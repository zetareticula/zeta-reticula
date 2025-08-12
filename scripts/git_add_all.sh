#!/usr/bin/env bash
set -euo pipefail

# Add all non-ignored files and directories, then commit if there are changes.

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || echo "")"
if [[ -z "$REPO_ROOT" ]]; then
  echo "Error: not inside a git repository" >&2
  exit 1
fi
cd "$REPO_ROOT"

# Show current branch
BRANCH=$(git rev-parse --abbrev-ref HEAD)
echo "On branch: $BRANCH"

echo "Staging all changes (respecting .gitignore)..."
# -A stages modifications, deletions, and new files
git add -A

# Check if there is anything to commit
if git diff --cached --quiet; then
  echo "No staged changes to commit."
  # Show short status for visibility
  git status -s
  exit 0
fi

# Prepare a concise commit message with timestamp
TS=$(date +"%Y-%m-%d %H:%M:%S %z")
COMMIT_MSG="chore: stage all folders/files and commit changes ($TS)"

echo "Committing..."
git commit -m "$COMMIT_MSG"

# Show last commit summary
echo "Last commit:"
LAST=$(git log -n 1 --oneline)
echo "$LAST"

echo "Done."
