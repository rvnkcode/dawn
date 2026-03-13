#!/bin/bash

FILE_PATH=$(jq -r '.tool_input.file_path // empty')

# Handle only Rust files
if [[ ! "$FILE_PATH" =~ \.rs$ ]]; then
    exit 0
fi

# Run cargo clippy and capture output
CLIPPY_OUTPUT=$(cargo clippy --all-features --message-format=short 2>&1)

# Ref: https://code.claude.com/docs/en/hooks#posttooluse-decision-control
if echo "$CLIPPY_OUTPUT" | grep -q "warning\|error"; then
    ESCAPED_OUTPUT=$(printf '%s' "$CLIPPY_OUTPUT" | jq -Rs .)
    cat <<EOF
{
  "decision": "block",
  "reason": "Clippy found issues that need to be addressed",
  "hookSpecificOutput": {
    "hookEventName": "PostToolUse",
    "additionalContext": $ESCAPED_OUTPUT
  }
}
EOF
fi
