# StykAnimate MCP — community fork

[![License: GPL v3 or later](https://img.shields.io/badge/License-GPLv3%2B-blue.svg)](LICENSE)

Animate stick figures from **Claude Code or Codex** using a local MCP server.
This fork adds 13 tools for inspecting the editor, posing joints, creating
keyframes, duplicating figures and previewing your animation.

Maintained in [raju1979/StykAnimate-App](https://github.com/raju1979/StykAnimate-App).
Based on [StykAnimate by filippocalippo and its contributors](https://github.com/filippocalippo/StykAnimate-App).
This is an independent community fork; the upstream website, browser app and
download links below refer to the original project. **To use this fork's MCP
features, run this repository's desktop app from source.**

[Connect through MCP](#ai-control-with-mcp) · [Tool reference](docs/MCP.md#tools) · [License](#license)

<p align="center">
  <img src="assets/brand/svg/stykanimate-horizontal-primary.svg" alt="StykAnimate" width="420">
</p>

<p align="center">
  A free, open-source stick figure animation app for Windows, Linux and the browser.<br>
  Opens Pivot (<code>.piv</code>, <code>.stk</code>) and Stick Nodes projects, and keeps your work on your device.
</p>

<p align="center">
  <a href="https://stykanimate.com">Website</a> ·
  <a href="https://app.stykanimate.com">Open in the browser</a> ·
  <a href="https://github.com/filippocalippo/styk-animate-releases/releases/latest">Download</a> ·
  <a href="https://discord.gg/WTssQKBaJp">Discord</a> ·
  <a href="https://www.youtube.com/channel/UCZg7hZc23PNufzxnX8mCaCw">YouTube</a>
</p>

## Features

- **Pose stick figures** with fixed-length joints, IK for arms and legs, stretchy nodes and 3D figures with depth.
- **Build your own figures** from lines, circles, polygons, text and images, with a shared rig and per-part styles.
- **Two timelines:** a Simple, Pivot-style frame strip and an Advanced timeline with layers, folders, keys and easing.
- **Motion paths, onion skins and camera moves,** including camera follow.
- **Particle effects** (explosions, auras, beams, smoke) and brushes for drawing.
- **Community library:** insert figures and movieclips shared by other animators.
- **Export** MP4, WebM (with transparency), GIF, APNG, PNG/JPEG/SVG frames, and editable `.piv`/`.stk`.
- **27 interface languages**, light and dark themes, a touch interface for phones and tablets.

Everything (decoding, editing, saving and rendering) runs locally. Optional, consent-based diagnostics are described in [docs/telemetry.md](docs/telemetry.md).

## Get StykAnimate

- **Windows and Linux:** download Setup, Portable or the AppImage from the [releases page](https://github.com/filippocalippo/styk-animate-releases/releases/latest). Installed apps update themselves.
- **Browser:** open [app.stykanimate.com](https://app.stykanimate.com). It works offline once loaded.

## Build from source

Requirements: Node.js 22+ (CI uses 24.11) and Python 3.11+ (CI uses 3.12). Windows and Linux are tested on every change; macOS has an untested build target.

```sh
npm ci
python -m pip install -r backend/requirements-fast.txt
npm start            # desktop app (Electron)
npm run serve        # or the editor in your browser at http://127.0.0.1:47831
```

Set `STYK_PYTHON` to choose a Python interpreter. Optional native helpers speed up large imports (`npm run build:frame-scan`); the Python fallbacks always work.

### Optional Rust acceleration

Keep the existing app and enable its first Rust component: Pivot v5 frame
validation during import. Install Rust/Cargo 1.85+ and a platform linker
(Visual Studio C++ Build Tools with the Windows SDK on Windows), then run:

```sh
npm run build:frame-scan:rust
npm start
```

The app prefers the built Rust helper and retains C++ and Python fallbacks.
Rust is a build-time requirement; end users of a packaged build do not need
Cargo. The interface, MCP tools, animation model and undo history stay the same.
See [Rust setup, benchmarks and limitations](docs/RUST-ACCELERATION.md).

Packaging, on the matching operating system:

```sh
python -m pip install -r scripts/requirements-build.txt
npm run build:sidecar
npm run pack && npm run verify:package
npm run dist:win     # or dist:linux, dist:mac
```

The browser build: `npm run web:build`, then `npm run web:preview`. See [Web build](docs/WEB.md), [Linux](docs/LINUX.md) and [Touch interface](docs/TOUCH.md).

## AI control with MCP

The MCP server connects Claude Code or Codex to the **running desktop editor**.
Edits appear in the app and participate in its normal undo/redo history. The
connection stays on your computer; StykAnimate itself needs no AI API key.

### 1. Install this fork

Install Node.js 22+ (24.11 is used by CI), Python 3.11+, Git, and either the
[Codex CLI](https://developers.openai.com/codex/cli) or
[Claude Code](https://code.claude.com/docs/en/overview).

```sh
git clone https://github.com/raju1979/StykAnimate-App.git
cd StykAnimate-App
npm ci
python -m pip install -r backend/requirements-fast.txt
```

### 2. Configure the local connection

In **PowerShell**, from the repository root, run:

```powershell
$env:STYK_MCP_TOKEN = node -e "console.log(require('node:crypto').randomBytes(32).toString('hex'))"
$env:STYK_MCP = '1'
$env:STYK_MCP_PORT = '47832'
$env:STYK_STARTUP_VIEW = 'editor'
$mcpServer = Join-Path (Get-Location) 'scripts/mcp-server.mjs'
```

Then register **one or both clients in that same terminal**, before starting
the app. The absolute server path is calculated from your checkout location.

**Codex:**

```powershell
codex mcp add stykanimate --env "STYK_MCP_TOKEN=$env:STYK_MCP_TOKEN" --env "STYK_MCP_PORT=$env:STYK_MCP_PORT" -- node "$mcpServer"
```

**Claude Code:**

```powershell
claude mcp add --transport stdio --env "STYK_MCP_TOKEN=$env:STYK_MCP_TOKEN" --env "STYK_MCP_PORT=$env:STYK_MCP_PORT" stykanimate -- node "$mcpServer"
```

See [Linux/macOS commands and detailed setup](docs/MCP.md) for other shells.
The MCP client launches the stdio server automatically; do not start a second
`npm run mcp` process by hand.

### 3. Start animating

Start the desktop app **in the same PowerShell terminal**:

```powershell
npm start
```

Open or restart your chosen AI client, check that `stykanimate` is connected
(`codex mcp list` or Claude Code's `/mcp`), and try:

> Inspect my current StykAnimate project and show me a preview.

> Create a 24 FPS walk animation using the figure's hand and foot IK controls.

> Duplicate the figure, color the second one orange, and create a short sparring animation.

Keep the app open, pause playback, dismiss dialogs, and return to **Animate**
before asking the AI to edit. The tools act on the active project tab. Save your
project and export media through the app when you are happy with the result.

### Available tools

| Task | MCP tools |
| --- | --- |
| Inspect and preview | `get_project`, `get_figure`, `get_frame_preview` |
| Create and style a second figure | `duplicate_figure`, `set_figure_color` |
| Animate | `insert_keyframe`, `pose_joint`, `configure_ik`, `move_figure`, `seek`, `set_fps` |
| Correct edits | `undo`, `redo` |

See the [complete argument reference](docs/MCP.md#tools). This initial integration
does not create new rigs or expose saving and video export as MCP tools. The
browser app does not host this local MCP bridge.

### Connection help

- **Connection refused:** launch this fork with `STYK_MCP=1`; check that app and client use the same port.
- **Unauthorized:** app and client must use the same `STYK_MCP_TOKEN`.
- **Finish the current interaction:** pause playback, close dialogs, leave Home/Build and finish any drag.
- **New terminal or app restart:** restore the same token and environment settings before launching the app, or re-register the client with a newly generated token. The registration commands store the token in local client configuration; keep it private and out of Git.
- **PowerShell npm launcher error:** try `npm.cmd` in place of `npm`.

Automation is disabled by default. When enabled, the bridge listens only on
`127.0.0.1` and requires a token of at least 32 characters.

### Reproducible demos

```sh
node scripts/demo-mcp-walk.mjs          # 48-frame walk at 24 FPS
node scripts/demo-mcp-walk.mjs --fight  # 144-frame, two-figure sparring scene
```

These demos launch an isolated editor, author the animation through MCP, save
an editable project, capture previews and leave playback open. Use empty output
directories; [the demo guide](docs/MCP.md#walking-demo) explains file locations.

## Tests

```sh
npm run check        # syntax, stylesheet lint
npm test             # unit tests
python -m unittest discover -s backend -p "test_*.py"
```

Browser tests (`tests/*.browser.mjs`) run against `npm run serve` with Chrome or Playwright Chromium, for example `node tests/timeline.browser.mjs`. Some tests read a local corpus of Pivot projects that is not distributed; they skip and name the missing file. [`.github/workflows/ci.yml`](.github/workflows/ci.yml) shows what runs on every pull request.

## How the code is organized

| Location | Responsibility |
| --- | --- |
| `src/editor*.mjs`, `src/pivot/` | Document model, commands, undo/redo, rigs, native evaluation and validation |
| `src/renderer*.mjs`, `src/prepared-frames.mjs` | WebGPU rendering, incremental edits, GPU playback packets |
| `src/playback*.mjs`, `src/document-transport.mjs` | Worker evaluation, playback caches, bounded document snapshots |
| `src/import-*.mjs`, `backend/`, `codecs/`, `native/` | Pivot and Stick Nodes decoding, the Python codec service, native helpers |
| `src/document-export*.mjs`, `src/image-*.mjs`, `src/video-export.mjs` | Saving, recovery and image/video export |
| `src/app.mjs`, `src/shell/`, `src/ui/`, `src/styles/` | Interface, workspaces, timelines and styles |
| `src/locales/`, `src/localization*.mjs` | Translations |
| `desktop/` | Electron main process, preload bridge, file access, updates |
| `web/` | Browser build: service worker and Cloudflare configuration |
| `tests/`, `scripts/` | Tests, smoke tests, benchmarks, build and release scripts |
| `docs/` | Feature contracts and architecture |

Start with [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md). Feature behavior is documented in `docs/`, for example [the timeline](docs/TIMELINE.md), [scene editing](docs/SCENE-EDITING.md) and [the renderer](docs/renderer.md).

## Contributing

Bug reports, ideas, translations and pull requests are welcome. Read [CONTRIBUTING.md](CONTRIBUTING.md) first, and follow the [Code of Conduct](CODE_OF_CONDUCT.md). Report security problems privately as described in [SECURITY.md](SECURITY.md).

## License

This fork, including its MCP integration, is published under the
[GNU General Public License v3.0 or later (GPL-3.0-or-later)](LICENSE), preserving
the upstream project's licensing and attribution. StykAnimate is free software:
you can redistribute it and/or modify it under version 3 of the GPL or, at your
option, any later version. The original project is by
[filippocalippo and its contributors](https://github.com/filippocalippo/StykAnimate-App).
Bundled third-party components keep their own licenses; see
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

## Trademarks

The StykAnimate name and logo identify the official project. The GPL lets you fork and redistribute the code, but please give modified versions a different name and icon so people can tell them apart from official releases.

StykAnimate is an independent project. It is not affiliated with or endorsed by Pivot Animator or Stick Nodes; those names are used only to describe file compatibility.
