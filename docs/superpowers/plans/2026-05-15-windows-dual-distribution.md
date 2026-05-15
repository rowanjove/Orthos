# Windows Dual Distribution Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Keep the existing installer release path and add a clearly named standalone Windows executable for Win10/11 users.

**Architecture:** Reuse the existing Tauri release executable as the portable artifact, preserve the NSIS installer build, and update the user-facing docs so both downloads are easy to distinguish. No runtime behavior changes are required.

**Tech Stack:** Tauri 2.x, Rust, Windows release artifacts, Markdown docs

---

### Task 1: Clarify download options in the README

**Files:**
- Modify: `README.md`

- [ ] **Step 1:** Replace the single-installer wording with two Windows download options: installer for older systems / guided setup, portable `.exe` for Windows 10/11 double-click use.
- [ ] **Step 2:** Keep the current installer filename and add the new portable filename convention.
- [ ] **Step 3:** Review the surrounding copy to make sure the distinction is immediately visible.

### Task 2: Produce the portable executable artifact

**Files:**
- Read from: `src-tauri/target/release/lintdrop.exe`
- Create: `dist/LintDrop_1.0.3_x64_portable.exe`

- [ ] **Step 1:** Run the release build command that preserves the installer output and also refreshes the raw executable.
- [ ] **Step 2:** Copy the raw release executable into `dist/` with the portable filename.
- [ ] **Step 3:** Confirm both the portable executable and installer artifact exist after the build.

### Task 3: Verify and commit

**Files:**
- Modify: `README.md`
- Create: `dist/LintDrop_1.0.3_x64_portable.exe`

- [ ] **Step 1:** Run a fresh build verification command and inspect the outputs.
- [ ] **Step 2:** Check git status so only intentional repo changes are staged.
- [ ] **Step 3:** Commit the documentation update; keep the generated executable available for delivery but do not commit it unless explicitly desired.
