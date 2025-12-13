//! CLI entry point for `amble_script`.
//! Typical usage:
//! - `cargo run -p amble_script -- compile-dir /path/to/root/data/dir --out-dir amble_engine/data`
//! - `cargo run -p amble_script -- lint amble_script/data/Amble --deny-missing`

use std::{env, fs, process};

use amble_script::{
    ActionAst, ActionStmt, ConditionAst, GoalCondAst, compile_goals_to_toml, compile_npcs_to_toml,
    compile_rooms_to_toml, compile_spinners_to_toml, compile_triggers_to_toml, parse_program_full,
};
use std::collections::{HashMap, HashSet};
use toml_edit::Document;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Accept either:
    // 1) cargo run: <bin> -- <cmd> <args>
    // 2) direct:    <bin> <cmd> <args>
    // Extract subcommand and collect the rest for flags/positional
    let rest: Vec<String> = match args.as_slice() {
        [_, flag, cmd, tail @ ..] if flag == "--" && (cmd == "compile" || cmd == "lint" || cmd == "compile-dir") => {
            let mut v = vec![cmd.clone()];
            v.extend_from_slice(tail);
            v
        },
        [_, cmd, tail @ ..] if cmd == "compile" || cmd == "lint" || cmd == "compile-dir" => {
            let mut v = vec![cmd.clone()];
            v.extend_from_slice(tail);
            v
        },
        _ => {
            eprintln!(
                "Usage:\n  amble_script compile <file.amble> [--out-triggers <triggers.toml>] [--out-rooms <rooms.toml>] [--out-items <items.toml>] [--out-spinners <spinners.toml>] [--out-npcs <npcs.toml>] [--out-goals <goals.toml>]\n  amble_script compile-dir <src_dir> --out-dir <engine_data_dir> [--only triggers,rooms,items,spinners,npcs,goals]\n  amble_script lint <file.amble|dir> [--data-dir <dir>] [--deny-missing]\n\nNotes:\n- compile-dir overwrites each category file; when a category has no entries, an empty skeleton is written (e.g., 'triggers = []').\n- Use --only to restrict which category files are written; excluded categories are left untouched."
            );
            process::exit(2);
        },
    };
    let cmd = &rest[0];
    if cmd == "compile" {
        run_compile(&rest[1..]);
    } else if cmd == "compile-dir" {
        run_compile_dir(&rest[1..]);
    } else if cmd == "lint" {
        run_lint(&rest[1..]);
    } else {
        eprintln!("unknown command: {cmd}");
        process::exit(2);
    }
}

fn run_compile(args: &[String]) {
    use std::process;
    let mut path: Option<String> = None;
    let mut out_path: Option<String> = None; // triggers
    let mut out_rooms: Option<String> = None; // rooms
    let mut out_spinners: Option<String> = None; // spinners
    let mut out_items: Option<String> = None; // items
    let mut out_npcs: Option<String> = None; // npcs
    let mut out_goals: Option<String> = None; // goals
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--out" {
            if i + 1 >= args.len() {
                eprintln!("--out requires a filepath");
                process::exit(2);
            }
            out_path = Some(args[i + 1].clone());
            eprintln!("warning: --out is deprecated; use --out-triggers instead");
            i += 2;
            continue;
        }
        if args[i] == "--out-triggers" {
            if i + 1 >= args.len() {
                eprintln!("--out-triggers requires a filepath");
                process::exit(2);
            }
            out_path = Some(args[i + 1].clone());
            i += 2;
            continue;
        }
        if args[i] == "--out-rooms" {
            if i + 1 >= args.len() {
                eprintln!("--out-rooms requires a filepath");
                process::exit(2);
            }
            out_rooms = Some(args[i + 1].clone());
            i += 2;
            continue;
        }
        if args[i] == "--out-spinners" {
            if i + 1 >= args.len() {
                eprintln!("--out-spinners requires a filepath");
                process::exit(2);
            }
            out_spinners = Some(args[i + 1].clone());
            i += 2;
            continue;
        }
        if args[i] == "--out-items" {
            if i + 1 >= args.len() {
                eprintln!("--out-items requires a filepath");
                process::exit(2);
            }
            out_items = Some(args[i + 1].clone());
            i += 2;
            continue;
        }
        if args[i] == "--out-npcs" {
            if i + 1 >= args.len() {
                eprintln!("--out-npcs requires a filepath");
                process::exit(2);
            }
            out_npcs = Some(args[i + 1].clone());
            i += 2;
            continue;
        }
        if args[i] == "--out-goals" {
            if i + 1 >= args.len() {
                eprintln!("--out-goals requires a filepath");
                process::exit(2);
            }
            out_goals = Some(args[i + 1].clone());
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
        eprintln!("error: unable to read '{path}': {e}");
        process::exit(1);
    });
    let (triggers, rooms, items, spinners, npcs, goals) = parse_program_full(&src).unwrap_or_else(|e| {
        eprintln!("parse error: {e}");
        process::exit(1);
    });
    for t in &triggers {
        if t.actions.is_empty() {
            eprintln!("warning: trigger '{}' has no actions (empty block?)", t.name);
        }
    }
    // Emit triggers
    if !triggers.is_empty() {
        match compile_triggers_to_toml(&triggers) {
            Ok(toml) => {
                let header = format!(
                    "# Generated by amble_script from {}\n# Do not edit: this file is compiled from DSL.\n# Source Hash (fnv64): {:016x}\n\n",
                    &path,
                    fnv64(&src)
                );
                let toml = format!("{header}{toml}");
                if let Some(out) = out_path.clone() {
                    if let Err(e) = fs::write(&out, &toml) {
                        eprintln!("error: writing '{out}': {e}");
                        process::exit(1);
                    }
                } else if rooms.is_empty()
                    && spinners.is_empty()
                    && items.is_empty()
                    && npcs.is_empty()
                    && goals.is_empty()
                {
                    // Preserve old behavior: print to stdout if only triggers are present
                    println!("{toml}");
                }
            },
            Err(e) => {
                eprintln!("compile error: {e}");
                process::exit(1);
            },
        }
    }
    // Emit items
    if !items.is_empty() {
        match amble_script::compile_items_to_toml(&items) {
            Ok(toml) => {
                let header = format!(
                    "# Generated by amble_script from {}\n# Do not edit: this file is compiled from DSL.\n# Source Hash (fnv64): {:016x}\n\n",
                    &path,
                    fnv64(&src)
                );
                let toml = format!("{header}{toml}");
                if let Some(out) = out_items.clone() {
                    if let Err(e) = fs::write(&out, &toml) {
                        eprintln!("error: writing '{out}': {e}");
                        process::exit(1);
                    }
                } else if triggers.is_empty()
                    && rooms.is_empty()
                    && spinners.is_empty()
                    && npcs.is_empty()
                    && goals.is_empty()
                    && out_path.is_none()
                    && out_rooms.is_none()
                    && out_spinners.is_none()
                {
                    // If only items present and no other outputs, print to stdout
                    println!("{toml}");
                }
            },
            Err(e) => {
                eprintln!("compile error (items): {e}");
                process::exit(1);
            },
        }
    }
    // Emit rooms (Step 2 minimal emission)
    if !rooms.is_empty() {
        match compile_rooms_to_toml(&rooms) {
            Ok(toml) => {
                let header = format!(
                    "# Generated by amble_script from {}\n# Do not edit: this file is compiled from DSL.\n# Source Hash (fnv64): {:016x}\n\n",
                    &path,
                    fnv64(&src)
                );
                let toml = format!("{header}{toml}");
                if let Some(out) = out_rooms.clone() {
                    if let Err(e) = fs::write(&out, &toml) {
                        eprintln!("error: writing '{out}': {e}");
                        process::exit(1);
                    }
                } else if triggers.is_empty()
                    && items.is_empty()
                    && spinners.is_empty()
                    && npcs.is_empty()
                    && goals.is_empty()
                    && out_path.is_none()
                {
                    // If only rooms present and no triggers output path, print rooms to stdout
                    println!("{toml}");
                }
            },
            Err(e) => {
                eprintln!("compile error (rooms): {e}");
                process::exit(1);
            },
        }
    }
    // Emit spinners
    if !spinners.is_empty() {
        match compile_spinners_to_toml(&spinners) {
            Ok(toml) => {
                let header = format!(
                    "# Generated by amble_script from {}\n# Do not edit: this file is compiled from DSL.\n# Source Hash (fnv64): {:016x}\n\n",
                    &path,
                    fnv64(&src)
                );
                let toml = format!("{header}{toml}");
                if let Some(out) = out_spinners.as_ref() {
                    if let Err(e) = fs::write(out, &toml) {
                        eprintln!("error: writing '{out}': {e}");
                        process::exit(1);
                    }
                } else if triggers.is_empty()
                    && rooms.is_empty()
                    && items.is_empty()
                    && npcs.is_empty()
                    && goals.is_empty()
                    && out_path.is_none()
                    && out_rooms.is_none()
                {
                    // If only spinners present and no other outputs, print to stdout
                    println!("{toml}");
                }
            },
            Err(e) => {
                eprintln!("compile error (spinners): {e}");
                process::exit(1);
            },
        }
    }
    // Emit NPCs
    if !npcs.is_empty() {
        match compile_npcs_to_toml(&npcs) {
            Ok(toml) => {
                let header = format!(
                    "# Generated by amble_script from {}\n# Do not edit: this file is compiled from DSL.\n# Source Hash (fnv64): {:016x}\n\n",
                    &path,
                    fnv64(&src)
                );
                let toml = format!("{header}{toml}");
                if let Some(out) = out_npcs.clone() {
                    if let Err(e) = fs::write(&out, &toml) {
                        eprintln!("error: writing '{out}': {e}");
                        process::exit(1);
                    }
                } else if triggers.is_empty()
                    && rooms.is_empty()
                    && spinners.is_empty()
                    && items.is_empty()
                    && goals.is_empty()
                    && out_path.is_none()
                    && out_rooms.is_none()
                    && out_items.is_none()
                {
                    // If only npcs present and no other outputs, print to stdout
                    println!("{toml}");
                }
            },
            Err(e) => {
                eprintln!("compile error (npcs): {e}");
                process::exit(1);
            },
        }
    }
    // Emit goals
    if !goals.is_empty() {
        match compile_goals_to_toml(&goals) {
            Ok(toml) => {
                let header = format!(
                    "# Generated by amble_script from {}\n# Do not edit: this file is compiled from DSL.\n# Source Hash (fnv64): {:016x}\n\n",
                    &path,
                    fnv64(&src)
                );
                let toml = format!("{header}{toml}");
                if let Some(out) = out_goals.clone() {
                    if let Err(e) = fs::write(&out, &toml) {
                        eprintln!("error: writing '{out}': {e}");
                        process::exit(1);
                    }
                } else if triggers.is_empty()
                    && rooms.is_empty()
                    && spinners.is_empty()
                    && items.is_empty()
                    && npcs.is_empty()
                    && out_path.is_none()
                    && out_rooms.is_none()
                    && out_spinners.is_none()
                    && out_items.is_none()
                    && out_npcs.is_none()
                {
                    println!("{toml}");
                }
            },
            Err(e) => {
                eprintln!("compile error (goals): {e}");
                process::exit(1);
            },
        }
    }
}

fn run_compile_dir(args: &[String]) {
    use std::path::Path;
    let mut src_dir: Option<String> = None;
    let mut out_dir: Option<String> = None;
    let mut only: Option<std::collections::HashSet<String>> = None;
    let mut verbose = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--out-dir" => {
                if i + 1 >= args.len() {
                    eprintln!("--out-dir requires a path to amble_engine/data");
                    process::exit(2);
                }
                out_dir = Some(args[i + 1].clone());
                i += 2;
            },
            "--only" => {
                if i + 1 >= args.len() {
                    eprintln!("--only requires a comma-separated list: triggers,rooms,items,spinners,npcs,goals");
                    process::exit(2);
                }
                let list = args[i + 1]
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                only = Some(list);
                i += 2;
            },
            "--verbose" | "-v" => {
                verbose = true;
                i += 1;
            },
            s => {
                if src_dir.is_none() {
                    src_dir = Some(s.to_string());
                }
                i += 1;
            },
        }
    }
    if src_dir.is_none() || out_dir.is_none() {
        eprintln!(
            "Usage: amble_script compile-dir <src_dir> --out-dir <engine_data_dir> [--only triggers,rooms,items,spinners,npcs,goals]\n\nNote: Writes empty skeleton TOMLs for categories with no entries unless filtered by --only."
        );
        process::exit(2);
    }
    let src_dir = src_dir.unwrap();
    let out_dir = out_dir.unwrap();
    // Collect DSL files
    let mut files = Vec::new();
    collect_dsl_files_recursive(&src_dir, &mut files);
    if files.is_empty() {
        eprintln!("compile-dir: no .amble/.able files in '{}'", &src_dir);
        process::exit(1);
    }
    // Aggregate ASTs
    let mut trigs = Vec::new();
    let mut rooms = Vec::new();
    let mut items = Vec::new();
    let mut spinners = Vec::new();
    let mut npcs = Vec::new();
    let mut goals = Vec::new();
    // Build combined source hash (fnv64) using file path + content for determinism
    let mut concat = String::new();
    files.sort();
    let mut total_t = 0usize;
    let mut total_r = 0usize;
    let mut total_i = 0usize;
    let mut total_sp = 0usize;
    let mut total_n = 0usize;
    let mut total_g = 0usize;
    let mut had_error = false;
    for f in &files {
        let src = match fs::read_to_string(f) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("compile-dir: cannot read '{f}': {e}");
                had_error = true;
                continue;
            },
        };
        concat.push_str(f);
        concat.push('\n');
        concat.push_str(&src);
        match parse_program_full(&src) {
            Ok((t, r, it, sp, n, g)) => {
                trigs.extend(t);
                rooms.extend(r);
                items.extend(it);
                spinners.extend(sp);
                npcs.extend(n);
                goals.extend(g);
                if verbose {
                    eprintln!(
                        "{f}: triggers={}, rooms={}, items={}, spinners={}, npcs={}, goals={}",
                        trigs.len(),
                        rooms.len(),
                        items.len(),
                        spinners.len(),
                        npcs.len(),
                        goals.len()
                    );
                }
                total_t = trigs.len();
                total_r = rooms.len();
                total_i = items.len();
                total_sp = spinners.len();
                total_n = npcs.len();
                total_g = goals.len();
            },
            Err(e) => {
                eprintln!("compile-dir: parse error in '{f}': {e}");
                had_error = true;
            },
        }
    }
    if had_error {
        eprintln!("compile-dir: aborting due to previous errors");
        process::exit(1);
    }
    let header = |_kind: &str, src_desc: &str, body: &str| -> String {
        format!(
            "# Generated by amble_script from {} ({} files)\n# Do not edit: this file is compiled from DSL.\n# Source Hash (fnv64): {:016x}\n\n{}",
            src_desc,
            files.len(),
            fnv64(&concat),
            body
        )
    };

    // Ensure out_dir exists
    if !Path::new(&out_dir).exists()
        && let Err(e) = fs::create_dir_all(&out_dir) {
            eprintln!("compile-dir: cannot create out-dir '{out_dir}': {e}");
            process::exit(1);
        }
    // Write each category; emit empty skeletons when no entries to avoid stale cross-file refs
    let allows = |k: &str| -> bool { only.as_ref().map(|s| s.contains(k)).unwrap_or(true) };
    if allows("triggers") {
        let p = format!("{out_dir}/triggers.toml");
        if !trigs.is_empty() {
            match compile_triggers_to_toml(&trigs) {
                Ok(t) => {
                    let text = header("triggers", &src_dir, &t);
                    fs::write(&p, text).unwrap_or_else(|e| {
                        eprintln!("write '{p}': {e}");
                        process::exit(1);
                    });
                },
                Err(e) => {
                    eprintln!("compile-dir error (triggers): {e}");
                    process::exit(1);
                },
            }
        } else {
            let text = header("triggers", &src_dir, "triggers = []\n");
            fs::write(&p, text).unwrap_or_else(|e| {
                eprintln!("write '{p}': {e}");
                process::exit(1);
            });
        }
    }
    if allows("rooms") {
        let p = format!("{out_dir}/rooms.toml");
        if !rooms.is_empty() {
            match compile_rooms_to_toml(&rooms) {
                Ok(t) => {
                    let text = header("rooms", &src_dir, &t);
                    fs::write(&p, text).unwrap_or_else(|e| {
                        eprintln!("write '{p}': {e}");
                        process::exit(1);
                    });
                },
                Err(e) => {
                    eprintln!("compile-dir error (rooms): {e}");
                    process::exit(1);
                },
            }
        } else {
            let text = header("rooms", &src_dir, "rooms = []\n");
            fs::write(&p, text).unwrap_or_else(|e| {
                eprintln!("write '{p}': {e}");
                process::exit(1);
            });
        }
    }
    if allows("items") {
        let p = format!("{out_dir}/items.toml");
        if !items.is_empty() {
            match amble_script::compile_items_to_toml(&items) {
                Ok(t) => {
                    let text = header("items", &src_dir, &t);
                    fs::write(&p, text).unwrap_or_else(|e| {
                        eprintln!("write '{p}': {e}");
                        process::exit(1);
                    });
                },
                Err(e) => {
                    eprintln!("compile-dir error (items): {e}");
                    process::exit(1);
                },
            }
        } else {
            let text = header("items", &src_dir, "items = []\n");
            fs::write(&p, text).unwrap_or_else(|e| {
                eprintln!("write '{p}': {e}");
                process::exit(1);
            });
        }
    }
    if allows("spinners") {
        let p = format!("{out_dir}/spinners.toml");
        if !spinners.is_empty() {
            match compile_spinners_to_toml(&spinners) {
                Ok(t) => {
                    let text = header("spinners", &src_dir, &t);
                    fs::write(&p, text).unwrap_or_else(|e| {
                        eprintln!("write '{p}': {e}");
                        process::exit(1);
                    });
                },
                Err(e) => {
                    eprintln!("compile-dir error (spinners): {e}");
                    process::exit(1);
                },
            }
        } else {
            let text = header("spinners", &src_dir, "spinners = []\n");
            fs::write(&p, text).unwrap_or_else(|e| {
                eprintln!("write '{p}': {e}");
                process::exit(1);
            });
        }
    }
    if allows("npcs") {
        let p = format!("{out_dir}/npcs.toml");
        if !npcs.is_empty() {
            match compile_npcs_to_toml(&npcs) {
                Ok(t) => {
                    let text = header("npcs", &src_dir, &t);
                    fs::write(&p, text).unwrap_or_else(|e| {
                        eprintln!("write '{p}': {e}");
                        process::exit(1);
                    });
                },
                Err(e) => {
                    eprintln!("compile-dir error (npcs): {e}");
                    process::exit(1);
                },
            }
        } else {
            let text = header("npcs", &src_dir, "npcs = []\n");
            fs::write(&p, text).unwrap_or_else(|e| {
                eprintln!("write '{p}': {e}");
                process::exit(1);
            });
        }
    }
    if allows("goals") {
        let p = format!("{out_dir}/goals.toml");
        if !goals.is_empty() {
            match compile_goals_to_toml(&goals) {
                Ok(t) => {
                    let text = header("goals", &src_dir, &t);
                    fs::write(&p, text).unwrap_or_else(|e| {
                        eprintln!("write '{p}': {e}");
                        process::exit(1);
                    });
                },
                Err(e) => {
                    eprintln!("compile-dir error (goals): {e}");
                    process::exit(1);
                },
            }
        } else {
            let text = header("goals", &src_dir, "goals = []\n");
            fs::write(&p, text).unwrap_or_else(|e| {
                eprintln!("write '{p}': {e}");
                process::exit(1);
            });
        }
    }
    if verbose {
        eprintln!(
            "Summary: triggers={total_t}, rooms={total_r}, items={total_i}, spinners={total_sp}, npcs={total_n}, goals={total_g}"
        );
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
                if (ext == "amble" || ext == "able")
                    && let Some(s) = p.to_str() {
                        out.push(s.to_string());
                    }
            }
        }
    }
}

fn lint_one_file(path: &str, world: &WorldRefs) -> usize {
    let src = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("lint: cannot read '{path}': {e}");
            return 0;
        },
    };
    let (asts, rooms_asts, item_asts, spinner_asts, npc_asts, goal_asts) = match parse_program_full(&src) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("lint: parse error in '{path}': {e}");
            return 0;
        },
    };
    let mut refs: HashMap<&'static str, HashSet<String>> = HashMap::new();
    refs.insert("item", HashSet::new());
    refs.insert("room", HashSet::new());
    refs.insert("npc", HashSet::new());
    refs.insert("spinner", HashSet::new());
    refs.insert("flag", HashSet::new());
    // Collect ids defined in this DSL file so references can target them without false positives
    let mut defined_rooms: HashSet<String> = HashSet::new();
    let mut defined_items: HashSet<String> = HashSet::new();
    let mut defined_npcs: HashSet<String> = HashSet::new();
    let mut defined_spinners: HashSet<String> = HashSet::new();
    let mut defined_goals: HashSet<String> = HashSet::new();
    for r in &rooms_asts {
        defined_rooms.insert(r.id.clone());
    }
    for it in &item_asts {
        defined_items.insert(it.id.clone());
    }
    for n in &npc_asts {
        defined_npcs.insert(n.id.clone());
    }
    for sp in &spinner_asts {
        defined_spinners.insert(sp.id.clone());
    }
    for g in &goal_asts {
        defined_goals.insert(g.id.clone());
    }
    for t in &asts {
        gather_refs_from_condition(&t.event, &mut refs);
        for c in &t.conditions {
            gather_refs_from_condition(c, &mut refs);
        }
        for stmt in &t.actions {
            gather_refs_from_action(stmt, &mut refs);
        }
    }
    for r in &rooms_asts {
        gather_refs_from_room(r, &mut refs);
    }
    // Lint NPC dialogue bucket duplicates and movement room references
    if !npc_asts.is_empty() {
        for n in &npc_asts {
            // warn on duplicate dialogue states
            let mut seen_states: HashSet<&str> = HashSet::new();
            for (state_key, _lines) in &n.dialogue {
                if !seen_states.insert(state_key.as_str()) {
                    eprintln!(
                        "lint: warning: NPC '{}' has duplicate dialogue bucket '{}'",
                        n.id, state_key
                    );
                }
            }
        }
    }
    let mut missing = 0usize;
    for id in &refs["item"] {
        if !world.items.contains(id) && !defined_items.contains(id) {
            report_missing_with_location(path, &src, "item", id, &world.items);
            missing += 1;
        }
    }
    for id in &refs["room"] {
        if !world.rooms.contains(id) && !defined_rooms.contains(id) {
            let mut cands = world.rooms.clone();
            cands.extend(defined_rooms.iter().cloned());
            report_missing_with_location(path, &src, "room", id, &cands);
            missing += 1;
        }
    }
    // Lint NPC movement rooms
    if !npc_asts.is_empty() {
        for n in &npc_asts {
            if let Some(mv) = &n.movement {
                for rid in &mv.rooms {
                    if !world.rooms.contains(rid) && !defined_rooms.contains(rid) {
                        let mut cands = world.rooms.clone();
                        cands.extend(defined_rooms.iter().cloned());
                        report_missing_with_location(path, &src, "room", rid, &cands);
                        missing += 1;
                    }
                }
            }
        }
    }
    // Lint goals conditions
    for g in &goal_asts {
        let check = |cond: &GoalCondAst, missing: &mut usize| {
            match cond {
                GoalCondAst::HasFlag(f)
                | GoalCondAst::MissingFlag(f)
                | GoalCondAst::FlagInProgress(f)
                | GoalCondAst::FlagComplete(f) => {
                    // Skip empty sentinel used by parser for missing "start when" (activate_when)
                    if f.trim().is_empty() {
                        return;
                    }
                    let base = f.split('#').next().unwrap_or(f);
                    if !world.flags.contains(base) {
                        report_missing_with_location(path, &src, "flag", f, &world.flags);
                        *missing += 1;
                    }
                },
                GoalCondAst::HasItem(i) => {
                    if !world.items.contains(i) {
                        report_missing_with_location(path, &src, "item", i, &world.items);
                        *missing += 1;
                    }
                },
                GoalCondAst::ReachedRoom(r) => {
                    if !world.rooms.contains(r) {
                        report_missing_with_location(path, &src, "room", r, &world.rooms);
                        *missing += 1;
                    }
                },
                GoalCondAst::GoalComplete(id) => {
                    if !world.goals.contains(id) && !defined_goals.contains(id) {
                        // Suggest from both world and locally-defined goal ids for better hints
                        let mut cands = world.goals.clone();
                        cands.extend(defined_goals.iter().cloned());
                        report_missing_with_location(path, &src, "goal", id, &cands);
                        *missing += 1;
                    }
                },
            }
        };
        check(&g.finished_when, &mut missing);
        if let Some(cond) = &g.activate_when {
            check(cond, &mut missing);
        }
        if let Some(cond) = &g.failed_when {
            check(cond, &mut missing);
        }
    }
    for id in &refs["npc"] {
        if !world.npcs.contains(id) && !defined_npcs.contains(id) {
            report_missing_with_location(path, &src, "npc", id, &world.npcs);
            missing += 1;
        }
    }
    for id in &refs["spinner"] {
        if !world.spinners.contains(id) && !defined_spinners.contains(id) {
            report_missing_with_location(path, &src, "spinner", id, &world.spinners);
            missing += 1;
        }
    }
    for id in &refs["flag"] {
        let base = id.split('#').next().unwrap_or(id).to_string();
        if !world.flags.contains(&base) {
            report_missing_with_location(path, &src, "flag", id, &world.flags);
            missing += 1;
        }
    }
    missing
}

fn report_missing_with_location(
    path: &str,
    src: &str,
    kind: &str,
    id: &str,
    candidates: &std::collections::HashSet<String>,
) {
    if let Some((line_no, col, line)) = find_position_for_id(src, kind, id) {
        let suggestions = suggest_ids(id, candidates);
        if suggestions.is_empty() {
            eprintln!(
                "{}:{}:{}: unknown {} '{}'\n{}\n{}^",
                path,
                line_no,
                col,
                kind,
                id,
                line,
                " ".repeat(col.saturating_sub(1))
            );
        } else {
            eprintln!(
                "{}:{}:{}: unknown {} '{}' (did you mean: {}?)\n{}\n{}^",
                path,
                line_no,
                col,
                kind,
                id,
                suggestions.join(", "),
                line,
                " ".repeat(col.saturating_sub(1))
            );
        }
    } else {
        let suggestions = suggest_ids(id, candidates);
        if suggestions.is_empty() {
            eprintln!("{path}: unknown {kind} '{id}'");
        } else {
            eprintln!(
                "{path}: unknown {kind} '{id}' (did you mean: {}?)",
                suggestions.join(", ")
            );
        }
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
        "goal" => vec![format!(" goal complete {}", id)],
        "flag" => vec![
            format!(" flag {}", id),
            format!(" missing flag {}", id),
            format!(" has flag {}", id),
            format!(" flag in progress {}", id),
            format!(" flag complete {}", id),
            format!(" required_flags({}", id),
        ],
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

fn suggest_ids(target: &str, candidates: &std::collections::HashSet<String>) -> Vec<String> {
    let mut scored: Vec<(usize, String)> = Vec::new();
    for c in candidates {
        if c == target {
            continue;
        }
        if c.starts_with(target) || c.contains(target) || target.starts_with(c) {
            scored.push((0, c.clone()));
            continue;
        }
        let d = edit_distance_bounded(target, c, 3);
        if d <= 2 {
            scored.push((d, c.clone()));
        }
    }
    scored.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    scored.into_iter().take(3).map(|(_, s)| s).collect()
}

fn edit_distance_bounded(a: &str, b: &str, _max: usize) -> usize {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let (n, m) = (a.len(), b.len());
    if n == 0 {
        return m;
    }
    if m == 0 {
        return n;
    }
    let mut prev: Vec<usize> = (0..=m).collect();
    let mut cur = vec![0usize; m + 1];
    for i in 1..=n {
        cur[0] = i;
        let ac = a[i - 1];
        for j in 1..=m {
            let cost = if ac == b[j - 1] { 0 } else { 1 };
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[m]
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
        | ConditionAst::TouchItem(i)
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
        ConditionAst::Ingest { item, .. } => {
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
        ConditionAst::NpcDeath(npc) => {
            out.get_mut("npc").unwrap().insert(npc.clone());
        },
        ConditionAst::PlayerDeath => { /* no references */ },
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
        ConditionAst::MissingFlag(f)
        | ConditionAst::HasFlag(f)
        | ConditionAst::FlagInProgress(f)
        | ConditionAst::FlagComplete(f) => {
            out.get_mut("flag").unwrap().insert(f.clone());
        },
    }
}

fn gather_refs_from_action(stmt: &ActionStmt, out: &mut HashMap<&'static str, HashSet<String>>) {
    match &stmt.action {
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
        | ActionAst::SetItemMovability { item: i, .. } => {
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
        ActionAst::SpawnNpcIntoRoom { npc, room } => {
            out.get_mut("npc").unwrap().insert(npc.clone());
            out.get_mut("room").unwrap().insert(room.clone());
        },
        ActionAst::SetItemDescription { item, .. } => {
            out.get_mut("item").unwrap().insert(item.clone());
        },
        ActionAst::NpcSays { npc, .. }
        | ActionAst::DespawnNpc(npc)
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
        ActionAst::ResetFlag(f) | ActionAst::AdvanceFlag(f) | ActionAst::RemoveFlag(f) => {
            out.get_mut("flag").unwrap().insert(f.clone());
        },
        _ => {},
    }
}

fn gather_refs_from_room(r: &amble_script::RoomAst, out: &mut HashMap<&'static str, HashSet<String>>) {
    // Exits: target rooms
    for (_, ex) in &r.exits {
        out.get_mut("room").unwrap().insert(ex.to.clone());
        // required_items are item ids (string symbols)
        for it in &ex.required_items {
            out.get_mut("item").unwrap().insert(it.clone());
        }
        for fl in &ex.required_flags {
            out.get_mut("flag").unwrap().insert(fl.clone());
        }
    }
    // Overlays: collect referenced items/npcs/rooms
    for ov in &r.overlays {
        for c in &ov.conditions {
            use amble_script::OverlayCondAst as O;
            match c {
                O::ItemPresent(i) | O::ItemAbsent(i) | O::PlayerHasItem(i) | O::PlayerMissingItem(i) => {
                    out.get_mut("item").unwrap().insert(i.clone());
                },
                O::NpcPresent(n) | O::NpcAbsent(n) | O::NpcInState { npc: n, .. } => {
                    out.get_mut("npc").unwrap().insert(n.clone());
                },
                O::ItemInRoom { item, room } => {
                    out.get_mut("item").unwrap().insert(item.clone());
                    out.get_mut("room").unwrap().insert(room.clone());
                },
                O::FlagSet(f) | O::FlagUnset(f) | O::FlagComplete(f) => {
                    out.get_mut("flag").unwrap().insert(f.clone());
                },
            }
        }
    }
}

struct WorldRefs {
    items: HashSet<String>,
    rooms: HashSet<String>,
    npcs: HashSet<String>,
    spinners: HashSet<String>,
    flags: HashSet<String>,
    goals: HashSet<String>,
}

fn load_world_refs(dir: &str) -> Result<WorldRefs, String> {
    let mut items = HashSet::new();
    let mut rooms = HashSet::new();
    let mut npcs = HashSet::new();
    let mut spinners = HashSet::new();
    let mut flags = HashSet::new();
    let mut goals = HashSet::new();

    fn load_ids(doc: &Document, key: &str, field: &str) -> HashSet<String> {
        let mut set = HashSet::new();
        if let Some(item) = doc.as_table().get(key)
            && let Some(aot) = item.as_array_of_tables()
        {
            for t in aot.iter() {
                if let Some(v) = t.get(field)
                    && let Some(s) = v.as_str()
                {
                    set.insert(s.to_string());
                }
            }
        }

        set
    }

    let items_path = format!("{dir}/items.toml");
    let rooms_path = format!("{dir}/rooms.toml");
    let npcs_path = format!("{dir}/npcs.toml");
    let spinners_path = format!("{dir}/spinners.toml");
    let goals_path = format!("{dir}/goals.toml");
    let triggers_path = format!("{dir}/triggers.toml");

    let read = |p: &str| -> Result<Document, String> {
        let s = fs::read_to_string(p).map_err(|e| format!("{e}"))?;
        s.parse::<Document>().map_err(|e| format!("{e}"))
    };

    if let Ok(doc) = read(&items_path) {
        items = load_ids(&doc, "items", "id");
    }
    if let Ok(doc) = read(&rooms_path) {
        if let Ok(raw) = fs::read_to_string(&rooms_path) {
            warn_if_stale_generated(&raw, dir, "rooms");
        }
        rooms = load_ids(&doc, "rooms", "id");
    }
    if let Ok(doc) = read(&npcs_path) {
        npcs = load_ids(&doc, "npcs", "id");
    }
    if let Ok(doc) = read(&spinners_path) {
        spinners = load_ids(&doc, "spinners", "spinnerType");
    }
    if let Ok(doc) = read(&goals_path) {
        goals = load_ids(&doc, "goals", "id");
    }
    if let Ok(doc) = read(&triggers_path) {
        if let Ok(raw) = fs::read_to_string(&triggers_path) {
            // Try to extract source path + hash; if stale, prefer registry from DSL
            if let Some((src_path, want_hash)) = parse_generated_header(&raw) {
                let src_abs = if std::path::Path::new(&src_path).is_absolute() {
                    src_path.clone()
                } else {
                    let base = std::path::Path::new(dir)
                        .parent()
                        .and_then(|p| p.parent())
                        .unwrap_or(std::path::Path::new("."));
                    base.join(&src_path).to_string_lossy().to_string()
                };
                if let Ok(src_text) = fs::read_to_string(&src_abs) {
                    let have = format!("{:016x}", fnv64(&src_text));
                    if have != want_hash {
                        eprintln!(
                            "lint: warning: triggers.toml hash mismatch vs '{src_abs}'; TOML may be stale (expected {have}, found {want_hash})"
                        );
                        flags = flags_from_triggers_dsl(&src_text);
                    } else {
                        flags = load_flags_from_triggers(&doc);
                    }
                } else {
                    // Fall back to TOML if source not found
                    flags = load_flags_from_triggers(&doc);
                }
            } else {
                // No header; fall back to TOML
                flags = load_flags_from_triggers(&doc);
            }
        } else {
            flags = load_flags_from_triggers(&doc);
        }
    }

    Ok(WorldRefs {
        items,
        rooms,
        npcs,
        spinners,
        flags,
        goals,
    })
}

fn load_flags_from_triggers(doc: &Document) -> HashSet<String> {
    let mut set = HashSet::new();
    if let Some(item) = doc.as_table().get("triggers")
        && let Some(aot) = item.as_array_of_tables()
    {
        for t in aot.iter() {
            if let Some(actions) = t.get("actions").and_then(|a| a.as_array()) {
                collect_flags_from_actions(actions, &mut set);
            }
        }
    }

    set
}

fn collect_flags_from_actions(actions: &toml_edit::Array, out: &mut HashSet<String>) {
    for act in actions.iter() {
        if let Some(at) = act.as_inline_table()
            && let Some(ty) = at.get("type").and_then(|v| v.as_str()) {
                match ty {
                    "addFlag" => {
                        if let Some(flag_item) = at.get("flag")
                            && let Some(ftab) = flag_item.as_inline_table()
                            && let Some(name) = ftab.get("name").and_then(|v| v.as_str())
                        {
                            out.insert(name.to_string());
                        }
                    },
                    // Recurse into scheduled actions
                    "scheduleIn" | "scheduleOn" | "scheduleInIf" | "scheduleOnIf" => {
                        if let Some(sub) = at.get("actions").and_then(|a| a.as_array()) {
                            collect_flags_from_actions(sub, out);
                        }
                    },
                    _ => {},
                }
            }
    }
}

fn warn_if_stale_generated(toml_text: &str, data_dir: &str, kind: &str) {
    // Expect header like:
    // # Generated by amble_script from <path>
    // # Do not edit...
    // # Source Hash (fnv64): <hex>
    if let Some((path, hh)) = parse_generated_header(toml_text) {
        // Resolve relative to repo root; allow both absolute and relative
        let src = if std::path::Path::new(&path).is_absolute() {
            path
        } else {
            // data_dir is usually amble_engine/data; go up two to repo root
            let base = std::path::Path::new(data_dir)
                .parent()
                .and_then(|p| p.parent())
                .unwrap_or(std::path::Path::new("."));
            base.join(path).to_string_lossy().to_string()
        };
        if let Ok(s) = fs::read_to_string(&src) {
            let h = format!("{:016x}", fnv64(&s));
            if h != hh {
                eprintln!(
                    "lint: warning: {kind}.toml hash mismatch vs '{src}'; TOML may be stale (expected {h}, found {hh})"
                );
            }
        }
    }
}

fn parse_generated_header(toml_text: &str) -> Option<(String, String)> {
    let mut src_path: Option<String> = None;
    let mut hash_hex: Option<String> = None;
    for line in toml_text.lines().take(4) {
        if let Some(rest) = line.strip_prefix("# Generated by amble_script from ") {
            src_path = Some(rest.trim().to_string());
        }
        if let Some(rest) = line.strip_prefix("# Source Hash (fnv64): ") {
            hash_hex = Some(rest.trim().to_string());
        }
    }
    match (src_path, hash_hex) {
        (Some(p), Some(h)) => Some((p, h)),
        _ => None,
    }
}

fn fnv64(s: &str) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x00000100000001B3;
    let mut hash = FNV_OFFSET;
    for &b in s.as_bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn flags_from_triggers_dsl(src: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    if let Ok((trigs, _rooms, _items, _spinners, _npcs, _goals)) = parse_program_full(src) {
        for t in trigs {
            collect_flags_from_actions_ast(&t.actions, &mut out);
        }
    }
    out
}

fn collect_flags_from_actions_ast(actions: &[ActionStmt], out: &mut HashSet<String>) {
    for stmt in actions {
        match &stmt.action {
            ActionAst::AddFlag(name) => {
                out.insert(name.clone());
            },
            ActionAst::AddSeqFlag { name, .. } => {
                out.insert(name.clone());
            },
            ActionAst::ScheduleIn { actions, .. }
            | ActionAst::ScheduleOn { actions, .. }
            | ActionAst::ScheduleInIf { actions, .. }
            | ActionAst::ScheduleOnIf { actions, .. } => {
                collect_flags_from_actions_ast(actions, out);
            },
            _ => {},
        }
    }
}
