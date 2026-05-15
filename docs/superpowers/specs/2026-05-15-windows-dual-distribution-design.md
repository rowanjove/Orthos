# LintDrop dual Windows distribution design

Date: 2026-05-15

## Goal
Keep the existing installer for older or less-prepared Windows machines, and add a second Windows 10/11-friendly build that users can launch directly by double-clicking a single `.exe` file.

## Distribution shape
- Installer build: keep the current NSIS installer for users who need the app to help provision runtime dependencies.
- Portable build: publish the raw Windows executable as a separate download for Windows 10/11 users who want zero installation friction.

## Naming
- Installer: `LintDrop_<version>_x64-setup.exe`
- Portable executable: `LintDrop_<version>_x64_portable.exe`

## User-facing copy
- Installer: recommended for older systems or users who want the guided installation flow.
- Portable executable: for Windows 10/11 users; download and double-click to run.

## Implementation notes
- Reuse the existing Tauri release build output executable rather than changing app behavior.
- Preserve the current installer workflow.
- Update README/release guidance so the two downloads are visibly distinct.

## Verification
- Build the release executable.
- Confirm the executable exists and launches as the standalone app artifact.
- Confirm the installer path still remains available in the normal bundle output.
