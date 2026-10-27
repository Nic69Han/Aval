# Traceability

Aval keeps three layers separate and links them by scenario ID:

| Layer | Location |
| --- | --- |
| Language reference | `docs/spec/` |
| Requirements and scenarios (acceptance criteria) | `openspec/specs/<capability>/spec.md` |
| Executable tests | `tests/conformance/<capability>/<ID>-*.aval` |

Rules:

1. Every scenario has at least one test, and every test names an existing scenario
   (`python3 scripts/trace.py`, checked in CI).
2. Scenario IDs are never renumbered or reused.
3. Tests are derived from the scenarios, not from the implementation. If a test looks wrong, the
   scenario is discussed and fixed first.
