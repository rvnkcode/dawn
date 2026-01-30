---
name: cli-reviewer
description: Compare Dawn's CLI implementation with Taskwarrior for specific commands or features
argument-hint: [command-or-feature]
allowed-tools: Bash, Grep, Glob, Read
model: opus
---

# CLI Implementation Review: Dawn vs Taskwarrior

Compare Dawn's CLI implementation with the original Taskwarrior for the specified command or feature: **$ARGUMENTS**

## Review Scope

If no specific command/feature is provided in **$ARGUMENTS**, review the diff between `origin/cli` branch and the current branch:

!`git diff origin/cli...HEAD --stat 2>/dev/null || echo "No origin/cli branch found"`

If a specific command/feature is specified, focus the review on that particular functionality.

## Review Checklist

For the feature **$ARGUMENTS** (or all changes in the diff):

1. **Verify Taskwarrior behavior**: Understand how the original implementation works
2. **Analyze Dawn implementation**: Review if current Clap structure is appropriate
3. **Categorize differences**: Distinguish intentional differences vs missing functionality
4. **Suggest improvements**: Propose pragmatic implementation using Clap idioms

## Verification

```bash
# Verify Taskwarrior behavior
task [command]

# Verify Dawn behavior
cargo run --features cli -- [command]
```

## Output Format

1. Taskwarrior behavior summary
2. Dawn current implementation status
3. Intentional differences (Clap-idiomatic choices)
4. Actual gaps and improvement suggestions
