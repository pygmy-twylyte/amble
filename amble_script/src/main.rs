//! CLI entry point for amble_script.
//! Usage: cargo run -p amble_script -- compile examples/first.able

use std::{env, fs, process};

use amble_script::{compile_triggers_to_toml, parse_program};

fn main() {
    let args: Vec<String> = env::args().collect();

    // Accept either:
    // 1) cargo run: <bin> -- compile <path>
    // 2) direct:    <bin> compile <path>
    // Extract subcommand and collect the rest for flags/positional
    let rest: Vec<String> = match args.as_slice() {
        [_, flag, cmd, tail @ ..] if flag == "--" && cmd == "compile" => tail.to_vec(),
        [_, cmd, tail @ ..] if cmd == "compile" => tail.to_vec(),
        _ => {
            eprintln!("Usage: amble_script compile <path/to/file.able> [--out <out.toml>]");
            process::exit(2);
        }
    };
    if rest.is_empty() {
        eprintln!("Usage: amble_script compile <path/to/file.able> [--out <out.toml>]");
        process::exit(2);
    }
    let mut path: Option<String> = None;
    let mut out_path: Option<String> = None;
    let mut i = 0;
    while i < rest.len() {
        if rest[i] == "--out" {
            if i + 1 >= rest.len() {
                eprintln!("--out requires a filepath");
                process::exit(2);
            }
            out_path = Some(rest[i + 1].clone());
            i += 2;
        } else {
            if path.is_none() {
                path = Some(rest[i].clone());
            }
            i += 1;
        }
    }
    let path = path.expect("input path");

    let src = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: unable to read '{}': {}", &path, e);
            process::exit(1);
        }
    };

    let asts = match parse_program(&src) {
        Ok(asts) => asts,
        Err(e) => {
            eprintln!("parse error: {}", e);
            process::exit(1);
        }
    };

    match compile_triggers_to_toml(&asts) {
        Ok(toml) => {
            if let Some(out) = out_path {
                if let Err(e) = fs::write(&out, toml) {
                    eprintln!("error: writing '{}': {}", &out, e);
                    process::exit(1);
                }
            } else {
                println!("{}", toml);
            }
        }
        Err(e) => {
            eprintln!("compile error: {}", e);
            process::exit(1);
        }
    }
}
