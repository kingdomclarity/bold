// bld, the CLI for BOLD, the in-house programming language of BOLD Ventures.
// Built in Rust. One binary, no runtime dependencies.
//
//   bld run <file>.bold     lex, parse, type-check, execute
//   bld check <file>.bold   lex, parse, type-check only
//   bld fmt <file>.bold     normalize indentation in place
//
// Designed and Developed by BOLD Studios.

mod codegen;
mod lexer;
mod parser;
mod typechecker;

use std::fs;
use std::process::exit;
use std::time::Instant;

const GOLD: &str = "\x1b[38;5;179m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        usage();
        exit(1);
    }
    let cmd = args[1].as_str();
    let file = args[2].as_str();

    if !file.ends_with(".bold") {
        eprintln!("{}error{} BOLD source files end in .bold", RED, RESET);
        exit(1);
    }

    let source = match fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}error{} cannot read {}: {}", RED, RESET, file, e);
            exit(1);
        }
    };

    match cmd {
        "run" => {
            let prog = compile(&source, file);
            let t0 = Instant::now();
            if let Err((line, msg)) = codegen::Runtime::run(&prog) {
                diagnostic(file, &source, line, &msg, "runtime error");
                exit(1);
            }
            eprintln!(
                "{}{} finished in {:?}{}",
                DIM,
                file,
                t0.elapsed(),
                RESET
            );
        }
        "check" => {
            let t0 = Instant::now();
            let prog = compile(&source, file);
            println!(
                "{}ok{}    {} compiles clean ({} function(s), {} struct(s)) in {:?}",
                GREEN,
                RESET,
                file,
                prog.fns.len(),
                prog.structs.len(),
                t0.elapsed()
            );
        }
        "fmt" => {
            let formatted = format_source(&source);
            if formatted != source {
                if let Err(e) = fs::write(file, &formatted) {
                    eprintln!("{}error{} cannot write {}: {}", RED, RESET, file, e);
                    exit(1);
                }
                println!("{}ok{}    formatted {}", GREEN, RESET, file);
            } else {
                println!("{}ok{}    {} already formatted", GREEN, RESET, file);
            }
        }
        other => {
            eprintln!("{}error{} unknown command: {}", RED, RESET, other);
            usage();
            exit(1);
        }
    }
}

fn compile(source: &str, file: &str) -> parser::Program {
    let toks = match lexer::lex(source) {
        Ok(t) => t,
        Err((line, msg)) => {
            diagnostic(file, source, line, &msg, "syntax error");
            exit(1);
        }
    };
    let mut p = parser::Parser::new(toks);
    let prog = match p.parse_program() {
        Ok(p) => p,
        Err((line, msg)) => {
            diagnostic(file, source, line, &msg, "parse error");
            exit(1);
        }
    };
    if let Err((line, msg)) = typechecker::Checker::check(&prog) {
        diagnostic(file, source, line, &msg, "type error");
        exit(1);
    }
    prog
}

fn diagnostic(file: &str, source: &str, line: usize, msg: &str, kind: &str) {
    eprintln!("{}{}{} [{}:{}] {}", RED, kind, RESET, file, line, msg);
    if line >= 1 {
        if let Some(src_line) = source.lines().nth(line - 1) {
            eprintln!("  {}{} |{} {}", GOLD, line, RESET, src_line.trim_end());
        }
    }
}

fn format_source(source: &str) -> String {
    // v1 formatter: normalize indentation to 4 spaces per brace depth.
    let mut out = String::new();
    let mut depth: usize = 0;
    for raw in source.lines() {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            out.push('\n');
            continue;
        }
        let opens = braces(trimmed, '{');
        let closes = braces(trimmed, '}');
        let this_depth = if trimmed.starts_with('}') {
            depth.saturating_sub(1)
        } else {
            depth
        };
        out.push_str(&"    ".repeat(this_depth));
        out.push_str(trimmed);
        out.push('\n');
        depth = (depth + opens).saturating_sub(closes);
    }
    out
}

fn braces(line: &str, which: char) -> usize {
    let mut count = 0;
    let mut in_str = false;
    let mut in_comment = false;
    let chars: Vec<char> = line.chars().collect();
    for i in 0..chars.len() {
        let c = chars[i];
        if in_comment {
            break;
        }
        if c == '"' {
            in_str = !in_str;
        }
        if !in_str && c == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            in_comment = true;
            continue;
        }
        if !in_str && c == which {
            count += 1;
        }
    }
    count
}

fn usage() {
    eprintln!("{}bld{} 1.0.0, the CLI for the BOLD programming language", GOLD, RESET);
    eprintln!("Usage:");
    eprintln!("  bld run <file>.bold     compile and execute");
    eprintln!("  bld check <file>.bold   compile only, report diagnostics");
    eprintln!("  bld fmt <file>.bold     format source in place");
}
