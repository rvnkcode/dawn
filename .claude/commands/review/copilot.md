---
description: Read and review GitHub Copilot's PR review comments on the current branch.
argument-hint: [pr-number]
allowed-tools: Bash, Read, Grep, Glob
---

# Copilot PR Review

Review GitHub Copilot's review comments on a pull request and provide recommendations.

## Steps

1. **Find the PR**: Resolve the PR number in this order, stopping at the first that succeeds.
   1. `$ARGUMENTS` — if it holds a PR number or a PR URL, use it.
   2. `BRANCH=$(git branch --show-current)`. If `$BRANCH` is non-empty, run
      `gh pr list --head "$BRANCH" --json number,title,url`.
   3. `$BRANCH` is empty. This repo is jujutsu colocated, so a detached HEAD is normal and no
      local branch necessarily maps to the PR — `jj bookmark list` and `git rev-parse HEAD` are
      no help either, since the PR head branch may exist only on the remote. Run
      `gh pr list --json number,title,url,headRefName` and use the single open PR; if more than
      one is open, ask the user which to review.
   - If no PR is found, inform the user and stop.

2. **Fetch Copilot comments**: Get every review comment in one paginated call, keeping the
   comment `id` and its reply linkage.

   ```bash
   gh api --paginate repos/{owner}/{repo}/pulls/{number}/comments \
     --jq '.[] | {id, user: .user.login, path, line, in_reply_to: .in_reply_to_id, body}'
   ```

   - `--paginate` is required. Without it the API returns only the first page (30 comments) and
     the rest are dropped with no warning.
   - Copilot's own comments are those whose `user` is `Copilot` or `copilot-pull-request-reviewer`.
     Both logins occur in practice: inline comments come from `Copilot`, the review summary from
     `copilot-pull-request-reviewer`.
   - A Copilot comment whose `id` appears as another comment's `in_reply_to` has already been
     answered. Report it as already addressed rather than raising it again.
   - `line` is `null` when the comment is outdated, meaning the diff moved after the review. Mark
     it outdated and anchor on the comment's `diff_hunk` instead of a line number.

   Also fetch the overall review summary:

   ```bash
   gh pr view {number} --json reviews \
     --jq '.reviews[] | select(.author.login == "Copilot" or .author.login == "copilot-pull-request-reviewer") | {state, body}'
   ```

3. **Read relevant source files**: For each comment, read the referenced file and line to
   understand the context. For an outdated comment, locate the code by its `diff_hunk`.

4. **Evaluate each comment**: For each Copilot comment, provide your assessment:
   - **Verify the claim by running it.** When a comment asserts tool behavior — a linter warning,
     a compile error, a runtime result — run that tool against the real file and quote the output.
     Reading the source is not sufficient. Treat an unverified Copilot premise as unproven.
   - Whether you agree or disagree with the suggestion
   - Why (with reference to the project's coding standards, architecture, and domain requirements)
   - Whether it should be applied, ignored, or modified

5. **Present findings**: Summarize all comments in a numbered list with:
   - File path and line number, or `outdated` when `line` is null
   - Brief description of Copilot's suggestion
   - The evidence that settled it: the command that was run and its output
   - Your recommendation (apply / ignore / modify) with reasoning
   - Whether the thread already has a reply

   To reply to a thread, use its `id`:

   ```bash
   gh api repos/{owner}/{repo}/pulls/{number}/comments \
     --method POST -F in_reply_to=<id> -f body="<reply>"
   ```
