#!/bin/bash

HOOK_NAME="shellscript"
# shellcheck source-path=SCRIPTDIR
source "$(dirname "${BASH_SOURCE[0]}")/lib/hook.sh"

hook_accept '\.sh$'
hook_require shfmt shellcheck

# Format indent 4 spaces
FMT_OUTPUT=$(shfmt -i 4 -w "$FILE_PATH" 2>&1)
FMT_EXIT_CODE=$?

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

hook_block "$REASON" "$(hook_join "$FMT_OUTPUT" "$LINT_OUTPUT")"
