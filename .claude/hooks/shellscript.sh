#!/bin/bash

command -v jq >/dev/null || {
    echo "shellscript hook: jq not found" >&2
    exit 1
}
command -v shfmt >/dev/null || {
    echo "shellscript hook: shfmt not found" >&2
    exit 1
}
command -v shellcheck >/dev/null || {
    echo "shellscript hook: shellcheck not found" >&2
    exit 1
}

# Read stdin first (consumed by jq)
INPUT=$(cat)
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty')

# Handle only shell scripts within the project directory
PROJECT_DIR=$(git rev-parse --show-toplevel 2>/dev/null) || exit 0
if [[ ! "$FILE_PATH" =~ \.sh$ ]] || [[ "$FILE_PATH" != "$PROJECT_DIR"/* ]]; then
    exit 0
fi

# Format: indent 4 spaces
FMT_OUTPUT=$(shfmt -i 4 -w "$FILE_PATH" 2>&1)
FMT_EXIT_CODE=$?

# The linter only reports; exit 1 means findings, any other code is a tool failure
LINT_OUTPUT=$(shellcheck -x "$FILE_PATH" 2>&1)
LINT_EXIT_CODE=$?

[[ $FMT_EXIT_CODE -eq 0 && $LINT_EXIT_CODE -eq 0 ]] && exit 0

if [[ $FMT_EXIT_CODE -ne 0 ]]; then
    REASON="shfmt could not format the file (exit $FMT_EXIT_CODE)"
elif [[ $LINT_EXIT_CODE -eq 1 ]]; then
    REASON="ShellCheck found issues that need a manual fix"
else
    REASON="shellcheck failed (exit $LINT_EXIT_CODE)"
fi

if [[ -n "$FMT_OUTPUT" ]]; then
    REPORT_OUTPUT=$(printf '%s\n%s' "$FMT_OUTPUT" "$LINT_OUTPUT")
else
    REPORT_OUTPUT="$LINT_OUTPUT"
fi

ESCAPED_OUTPUT=$(printf '%s' "$REPORT_OUTPUT" | jq -Rs .)
cat <<EOF
{
  "decision": "block",
  "reason": "$REASON",
  "hookSpecificOutput": {
    "hookEventName": "PostToolUse",
    "additionalContext": $ESCAPED_OUTPUT
  }
}
EOF
