# Tasks not shipped

MBPP+ tasks (378 in the dataset) that are not in this directory.

## Reference could not be written within Aval v0.1 (a language finding)

The importer skips every id in a row of this table.

| Id | Reason |
| --- | --- |
| MB-006 | needs bitwise XOR/AND on negative integers (no bitwise operators in Aval v0.1; the cases include negative operands) |
| MB-119 | the hidden cases include arrays for which the expected value is the XOR of all elements (no bitwise XOR in Aval v0.1) |
| MB-253 | degenerate task: type inference gives List<List<List<Int>>> and every shipped case expects 0 (the visible cases mix element types that Aval cannot express) |
| MB-427 | regular expressions: the hidden cases encode `re.sub` semantics on malformed inputs (for example "2021-1-026" -> "02-1-20216"), which cannot be written without a regex or substitution library |
| MB-457 | type mismatch inside the task: it returns `List<List<String>>` but the minimal sublist is a `List<String>`; every hidden expected value is `[]`, so only a vacuous reference is possible |

| MB-638 | needs Float power / exp / ln (`v ** 0.16` in the wind chill formula); no `powf` in Aval v0.1 |
| MB-724 | needs arbitrary-precision integers (hidden cases need 5^1000, 7^200, 11^100; `Int` is i64) |

## Hidden values do not follow the statement

A model cannot infer these behaviours from the statement, so the task measures guessing rather than
programming (equally for both languages). Ambiguous but reasonable tasks (MB-435, 559, 589, 604, 757,
771, 777) are kept.

| Id | Reason |
| --- | --- |
| MB-016 | "contains sequences of lowercase letters joined with an underscore": the values follow a whole-string match with the underscore optional (`"abc"` expects true); the search reading disagrees with 81 of 115 cases (70.4 %) |
| MB-092 | numbers with fewer than three digits (`82`, `40`) expect true; the usual definition (alternating digits, at least three) disagrees with 69 of 103 cases (67.0 %) |
| MB-102 | leading, trailing and doubled underscores keep one underscore per empty segment (`"___python_program"` -> `"___PythonProgram"`); dropping them disagrees with 100 of 112 cases (89.3 %) |
| MB-113 | surrounding whitespace and a leading `+` are accepted; the strict reading (digits only) disagrees with 12 of 119 cases (10.1 %) |
| MB-130 | 42 of 104 cases have a tie for the maximum frequency and the tie rule is unstated: choosing the last tied item disagrees with 27 cases (26.0 %), the smallest with 4 (3.8 %) |
| MB-418 | 17 of 43 cases have several longest lists and the tie rule is unstated: choosing the last longest list disagrees with 17 cases (39.5 %) |
| MB-138 | "non-zero powers of 2": the reading "any positive integer power" (exponent unstated) disagrees with 23/101 = 22.8 % of hidden cases |
| MB-425 | both parameters are inferred as `List<List<String>>` and every expected value is 0 (degenerate) |
| MB-448 | every negative n expects 5 (the Python reference falls through its loops) |
| MB-451 | only the space character is removed; tabs and newlines are kept although the statement says whitespace |
| MB-459 | only lowercase ASCII letters are kept; punctuation is dropped although the statement removes uppercase only |
| MB-468 | expected values match no algorithm for the stated problem (78 of 112 cases disagree with the true maximum product) |
| MB-558 | values pair digit strings position by position (zip), not the digit sum of the difference |
| MB-576 | hidden cases test a subsequence, the statement asks for a sublist |
| MB-577 | non-integral floats expect 0 |
| MB-620 | values come from a DP over adjacent elements in the original order, not the stated pairwise divisibility |
| MB-622 | most inputs violate the precondition; values follow the Python two-pointer loop |
| MB-639 | counts names that are capitalised with a lowercase remainder, not names that do not start lowercase |
| MB-735 | n = 1 expects 3 |
| MB-739 | values follow a float formula, not the exact smallest triangular number |
| MB-790 | every index must satisfy `nums[i] % 2 == i % 2`, not only even positions |
| MB-806 | values reproduce a quirk of the Python reference (e.g. "Aaa" -> 0) |

## Harness limit

| Id | Reason |
| --- | --- |
| MB-462 | expected values up to 2,048 nested lists: compiling the generated Rust test binary takes more than 3 minutes (above the per-command timeout) |

## Signature differs from the original problem

| Id | Reason |
| --- | --- |
| MB-069 | the original arguments are `List<Int>`; 79 of the hidden cases use `List<String>` (mutated inputs), so the shipped element type is not the problem's |
| MB-297 | the original input mixes integers and lists at the same level (`[0, 10, [20, 30], 40, ...]`), which Aval's `List<T>` cannot express; the inferred depth (four levels) is arbitrary |
| MB-562 | with the signature fixed to the original depth (`List<List<Int>>`) fewer than 20 hidden cases remain |
| MB-612 | with the signature fixed to the original depth (`List<List<String>>`) fewer than 20 hidden cases remain |

## Screened out by the importer (values or tests not expressible as an Aval equality test)

| Id | Reason |
| --- | --- |
| MB-002 | the expected result is compared as a set in MBPP+ (order unspecified), so a list-returning task has no single correct answer (unordered_result) |
| MB-007 | the expected result is compared as a set in MBPP+ (order unspecified), so a list-returning task has no single correct answer (unordered_result) |
| MB-058 | fewer than 20 hidden cases remain after mapping the values to Aval's types (few_cases) |
| MB-065 | fewer than 20 hidden cases remain after mapping the values to Aval's types (few_cases) |
| MB-088 | the test uses a custom comparison (reference function or boolean coercion), not an equality with an expected value (no_plain_assertion) |
| MB-111 | the expected result is compared as a set in MBPP+ (order unspecified), so a list-returning task has no single correct answer (unordered_result) |
| MB-124 | the test values are not plain literals (cannot be imported safely) (unparsable) |
| MB-127 | fewer than 20 hidden cases remain after mapping the values to Aval's types (few_cases) |
| MB-140 | the expected result is compared as a set in MBPP+ (order unspecified), so a list-returning task has no single correct answer (unordered_result) |
| MB-232 | the expected result is compared as a set in MBPP+ (order unspecified), so a list-returning task has no single correct answer (unordered_result) |
| MB-251 | fewer than 20 hidden cases remain after mapping the values to Aval's types (few_cases) |
| MB-252 | the test values are not plain literals (cannot be imported safely) (unparsable) |
| MB-255 | the test uses a custom comparison (reference function or boolean coercion), not an equality with an expected value (no_plain_assertion) |
| MB-395 | fewer than 20 hidden cases remain after mapping the values to Aval's types (few_cases) |
| MB-439 | fewer than 20 hidden cases remain after mapping the values to Aval's types (few_cases) |
| MB-455 | fewer than 20 hidden cases remain after mapping the values to Aval's types (few_cases) |
| MB-471 | fewer than 20 hidden cases remain after mapping the values to Aval's types (few_cases) |
| MB-473 | the test values are not plain literals (cannot be imported safely) (unparsable) |
| MB-573 | fewer than 20 hidden cases remain after mapping the values to Aval's types (few_cases) |
| MB-579 | the expected result is compared as a set in MBPP+ (order unspecified), so a list-returning task has no single correct answer (unordered_result) |
| MB-583 | fewer than 20 hidden cases remain after mapping the values to Aval's types (few_cases) |
| MB-590 | the test values are not plain literals (cannot be imported safely) (unparsable) |
| MB-591 | fewer than 20 hidden cases remain after mapping the values to Aval's types (few_cases) |
| MB-630 | the test uses a custom comparison (reference function or boolean coercion), not an equality with an expected value (no_plain_assertion) |
| MB-635 | fewer than 20 hidden cases remain after mapping the values to Aval's types (few_cases) |
| MB-734 | fewer than 20 hidden cases remain after mapping the values to Aval's types (few_cases) |
| MB-737 | the test uses a custom comparison (reference function or boolean coercion), not an equality with an expected value (no_plain_assertion) |
| MB-762 | fewer than 20 hidden cases remain after mapping the values to Aval's types (few_cases) |
| MB-769 | the expected result is compared as a set in MBPP+ (order unspecified), so a list-returning task has no single correct answer (unordered_result) |
| MB-778 | fewer than 20 hidden cases remain after mapping the values to Aval's types (few_cases) |
| MB-787 | the test uses a custom comparison (reference function or boolean coercion), not an equality with an expected value (no_plain_assertion) |
| MB-792 | fewer than 20 hidden cases remain after mapping the values to Aval's types (few_cases) |
| MB-793 | fewer than 20 hidden cases remain after mapping the values to Aval's types (few_cases) |
| MB-794 | the test uses a custom comparison (reference function or boolean coercion), not an equality with an expected value (no_plain_assertion) |

74 further MBPP+ tasks use tuple, dict, set or None values in their parameters or results (no Aval type) and are not listed individually.

## Kept with a note

- MB-223: hidden values use the parameter `n` as the array length, which differs from `len(arr)` in 62 of 71 cases; the literal reading of the signature (use `n`) agrees with all of them, so the task is kept.
