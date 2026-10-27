//! Conformance runner: executes the real `aval` binary over `tests/conformance/**` as described
//! in `tests/conformance/README.md`. Set `AVAL_CONFORMANCE_FILTER=<substring>` to select tests.

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Default, Debug)]
struct Header {
    scenario: Vec<String>,
    cmd: Option<String>,
    exit: Option<i32>,
    expect: Vec<Value>,
    stdout_contains: Vec<String>,
    stderr_contains: Vec<String>,
    no_file: bool,
    skip_output_count: bool,
    lock_none: bool,
    rename: Option<String>,
}

fn parse_header(text: &str) -> Result<Header, String> {
    let mut h = Header::default();
    for line in text.lines() {
        let Some(rest) = line.strip_prefix('#') else {
            if line.trim().is_empty() {
                continue;
            }
            break;
        };
        let rest = rest.trim();
        let Some((key, value)) = rest.split_once(':') else {
            continue;
        };
        let (key, value) = (key.trim(), value.trim());
        match key {
            "scenario" => h.scenario = value.split_whitespace().map(String::from).collect(),
            "cmd" => h.cmd = Some(value.to_string()),
            "exit" => h.exit = Some(value.parse().map_err(|_| format!("bad exit: {}", value))?),
            "expect" => h.expect.push(
                serde_json::from_str(value)
                    .map_err(|e| format!("bad expect JSON `{}`: {}", value, e))?,
            ),
            "expect-codes" => {
                for c in value.split_whitespace() {
                    h.expect.push(serde_json::json!({ "code": c }));
                }
            }
            "stdout-contains" => h.stdout_contains.push(value.to_string()),
            "stderr-contains" => h.stderr_contains.push(value.to_string()),
            "no-file" => h.no_file = value.is_empty() || value == "true",
            "skip-output-count" => h.skip_output_count = value.is_empty() || value == "true",
            "lock" => h.lock_none = value == "none",
            "rename" => h.rename = Some(value.to_string()),
            _ => {}
        }
    }
    Ok(h)
}

fn json_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x.as_f64() == y.as_f64(),
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(p, q)| json_eq(p, q))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len()
                && x.iter()
                    .all(|(k, v)| y.get(k).map(|w| json_eq(v, w)).unwrap_or(false))
        }
        _ => a == b,
    }
}

/// Every key of `expected` is present in `actual` with an equal value.
fn is_subset(expected: &Value, actual: &Value) -> bool {
    match (expected, actual) {
        (Value::Object(e), Value::Object(a)) => e
            .iter()
            .all(|(k, v)| a.get(k).map(|w| json_eq(v, w)).unwrap_or(false)),
        _ => json_eq(expected, actual),
    }
}

fn expects_error(h: &Header) -> bool {
    h.expect.iter().any(|e| {
        e.get("code")
            .and_then(Value::as_str)
            .map(|c| c.starts_with('E') || c.starts_with('T'))
            .unwrap_or(false)
    })
}

fn build_args(h: &Header, file: &str) -> Vec<String> {
    let cmd = h.cmd.clone().unwrap_or_else(|| "check".to_string());
    let mut toks: Vec<String> = cmd.split_whitespace().map(String::from).collect();
    let has_file = toks.iter().any(|t| t.contains("$FILE"));
    if !has_file
        && !h.no_file
        && matches!(
            toks.first().map(String::as_str),
            Some("check") | Some("test")
        )
    {
        toks.insert(1, "$FILE".to_string());
    }
    toks.iter().map(|t| t.replace("$FILE", file)).collect()
}

struct Output {
    code: i32,
    stdout: String,
    stderr: String,
}

fn run_aval(args: &[String], cwd: Option<&Path>) -> Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_aval"));
    c.args(args);
    if let Some(d) = cwd {
        c.current_dir(d);
    }
    let out = c.output().expect("cannot run aval");
    Output {
        code: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

fn verify(h: &Header, out: &Output) -> Result<(), String> {
    let want_exit = h.exit.unwrap_or(if expects_error(h) { 1 } else { 0 });
    if out.code != want_exit {
        return Err(format!(
            "exit code {} (expected {})\nstdout:\n{}\nstderr:\n{}",
            out.code, want_exit, out.stdout, out.stderr
        ));
    }
    let lines: Vec<&str> = out
        .stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .collect();
    if !h.expect.is_empty() {
        let parsed: Vec<Option<Value>> = lines
            .iter()
            .map(|l| serde_json::from_str::<Value>(l).ok())
            .collect();
        if h.skip_output_count {
            let objs: Vec<&Value> = parsed.iter().flatten().filter(|v| v.is_object()).collect();
            let mut from = 0;
            for e in &h.expect {
                match objs[from..].iter().position(|a| is_subset(e, a)) {
                    Some(p) => from += p + 1,
                    None => {
                        return Err(format!(
                            "no output line matches {}\nstdout:\n{}",
                            e, out.stdout
                        ))
                    }
                }
            }
        } else {
            if lines.len() != h.expect.len() {
                return Err(format!(
                    "expected {} output line(s), got {}\nstdout:\n{}",
                    h.expect.len(),
                    lines.len(),
                    out.stdout
                ));
            }
            for (i, e) in h.expect.iter().enumerate() {
                match &parsed[i] {
                    Some(a) if is_subset(e, a) => {}
                    _ => {
                        return Err(format!(
                            "line {} `{}` does not contain {}",
                            i + 1,
                            lines[i],
                            e
                        ))
                    }
                }
            }
        }
    } else if h.stdout_contains.is_empty() && !h.skip_output_count && !lines.is_empty() {
        return Err(format!("expected empty stdout, got:\n{}", out.stdout));
    }
    for s in &h.stdout_contains {
        if !out.stdout.contains(s.as_str()) {
            return Err(format!("stdout lacks `{}`\nstdout:\n{}", s, out.stdout));
        }
    }
    for s in &h.stderr_contains {
        if !out.stderr.contains(s.as_str()) {
            return Err(format!("stderr lacks `{}`\nstderr:\n{}", s, out.stderr));
        }
    }
    Ok(())
}

fn run_single(path: &Path) -> Result<(), String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let h = parse_header(&text)?;
    let args = build_args(&h, &path.to_string_lossy());
    verify(&h, &run_aval(&args, None))
}

fn run_lock_dir(dir: &Path, n: usize) -> Result<(), String> {
    let before = fs::read_to_string(dir.join("before.aval")).map_err(|e| e.to_string())?;
    let after = fs::read_to_string(dir.join("after.aval")).map_err(|e| e.to_string())?;
    let hb = parse_header(&before)?;
    let ha = parse_header(&after)?;
    let tmp = std::env::temp_dir().join(format!("aval-conf-{}-{}", std::process::id(), n));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
    let result = (|| {
        fs::write(tmp.join("main.aval"), &before).map_err(|e| e.to_string())?;
        if !hb.lock_none {
            let o = run_aval(
                &["spec".into(), "lock".into(), "main.aval".into()],
                Some(&tmp),
            );
            if o.code != 0 {
                return Err(format!(
                    "`aval spec lock` failed ({}):\n{}{}",
                    o.code, o.stdout, o.stderr
                ));
            }
        }
        let name = ha.rename.clone().unwrap_or_else(|| "main.aval".to_string());
        if name != "main.aval" {
            fs::remove_file(tmp.join("main.aval")).map_err(|e| e.to_string())?;
        }
        fs::write(tmp.join(&name), &after).map_err(|e| e.to_string())?;
        let args = build_args(&ha, &name);
        verify(&ha, &run_aval(&args, Some(&tmp)))
    })();
    let _ = fs::remove_dir_all(&tmp);
    result
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<PathBuf> = match fs::read_dir(dir) {
        Ok(rd) => rd.flatten().map(|e| e.path()).collect(),
        Err(_) => return,
    };
    entries.sort();
    for p in entries {
        if p.is_dir() {
            if p.join("before.aval").is_file() && p.join("after.aval").is_file() {
                out.push(p);
            } else {
                collect(&p, out);
            }
        } else if p.extension().map(|e| e == "aval").unwrap_or(false) {
            out.push(p);
        }
    }
}

#[test]
fn conformance_suite() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/conformance");
    let filter = std::env::var("AVAL_CONFORMANCE_FILTER").unwrap_or_default();
    let mut tests = Vec::new();
    collect(&root, &mut tests);
    let mut failures = Vec::new();
    let mut ran = 0;
    for (n, t) in tests.iter().enumerate() {
        let name = t
            .strip_prefix(&root)
            .unwrap_or(t)
            .to_string_lossy()
            .into_owned();
        if !filter.is_empty() && !name.contains(&filter) {
            continue;
        }
        ran += 1;
        let r = if t.is_dir() {
            run_lock_dir(t, n)
        } else {
            run_single(t)
        };
        if let Err(msg) = r {
            failures.push(format!("FAIL {}\n{}", name, msg));
        }
    }
    eprintln!(
        "conformance: {} test(s) run, {} failed",
        ran,
        failures.len()
    );
    assert!(failures.is_empty(), "\n{}\n", failures.join("\n\n"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_parsing() {
        let h = parse_header("# scenario: A-1 A-2\n# cmd: test $FILE --cases 0\n# expect: {\"code\":\"T100\"}\n# expect-codes: E101\nmodule t\n").unwrap();
        assert_eq!(h.scenario, vec!["A-1", "A-2"]);
        assert_eq!(h.expect.len(), 2);
        assert!(expects_error(&h));
        assert_eq!(
            build_args(&h, "f.aval"),
            vec!["test", "f.aval", "--cases", "0"]
        );
    }

    #[test]
    fn shorthand_and_subset() {
        let h = parse_header("# cmd: check\nmodule t\n").unwrap();
        assert_eq!(build_args(&h, "x"), vec!["check", "x"]);
        assert!(is_subset(
            &serde_json::json!({"a": 1}),
            &serde_json::json!({"a": 1.0, "b": 2})
        ));
        assert!(!is_subset(
            &serde_json::json!({"a": [1]}),
            &serde_json::json!({"a": [1, 2]})
        ));
    }
}
