---
description: Full pipeline - test, build, commit, PR, and merge in one shot
---

# Yolo Workflow

> Runs the full pipeline automatically. Supports three modes:
>
> `/yolo local` — 🧪 Test → 🔨 Build → ✅ Verify Local **(STOP)**
> `/yolo deploy` — 📝 Commit → 📝 PR → 🔀 Merge → 📦 Publish Extension **(use after `/yolo local`)**
> `/yolo` — 🧪 Test → 🔨 Build → 📝 Commit → 📝 PR → 🔀 Merge → 📦 Publish Extension

// turbo-all

---

## `/yolo local` — Verify Only

1. **Lint**:

   ```bash
   cargo clippy --workspace -- -D warnings
   ```

2. **Format**:

   ```bash
   cargo fmt --all
   ```

3. **Test**:

   ```bash
   cargo test --workspace
   ```

4. **Report** results to user. **STOP HERE.**

---

## `/yolo deploy` — Commit + PR + Merge

> Use this after `/yolo local` has passed.

5. **Check branch** (never commit to main):

   ```bash
   git branch --show-current
   ```

6. If on `main`, create a feature branch:

   ```bash
   git checkout -b feat/<descriptive-name>
   ```

7. **Stage and commit**:

   ```bash
   git add -A
   git commit -m "<type>(<scope>): <description>"
   ```

8. **Push**:

   ```bash
   git push -u origin HEAD
   ```

9. **Create PR** using GitKraken MCP:
   - `provider`: github
   - `source_branch`: current branch
   - `target_branch`: main
   - Title in conventional format
   - Body summarizing changes + test results

10. **Merge PR** and clean up:

    ```bash
    gh pr merge <PR_NUMBER> --merge --delete-branch
    ```

11. **Sync main**:

    ```bash
    git checkout main
    git pull origin main
    ```

12. **Build & Publish VS Code extension** (if `fd-vscode/` was changed):

    ```bash
    cd fd-vscode && pnpm run compile
    ```

    Then publish to both registries:

    ```bash
    cd fd-vscode && pnpm vsce publish && pnpm ovsx publish
    ```

    > Skip publish if the change is local-only or version wasn't bumped.

13. Report PR URL, merge status, and publish results to user.

---

## `/yolo` — Full Pipeline

Runs **all steps 1–13** in sequence (local + deploy).
