# Verification · 2026-10-06

## Passed

- `npm test`: 7 tests. Fixed/running merge, executable/AUMID aliases, shortcut target matching, independent Shell identities, filtering, stable ordering.
- `npm run build`: Vue TypeScript checking and Vite production build.
- `cargo test --manifest-path src-tauri/Cargo.toml --offline -- --nocapture`: 5 tests. Identity normalization, double-Win taps/chords, conditional restoration, Windows adapter integration.
- Native integration: 1 monitor detected; this execution environment exposed 0 eligible application groups. Explorer icon extracted and PNG decoded at 64×64. Real temporary Shell shortcut created, parsed and removed; executable target matched without launching it.
- `npm run tauri build`: optimized Windows executable and NSIS x64 installer built successfully.
- Browser UI: collapse/expand, theme change, Switch binding, saved/dirty state, search panel, preview-only error message and dismissal checked through the actual UI.
- Independent code review findings addressed: watchdog/startup restoration race, Explorer restart retry, executable/Shell identity mismatch, draft preservation, menu clipping, dynamic hit regions and DPI invalidation.

## Artifacts

- `src-tauri/target/release/desktop-island.exe`
- `src-tauri/target/release/bundle/nsis/Desktop Island_0.1.0_x64-setup.exe`
- `artifacts/settings-preview.jpg`, `artifacts/island-preview.jpg`, `artifacts/dock-preview.jpg`

## Manual acceptance still required

The build and read-only checks do not validate every desktop integration scenario. Test activation/minimization with ordinary and elevated applications; multiwindow selection; taskbar and icon visibility followed by normal exit/crash; Explorer restart; display unplug/DPI changes; exclusive fullscreen games. Weather is disabled by default and its live network request was not part of this verification. Browser screenshots show UI preview, not Windows system control results.

Music, downloads, device event animations, Workspace, plugins and AI remain the later versions described in the shared conversation.
