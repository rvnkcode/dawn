#!/bin/bash

# Read stdin first (consumed by jq)
INPUT=$(cat)
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty')

PROJECT_DIR=$(git rev-parse --show-toplevel 2>/dev/null) || exit 0
GUI_DIR="$PROJECT_DIR/gui"

# Only handle frontend files inside gui/, excluding the Rust side and build/deps
case "$FILE_PATH" in
    "$GUI_DIR"/src-tauri/* | "$GUI_DIR"/node_modules/* | "$GUI_DIR"/dist/*)
        exit 0 ;;
    "$GUI_DIR"/*.ts | "$GUI_DIR"/*.js | "$GUI_DIR"/*.svelte | "$GUI_DIR"/*.json | "$GUI_DIR"/*.css | "$GUI_DIR"/*.html)
        ;;
    *)
        exit 0 ;;
esac

# Run the GUI check (svelte-check + biome check --write)
CHECK_OUTPUT=$(cd "$GUI_DIR" && bun run check 2>&1)
CHECK_EXIT_CODE=$?

# Ref: https://code.claude.com/docs/en/hooks#posttooluse-decision-control
if [[ $CHECK_EXIT_CODE -ne 0 ]]; then
    ESCAPED_OUTPUT=$(printf '%s' "$CHECK_OUTPUT" | jq -Rs .)
    cat <<EOF
{
  "decision": "block",
  "reason": "bun run check found issues that need to be addressed",
  "hookSpecificOutput": {
    "hookEventName": "PostToolUse",
    "additionalContext": $ESCAPED_OUTPUT
  }
}
EOF
fi
