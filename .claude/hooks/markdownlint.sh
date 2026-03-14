#!/bin/bash

# Read stdin first (consumed by jq)
INPUT=$(cat)
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty')

# Handle only Markdown files within the project directory
PROJECT_DIR=$(git rev-parse --show-toplevel 2>/dev/null) || exit 0
if [[ ! "$FILE_PATH" =~ \.md$ ]] || [[ "$FILE_PATH" != "$PROJECT_DIR"/* ]]; then
    exit 0
fi

# Run markdownlint-cli2 to fix (suppress output to avoid JSON pollution)
markdownlint-cli2 "$FILE_PATH" >/dev/null 2>&1

# Run again to check for remaining issues
LINT_OUTPUT=$(markdownlint-cli2 "$FILE_PATH" 2>&1)
LINT_EXIT_CODE=$?

if [[ $LINT_EXIT_CODE -ne 0 ]]; then
    ESCAPED_OUTPUT=$(printf '%s' "$LINT_OUTPUT" | jq -Rs .)
    cat <<EOF
{
  "decision": "block",
  "reason": "Markdownlint found issues that --fix could not resolve",
  "hookSpecificOutput": {
    "hookEventName": "PostToolUse",
    "additionalContext": $ESCAPED_OUTPUT
  }
}
EOF
fi
