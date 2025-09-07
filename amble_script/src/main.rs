//! CLI entry point for amble_script.
//! Usage: cargo run -p amble_script -- compile examples/first.able

use std::{env, fs, process};

use amble_script::{ActionAst, ConditionAst, compile_triggers_to_toml, parse_program};
use std::collections::{HashMap, HashSet};
use toml_edit::Document;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Accept either:
    // 1) cargo run: <bin> -- <cmd> <args>
    // 2) direct:    <bin> <cmd> <args>
    // Extract subcommand and collect the rest for flags/positional
    let rest: Vec<String> = match args.as_slice() {
        [_, flag, cmd, tail @ ..] if flag == "--" && (cmd == "compile" || cmd == "lint") => {
            let mut v = vec![cmd.clone()];
            v.extend_from_slice(tail);
            v
        },
        [_, cmd, tail @ ..] if cmd == "compile" || cmd == "lint" => {
            let mut v = vec![cmd.clone()];
            v.extend_from_slice(tail);
            v
        },
        _ => {
            eprintln!(
                "Usage:\n  amble_script compile <file.amble> [--out <out.toml>]\n  amble_script lint <file.amble> [--data-dir <dir>] [--deny-missing]"
            );
            process::exit(2);
        },
    };
    let cmd = &rest[0];
    if cmd == "compile" {
        run_compile(&rest[1..]);
    } else if cmd == "lint" {
        run_lint(&rest[1..]);
    } else {
        eprintln!("unknown command: {}", cmd);
        process::exit(2);
    }
}

fn run_compile(args: &[String]) {
    use std::process;
    let mut path: Option<String> = None;
    let mut out_path: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--out" {
            if i + 1 >= args.len() {
                eprintln!("--out requires a filepath");
                process::exit(2);
            }
            out_path = Some(args[i + 1].clone());
            i += 2;
            continue;
        }
        if path.is_none() {
            path = Some(args[i].clone());
        }
        i += 1;
    }
    if path.is_none() {
        eprintln!("Usage: amble_script compile <file.amble> [--out <out.toml>]");
        process::exit(2);
    }
    let path = path.unwrap();
    let src = fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("error: unable to read '{}': {}", &path, e);
        process::exit(1);
    });
    let asts = parse_program(&src).unwrap_or_else(|e| {
        eprintln!("parse error: {}", e);
        process::exit(1);
    });
    for t in &asts {
        if t.actions.is_empty() {
            eprintln!("warning: trigger '{}' has no actions (empty block?)", t.name);
        }
    }
    match compile_triggers_to_toml(&asts) {
        Ok(toml) => {
            if let Some(out) = out_path {
                fs::write(&out, toml).unwrap_or_else(|e| {
                    eprintln!("error: writing '{}': {}", &out, e);
                    process::exit(1);
                });
            } else {
                println!("{}", toml);
            }
        },
        Err(e) => {
            eprintln!("compile error: {}", e);
            process::exit(1);
        },
    }
}

fn run_lint(args: &[String]) {
    use std::process;
    let mut path: Option<String> = None;
    let mut data_dir: Option<String> = None;
    let mut deny_missing = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--data-dir" => {
                if i + 1 >= args.len() {
                    eprintln!("--data-dir requires a path to amble_engine/data");
                    process::exit(2);
                }
                data_dir = Some(args[i + 1].clone());
                i += 2;
                continue;
            },
            "--deny-missing" => {
                deny_missing = true;
                i += 1;
                continue;
            },
            s => {
                if path.is_none() {
                    path = Some(s.to_string());
                }
                i += 1;
            },
        }
    }
    if path.is_none() {
        eprintln!("Usage: amble_script lint <file.amble> [--data-dir <dir>] [--deny-missing]");
        process::exit(2);
    }
    let path = path.unwrap();
    let data_dir = data_dir.unwrap_or_else(|| "amble_engine/data".to_string());
    let world = load_world_refs(&data_dir).unwrap_or_else(|e| {
        eprintln!("lint: failed to load data dir '{}': {}", &data_dir, e);
        process::exit(2);
    });

    // Support linting a single file or a directory of files (recursive)
    let mut files = Vec::new();
    let md = fs::metadata(&path).unwrap_or_else(|e| {
        eprintln!("error: stat '{}': {}", &path, e);
        process::exit(1);
    });
    if md.is_dir() {
        collect_dsl_files_recursive(&path, &mut files);
        if files.is_empty() {
            eprintln!("lint: no .amble/.able files in directory '{}'", &path);
        }
    } else {
        files.push(path.clone());
    }

    let mut any_missing = 0usize;
    for f in files {
        any_missing += lint_one_file(&f, &world);
    }
    if any_missing == 0 {
        eprintln!("lint: OK (no missing cross references)");
    }
    if deny_missing && any_missing > 0 {
        process::exit(1);
    }
}

fn collect_dsl_files_recursive(dir: &str, out: &mut Vec<String>) {
    if let Ok(rd) = fs::read_dir(dir) {
        for ent in rd.flatten() {
            let p = ent.path();
            if p.is_dir() {
                if let Some(s) = p.to_str() {
                    collect_dsl_files_recursive(s, out);
                }
                continue;
            }
            if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                if ext == "amble" || ext == "able" {
                    if let Some(s) = p.to_str() {
                        out.push(s.to_string());
                    }
                }
            }
        }
    }
}

fn lint_one_file(path: &str, world: &WorldRefs) -> usize {
    let src = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("lint: cannot read '{}': {}", path, e);
            return 0;
        },
    };
    let asts = match parse_program(&src) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("lint: parse error in '{}': {}", path, e);
            return 0;
        },
    };
    let mut refs: HashMap<&'static str, HashSet<String>> = HashMap::new();
    refs.insert("item", HashSet::new());
    refs.insert("room", HashSet::new());
    refs.insert("npc", HashSet::new());
    refs.insert("spinner", HashSet::new());
    for t in &asts {
        gather_refs_from_condition(&t.event, &mut refs);
        for c in &t.conditions {
            gather_refs_from_condition(c, &mut refs);
        }
        for a in &t.actions {
            gather_refs_from_action(a, &mut refs);
        }
    }
    let mut missing = 0usize;
    for id in &refs["item"] {
        if !world.items.contains(id) {
            report_missing_with_location(path, &src, "item", id);
            missing += 1;
        }
    }
    for id in &refs["room"] {
        if !world.rooms.contains(id) {
            report_missing_with_location(path, &src, "room", id);
            missing += 1;
        }
    }
    for id in &refs["npc"] {
        if !world.npcs.contains(id) {
            report_missing_with_location(path, &src, "npc", id);
            missing += 1;
        }
    }
    for id in &refs["spinner"] {
        if !world.spinners.contains(id) {
            report_missing_with_location(path, &src, "spinner", id);
            missing += 1;
        }
    }
    missing
}

fn report_missing_with_location(path: &str, src: &str, kind: &str, id: &str) {
    if let Some((line_no, col, line)) = find_position_for_id(src, kind, id) {
        eprintln!(
            "{}:{}:{}: missing {} '{}'\n{}\n{}^",
            path,
            line_no,
            col,
            kind,
            id,
            line,
            " ".repeat(col.saturating_sub(1))
        );
    } else {
        eprintln!("{}: missing {} '{}' (no position)", path, kind, id);
    }
}

fn find_position_for_id(src: &str, kind: &str, id: &str) -> Option<(usize, usize, String)> {
    let patterns: Vec<String> = match kind {
        "item" => vec![
            format!(" item {}", id),
            format!(" container {}", id),
            format!(" description {}", id),
            format!(" restrict item {}", id),
        ],
        "room" => vec![format!(" room {}", id), format!(" to {}", id), format!(" from {}", id)],
        "npc" => vec![
            format!(" npc {}", id),
            format!(" from npc {}", id),
            format!(" with npc {}", id),
        ],
        "spinner" => vec![format!(" spinner {}", id), format!(" ambient {}", id)],
        _ => vec![id.to_string()],
    };
    let bytes = src.as_bytes();
    for pat in patterns {
        if let Some(idx) = src.find(&pat) {
            return byte_index_to_line_col(src, idx + pat.find(id).unwrap_or(0));
        }
    }
    // Fallback: find id as a whole word
    let mut i = 0usize;
    while let Some(pos) = src[i..].find(id) {
        let abs = i + pos;
        if is_word_boundary(bytes, abs.saturating_sub(1)) && is_word_boundary(bytes, abs + id.len()) {
            return byte_index_to_line_col(src, abs);
        }
        i = abs + id.len();
    }
    None
}

fn is_word_boundary(bytes: &[u8], idx: usize) -> bool {
    if idx >= bytes.len() {
        return true;
    }
    let c = bytes[idx] as char;
    !(c.is_alphanumeric() || c == '_' || c == '-' || c == ':')
}

fn byte_index_to_line_col(src: &str, idx: usize) -> Option<(usize, usize, String)> {
    let mut line_no = 1usize;
    let mut col = 1usize;

    let mut line_start = 0usize;
    for (pos, ch) in src.char_indices() {
        if pos >= idx {
            col = idx.saturating_sub(line_start) + 1;
            break;
        }
        if ch == '\n' {
            line_no += 1;
            line_start = pos + 1;
        }
    }
    let line_end = src[idx..].find('\n').map(|off| idx + off).unwrap_or(src.len());
    let line = src[line_start..line_end].to_string();
    Some((line_no, col, line))
}

fn gather_refs_from_condition(c: &ConditionAst, out: &mut HashMap<&'static str, HashSet<String>>) {
    match c {
        ConditionAst::EnterRoom(r)
        | ConditionAst::LeaveRoom(r)
        | ConditionAst::PlayerInRoom(r)
        | ConditionAst::HasVisited(r) => {
            out.get_mut("room").unwrap().insert(r.clone());
        },
        ConditionAst::TakeItem(i)
        | ConditionAst::OpenItem(i)
        | ConditionAst::LookAtItem(i)
        | ConditionAst::DropItem(i)
        | ConditionAst::UnlockItem(i)
        | ConditionAst::HasItem(i)
        | ConditionAst::MissingItem(i) => {
            out.get_mut("item").unwrap().insert(i.clone());
        },
        ConditionAst::TalkToNpc(n) | ConditionAst::WithNpc(n) => {
            out.get_mut("npc").unwrap().insert(n.clone());
        },
        ConditionAst::UseItem { item, .. } => {
            out.get_mut("item").unwrap().insert(item.clone());
        },
        ConditionAst::GiveToNpc { item, npc } => {
            out.get_mut("item").unwrap().insert(item.clone());
            out.get_mut("npc").unwrap().insert(npc.clone());
        },
        ConditionAst::UseItemOnItem { tool, target, .. } => {
            out.get_mut("item").unwrap().insert(tool.clone());
            out.get_mut("item").unwrap().insert(target.clone());
        },
        ConditionAst::ActOnItem { target, .. } => {
            out.get_mut("item").unwrap().insert(target.clone());
        },
        ConditionAst::TakeFromNpc { item, npc } => {
            out.get_mut("item").unwrap().insert(item.clone());
            out.get_mut("npc").unwrap().insert(npc.clone());
        },
        ConditionAst::InsertItemInto { item, container } => {
            out.get_mut("item").unwrap().insert(item.clone());
            out.get_mut("item").unwrap().insert(container.clone());
        },
        ConditionAst::NpcHasItem { npc, item } => {
            out.get_mut("npc").unwrap().insert(npc.clone());
            out.get_mut("item").unwrap().insert(item.clone());
        },
        ConditionAst::NpcInState { npc, .. } => {
            out.get_mut("npc").unwrap().insert(npc.clone());
        },
        ConditionAst::ContainerHasItem { container, item } => {
            out.get_mut("item").unwrap().insert(container.clone());
            out.get_mut("item").unwrap().insert(item.clone());
        },
        ConditionAst::Ambient { spinner, rooms } => {
            out.get_mut("spinner").unwrap().insert(spinner.clone());
            if let Some(rs) = rooms {
                for r in rs {
                    out.get_mut("room").unwrap().insert(r.clone());
                }
            }
        },
        ConditionAst::ChancePercent(_) | ConditionAst::Always => {},
        ConditionAst::All(kids) | ConditionAst::Any(kids) => {
            for k in kids {
                gather_refs_from_condition(k, out);
            }
        },
        ConditionAst::MissingFlag(_)
        | ConditionAst::HasFlag(_)
        | ConditionAst::FlagInProgress(_)
        | ConditionAst::FlagComplete(_) => { /* flags not cross-checked */ },
    }
}

fn gather_refs_from_action(a: &ActionAst, out: &mut HashMap<&'static str, HashSet<String>>) {
    match a {
        ActionAst::ReplaceItem { old_sym, new_sym } | ActionAst::ReplaceDropItem { old_sym, new_sym } => {
            out.get_mut("item").unwrap().insert(old_sym.clone());
            out.get_mut("item").unwrap().insert(new_sym.clone());
        },
        ActionAst::SpawnItemIntoRoom { item, room } => {
            out.get_mut("item").unwrap().insert(item.clone());
            out.get_mut("room").unwrap().insert(room.clone());
        },
        ActionAst::DespawnItem(i)
        | ActionAst::LockItem(i)
        | ActionAst::UnlockItemAction(i)
        | ActionAst::RestrictItem(i) => {
            out.get_mut("item").unwrap().insert(i.clone());
        },
        ActionAst::PushPlayerTo(r) => {
            out.get_mut("room").unwrap().insert(r.clone());
        },
        ActionAst::GiveItemToPlayer { npc, item } => {
            out.get_mut("npc").unwrap().insert(npc.clone());
            out.get_mut("item").unwrap().insert(item.clone());
        },
        ActionAst::SpawnItemInInventory(i) | ActionAst::SpawnItemCurrentRoom(i) => {
            out.get_mut("item").unwrap().insert(i.clone());
        },
        ActionAst::SpawnItemInContainer { item, container } => {
            out.get_mut("item").unwrap().insert(item.clone());
            out.get_mut("item").unwrap().insert(container.clone());
        },
        ActionAst::SetItemDescription { item, .. } => {
            out.get_mut("item").unwrap().insert(item.clone());
        },
        ActionAst::NpcSays { npc, .. }
        | ActionAst::NpcSaysRandom { npc }
        | ActionAst::NpcRefuseItem { npc, .. }
        | ActionAst::SetNpcState { npc, .. } => {
            out.get_mut("npc").unwrap().insert(npc.clone());
        },
        ActionAst::SetContainerState { item, .. } => {
            out.get_mut("item").unwrap().insert(item.clone());
        },
        ActionAst::SpinnerMessage { spinner } | ActionAst::AddSpinnerWedge { spinner, .. } => {
            out.get_mut("spinner").unwrap().insert(spinner.clone());
        },
        ActionAst::SetBarredMessage { exit_from, exit_to, .. } | ActionAst::RevealExit { exit_from, exit_to, .. } => {
            out.get_mut("room").unwrap().insert(exit_from.clone());
            out.get_mut("room").unwrap().insert(exit_to.clone());
        },
        ActionAst::LockExit { from_room, .. } | ActionAst::UnlockExit { from_room, .. } => {
            out.get_mut("room").unwrap().insert(from_room.clone());
        },
        ActionAst::ScheduleIn { actions, .. } | ActionAst::ScheduleOn { actions, .. } => {
            for aa in actions {
                gather_refs_from_action(aa, out);
            }
        },
        ActionAst::ScheduleInIf { actions, condition, .. } | ActionAst::ScheduleOnIf { actions, condition, .. } => {
            gather_refs_from_condition(condition, out);
            for aa in actions {
                gather_refs_from_action(aa, out);
            }
        },
        _ => {},
    }
}

struct WorldRefs {
    items: HashSet<String>,
    rooms: HashSet<String>,
    npcs: HashSet<String>,
    spinners: HashSet<String>,
}

fn load_world_refs(dir: &str) -> Result<WorldRefs, String> {
    let mut items = HashSet::new();
    let mut rooms = HashSet::new();
    let mut npcs = HashSet::new();
    let mut spinners = HashSet::new();

    fn load_ids(doc: &Document, key: &str, field: &str) -> HashSet<String> {
        let mut set = HashSet::new();
        if let Some(item) = doc.as_table().get(key) {
            if let Some(aot) = item.as_array_of_tables() {
                for t in aot.iter() {
                    if let Some(v) = t.get(field) {
                        if let Some(s) = v.as_str() {
                            set.insert(s.to_string());
                        }
                    }
                }
            }
        }
        set
    }

    let items_path = format!("{}/items.toml", dir);
    let rooms_path = format!("{}/rooms.toml", dir);
    let npcs_path = format!("{}/npcs.toml", dir);
    let spinners_path = format!("{}/spinners.toml", dir);

    let read = |p: &str| -> Result<Document, String> {
        let s = fs::read_to_string(p).map_err(|e| format!("{}", e))?;
        s.parse::<Document>().map_err(|e| format!("{}", e))
    };

    if let Ok(doc) = read(&items_path) {
        items = load_ids(&doc, "items", "id");
    }
    if let Ok(doc) = read(&rooms_path) {
        rooms = load_ids(&doc, "rooms", "id");
    }
    if let Ok(doc) = read(&npcs_path) {
        npcs = load_ids(&doc, "npcs", "id");
    }
    if let Ok(doc) = read(&spinners_path) {
        spinners = load_ids(&doc, "spinners", "spinnerType");
    }

    Ok(WorldRefs {
        items,
        rooms,
        npcs,
        spinners,
    })
}
