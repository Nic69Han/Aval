# E2 results: Aval vs Rust, in-context

- Provider: `command`
- Tasks: 40; repairs allowed: 3
- Skill (`aval skill print`) SHA-256: `4473483980d2cb9f8dd8a6338d4def88f7dcf2df33c27b166c9f4a4ff1f2e108`
- Criterion (pre-registered): Aval pass@1 >= 0.8 x Rust pass@1

Verdict: PASS (Aval pass@1 = 0.950, Rust pass@1 = 1.000, needs >= 0.800)

| Metric | Aval | Rust |
| --- | --- | --- |
| compile@1 | 0.950 | 1.000 |
| pass@1 | 0.950 | 1.000 |
| pass@R (R = 3) | 1.000 | 1.000 |
| solved tasks | 40 / 40 | 40 / 40 |
| tokens/solved (with system) | 2931 | 220 |
| tokens/solved (without system) | 343 | 193 |
| provider errors (excluded) | 0 | 0 |
| tokens in / out (total) | 109684 / 7546 | 5076 / 3714 |
| token source | tiktoken | tiktoken |

tokens/solved (with system) = all tokens (prompt + answer, every attempt of every task) divided by the number of solved tasks; the system prompt is the whole `aval skill print` reference for Aval and one line for Rust. Without system: the system-prompt tokens of every attempt are subtracted first.


## Per task

| Task | Aval (solved at / attempts) | Rust (solved at / attempts) |
| --- | --- | --- |
| HE-000 | 1 / 1 | 1 / 1 |
| HE-002 | 1 / 1 | 1 / 1 |
| HE-003 | 1 / 1 | 1 / 1 |
| HE-004 | 1 / 1 | 1 / 1 |
| HE-007 | 1 / 1 | 1 / 1 |
| HE-013 | 1 / 1 | 1 / 1 |
| HE-014 | 1 / 1 | 1 / 1 |
| HE-015 | 1 / 1 | 1 / 1 |
| HE-016 | 1 / 1 | 1 / 1 |
| HE-018 | 1 / 1 | 1 / 1 |
| HE-024 | 1 / 1 | 1 / 1 |
| HE-025 | 1 / 1 | 1 / 1 |
| HE-026 | 1 / 1 | 1 / 1 |
| HE-027 | 1 / 1 | 1 / 1 |
| HE-031 | 1 / 1 | 1 / 1 |
| HE-036 | 1 / 1 | 1 / 1 |
| HE-040 | 1 / 1 | 1 / 1 |
| HE-043 | 1 / 1 | 1 / 1 |
| HE-044 | 1 / 1 | 1 / 1 |
| HE-046 | 2 / 2 | 1 / 1 |
| HE-047 | 1 / 1 | 1 / 1 |
| HE-048 | 1 / 1 | 1 / 1 |
| HE-049 | 1 / 1 | 1 / 1 |
| HE-051 | 1 / 1 | 1 / 1 |
| HE-054 | 1 / 1 | 1 / 1 |
| HE-056 | 1 / 1 | 1 / 1 |
| HE-057 | 1 / 1 | 1 / 1 |
| HE-058 | 1 / 1 | 1 / 1 |
| HE-062 | 1 / 1 | 1 / 1 |
| HE-064 | 1 / 1 | 1 / 1 |
| HE-065 | 1 / 1 | 1 / 1 |
| HE-067 | 1 / 1 | 1 / 1 |
| HE-068 | 1 / 1 | 1 / 1 |
| HE-070 | 1 / 1 | 1 / 1 |
| HE-072 | 1 / 1 | 1 / 1 |
| HE-074 | 1 / 1 | 1 / 1 |
| HE-076 | 3 / 3 | 1 / 1 |
| HE-080 | 1 / 1 | 1 / 1 |
| HE-086 | 1 / 1 | 1 / 1 |
| HE-120 | 1 / 1 | 1 / 1 |
