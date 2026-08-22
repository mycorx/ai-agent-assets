---
name: git-rewrite-email
description: Rewrite Git history to remove an unwanted email, auto-install filter-repo if missing, and optionally restore the origin remote.
argument-hint: [action old-email new-email]
---

# Git Rewrite Email Skill

This Skill rewrites Git commit history to replace an unwanted email address (author + committer) using `git-filter-repo`. If `git-filter-repo` is not installed, the Skill will install it using `pipx` (and install `pipx` if missing). It prompts the user to re-add the original `origin` remote after rewriting and asks for explicit confirmation before any force push.

> **Important safety notes**
>
> - The Skill **never** force-pushes without explicit user confirmation.
> - The Skill **never** re-adds `origin` without an explicit yes/no confirmation.
> - The Skill **never** runs a rewrite without explicit `old-email` and `new-email` arguments.
> - The Skill stores the original `origin` URL (if any) before rewriting so it can be restored if the user chooses.

---

## Usage summary

- **Rewrite history:**
  ```bash
  /git-rewrite-email rewrite old@example.com new@example.com
  ```

- **Verify removal:**
  ```bash
  /git-rewrite-email verify old@example.com
  ```

- **Push rewritten history (requires confirmation):**
  ```bash
  /git-rewrite-email push
  ```

---

## Action: rewrite

**Purpose:** Replace all occurrences of `OLD_EMAIL` with `NEW_EMAIL` in author and committer fields across the repository history.

**Usage:**
```bash
/git-rewrite-email rewrite old@example.com new@example.com
```

**Behavior (what Claude will do):**

1. **Validate arguments.** If `old-email` or `new-email` are missing, the Skill will explain the correct usage and abort.
2. **Detect and install prerequisites** (only if missing):
   - Check for `git-filter-repo`.
   - If missing, check for `pipx`.
   - If `pipx` is missing, install it using the system package manager (the Skill uses `sudo apt` by default; the Skill will inform the user if a different OS/package manager is required).
   - Install `git-filter-repo` via `pipx`.
3. **Capture the current origin URL** (if any) and store it in a variable so it can be restored later:
   ```bash
   ORIGIN_URL=$(git config --get remote.origin.url || true)
   ```
4. **Run the rewrite** using `git-filter-repo` with a safe, explicit email-callback. The Skill substitutes the provided arguments into the callback:
   ```bash
   git filter-repo --force --email-callback '
   return b"NEW_EMAIL" if email == b"OLD_EMAIL" else email
   '
   ```
   *(When executed by Claude, OLD_EMAIL and NEW_EMAIL are replaced with the user-supplied $1 and $2 values.)*
5. **Repack and clean** (handled automatically by `git-filter-repo`).
6. **Prompt the user:** “Do you want to re-add the origin remote that existed before the rewrite? (yes/no)”
   - If the user answers yes and `ORIGIN_URL` is non-empty, the Skill runs:
     ```bash
     git remote add origin "$ORIGIN_URL"
     ```
   - If `ORIGIN_URL` is empty, the Skill informs the user there was no origin to restore.
   - If the user answers no, the Skill leaves remotes untouched.

**Example (what the Skill will run conceptually):**
```bash
# prerequisite checks and installs (only if needed)
if ! command -v git-filter-repo >/dev/null 2>&1; then
  NEED_INSTALL=1
fi

if [ "${NEED_INSTALL:-0}" -eq 1 ]; then
  if ! command -v pipx >/dev/null 2>&1; then
    echo "pipx not found. Installing pipx..."
    sudo apt update
    sudo apt install -y pipx
    pipx ensurepath
  fi
  echo "Installing git-filter-repo via pipx..."
  pipx install git-filter-repo
fi

# capture origin
ORIGIN_URL=$(git config --get remote.origin.url || true)

# run rewrite (replace OLD_EMAIL and NEW_EMAIL with user args)
git filter-repo --force --email-callback '
return b"NEW_EMAIL" if email == b"OLD_EMAIL" else email
'
```
> **Note:** The Skill will substitute the actual `old-email` and `new-email` values provided by the user into the command when executing.

---

## Action: verify

**Purpose:** Confirm whether `OLD_EMAIL` still appears anywhere in the repository history (author, committer, or other commit metadata).

**Usage:**
```bash
/git-rewrite-email verify old@example.com
```

**What the Skill runs:**
```bash
# Search author and committer emails
git log --all --pretty=format:"%H %ae %ce" | grep -i "old@example.com" || true

# Deep search across all objects (stronger check)
git rev-list --all | xargs -n1 git grep -i "old@example.com" || true
```

**Interpretation:**
- If the commands return no output, the Skill reports: **PASS** — the email was not found in the repository history.
- If any lines are returned, the Skill reports **FAIL** and shows the matching lines so the user can inspect them.

---

## Action: push

**Purpose:** Force-push the rewritten history to the remote. This is destructive on the remote and requires explicit confirmation.

**Usage:**
```bash
/git-rewrite-email push
```

**Behavior:**
1. The Skill asks: *“Are you sure you want to force-push the rewritten history to the remote? This will overwrite remote history. Type 'yes' to proceed.”*
2. If the user confirms with yes, the Skill runs:
   ```bash
   git push --force --all
   git push --force --tags
   ```
3. If the user answers anything else, the Skill aborts the push and reports that no push was performed.

**Safety reminder shown before push:**
- The Skill will remind the user to ensure collaborators are aware and to coordinate any necessary steps (e.g., informing team members, protecting important branches).

---

## Implementation details and argument handling

- **Argument mapping:** When the user runs `/git-rewrite-email rewrite old@example.com new@example.com`, the Skill treats:
  - `$1` → `old@example.com`
  - `$2` → `new@example.com`
- **Validation:** The Skill checks that both `$1` and `$2` are present and look like emails. If validation fails, the Skill explains the correct usage and stops.
- **Prompts:** All prompts (re-add origin, confirm push) are explicit yes/no prompts. The Skill will not proceed with the corresponding action unless the user replies with a clear yes.
- **OS/package manager note:** The Skill uses `sudo apt` to install `pipx` when missing. If the user's environment uses a different package manager (Homebrew, yum, pacman, etc.), the Skill will inform the user and provide the appropriate alternative command instead of attempting an incompatible install.

---

## Safety Rules (recap)

- No automatic pushes.
- No automatic remote re-add without confirmation.
- No rewrite without explicit `old-email` and `new-email`.
- The Skill stores and can restore the pre-rewrite origin URL only when the user agrees.
- The Skill installs `git-filter-repo` via `pipx` only when the user invoked rewrite and the tool is missing; it will attempt to install `pipx` only if necessary and will inform the user before performing system-level installs.

---

## Available actions (quick reference)

```bash
- rewrite old-email new-email   # rewrite history replacing old-email with new-email
- verify old-email              # search history for old-email
- push                          # force-push rewritten history (requires confirmation)
```

---

## Example full workflow (user interaction)

1. **User runs:**
   ```bash
   /git-rewrite-email rewrite daniel.personal@gmail.com 12345678+mycorx@users.noreply.github.com
   ```
2. **Skill installs prerequisites** if needed, captures origin, runs the rewrite, then asks:
   > “Do you want to re-add the origin remote that existed before the rewrite? (yes/no)”
3. **If user answers yes**, Skill restores origin:
   ```bash
   git remote add origin "https://github.com/yourorg/yourrepo.git"
   ```
4. **User runs:**
   ```bash
   /git-rewrite-email verify daniel.personal@gmail.com
   ```
   *Skill reports whether the email remains.*
5. **When ready, user runs:**
   ```bash
   /git-rewrite-email push
   ```
   *Skill asks for final confirmation and, if confirmed, force-pushes.*
