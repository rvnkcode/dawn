---
description: Read and review GitHub Copilot's PR review comments on the current branch.
---

# Copilot PR Review

Review GitHub Copilot's review comments on the PR for the current branch and provide recommendations.

## Steps

1. **Find the PR**: Run `gh pr list --head <current-branch> --json number,title,url` to find the PR associated with the current branch.
   - If no PR exists, inform the user and stop.

2. **Fetch Copilot reviews**: Run `gh api repos/{owner}/{repo}/pulls/{number}/comments --jq '.[] | select(.user.login == "Copilot" or .user.login == "copilot-pull-request-reviewer") | {path: .path, line: .line, body: .body}'` to get Copilot's inline comments.
   - Also check `gh pr view {number} --json reviews --jq '.reviews[] | select(.author.login == "copilot-pull-request-reviewer") | {state: .state, body: .body}'` for the overall review summary.

3. **Read relevant source files**: For each comment, read the referenced file and line to understand the context.

4. **Evaluate each comment**: For each Copilot comment, provide your assessment:
   - Whether you agree or disagree with the suggestion
   - Why (with reference to the project's coding standards, architecture, and domain requirements)
   - Whether it should be applied, ignored, or modified

5. **Present findings**: Summarize all comments in a numbered list with:
   - File path and line number
   - Brief description of Copilot's suggestion
   - Your recommendation (apply / ignore / modify) with reasoning
