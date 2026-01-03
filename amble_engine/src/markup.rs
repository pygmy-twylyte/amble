//! Lightweight inline markup for terminal output.
//!
//! Amble content is data-first: most output strings originate in `.amble` DSL and
//! are compiled into `WorldDef` data (`world.ron`). This module allows authors to add a small amount of
//! formatting inside those strings without touching Rust code.
//!
//! # Syntax
//! Tags use the `[[tag]]...[[/tag]]` form.
//!
//! Inline tags:
//! - `[[b]]` / `[[/b]]` (bold)
//! - `[[u]]` / `[[/u]]` (underline)
//! - `[[i]]` / `[[/i]]` (italic)
//! - `[[dim]]` / `[[/dim]]` (dim)
//! - Color tags like `[[red]]...[[/red]]` and `[[color=red]]...[[/color]]`
//! - Theme tags like `[[item]]`, `[[npc]]`, `[[room]]`, `[[highlight]]`, `[[triggered]]`
//!
//! Block tags (only recognized by [`render_wrapped`]):
//! - `[[center]]...[[/center]]` centers the wrapped lines
//! - `[[box]]...[[/box]]` draws a simple box around the wrapped lines
//!   - Optional title: `[[box:Title]]...[[/box]]`
//!
//! Escaping:
//! - Prefix `[[` or `]]` with a backslash to render it literally: `\\[[` or `\\]]`.

use std::fmt::Write;

use colored::{Color, ColoredString, Colorize};
use textwrap::{Options, fill, wrap_algorithms::Penalties};

use crate::style::GameStyle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WrapMode {
    Normal,
    Indented,
}

impl WrapMode {
    fn indent_str(self) -> &'static str {
        match self {
            WrapMode::Normal => "",
            WrapMode::Indented => "    ",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleKind {
    Plain,
    Description,
    Overlay,
    ItemText,
    Item,
    Npc,
    Room,
    Highlight,
    Error,
    Triggered,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct StyleMods {
    pub bold: bool,
    pub underline: bool,
    pub italic: bool,
    pub dimmed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StyleState {
    kind: StyleKind,
    mods: StyleMods,
    fg: Option<Color>,
}

impl StyleState {
    fn new(kind: StyleKind, mods: StyleMods) -> Self {
        Self { kind, mods, fg: None }
    }

    fn apply_to(self, text: &str) -> String {
        if text.is_empty() {
            return String::new();
        }

        if self.kind == StyleKind::Plain && self.fg.is_none() && self.mods == StyleMods::default() {
            return text.to_string();
        }

        let mut styled: ColoredString = match self.kind {
            StyleKind::Plain => text.normal(),
            StyleKind::Description => text.description_style(),
            StyleKind::Overlay => text.overlay_style(),
            StyleKind::ItemText => text.item_text_style(),
            StyleKind::Item => text.item_style(),
            StyleKind::Npc => text.npc_style(),
            StyleKind::Room => text.room_style(),
            StyleKind::Highlight => text.highlight(),
            StyleKind::Error => text.error_style(),
            StyleKind::Triggered => text.triggered_style(),
        };

        if let Some(fg) = self.fg {
            styled = styled.color(fg);
        }
        if self.mods.bold {
            styled = styled.bold();
        }
        if self.mods.underline {
            styled = styled.underline();
        }
        if self.mods.italic {
            styled = styled.italic();
        }
        if self.mods.dimmed {
            styled = styled.dimmed();
        }

        styled.to_string()
    }
}

/// Render inline markup without wrapping or block processing.
pub fn render_inline(text: &str, base: StyleKind, base_mods: StyleMods) -> String {
    render_inline_inner(text, StyleState::new(base, base_mods), /*allow_block_tags=*/ false)
}

/// Render markup with wrapping and support for `[[center]]` and `[[box]]` blocks.
pub fn render_wrapped(text: &str, width: usize, wrap: WrapMode, base: StyleKind, base_mods: StyleMods) -> String {
    let blocks = parse_blocks(text);
    let mut out = String::new();

    for (idx, block) in blocks.into_iter().enumerate() {
        if idx > 0 {
            out.push('\n');
        }
        let rendered = match block.kind {
            BlockKind::Normal => {
                let ansi = render_inline_inner(
                    &block.text,
                    StyleState::new(base, base_mods),
                    /*allow_block_tags=*/ true,
                );
                let opts = wrap_options(width, wrap.indent_str());
                fill(&ansi, opts)
            },
            BlockKind::Center => render_center_block(&block.text, width, wrap, base, base_mods),
            BlockKind::Box { title } => render_box_block(&block.text, title.as_deref(), width, wrap, base, base_mods),
        };
        out.push_str(&rendered);
    }

    out
}

fn wrap_options(width: usize, indent: &'static str) -> Options<'static> {
    Options::new(width)
        .initial_indent(indent)
        .subsequent_indent(indent)
        .wrap_algorithm(textwrap::WrapAlgorithm::OptimalFit(Penalties::new()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BlockKind {
    Normal,
    Center,
    Box { title: Option<String> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Block {
    kind: BlockKind,
    text: String,
}

fn parse_blocks(input: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut current = Block {
        kind: BlockKind::Normal,
        text: String::new(),
    };

    let mut idx = 0;
    while idx < input.len() {
        // Preserve escapes for inline parsing.
        if input[idx..].starts_with("\\[[") || input[idx..].starts_with("\\]]") {
            let esc = &input[idx..idx + 3];
            current.text.push_str(esc);
            idx += 3;
            continue;
        }

        if input[idx..].starts_with("[[") {
            let Some(close_rel) = input[idx + 2..].find("]]") else {
                // Unclosed tag; treat remainder as literal.
                current.text.push_str(&input[idx..]);
                break;
            };
            let close_idx = idx + 2 + close_rel;
            let raw_tag = &input[idx + 2..close_idx];
            let tag = raw_tag.trim();

            let tag_span = &input[idx..close_idx + 2];

            if let Some(block_tag) = parse_block_tag(tag) {
                match (&current.kind, block_tag) {
                    (BlockKind::Normal, BlockTag::Start(kind)) => {
                        if !current.text.is_empty() {
                            blocks.push(current);
                        }
                        current = Block {
                            kind,
                            text: String::new(),
                        };
                        idx = close_idx + 2;
                        continue;
                    },
                    (BlockKind::Center, BlockTag::EndCenter) | (BlockKind::Box { .. }, BlockTag::EndBox) => {
                        blocks.push(current);
                        current = Block {
                            kind: BlockKind::Normal,
                            text: String::new(),
                        };
                        idx = close_idx + 2;
                        continue;
                    },
                    _ => {
                        // Block tags are only recognized at top-level; render literally otherwise.
                        current.text.push_str(tag_span);
                        idx = close_idx + 2;
                        continue;
                    },
                }
            }

            // Not a block tag: keep it for inline parsing.
            current.text.push_str(tag_span);
            idx = close_idx + 2;
            continue;
        }

        let ch = input[idx..].chars().next().expect("idx is in-bounds");
        current.text.push(ch);
        idx += ch.len_utf8();
    }

    if !current.text.is_empty() || blocks.is_empty() {
        blocks.push(current);
    }
    blocks
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BlockTag {
    Start(BlockKind),
    EndCenter,
    EndBox,
}

fn parse_block_tag(tag: &str) -> Option<BlockTag> {
    let trimmed = tag.trim();
    let lower = trimmed.to_ascii_lowercase();

    if lower == "center" {
        return Some(BlockTag::Start(BlockKind::Center));
    }
    if lower == "/center" {
        return Some(BlockTag::EndCenter);
    }
    if lower == "box" {
        return Some(BlockTag::Start(BlockKind::Box { title: None }));
    }
    if lower == "/box" {
        return Some(BlockTag::EndBox);
    }

    if lower.starts_with("box:") {
        let title = trimmed[4..].trim();
        return Some(BlockTag::Start(BlockKind::Box {
            title: if title.is_empty() {
                None
            } else {
                Some(title.to_string())
            },
        }));
    }
    if lower.starts_with("box=") {
        let title = trimmed[4..].trim();
        return Some(BlockTag::Start(BlockKind::Box {
            title: if title.is_empty() {
                None
            } else {
                Some(title.to_string())
            },
        }));
    }

    None
}

fn strip_markup_tags(text: &str) -> String {
    let mut out = String::new();
    let mut idx = 0;
    while idx < text.len() {
        if text[idx..].starts_with("\\[[") {
            out.push_str("[[");
            idx += 3;
            continue;
        }
        if text[idx..].starts_with("\\]]") {
            out.push_str("]]");
            idx += 3;
            continue;
        }
        if text[idx..].starts_with("\\\\") {
            out.push('\\');
            idx += 2;
            continue;
        }
        if text[idx..].starts_with("[[") {
            if let Some(close_rel) = text[idx + 2..].find("]]") {
                idx = idx + 2 + close_rel + 2;
                continue;
            }
            out.push_str(&text[idx..]);
            break;
        }
        let ch = text[idx..].chars().next().expect("idx is in-bounds");
        out.push(ch);
        idx += ch.len_utf8();
    }
    out
}

fn render_center_block(text: &str, width: usize, wrap: WrapMode, base: StyleKind, base_mods: StyleMods) -> String {
    let indent = wrap.indent_str();
    let indent_width = textwrap::core::display_width(indent);
    let content_width = width.saturating_sub(indent_width);

    let ansi = render_inline_inner(text, StyleState::new(base, base_mods), /*allow_block_tags=*/ true);
    let opts = wrap_options(content_width.max(1), "");
    let wrapped = fill(&ansi, opts);

    let mut out = String::new();
    for (i, line) in wrapped.lines().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        let line_width = textwrap::core::display_width(line);
        let pad = content_width.saturating_sub(line_width) / 2;
        out.push_str(indent);
        out.push_str(&" ".repeat(pad));
        out.push_str(line);
    }
    out
}

fn render_box_block(
    text: &str,
    title: Option<&str>,
    width: usize,
    wrap: WrapMode,
    base: StyleKind,
    base_mods: StyleMods,
) -> String {
    let indent = wrap.indent_str();
    let indent_width = textwrap::core::display_width(indent);
    let content_width = width.saturating_sub(indent_width);
    let inner_wrap_width = content_width.saturating_sub(4).max(1);

    let ansi = render_inline_inner(text, StyleState::new(base, base_mods), /*allow_block_tags=*/ true);
    let opts = wrap_options(inner_wrap_width, "");
    let wrapped = fill(&ansi, opts);
    let mut lines: Vec<&str> = wrapped.lines().collect();
    if lines.is_empty() {
        lines.push("");
    }

    let mut max_len = 0usize;
    for line in &lines {
        max_len = max_len.max(textwrap::core::display_width(line));
    }

    let border_mods = if base == StyleKind::Plain {
        StyleMods::default()
    } else {
        StyleMods {
            dimmed: true,
            ..StyleMods::default()
        }
    };
    let border_style = StyleState::new(base, border_mods);
    let box_width = max_len + 4;
    let center_pad_width = content_width.saturating_sub(box_width) / 2;
    let left_pad = " ".repeat(center_pad_width);

    let mut out = String::new();

    // Top border
    out.push_str(indent);
    out.push_str(&left_pad);
    out.push_str(&border_style.apply_to("┌"));
    if let Some(title) = title.filter(|t| !t.trim().is_empty()) {
        let trimmed = title.trim();
        let visible = strip_markup_tags(trimmed);
        let visible_with_padding = format!(" {visible} ");
        let title_len = textwrap::core::display_width(&visible_with_padding);
        let dash_total = max_len + 2;
        let left = dash_total.saturating_sub(title_len) / 2;
        let right = dash_total.saturating_sub(title_len).saturating_sub(left);
        out.push_str(&border_style.apply_to(&"─".repeat(left)));
        out.push_str(&border_style.apply_to(" "));
        out.push_str(&render_inline_inner(
            trimmed,
            StyleState::new(base, base_mods),
            /*allow_block_tags=*/ false,
        ));
        out.push_str(&border_style.apply_to(" "));
        out.push_str(&border_style.apply_to(&"─".repeat(right)));
    } else {
        out.push_str(&border_style.apply_to(&"─".repeat(max_len + 2)));
    }
    out.push_str(&border_style.apply_to("┐"));
    out.push('\n');

    // Body
    for (idx, line) in lines.iter().enumerate() {
        if idx > 0 {
            out.push('\n');
        }
        let line_len = textwrap::core::display_width(line);
        let pad_right = max_len.saturating_sub(line_len);
        out.push_str(indent);
        out.push_str(&left_pad);
        out.push_str(&border_style.apply_to("│ "));
        out.push_str(line);
        out.push_str(&" ".repeat(pad_right));
        out.push_str(&border_style.apply_to(" │"));
    }

    // Bottom border
    out.push('\n');
    out.push_str(indent);
    out.push_str(&left_pad);
    out.push_str(&border_style.apply_to("└"));
    out.push_str(&border_style.apply_to(&"─".repeat(max_len + 2)));
    out.push_str(&border_style.apply_to("┘"));

    out
}

fn render_inline_inner(text: &str, base_state: StyleState, allow_block_tags: bool) -> String {
    // Inline tags are always supported. Block tags are only removed when wrapping.
    let mut out = String::new();
    let mut buf = String::new();
    let mut stack: Vec<(String, StyleState)> = vec![("root".to_string(), base_state)];

    let mut idx = 0;
    while idx < text.len() {
        if text[idx..].starts_with("\\[[") {
            buf.push_str("[[");
            idx += 3;
            continue;
        }
        if text[idx..].starts_with("\\]]") {
            buf.push_str("]]");
            idx += 3;
            continue;
        }
        if text[idx..].starts_with("\\\\") {
            buf.push('\\');
            idx += 2;
            continue;
        }

        if text[idx..].starts_with("[[") {
            let Some(close_rel) = text[idx + 2..].find("]]") else {
                buf.push_str(&text[idx..]);
                break;
            };
            let close_idx = idx + 2 + close_rel;
            let raw_tag = &text[idx + 2..close_idx];
            let tag = raw_tag.trim();

            // Optionally ignore block tags in inline mode so they render literally.
            if allow_block_tags && parse_block_tag(tag).is_some() {
                // Strip block tags from output; they are handled at the block layer.
                idx = close_idx + 2;
                continue;
            }

            if let Some(parsed) = parse_inline_tag(tag) {
                // flush pending text in current style before changing the stack
                if !buf.is_empty() {
                    let current_style = stack.last().expect("style stack never empty").1;
                    out.push_str(&current_style.apply_to(&buf));
                    buf.clear();
                }

                match parsed {
                    InlineTag::Start { name, change } => {
                        let current = stack.last().expect("style stack never empty").1;
                        stack.push((name, apply_change(current, change)));
                    },
                    InlineTag::End { name } => {
                        if let Some(pos) = stack.iter().rposition(|(n, _)| n == &name) {
                            stack.truncate(pos);
                        } else {
                            // Unknown close tag; render literally.
                            let _ = write!(buf, "[[{tag}]]");
                        }
                    },
                }

                idx = close_idx + 2;
                continue;
            }

            // Unknown inline tag: render literally.
            buf.push_str(&text[idx..close_idx + 2]);
            idx = close_idx + 2;
            continue;
        }

        let ch = text[idx..].chars().next().expect("idx is in-bounds");
        buf.push(ch);
        idx += ch.len_utf8();
    }

    if !buf.is_empty() {
        let current_style = stack.last().expect("style stack never empty").1;
        out.push_str(&current_style.apply_to(&buf));
    }

    out
}

fn apply_change(mut state: StyleState, change: StyleChange) -> StyleState {
    match change {
        StyleChange::Bold => state.mods.bold = true,
        StyleChange::Underline => state.mods.underline = true,
        StyleChange::Italic => state.mods.italic = true,
        StyleChange::Dimmed => state.mods.dimmed = true,
        StyleChange::SetKind(kind) => state.kind = kind,
        StyleChange::SetFg(color) => state.fg = Some(color),
    }
    state
}

#[derive(Debug)]
enum InlineTag {
    Start { name: String, change: StyleChange },
    End { name: String },
}

#[derive(Debug, Clone, Copy)]
enum StyleChange {
    Bold,
    Underline,
    Italic,
    Dimmed,
    SetKind(StyleKind),
    SetFg(Color),
}

fn parse_inline_tag(tag: &str) -> Option<InlineTag> {
    let trimmed = tag.trim();
    if trimmed.is_empty() {
        return None;
    }

    let (is_end, raw) = trimmed.strip_prefix('/').map_or((false, trimmed), |rest| (true, rest));
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }

    let normalized = normalize_tag_key(raw);

    // Close tags
    if is_end {
        let name = canonical_end_tag_name(&normalized)?;
        return Some(InlineTag::End { name });
    }

    // Start tags (possibly with parameters)
    if let Some((key, value)) = normalized.split_once('=') {
        return parse_key_value_tag(key.trim(), value.trim());
    }
    if let Some((key, value)) = normalized.split_once(':') {
        return parse_key_value_tag(key.trim(), value.trim());
    }

    // Shorthand tags (no parameters)
    let Some(name) = canonical_inline_tag_name(&normalized) else {
        // Treat bare color names as a shorthand.
        if let Some(color) = parse_color(&normalized) {
            return Some(InlineTag::Start {
                name: normalized,
                change: StyleChange::SetFg(color),
            });
        }
        return None;
    };

    let change = match name.as_str() {
        "b" => StyleChange::Bold,
        "u" => StyleChange::Underline,
        "i" => StyleChange::Italic,
        "dim" => StyleChange::Dimmed,
        "item" => StyleChange::SetKind(StyleKind::Item),
        "npc" => StyleChange::SetKind(StyleKind::Npc),
        "room" => StyleChange::SetKind(StyleKind::Room),
        "highlight" => StyleChange::SetKind(StyleKind::Highlight),
        "error" => StyleChange::SetKind(StyleKind::Error),
        "triggered" => StyleChange::SetKind(StyleKind::Triggered),
        _ => return None,
    };

    Some(InlineTag::Start { name, change })
}

fn canonical_inline_tag_name(lower: &str) -> Option<String> {
    Some(match lower {
        "b" | "bold" => "b".to_string(),
        "u" | "underline" => "u".to_string(),
        "i" | "italic" => "i".to_string(),
        "dim" | "dimmed" => "dim".to_string(),
        "item" => "item".to_string(),
        "npc" => "npc".to_string(),
        "room" => "room".to_string(),
        "hl" | "highlight" => "highlight".to_string(),
        "err" | "error" => "error".to_string(),
        "triggered" | "trig" => "triggered".to_string(),
        "color" | "fg" => "color".to_string(),
        // center and box block tags are handled elsewhere
        _ => return None,
    })
}

fn canonical_end_tag_name(lower: &str) -> Option<String> {
    if let Some(name) = canonical_inline_tag_name(lower) {
        return Some(name);
    }
    if parse_color(lower).is_some() {
        return Some(lower.to_string());
    }
    None
}

fn parse_key_value_tag(key: &str, value: &str) -> Option<InlineTag> {
    if key == "color" || key == "fg" {
        let color = parse_color(value)?;
        return Some(InlineTag::Start {
            name: "color".to_string(),
            change: StyleChange::SetFg(color),
        });
    }
    None
}

fn normalize_tag_key(tag: &str) -> String {
    tag.trim().to_ascii_lowercase().replace('_', "-")
}

fn parse_color(name: &str) -> Option<Color> {
    let n = normalize_tag_key(name);
    Some(match n.as_str() {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "white" => Color::White,
        "bright-black" | "gray" | "grey" => Color::BrightBlack,
        "bright-red" => Color::BrightRed,
        "bright-green" => Color::BrightGreen,
        "bright-yellow" => Color::BrightYellow,
        "bright-blue" => Color::BrightBlue,
        "bright-magenta" => Color::BrightMagenta,
        "bright-cyan" => Color::BrightCyan,
        "bright-white" => Color::BrightWhite,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_inline_bold_works() {
        let out = render_inline("Hello [[b]]World[[/b]]", StyleKind::Plain, StyleMods::default());
        assert!(out.contains("Hello "));
        assert!(!out.contains("[[b]]"));
        assert!(!out.contains("[[/b]]"));
        assert!(out.contains("World"));
    }

    #[test]
    fn render_inline_color_shorthand_works() {
        let out = render_inline("[[red]]Hi[[/red]]", StyleKind::Plain, StyleMods::default());
        assert!(!out.contains("[[red]]"));
        assert!(!out.contains("[[/red]]"));
        assert!(out.contains("Hi"));
    }

    #[test]
    fn render_wrapped_center_centers_line() {
        let out = render_wrapped(
            "[[center]]Hello[[/center]]",
            10,
            WrapMode::Normal,
            StyleKind::Plain,
            StyleMods::default(),
        );
        assert_eq!(out, "  Hello");
    }
}
