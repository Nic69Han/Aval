# E1 results: are Aval reserved words single tokens?

- Date (UTC): 2026-10-24 (run at 2026-10-24T12:11:40+00:00)
- Inputs: `keywords.txt` (46 keywords) and project name `Aval`
- Script: `measure.py`, Python 3.13.17, Linux-6.18.44-fc-v153-x86_64-with-glibc2.39
- Contexts: bare (`fn`), leading space (` fn`), after newline (`\nfn`). The after-newline delta is the raw count minus the count of `\n` alone; it is exact only if the newline does not merge with the keyword.
- No special tokens (no BOS) are added. Cell format in the main table: `bare / leading-space` token counts.

## Verdict

Success criterion: at most 3 keywords at 2 tokens per tokenizer. The primary verdict uses the leading-space context, because in code a keyword is usually preceded by whitespace. The bare-context verdict is reported as a secondary check. Keywords at 3 or more tokens are outside the literal criterion and are shown in the last two count columns.

| Tokenizer | Status | Single-token (leading space) | 2 tokens (leading space) | Verdict (leading space) | 2 tokens (bare) | Verdict (bare, secondary) | >2 tokens (bare / leading) |
|---|---|---|---|---|---|---|---|
| `tiktoken:o200k_base` | ok | 47/47 | 0 | **PASS** | 2 | PASS | 0 / 0 |
| `tiktoken:cl100k_base` | ok | 47/47 | 0 | **PASS** | 3 | PASS | 0 / 0 |
| `hf:unsloth/Llama-3.2-1B` | ok | 47/47 | 0 | **PASS** | 3 | PASS | 0 / 0 |
| `hf:NousResearch/Meta-Llama-3-8B` | ok | 47/47 | 0 | **PASS** | 3 | PASS | 0 / 0 |
| `hf:Qwen/Qwen2.5-7B-Instruct` | ok | 47/47 | 0 | **PASS** | 3 | PASS | 0 / 0 |
| `hf:Qwen/Qwen3-8B` | ok | 47/47 | 0 | **PASS** | 3 | PASS | 0 / 0 |
| `hf:deepseek-ai/DeepSeek-V3` | ok | 47/47 | 0 | **PASS** | 7 | FAIL | 0 / 0 |
| `hf:mistralai/Mistral-7B-v0.3` | ok | 41/47 | 6 | **FAIL** | 6 | FAIL | 0 / 0 |
| `claude:count_tokens` | not measured (no API key) | - | - | not measured | - | - | - |

Overall (leading space): 7 of 8 measured tokenizers PASS. 0 failed to load and 1 not measured, so coverage is partial.

## Keyword x tokenizer table (bare / leading space)

| Term | o200k_base | cl100k_base | Llama-3.2-1B | Meta-Llama-3-8B | Qwen2.5-7B-Instruct | Qwen3-8B | DeepSeek-V3 | Mistral-7B-v0.3 | count_tokens |
|---|---|---|---|---|---|---|---|---|---|
| `fn` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | n/a |
| `let` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | n/a |
| `return` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | n/a |
| `type` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | n/a |
| `enum` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | n/a |
| `module` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | n/a |
| `import` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | n/a |
| `pub` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | n/a |
| `if` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | n/a |
| `else` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | n/a |
| `match` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | n/a |
| `for` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | n/a |
| `in` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | n/a |
| `par` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | n/a |
| `requires` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | n/a |
| `ensures` | 2 / 1 | 2 / 1 | 2 / 1 | 2 / 1 | 2 / 1 | 2 / 1 | 2 / 1 | 1 / 1 | n/a |
| `result` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | n/a |
| `examples` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | n/a |
| `oracle` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 2 / 1 | 2 / 2 | n/a |
| `effects` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | n/a |
| `alloc` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | n/a |
| `cost` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | n/a |
| `latency` | 2 / 1 | 2 / 1 | 2 / 1 | 2 / 1 | 2 / 1 | 2 / 1 | 2 / 1 | 2 / 2 | n/a |
| `reuse` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 2 / 1 | 2 / 2 | n/a |
| `none` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | n/a |
| `resource` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | n/a |
| `using` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | n/a |
| `unsafe` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 2 / 1 | 1 / 1 | n/a |
| `true` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | n/a |
| `false` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | n/a |
| `and` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | n/a |
| `or` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | n/a |
| `not` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | n/a |
| `Int` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | n/a |
| `Float` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | n/a |
| `Bool` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 2 / 2 | n/a |
| `String` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | n/a |
| `Byte` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | n/a |
| `List` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | n/a |
| `Set` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | n/a |
| `Map` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | n/a |
| `Queue` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 2 / 2 | n/a |
| `Image` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | n/a |
| `Tensor` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | n/a |
| `Result` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | n/a |
| `Arena` | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 2 / 1 | 1 / 1 | n/a |
| `Aval` | 1 / 1 | 2 / 1 | 2 / 1 | 2 / 1 | 2 / 1 | 2 / 1 | 2 / 1 | 2 / 2 | n/a |

Column headers: `o200k_base` = `tiktoken:o200k_base`; `cl100k_base` = `tiktoken:cl100k_base`; `Llama-3.2-1B` = `hf:unsloth/Llama-3.2-1B`; `Meta-Llama-3-8B` = `hf:NousResearch/Meta-Llama-3-8B`; `Qwen2.5-7B-Instruct` = `hf:Qwen/Qwen2.5-7B-Instruct`; `Qwen3-8B` = `hf:Qwen/Qwen3-8B`; `DeepSeek-V3` = `hf:deepseek-ai/DeepSeek-V3`; `Mistral-7B-v0.3` = `hf:mistralai/Mistral-7B-v0.3`; `count_tokens` = `claude:count_tokens`.

## Terms with more than one token, per tokenizer

Listed when the bare or leading-space count is above 1. Pieces are shown for the leading-space form.

**`tiktoken:o200k_base`** (2 terms)
- `ensures`: bare 2, leading space 1 (pieces: ` ensures`)
- `latency`: bare 2, leading space 1 (pieces: ` latency`)

**`tiktoken:cl100k_base`** (3 terms)
- `ensures`: bare 2, leading space 1 (pieces: ` ensures`)
- `latency`: bare 2, leading space 1 (pieces: ` latency`)
- `Aval`: bare 2, leading space 1 (pieces: ` Aval`)

**`hf:unsloth/Llama-3.2-1B`** (3 terms)
- `ensures`: bare 2, leading space 1 (pieces: `Ġensures`)
- `latency`: bare 2, leading space 1 (pieces: `Ġlatency`)
- `Aval`: bare 2, leading space 1 (pieces: `ĠAval`)

**`hf:NousResearch/Meta-Llama-3-8B`** (3 terms)
- `ensures`: bare 2, leading space 1 (pieces: `Ġensures`)
- `latency`: bare 2, leading space 1 (pieces: `Ġlatency`)
- `Aval`: bare 2, leading space 1 (pieces: `ĠAval`)

**`hf:Qwen/Qwen2.5-7B-Instruct`** (3 terms)
- `ensures`: bare 2, leading space 1 (pieces: `Ġensures`)
- `latency`: bare 2, leading space 1 (pieces: `Ġlatency`)
- `Aval`: bare 2, leading space 1 (pieces: `ĠAval`)

**`hf:Qwen/Qwen3-8B`** (3 terms)
- `ensures`: bare 2, leading space 1 (pieces: `Ġensures`)
- `latency`: bare 2, leading space 1 (pieces: `Ġlatency`)
- `Aval`: bare 2, leading space 1 (pieces: `ĠAval`)

**`hf:deepseek-ai/DeepSeek-V3`** (7 terms)
- `ensures`: bare 2, leading space 1 (pieces: `Ġensures`)
- `oracle`: bare 2, leading space 1 (pieces: `Ġoracle`)
- `latency`: bare 2, leading space 1 (pieces: `Ġlatency`)
- `reuse`: bare 2, leading space 1 (pieces: `Ġreuse`)
- `unsafe`: bare 2, leading space 1 (pieces: `Ġunsafe`)
- `Arena`: bare 2, leading space 1 (pieces: `ĠArena`)
- `Aval`: bare 2, leading space 1 (pieces: `ĠAval`)

**`hf:mistralai/Mistral-7B-v0.3`** (6 terms)
- `oracle`: bare 2, leading space 2 (pieces: `▁or acle`)
- `latency`: bare 2, leading space 2 (pieces: `▁lat ency`)
- `reuse`: bare 2, leading space 2 (pieces: `▁re use`)
- `Bool`: bare 2, leading space 2 (pieces: `▁B ool`)
- `Queue`: bare 2, leading space 2 (pieces: `▁Que ue`)
- `Aval`: bare 2, leading space 2 (pieces: `▁A val`)

**`claude:count_tokens`**: not measured (no API key)

## After-newline context

Number of terms whose after-newline delta (`\n` + term minus `\n`) is exactly 1 token:

- `tiktoken:o200k_base`: 45/47 (the `\n` token alone counts 1)
- `tiktoken:cl100k_base`: 44/47 (the `\n` token alone counts 1)
- `hf:unsloth/Llama-3.2-1B`: 44/47 (the `\n` token alone counts 1)
- `hf:NousResearch/Meta-Llama-3-8B`: 44/47 (the `\n` token alone counts 1)
- `hf:Qwen/Qwen2.5-7B-Instruct`: 44/47 (the `\n` token alone counts 1)
- `hf:Qwen/Qwen3-8B`: 44/47 (the `\n` token alone counts 1)
- `hf:deepseek-ai/DeepSeek-V3`: 40/47 (the `\n` token alone counts 1)
- `hf:mistralai/Mistral-7B-v0.3`: 37/47 (the `\n` token alone counts 2)
Note: where the `\n` token alone is not 1 token (`hf:mistralai/Mistral-7B-v0.3`), the delta subtracts that count, so this column is less reliable for those tokenizers.

## Failures and limits

- No tokenizer failed to load.
- `claude:count_tokens`: not measured (no API key)
- Claude row: not measured, because ANTHROPIC_API_KEY was not set in this run.

## Versions and repository ids

| Tokenizer | Library | Repo / encoding | Revision (commit) | Method |
|---|---|---|---|---|
| `tiktoken:o200k_base` | tiktoken 0.14.0 | `o200k_base` | `-` | tiktoken get_encoding |
| `tiktoken:cl100k_base` | tiktoken 0.14.0 | `cl100k_base` | `-` | tiktoken get_encoding |
| `hf:unsloth/Llama-3.2-1B` | transformers 5.20.2 / tokenizers 0.23.3 | `unsloth/Llama-3.2-1B` | `9535bd9b1d1dea6acafbdc4813b728796aeb28da` | transformers.AutoTokenizer |
| `hf:NousResearch/Meta-Llama-3-8B` | transformers 5.20.2 / tokenizers 0.23.3 | `NousResearch/Meta-Llama-3-8B` | `315b20096dc791d381d514deb5f8bd9c8d6d3061` | transformers.AutoTokenizer |
| `hf:Qwen/Qwen2.5-7B-Instruct` | transformers 5.20.2 / tokenizers 0.23.3 | `Qwen/Qwen2.5-7B-Instruct` | `a09a35458c702b33eeacc393d103063234e8bc28` | transformers.AutoTokenizer |
| `hf:Qwen/Qwen3-8B` | transformers 5.20.2 / tokenizers 0.23.3 | `Qwen/Qwen3-8B` | `b968826d9c46dd6066d109eabc6255188de91218` | transformers.AutoTokenizer |
| `hf:deepseek-ai/DeepSeek-V3` | transformers 5.20.2 / tokenizers 0.23.3 | `deepseek-ai/DeepSeek-V3` | `e815299b0bcbac849fa540c768ef21845365c9eb` | transformers.AutoTokenizer |
| `hf:mistralai/Mistral-7B-v0.3` | transformers 5.20.2 / tokenizers 0.23.3 | `mistralai/Mistral-7B-v0.3` | `caa1feb0e54d415e2df31207e5f4e273e33509b1` | transformers.AutoTokenizer |
| `claude:count_tokens` | - | - | - | not measured (no API key) |

Reproduce: `python -I measure.py` inside a venv with `requirements.txt` installed. Raw per-keyword data, including pieces and after-newline values, is in `results.json`.
