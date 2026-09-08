#!/bin/bash
# Shared helpers for PostToolUse hooks

# Exit the hook when a required executable is missing
hook_require() {
    local tool
    for tool in "$@"; do
        command -v "$tool" >/dev/null || {
            echo "$HOOK_NAME hook: $tool not found" >&2
            exit 1
        }
    done
}

# Set FILE_PATH and PROJECT_DIR from the payload on stdin
hook_accept() {
    local extension_pattern=$1 input

    hook_require jq

    # Read stdin first (consumed by jq)
    input=$(cat)
    FILE_PATH=$(echo "$input" | jq -r '.tool_input.file_path // empty')

    PROJECT_DIR=$(git rev-parse --show-toplevel 2>/dev/null) || exit 0
    if [[ ! "$FILE_PATH" =~ $extension_pattern ]] || [[ "$FILE_PATH" != "$PROJECT_DIR"/* ]]; then
        exit 0
    fi
}

# Print the non-empty arguments joined by newlines
hook_join() {
    local part
    for part in "$@"; do
        [[ -n "$part" ]] && printf '%s\n' "$part"
    done
}

# https://code.claude.com/docs/en/hooks#posttooluse-decision-control
hook_block() {
    local reason=$1 report=$2
    cat <<EOF
{
  "decision": "block",
  "reason": $(printf '%s' "$reason" | jq -Rs .),
  "hookSpecificOutput": {
    "hookEventName": "PostToolUse",
    "additionalContext": $(printf '%s' "$report" | jq -Rs .)
  }
}
EOF
    exit 0
}
