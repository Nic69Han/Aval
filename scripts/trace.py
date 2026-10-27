#!/usr/bin/env python3
"""Traceability check: every scenario has a conformance test, every test names a known scenario.

Scenario IDs come from `#### Scenario: <ID> ...` lines in openspec/specs/**/spec.md and
openspec/changes/*/specs/**/spec.md (openspec/changes/archive is excluded). Test IDs come from
`# scenario:` headers of the files under tests/conformance (including before.aval/after.aval of
directory tests). Exit code 1 on any mismatch.
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCENARIO_RE = re.compile(r"^####\s+Scenario:\s*([A-Z]+-\d+)\b")
HEADER_RE = re.compile(r"^#\s*scenario:\s*(.*)$")
ID_RE = re.compile(r"^[A-Z]+-\d+$")


def spec_files():
    files = list((ROOT / "openspec" / "specs").glob("**/spec.md"))
    changes = ROOT / "openspec" / "changes"
    if changes.is_dir():
        for change in sorted(changes.iterdir()):
            if change.name == "archive" or not change.is_dir():
                continue
            files.extend((change / "specs").glob("**/spec.md"))
    return sorted(files)


def scenarios():
    found = {}
    for f in spec_files():
        for line in f.read_text(encoding="utf-8").splitlines():
            m = SCENARIO_RE.match(line)
            if m:
                found.setdefault(m.group(1), str(f.relative_to(ROOT)))
    return found


def test_ids():
    found = {}
    base = ROOT / "tests" / "conformance"
    for f in sorted(base.rglob("*")):
        if not f.is_file() or f.suffix not in (".aval", ".py"):
            continue
        for line in f.read_text(encoding="utf-8").splitlines():
            m = HEADER_RE.match(line)
            if m:
                for tid in m.group(1).split():
                    found.setdefault(tid, []).append(str(f.relative_to(ROOT)))
                break
    return found


def main():
    spec = scenarios()
    tests = test_ids()
    missing = sorted(set(spec) - set(tests))
    unknown = sorted(set(tests) - set(spec))
    malformed = sorted(t for t in tests if not ID_RE.match(t))
    for sid in missing:
        print(f"missing test for scenario {sid} ({spec[sid]})")
    for tid in unknown:
        print(f"unknown scenario {tid} named in {', '.join(tests[tid])}")
    for tid in malformed:
        print(f"malformed scenario id {tid}")
    print(f"{len(spec)} scenarios, {len(tests)} tested ids, {len(missing)} missing, {len(unknown)} unknown")
    return 1 if (missing or unknown or malformed) else 0


if __name__ == "__main__":
    sys.exit(main())
