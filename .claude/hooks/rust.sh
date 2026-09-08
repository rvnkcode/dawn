#!/bin/bash

HOOK_NAME="rust"
# shellcheck source-path=SCRIPTDIR
source "$(dirname "${BASH_SOURCE[0]}")/lib/hook.sh"

hook_accept '\.rs$'
hook_require rustfmt cargo

# Format from the edited file
FMT_OUTPUT=$(rustfmt "$FILE_PATH" 2>&1)
FMT_EXIT_CODE=$?

LINT_OUTPUT=$(cargo clippy --workspace --message-format=short -- -D warnings 2>&1)
LINT_EXIT_CODE=$?

[[ $FMT_EXIT_CODE -eq 0 && $LINT_EXIT_CODE -eq 0 ]] && exit 0

if [[ $FMT_EXIT_CODE -ne 0 ]]; then
    REASON="rustfmt could not format the file (exit $FMT_EXIT_CODE)"
else
    REASON="cargo clippy reported problems (exit $LINT_EXIT_CODE)"
fi

hook_block "$REASON" "$(hook_join "$FMT_OUTPUT" "$LINT_OUTPUT")"
