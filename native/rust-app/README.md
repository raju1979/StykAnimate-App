# Native Rust editor preview

This is the standalone native editor preview in the full Rust rewrite.
It uses Rust, egui and OpenGL, with no Electron, Node.js or Python at runtime.
**It is not a feature-complete replacement for StykAnimate.** The target remains
full feature parity; see the [acceptance matrix](../../docs/RUST-APP-PARITY.md).

The current editor supports multiple 2D stick figures, line/circle construction,
joint posing with two-bone IK, colors, independent figure layers, Camera and
Scene tracks, linear/hold evaluation, playback, onion skins, undo/redo, atomic project saving, and
PNG/SVG/GIF exports. GIF encoding runs in a cancellable background task.

Preview projects use **`.rustanim.json`**, a separate versioned format. The
preview cannot open `.styk`, `.stkz`, `.stykmov`, Pivot or Stick Nodes files.
It refuses to save to their extensions. Continue using the existing app for
those projects, audio, effects, advanced property timelines, movieclips and 3D rigs.

## Workspace and tracks

The light workspace follows the original app's arrangement: document toolbar,
stage controls, floating selection/build/pan tools, properties on the right,
and an exposure timeline below. Use **Fit** to reset viewport pan and zoom;
viewport navigation does not change the exported camera.

- **Camera**: select its timeline row, then edit center X/Y, zoom or rotation
  in Properties. Insert keys at different times to animate the camera. Camera
  evaluation affects the stage, PNG preview, SVG and GIF exports.
- **Scene**: select its row to edit the background color, duration and frame
  rate. Background colors hold until the next Scene key.
- **Figure layers**: use **+** beside Layers to add one. Select a layer row to
  rename it, add figures, or delete it. Use the eye/lock controls to toggle
  visibility and editing. Visibility also affects exports in this preview.
- **Figure properties**: select a joint/origin on the stage or a figure in its
  layer's Properties. Change position/color, duplicate it, move its complete
  animation to another layer, or move its layer forward/back.
- **Keys**: Keyframe, Blank and Delete key operate on the selected track.
  Blank creates an empty figure-layer exposure; the opening key is retained.
  Locked tracks reject edits. All document changes support undo/redo.

New projects have one constant opening key on Camera, Scene and Layer 1,
held for ten frames. Extend Duration in Scene properties to animate longer.
Layer folders, property channels, easing and range retiming remain pending.

Preview format version 2 stores independent tracks. Version 1 preview files
are migrated on opening with their figure poses and background retained;
version 2 saves require this updated preview.

## Build and run

Use Rust 1.85.1 or newer and platform development libraries. On Windows, install
Visual Studio C++ Build Tools and a Windows SDK. The lockfile is checked in.

```powershell
cd native/rust-app
cargo build --release --locked
.\target\release\stykanimate-rust-preview.exe
# Start an editable walking animation:
.\target\release\stykanimate-rust-preview.exe --walk
```

For a Windows build with the C runtime linked statically:

```powershell
$env:RUSTFLAGS = '-C target-feature=+crt-static'
cargo build --release --locked --target x86_64-pc-windows-msvc
```

The executable is then under `target/x86_64-pc-windows-msvc/release/`.
Windows x64 is the currently exercised target; Linux/macOS builds and packaging
remain unverified.

## Connect an MCP client

The same executable has two modes: the visible editor, and a stdio MCP server
started with `--mcp`. The server connects to an already running native editor.
The editor bridge is disabled by default and binds only to `127.0.0.1`.

Generate a private token of 32–128 ASCII characters. Set the same token and port
in the editor and MCP client environments. Keep tokens out of source control.
Use a different port from the Electron MCP app if both are running.

```powershell
$env:STYK_MCP = '1'
$env:STYK_MCP_PORT = '47833'
$env:STYK_MCP_TOKEN = 'REPLACE_WITH_YOUR_PRIVATE_RANDOM_TOKEN'
.\stykanimate-rust-preview.exe
```

For a client accepting an `mcpServers` JSON configuration:

```json
{
  "mcpServers": {
    "stykanimate-rust": {
      "command": "C:/absolute/path/stykanimate-rust-preview.exe",
      "args": ["--mcp"],
      "env": {
        "STYK_MCP_PORT": "47833",
        "STYK_MCP_TOKEN": "REPLACE_WITH_YOUR_PRIVATE_RANDOM_TOKEN"
      }
    }
  }
}
```

For a client accepting TOML server configuration:

```toml
[mcp_servers.stykanimate-rust]
command = 'C:\absolute\path\stykanimate-rust-preview.exe'
args = ['--mcp']

[mcp_servers.stykanimate-rust.env]
STYK_MCP_PORT = '47833'
STYK_MCP_TOKEN = 'REPLACE_WITH_YOUR_PRIVATE_RANDOM_TOKEN'
```

The server exposes 27 tools: `get_project`, `get_figure`, `get_frame_preview`,
`add_figure`, `duplicate_figure`, `set_figure_color`, `move_figure`, `pose_joint`,
`configure_ik`, `insert_keyframe`, `seek`, `set_fps`, `save_project`, `undo`,
`redo`, `add_layer`, `select_track`, `set_layer`, `delete_layer`, `move_to_layer`,
`reorder_layer`, `delete_figure`, `delete_keyframe`, `blank_keyframe`,
`set_camera`, `set_scene`, and `set_track_lock`. Query tool schemas for arguments. Ticks are zero-based; returned
joint IDs are one-based. Key operations affect the selected track. Figure and joint
identities belong to this preview and differ from existing app identities.

Edits are undoable and run through the editor command layer. Invalid edits
restore the prior document. Commands reject during a manual drag, playback or
unsaved-change prompt. If a request times out, inspect the project before
retrying an edit. The token grants access to the local project and saving
files under the user account; give it only to trusted clients.

Example prompt: “Inspect the scene, duplicate Blue, color the copy red, move it
200 units right, add pose keys at ticks 0 and 12, and preview the result.”

## Verification

```powershell
cd native/rust-app
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo fmt --check
cd ../..
node scripts/smoke-rust-app.mjs
# Or pass a release executable path as the first argument.
```

The smoke script starts a separate GUI and authenticated stdio client, then
checks live edits, undo/redo, duplication, posing, invalid-edit rollback,
independent layers, track keys, locks, visibility, save output, PNG preview,
and rejection of a wrong token. Its disposable test
files go under `artifacts/rust-app-smoke/`.

For an application-owned screenshot without UI automation:

```powershell
.\stykanimate-rust-preview.exe --walk --screenshot C:\absolute\path\preview.png
```

## License

GPL-3.0-or-later, consistent with the repository's [LICENSE](../../LICENSE).
Upstream StykAnimate copyright and attribution remain in effect. Dependency
licenses are separate; see [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
