#!/bin/bash

HOOK_NAME="markdownlint"
# shellcheck source-path=SCRIPTDIR
source "$(dirname "${BASH_SOURCE[0]}")/lib/hook.sh"

hook_accept '\.md$'
hook_require markdownlint-cli2

LINT_OUTPUT=$(markdownlint-cli2 "$FILE_PATH" 2>&1)
LINT_EXIT_CODE=$?

[[ $LINT_EXIT_CODE -eq 0 ]] && exit 0

if [[ $LINT_EXIT_CODE -eq 1 ]]; then
    REASON="Markdownlint found issues that --fix could not resolve"
else
    REASON="markdownlint-cli2 failed (exit $LINT_EXIT_CODE)"
fi

hook_block "$REASON" "$LINT_OUTPUT"
