#!/usr/bin/env sh
# Command provider for harness.py using the Claude Code CLI in print mode.
# The model gets no tools, no MCP servers, no settings and a neutral system prompt;
# the whole conversation built by the harness arrives on stdin.
# Model: set AVAL_E2_MODEL and pass it with `--pass-env AVAL_E2_MODEL` (CLI default otherwise).
set -eu
if [ -n "${AVAL_E2_MODEL:-}" ]; then
  set -- --model "$AVAL_E2_MODEL"
else
  set --
fi
exec claude -p "$@" \
  --tools "" \
  --strict-mcp-config \
  --setting-sources "" \
  --no-session-persistence \
  --system-prompt "You are an expert programmer. Follow the instructions in the conversation exactly and answer with the requested code block."
