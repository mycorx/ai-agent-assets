---
name: git-rewrite-email
description: Rewrite Git history to remove an unwanted email, auto-install filter-repo if missing, and optionally restore the origin remote.
argument-hint: [action old-email new-email]
---

# Git Rewrite Email Skill

This Skill rewrites Git commit history to replace an unwanted email address
(author + committer) using `git filter-repo`.  
If `git-filter-repo` is not installed, the Skill will install it using pipx
(and install pipx if missing).  
It also prompts the user to re-add the original origin remote after rewriting.

## Internal Setup Logic

Before any rewrite, the Skill performs:

```bash
# Detect git-filter-repo
if ! command -v git-filter-repo >/dev/null 2>&1; then
    echo "git-filter-repo not found. Preparing installation..."
    NEED_INSTALL=1
fi

# Install pipx if missing
if [ "$NEED_INSTALL" = "1" ]; then
    if ! command -v pipx >/dev/null 2>&1; then
        echo "pipx not found. Installing pipx..."
        sudo apt update
        sudo apt install -y pipx
        pipx ensurepath
    fi
fi

# Install git-filter-repo via pipx
if [ "$NEED_INSTALL" = "1" ]; then
    echo "Installing git-filter-repo via pipx..."
    pipx install git-filter-repo
fi
