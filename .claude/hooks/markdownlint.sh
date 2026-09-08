#!/bin/bash

command -v jq >/dev/null || {
    echo "markdownlint hook: jq not found" >&2
    exit 1
}

# Read stdin first (consumed by jq)
INPUT=$(cat)
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty')

# Handle only Markdown files within the project directory
PROJECT_DIR=$(git rev-parse --show-toplevel 2>/dev/null) || exit 0
if [[ ! "$FILE_PATH" =~ \.md$ ]] || [[ "$FILE_PATH" != "$PROJECT_DIR"/* ]]; then
    exit 0
fi

command -v markdownlint-cli2 >/dev/null || {
    echo "markdownlint hook: markdownlint-cli2 not found" >&2
    exit 1
}

LINT_OUTPUT=$(markdownlint-cli2 "$FILE_PATH" 2>&1)
LINT_EXIT_CODE=$?

[[ $LINT_EXIT_CODE -eq 0 ]] && exit 0

if [[ $LINT_EXIT_CODE -eq 1 ]]; then
    REASON="Markdownlint found issues that --fix could not resolve"
else
    REASON="markdownlint-cli2 failed (exit $LINT_EXIT_CODE)"
fi

ESCAPED_OUTPUT=$(printf '%s' "$LINT_OUTPUT" | jq -Rs .)
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
