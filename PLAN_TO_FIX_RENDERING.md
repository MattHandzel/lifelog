# Plan To Fix Tauri Rendering On Hyprland

## 1) Problem Statement

`npm run tauri dev` launches the app process, but the Tauri window is blank/white and eventually aborts with:

`Could not create default EGL display: EGL_BAD_PARAMETER. Aborting...`

This happens on laptop Wayland session (Hyprland), even when backend connectivity is working.

## 2) Current Environment Snapshot

- Host OS: NixOS 26.05
- Compositor: Hyprland 0.53.3
- Session vars observed: `XDG_SESSION_TYPE=wayland`, `WAYLAND_DISPLAY=wayland-1`, `DISPLAY=:0`
- XWayland: enabled (`hyprctl getoption xwayland:enabled -> int: 1`)
- GPU stack sanity: `eglinfo -B` succeeds outside dev shell for GBM/Wayland/X11 on Intel Mesa
- Repo state when reproducing: branch `main`, local dirty edits in Tauri + flake files

## 3) What Is Already Working

- Tauri Rust code compiles after config-related fixes.
- Interface process can connect to backend gRPC before rendering crash.
- Remote server connectivity can be tunneled and reached locally:
  - SSH tunnel created: `127.0.0.1:27183 -> server:127.0.0.1:7183`
  - `nc -vz 127.0.0.1 27183` passes

## 4) What Has Already Been Tried

### 4.1 Build/Dependency Fixes

- Added/adjusted Linux shared libs in `flake.nix` (webkit/gtk/libsoup/protobuf-related deps).
- Resolved compile errors from strict config changes in:
  - `interface/src-tauri/src/main.rs`
  - `interface/src-tauri/src/lib.rs`
  - `interface/src-tauri/src/bin/lifelog-server.rs`
  - `interface/src-tauri/src/bin/text-upload.rs`
- Added env-configurable gRPC endpoint via `LIFELOG_INTERFACE_GRPC_ADDR`.

### 4.2 Runtime Environment Variants Tested

Tried multiple launch-time env combinations; EGL crash remained:

- Backend selection:
  - `WRY_BACKEND=x11`
  - `GDK_BACKEND=x11`
  - `GDK_BACKEND=wayland`
- WebKit rendering toggles:
  - `WEBKIT_DISABLE_DMABUF_RENDERER=1`
  - `WEBKIT_FORCE_COMPOSITING_MODE=0`
  - `WEBKIT_DISABLE_COMPOSITING_MODE=1`
- GL fallback:
  - `LIBGL_ALWAYS_SOFTWARE=1`
- GIO modules:
  - cleared `GIO_EXTRA_MODULES` to remove gvfs warnings (warnings reduced, EGL crash unchanged)

### 4.3 Lockfile/Package Source Attempt

- Attempted `flake.lock` update to newer nixpkgs to pick up newer WebKitGTK.
- This caused rust toolchain fetch failures (`unknown source archive`) in current setup.
- Reverted lock update to restore working dev shell.

## 5) Research Findings (External)

- Similar `EGL_BAD_PARAMETER` failures are reported in Tauri/WRY/WebKitGTK Linux contexts.
- Known reports indicate some issues are version/interactions between:
  - WebKitGTK runtime
  - WRY/Tao integration
  - Wayland compositors/XWayland
- No single confirmed one-line fix yet for this exact Hyprland + pinned nixpkgs combo.

## 6) Main Hypotheses

H1. The crash is primarily in WebKitGTK/WRY runtime initialization on this compositor/runtime stack, not in our app logic.

H2. Our current nix shell is still missing one or more runtime components expected by WebKitGTK under Wayland/Hyprland.

H3. X11 fallback path (`WRY_BACKEND=x11`) is not actually providing a stable alternative in this session due to display/backend mismatch.

H4. The app may render white due to frontend dev server/path/CSP mismatch in some runs, but hard EGL abort indicates a lower-level graphics initialization blocker.

## 7) Gaps In Evidence

- We do not yet have a minimal, app-independent Tauri/WebKit smoke test run in the same shell/session to isolate app code vs platform/runtime.
- We have not captured GTK/WebKit debug logs with maximum verbosity focused on GL/EGL init path.
- We have not yet tested a pure web UI path (`npm run dev` only in browser) as a control baseline during same session.
- We have not yet pinned/compared specific WebKitGTK package versions in a controlled matrix without breaking toolchain.

## 8) Systematic Test Plan (Next)

## Phase A: Isolate App Code vs Runtime

1. Run web UI only (no Tauri) and verify normal rendering in browser.
2. Create/run minimal Tauri/Wry smoke app in same nix shell and Hyprland session.
3. If smoke app fails with same EGL error, classify as runtime/platform issue.

Exit criteria:
- If smoke app fails: stop app-level debugging and focus on runtime/deps strategy.
- If smoke app works: focus on project-specific Tauri config/CSP/frontend integration.

Phase A status: completed.
Outcome: web-only UI works, and app-independent WebKit smoke reproduces EGL failure, so issue is runtime/platform.

## Phase B: High-Signal Runtime Diagnostics

1. Run Tauri with expanded logging:
   - `RUST_LOG=trace`
   - relevant `WEBKIT_*`, `GDK_*`, `WAYLAND_DEBUG` knobs where feasible
2. Capture exact last successful initialization stage before EGL failure.
3. Verify loaded shared libraries (`ldd` on built binary, if practical).

Exit criteria:
- Clear failing subsystem identified (EGL display creation path, specific backend branch, or missing symbol/lib).

## Phase C: Backend/Display Matrix

Test matrix with explicit records for each run:

- Session backend:
  - Wayland
  - X11 via XWayland forcing
- Rendering mode:
  - default
  - software GL
  - dmabuf disabled
- Tauri mode:
  - `tauri dev`
  - built binary run (`cargo run`/`cargo build` artifacts)

Exit criteria:
- At least one stable rendering path OR all matrix rows fail consistently (confirming systemic runtime incompatibility).

## Phase D: Dependency Strategy

1. Create a safe branch for nixpkgs/WebKitGTK version experiments.
2. Upgrade only targeted graphics/webkit packages (not full lock churn if avoidable).
3. Re-test minimal smoke and full app after each dependency change.

Exit criteria:
- Identify version set where Tauri renders, then merge minimal dependency changes.

## 9) Contingency / Shortest Path If Blocked

If Tauri remains blocked on Hyprland in current stack:

- Use web interface workflow (`cd interface && npm run dev`) as immediate UI path.
- Keep server on remote host and point web UI/Tauri backend calls to remote endpoint.
- Track Tauri desktop runtime as a separate compatibility task with reproducible matrix results.

## 10) Tracking Template For Next Runs

Use this table for each attempted run:

| Date/Time | Command | Key Env Vars | Result | Error Signature | Notes |
|---|---|---|---|---|---|
| 2026-02-27 (local) | `cd interface && npm run dev -- --host 127.0.0.1 --port 1420` + `curl http://127.0.0.1:1420` | none | pass | none | Vite served HTML successfully; frontend content pipeline is healthy outside Tauri. |
| 2026-02-27 (local) | `cd interface && npm run tauri dev` | `LIFELOG_CONFIG_PATH=...` `LIFELOG_COLLECTOR_ID=laptop` `LIFELOG_INTERFACE_GRPC_ADDR=127.0.0.1:27183` | fail | `Could not create default EGL display: EGL_BAD_PARAMETER` | Logs also showed gRPC client initialized successfully before render abort. |
| 2026-02-27 (local) | standalone `webkit-smoke` (`gtk` + `webkit2gtk`) | `GIO_EXTRA_MODULES=` | fail | `Could not create default EGL display: EGL_BAD_PARAMETER` | Reproduces without lifelog code, isolating problem to WebKitGTK/EGL/runtime stack. |
| 2026-02-27 (local) | `webkit-smoke` matrix: wayland/x11/software/dmabuf variants | `GDK_BACKEND`, `WRY_BACKEND`, `LIBGL_ALWAYS_SOFTWARE`, `WEBKIT_DISABLE_DMABUF_RENDERER` | fail | `Could not create default EGL display: EGL_BAD_PARAMETER` | All tested combinations failed identically. |
| 2026-02-27 (local) | `webkit-smoke` extra toggles | `WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS=1`, mesa loader overrides | fail | `Could not create default EGL display: EGL_BAD_PARAMETER` | Sandbox toggle did not help; warning confirms `WEBKIT_FORCE_SANDBOX` no longer disables sandbox. |
| 2026-02-27 (local) | `gtk-smoke` (GTK only, no WebKit) | default session vars | pass | none | Simple GTK window rendered successfully on Hyprland (`GTK smoke OK`). |
| 2026-02-27 (local) | `webkit-smoke` inside vs outside `nix develop` | `GIO_EXTRA_MODULES=` | fail | `Could not create default EGL display: EGL_BAD_PARAMETER` | Same failure in both contexts; not nix-shell-only behavior. |
| 2026-02-27 (local) | `webkit-smoke` with EGL platform forcing | `EGL_PLATFORM=wayland/x11/surfaceless/device`, explicit Mesa vendor JSON | fail | `Could not create default EGL display: EGL_BAD_PARAMETER` | EGL platform hints did not change behavior. |
| 2026-02-27 (local) | `webkit-smoke` on nested X11 (`Xephyr :99`) | `DISPLAY=:99` | fail | `Could not create default EGL display: EGL_BAD_PARAMETER` | Still fails under nested X server. |

## 11) Definition Of Done

Done means all are true:

1. Tauri window renders non-white content reliably on Hyprland.
2. No EGL abort during launch or basic navigation.
3. Backend requests succeed to configured remote/local endpoint.
4. Repro commands documented and repeatable across fresh shell sessions.

## 12) Executed Results (2026-02-27)

### Phase B: High-Signal Diagnostics

- Ran Tauri with expanded diagnostics:
  - `RUST_LOG=trace RUST_BACKTRACE=1`
  - `GDK_DEBUG=gl-egl,gl-gles,opengl`
  - `LIBGL_DEBUG=verbose MESA_DEBUG=1`
  - `WEBKIT_DISABLE_DMABUF_RENDERER=1 WEBKIT_FORCE_COMPOSITING_MODE=0 WEBKIT_DISABLE_COMPOSITING_MODE=1`
  - `GIO_EXTRA_MODULES=`
- Result:
  - Tauri dev server started and Rust binary launched.
  - gRPC init executed (failed in this specific run due tunnel/backend state at that moment).
  - Rendering still aborted with identical signature:
    - `Could not create default EGL display: EGL_BAD_PARAMETER. Aborting...`

Additional runtime checks:
- `ldd` on `/tmp/webkit-smoke/target/debug/webkit-smoke` shows expected GTK/WebKit/EGL libs and no missing library lines.
- `libpango` and WebKitGTK are present (no `not found` linkage failure).

Inference:
- This is not a missing-shared-library issue anymore.
- Failure is occurring during EGL display initialization path inside WebKitGTK runtime.

### Phase C: Backend/Display Matrix (Executed)

Matrix executed against standalone `gtk + webkit2gtk` smoke app (no lifelog code):

- `baseline-wayland`
- `wayland-software` (`LIBGL_ALWAYS_SOFTWARE=1`)
- `wayland-dmabuf-off` (`WEBKIT_DISABLE_DMABUF_RENDERER=1` + compositing disables)
- `x11-forced` (`GDK_BACKEND=x11`, `WRY_BACKEND=x11`, `XDG_SESSION_TYPE=x11`)
- `x11-forced-software` (x11 + software GL)
- `wayland-explicit` (`GDK_BACKEND=wayland`)

Outcome:
- All matrix rows failed with the same error:
  - `Could not create default EGL display: EGL_BAD_PARAMETER. Aborting...`

Inference:
- Current runtime stack is systematically incompatible for WebKitGTK EGL init in this session.
- This is independent of app code and independent of basic Wayland/X11 toggle attempts.

## 13) Phase D: Next Experiments (Priority Order)

1. Run smoke app in a full native X11 login session (not Wayland + XWayland and not nested Xephyr).
   - Goal: verify if a true Xorg session avoids EGL failure.
2. Create a dedicated branch to test newer WebKitGTK/Nixpkgs inputs while pinning Rust toolchain compatibility.
   - Goal: find a package set where EGL init succeeds.
3. If available on this machine, test same smoke app under another compositor session (GNOME/KDE Wayland) to isolate Hyprland-specific interaction.
4. If still blocked, keep desktop app blocked and operate via web UI path (`npm run dev`) while tracking runtime compatibility as separate issue.

Current status after Phase D quick toggles:
- Still blocked on desktop Tauri rendering in this Hyprland setup.
- Practical path right now is web UI for local interface usage while continuing dependency/session compatibility testing.
- Strong evidence boundary:
  - GTK works.
  - WebKitGTK initialization fails at EGL display creation across all tested variants.

## 14) Resolution Path Executed

### Change Set

1. Upgraded Rust-side Tauri stack in `interface/src-tauri/Cargo.toml`:
   - `tauri` from beta to stable 2.x (`2.5.0` constraint, resolved to latest compatible 2.10.x in lock)
   - `tauri-plugin-shell` to stable 2.x
   - `tauri-plugin-dialog` to stable 2.x
2. Updated `flake.lock` nixpkgs input to a newer 2026-02 snapshot.
3. Removed `rust-overlay` input from flake and switched dev shells to nixpkgs-native Rust toolchain (`rustc`, `cargo`, `rust-analyzer`, `rustfmt`) to avoid broken rust-src archive fetches on updated nixpkgs.
4. Updated deprecated package names in `flake.nix` (`xorg.lib*` -> `libx11/libxi/libxtst`) and `LIBGL_DRIVERS_PATH` to `${pkgs.mesa}/lib/dri`.
5. Rebuilt interface binaries in the new shell to eliminate mixed-ABI runtime (`GLIBC_2.42 not found` was fixed by clean rebuild).

### Post-Change Verification

- Standalone `webkit-smoke` (gtk + webkit2gtk) no longer emits EGL error and remains running.
- `npm run tauri dev` result:
  - No `EGL_BAD_PARAMETER`.
  - frontend binary launches.
  - gRPC startup check succeeded when `LIFELOG_INTERFACE_GRPC_ADDR` includes scheme:
    - `LIFELOG_INTERFACE_GRPC_ADDR=http://127.0.0.1:27183`

Observed key log line:
- `[TAURI_MAIN] Initial gRPC client connection successful.`

### Required Runtime Notes

- `LIFELOG_INTERFACE_GRPC_ADDR` must include scheme (`http://...`) for tonic endpoint parsing.
- If `vite` port is occupied, stop stale process before relaunch (`pkill -x vite`).
