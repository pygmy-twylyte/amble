//! Terminal input handling for the Amble REPL.
//!
//! Wraps rustyline configuration, validation, and completion tailored to
//! the engine's command set and save-file workflow.

use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};

use lazy_static::lazy_static;
use log::{info, warn};
use pest_meta::ast::Expr;
use pest_meta::parser::{Rule as PestRule, consume_rules, parse};
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::history::DefaultHistory;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{Context, Helper};

use crate::save_files::{SAVE_DIR, collect_save_slots};

/// Outcome of reading a line from the REPL input.
pub enum InputEvent {
    Line(String),
    Eof,
    Interrupted,
}

lazy_static! {
    static ref COMMAND_TERMS: Vec<String> = build_command_terms();
}

const DEV_COMMANDS: &[&str] = &[
    ":help dev",
    ":npcs",
    ":flags",
    ":sched",
    ":schedule",
    ":schedule cancel",
    ":schedule delay",
    ":teleport",
    ":port",
    ":spawn",
    ":item",
    ":adv-seq",
    ":init-seq",
    ":reset-seq",
    ":set-flag",
];

const EXCLUDED_TERMS: &[&str] = &[
    "", "a", "an", "at", "down", "from", "in", "into", "on", "the", "to", "using", "with",
];

type ReplEditor = rustyline::Editor<AmbleHelper, DefaultHistory>;

#[derive(Default)]
struct AmbleHelper;

impl Helper for AmbleHelper {}

impl Completer for AmbleHelper {
    type Candidate = Pair;

    fn complete(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let (start, prefix) = current_prefix(line, pos);
        if prefix.is_empty() {
            return Ok((start, Vec::new()));
        }
        let lower = prefix.to_lowercase();
        if let Some((replacement_start, candidates)) = load_command_completions(&prefix, &lower, start) {
            return Ok((replacement_start, candidates));
        }
        let mut pairs = Vec::new();
        for term in COMMAND_TERMS.iter() {
            if term.starts_with(&lower) {
                pairs.push(Pair {
                    display: term.clone(),
                    replacement: term.clone(),
                });
            }
        }
        Ok((start, pairs))
    }
}

impl Hinter for AmbleHelper {
    type Hint = String;
}

impl Highlighter for AmbleHelper {}

impl Validator for AmbleHelper {
    fn validate(&self, ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        let _ = ctx;
        Ok(ValidationResult::Valid(None))
    }
}

fn current_prefix(line: &str, pos: usize) -> (usize, String) {
    let slice = &line[..pos];
    let trimmed = slice.trim_start_matches(char::is_whitespace);
    let start = pos - trimmed.len();
    (start, trimmed.to_string())
}

fn build_command_terms() -> Vec<String> {
    let mut terms = collect_grammar_terms();
    terms.extend(DEV_COMMANDS.iter().map(|cmd| cmd.to_string()));
    terms.sort_unstable();
    terms.dedup();
    terms
}

fn collect_grammar_terms() -> Vec<String> {
    let grammar = include_str!("../repl_grammar.pest");
    let parsed = parse(PestRule::grammar_rules, grammar).expect("failed to parse repl_grammar.pest for completions");
    let rules = consume_rules(parsed).expect("invalid repl grammar AST");
    let mut terms = Vec::new();

    for rule in rules {
        if let Some(sequences) = literal_sequences(&rule.expr) {
            for seq in sequences {
                if let Some(term) = normalize_sequence(&seq) {
                    if should_include(&term) {
                        terms.push(term);
                    }
                }
            }
        }
    }
    terms
}

fn literal_sequences(expr: &Expr) -> Option<Vec<Vec<String>>> {
    match expr {
        Expr::Str(s) | Expr::Insens(s) => {
            let token = s.trim();
            if token.is_empty() {
                Some(vec![vec![]])
            } else {
                Some(vec![vec![token.to_lowercase()]])
            }
        },
        Expr::Seq(lhs, rhs) => {
            let left = literal_sequences(lhs)?;
            let right = literal_sequences(rhs)?;
            let mut combined = Vec::new();
            for l in &left {
                for r in &right {
                    let mut merged = l.clone();
                    merged.extend(r.clone());
                    combined.push(merged);
                }
            }
            Some(combined)
        },
        Expr::Choice(lhs, rhs) => {
            let mut results = Vec::new();
            if let Some(mut left) = literal_sequences(lhs) {
                results.append(&mut left);
            }
            if let Some(mut right) = literal_sequences(rhs) {
                results.append(&mut right);
            }
            Some(results)
        },
        Expr::Opt(_)
        | Expr::Rep(_)
        | Expr::RepOnce(_)
        | Expr::RepExact(_, _)
        | Expr::RepMin(_, _)
        | Expr::RepMax(_, _)
        | Expr::RepMinMax(_, _, _) => {
            // Optional/Repeated expressions do not contribute to base command text.
            Some(vec![Vec::new()])
        },
        Expr::PosPred(_) | Expr::NegPred(_) | Expr::Push(_) => {
            // Lookahead and push do not consume input; ignore them for literals.
            Some(vec![Vec::new()])
        },
        Expr::PeekSlice(_, _) | Expr::Range(_, _) | Expr::Skip(_) | Expr::Ident(_) => None,
    }
}

fn normalize_sequence(sequence: &[String]) -> Option<String> {
    if sequence.is_empty() {
        return None;
    }
    let mut parts = Vec::new();
    for token in sequence {
        let trimmed = token.trim();
        if trimmed.is_empty() {
            continue;
        }
        parts.push(trimmed.to_string());
    }
    if parts.is_empty() { None } else { Some(parts.join(" ")) }
}

fn should_include(term: &str) -> bool {
    let normalized = term.trim();
    if normalized.is_empty() {
        return false;
    }
    let lower = normalized.to_lowercase();
    !EXCLUDED_TERMS.contains(&lower.as_str())
}

fn load_command_completions(prefix: &str, lower: &str, start: usize) -> Option<(usize, Vec<Pair>)> {
    let keyword = if matches_keyword(lower, "load") {
        "load"
    } else if matches_keyword(lower, "reload") {
        "reload"
    } else {
        return None;
    };

    if prefix.len() < keyword.len() {
        return None;
    }

    let command_part = &prefix[..keyword.len()];
    let after_keyword = &prefix[keyword.len()..];
    let trimmed_after = after_keyword.trim_start();
    let insertion_offset = prefix.len() - trimmed_after.len();
    let slots = available_save_slots();

    if after_keyword.is_empty() {
        let mut pairs = Vec::new();
        for slot in slots {
            pairs.push(Pair {
                display: format!("{command_part} {slot}"),
                replacement: format!(" {}", slot),
            });
        }
        return Some((start + prefix.len(), pairs));
    }

    let lower_partial = trimmed_after.to_lowercase();
    let mut pairs = Vec::new();
    for slot in slots
        .into_iter()
        .filter(|slot| lower_partial.is_empty() || slot.starts_with(&lower_partial))
    {
        pairs.push(Pair {
            display: slot.clone(),
            replacement: slot,
        });
    }
    Some((start + insertion_offset, pairs))
}

fn matches_keyword(lower: &str, keyword: &str) -> bool {
    if lower.len() < keyword.len() {
        return false;
    }
    if lower == keyword {
        return true;
    }
    if lower.starts_with(keyword) {
        return lower.chars().nth(keyword.len()).map_or(false, char::is_whitespace);
    }
    false
}

fn available_save_slots() -> Vec<String> {
    let dir = Path::new(SAVE_DIR);
    match collect_save_slots(dir) {
        Ok(slots) => {
            let mut names = Vec::new();
            for slot in slots {
                if names.last().map_or(true, |last| last != &slot.slot) {
                    names.push(slot.slot);
                }
            }
            names
        },
        Err(err) => {
            warn!("Failed to enumerate save slots for completion: {err}");
            Vec::new()
        },
    }
}

/// Helper responsible for managing the interactive input backend.
///
/// Prefers `rustyline` when an interactive terminal is available, falling back to
/// a basic stdin reader otherwise.
pub struct InputManager {
    backend: Backend,
}

impl InputManager {
    pub fn new() -> Self {
        let backend = if io::stdin().is_terminal() {
            match RustylineInput::new() {
                Ok(editor) => {
                    info!("using rustyline-backed REPL input");
                    Backend::Rustyline(editor)
                },
                Err(err) => {
                    warn!("failed to initialize rustyline ({}), falling back to basic stdin", err);
                    Backend::plain()
                },
            }
        } else {
            info!("stdin is not a TTY; using basic input mode");
            Backend::plain()
        };

        Self { backend }
    }

    /// Read a line from the current backend. If the interactive backend reports an
    /// unrecoverable error, switch to the plain stdin backend and retry once.
    pub fn read_line(&mut self, prompt: &str) -> io::Result<InputEvent> {
        match self.backend.read_line(prompt) {
            Ok(event) => Ok(event),
            Err(err) => {
                if self.backend.is_rustyline() {
                    warn!("rustyline input failed: {} -- switching to basic stdin", err);
                    self.backend = Backend::plain();
                    self.backend.read_line(prompt)
                } else {
                    Err(err)
                }
            },
        }
    }
}

enum Backend {
    Rustyline(RustylineInput),
    Plain(StdinInput),
}

impl Backend {
    fn plain() -> Self {
        Backend::Plain(StdinInput::default())
    }

    fn is_rustyline(&self) -> bool {
        matches!(self, Backend::Rustyline(_))
    }

    fn read_line(&mut self, prompt: &str) -> io::Result<InputEvent> {
        match self {
            Backend::Rustyline(editor) => editor.read_line(prompt),
            Backend::Plain(stdin) => stdin.read_line(prompt),
        }
    }
}

struct RustylineInput {
    editor: ReplEditor,
    history_path: Option<PathBuf>,
}

impl RustylineInput {
    fn new() -> io::Result<Self> {
        let mut editor = rustyline::Editor::<AmbleHelper, _>::new().map_err(map_io_err)?;
        editor.set_helper(Some(AmbleHelper::default()));
        let history_path = history_file_path();

        if let Some(path) = history_path.as_ref() {
            if let Some(dir) = path.parent() {
                if let Err(err) = fs::create_dir_all(dir) {
                    warn!("failed to create history directory {}: {}", dir.display(), err);
                }
            }

            if let Err(err) = editor.load_history(path) {
                match err {
                    ReadlineError::Io(ref io_err) if io_err.kind() == io::ErrorKind::NotFound => {
                        info!("no prior history found at {}, starting fresh", path.display());
                    },
                    other => {
                        warn!("failed to load history from {}: {}", path.display(), other);
                    },
                }
            }
        }

        Ok(Self { editor, history_path })
    }

    fn read_line(&mut self, prompt: &str) -> io::Result<InputEvent> {
        match self.editor.readline(prompt) {
            Ok(line) => {
                if !line.trim().is_empty() {
                    if let Err(err) = self.editor.add_history_entry(line.as_str()) {
                        warn!("failed to append to history: {}", err);
                    }
                    if let Some(path) = self.history_path.as_ref() {
                        if let Err(err) = self.editor.save_history(path) {
                            warn!("failed to persist history to {}: {}", path.display(), err);
                        }
                    }
                }
                Ok(InputEvent::Line(line))
            },
            Err(err) => convert_readline_error(err),
        }
    }
}

#[derive(Default)]
struct StdinInput {
    buffer: String,
}

impl StdinInput {
    fn read_line(&mut self, prompt: &str) -> io::Result<InputEvent> {
        print!("{}", prompt);
        io::stdout().flush()?;

        self.buffer.clear();
        let bytes = io::stdin().read_line(&mut self.buffer)?;
        if bytes == 0 {
            return Ok(InputEvent::Eof);
        }

        if self.buffer.ends_with('\n') {
            self.buffer.pop();
            if self.buffer.ends_with('\r') {
                self.buffer.pop();
            }
        }

        Ok(InputEvent::Line(self.buffer.clone()))
    }
}

fn convert_readline_error(err: ReadlineError) -> io::Result<InputEvent> {
    match err {
        ReadlineError::Interrupted => Ok(InputEvent::Interrupted),
        ReadlineError::Eof => Ok(InputEvent::Eof),
        ReadlineError::Io(io_err) => Err(io_err),
        other => Err(io::Error::new(io::ErrorKind::Other, other)),
    }
}

fn map_io_err(err: ReadlineError) -> io::Error {
    match err {
        ReadlineError::Io(io_err) => io_err,
        other => io::Error::new(io::ErrorKind::Other, other),
    }
}

fn history_file_path() -> Option<PathBuf> {
    dirs::data_dir()
        .or_else(dirs::data_local_dir)
        .map(|base| build_history_path(&base))
}

fn build_history_path(base: &Path) -> PathBuf {
    let mut path = base.to_path_buf();
    path.push("amble_engine");
    path.push("history.txt");
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn converts_readline_ctrl_c_to_interrupt() {
        let result = convert_readline_error(ReadlineError::Interrupted).unwrap();
        assert!(matches!(result, InputEvent::Interrupted));
    }

    #[test]
    fn history_path_appends_components() {
        let base = PathBuf::from("/tmp/amble-test");
        let path = build_history_path(&base);
        assert!(path.ends_with(Path::new("amble_engine/history.txt")));
    }

    #[test]
    fn grammar_terms_include_expected_commands() {
        assert!(COMMAND_TERMS.iter().any(|term| term == "inventory"));
        assert!(COMMAND_TERMS.iter().any(|term| term == "take"));
    }

    #[test]
    fn grammar_terms_exclude_articles() {
        assert!(!COMMAND_TERMS.iter().any(|term| term == "the"));
    }

    #[test]
    fn dev_commands_are_present() {
        assert!(COMMAND_TERMS.iter().any(|term| term == ":sched"));
    }
}
