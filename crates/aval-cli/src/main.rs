//! The `aval` command line tool.
#![forbid(unsafe_code)]

use aval_check::{check_source, lock, Checked};
use aval_eval::{run_tests, Options};
use aval_syntax::diag::{has_errors, sort_diags, Diag};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// The compact language reference, compiled into the binary.
pub const SKILL: &str = include_str!("../../../docs/skill/aval-skill.md");

const USAGE: &str = "usage:
  aval check FILE                         lex, parse, resolve, type-check, spec rules, spec lock
  aval test FILE [--cases N] [--seed S] [--max-steps N]
                                          examples, property tests, oracle (runs check first)
  aval spec lock FILE                     write aval.lock next to FILE (human command)
  aval skill print                        print the language reference for agents
  aval --version";

fn usage_error(msg: &str) -> ExitCode {
    eprintln!("aval: {}\n{}", msg, USAGE);
    ExitCode::from(2)
}

fn print_diags(diags: &[Diag]) {
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    for d in diags {
        let _ = writeln!(out, "{}", d.to_line());
    }
}

fn read_source(path: &str) -> Result<String, ExitCode> {
    match std::fs::read(path) {
        Ok(bytes) => String::from_utf8(bytes).map_err(|_| {
            eprintln!("aval: {} is not valid UTF-8", path);
            ExitCode::from(2)
        }),
        Err(e) => {
            eprintln!("aval: cannot read {}: {}", path, e);
            Err(ExitCode::from(2))
        }
    }
}

fn file_name(file: &str) -> String {
    Path::new(file)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| file.to_string())
}

fn lock_dir(file: &str) -> PathBuf {
    match Path::new(file).parent() {
        Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
        _ => PathBuf::from("."),
    }
}

fn lock_path(file: &str) -> PathBuf {
    match Path::new(file).parent() {
        Some(p) if !p.as_os_str().is_empty() => p.join("aval.lock"),
        _ => PathBuf::from("aval.lock"),
    }
}

/// Stages 1-5: everything `aval check` does.
fn full_check(file: &str, src: &str) -> Checked {
    let mut c = check_source(src);
    if !c.syntax_error {
        if let Ok(text) = std::fs::read_to_string(lock_path(file)) {
            c.diags.extend(lock::check(
                &c.module,
                &text,
                &file_name(file),
                &|name: &str| lock_dir(file).join(name).is_file(),
            ));
            sort_diags(&mut c.diags);
        }
    }
    c
}

fn cmd_check(args: &[String]) -> ExitCode {
    let [file] = args else {
        return usage_error("check takes exactly one FILE");
    };
    let src = match read_source(file) {
        Ok(s) => s,
        Err(c) => return c,
    };
    let c = full_check(file, &src);
    print_diags(&c.diags);
    exit_for(&c.diags)
}

fn exit_for(diags: &[Diag]) -> ExitCode {
    if has_errors(diags) {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn cmd_test(args: &[String]) -> ExitCode {
    let mut file: Option<&String> = None;
    let mut opts = Options::default();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--cases" | "--seed" | "--max-steps" => {
                let Some(v) = args.get(i + 1).and_then(|v| v.parse::<u64>().ok()) else {
                    return usage_error(&format!("{} needs a non-negative integer", args[i]));
                };
                if args[i] == "--cases" {
                    opts.cases = v as usize;
                } else if args[i] == "--max-steps" {
                    opts.max_steps = v;
                } else {
                    opts.seed = v;
                }
                i += 2;
            }
            a if a.starts_with("--") => return usage_error(&format!("unknown option {}", a)),
            _ => {
                if file.is_some() {
                    return usage_error("test takes exactly one FILE");
                }
                file = Some(&args[i]);
                i += 1;
            }
        }
    }
    let Some(file) = file else {
        return usage_error("test needs a FILE");
    };
    let src = match read_source(file) {
        Ok(s) => s,
        Err(c) => return c,
    };
    let c = full_check(file, &src);
    print_diags(&c.diags);
    if has_errors(&c.diags) {
        return ExitCode::from(1);
    }
    let Some(info) = &c.info else {
        return ExitCode::from(1);
    };
    let mut results = run_tests(&c.module, info, &opts);
    sort_diags(&mut results);
    print_diags(&results);
    exit_for(&results)
}

fn cmd_spec(args: &[String]) -> ExitCode {
    match args {
        [sub, file] if sub == "lock" => {
            let src = match read_source(file) {
                Ok(s) => s,
                Err(c) => return c,
            };
            let c = check_source(&src);
            if has_errors(&c.diags) {
                print_diags(&c.diags);
                return ExitCode::from(1);
            }
            let path = lock_path(file);
            let existing = std::fs::read_to_string(&path).ok();
            let text = lock::render(existing.as_deref(), &c.module, &file_name(file));
            if let Err(e) = std::fs::write(&path, text) {
                eprintln!("aval: cannot write {}: {}", path.display(), e);
                return ExitCode::from(2);
            }
            eprintln!(
                "aval: wrote {} ({} pub functions)",
                path.display(),
                lock::entries(&c.module, &file_name(file)).len()
            );
            ExitCode::SUCCESS
        }
        _ => usage_error("usage: aval spec lock FILE"),
    }
}

fn run() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("--version") | Some("-V") => {
            println!("aval {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Some("check") => cmd_check(&args[1..]),
        Some("test") => cmd_test(&args[1..]),
        Some("spec") => cmd_spec(&args[1..]),
        Some("skill") => match args.get(1).map(String::as_str) {
            Some("print") if args.len() == 2 => {
                print!("{}", SKILL);
                ExitCode::SUCCESS
            }
            _ => usage_error("usage: aval skill print"),
        },
        Some(other) => usage_error(&format!("unknown command `{}`", other)),
        None => usage_error("missing command"),
    }
}

fn main() -> ExitCode {
    // Deep recursion in user programs is bounded by the interpreter; give it room.
    let handle = std::thread::Builder::new()
        .stack_size(512 * 1024 * 1024)
        .spawn(run);
    match handle {
        Ok(h) => h.join().unwrap_or_else(|_| {
            eprintln!("aval: internal error");
            ExitCode::from(2)
        }),
        Err(_) => run(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aval_syntax::lexer::KEYWORDS;

    fn collect_codes(dir: &Path, out: &mut Vec<String>) {
        for entry in std::fs::read_dir(dir).unwrap().flatten() {
            let p = entry.path();
            if p.is_dir() {
                collect_codes(&p, out);
            } else if p.extension().map(|e| e == "rs").unwrap_or(false) {
                let text = std::fs::read_to_string(&p).unwrap();
                let b = text.as_bytes();
                for i in 0..b.len().saturating_sub(6) {
                    if b[i] == b'"'
                        && matches!(b[i + 1], b'E' | b'T' | b'W')
                        && b[i + 2..i + 5].iter().all(u8::is_ascii_digit)
                        && b[i + 5] == b'"'
                    {
                        out.push(text[i + 1..i + 5].to_string());
                    }
                }
            }
        }
    }

    #[test]
    fn skill_lists_every_emitted_code_and_all_keywords() {
        let mut codes = Vec::new();
        collect_codes(
            &Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .canonicalize()
                .unwrap(),
            &mut codes,
        );
        codes.sort();
        codes.dedup();
        assert!(codes.len() >= 20, "found only {:?}", codes);
        for c in &codes {
            assert!(
                SKILL.contains(c.as_str()),
                "skill misses diagnostic code {}",
                c
            );
        }
        let words: Vec<&str> = SKILL
            .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
            .collect();
        for k in KEYWORDS {
            assert!(words.contains(&k), "skill misses keyword {}", k);
        }
    }

    #[test]
    fn skill_code_blocks_are_valid_aval() {
        let mut blocks = Vec::new();
        let mut cur: Option<String> = None;
        for line in SKILL.lines() {
            match (&mut cur, line.trim_end()) {
                (None, "```aval") => cur = Some(String::new()),
                (Some(_), "```") => blocks.push(cur.take().unwrap()),
                (Some(b), l) => {
                    b.push_str(l);
                    b.push('\n');
                }
                _ => {}
            }
        }
        assert!(!blocks.is_empty());
        for b in blocks {
            let c = check_source(&b);
            assert!(!c.has_errors(), "{:?}\n{}", c.diags, b);
        }
    }
}
