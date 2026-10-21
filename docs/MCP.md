# Claude Code and Codex integration

The desktop app offers an experimental local MCP integration. Start the app
with automation enabled, then connect a stdio MCP client. The same server works
with Claude Code and Codex. The browser editor does not expose a local server.

## Start the app

Install the normal source prerequisites from README.md first. In PowerShell,
from the repository root:

```powershell
npm ci
$env:STYK_MCP_TOKEN = node -e "console.log(require('node:crypto').randomBytes(32).toString('hex'))"
$env:STYK_MCP = '1'
$env:STYK_MCP_PORT = '47832'
$env:STYK_STARTUP_VIEW = 'editor'
npm start
```

Keep the token available to the client process too. The app binds only to
127.0.0.1 and requires the token for every command. Automation is disabled unless
STYK_MCP=1. Do not publish the token or commit it in configuration. Each running
app needs its own port; configure its client with the matching port and token.
The server operates on that app's active project tab.

## Connect a client

Run one of these commands with your actual absolute checkout path:

```powershell
codex mcp add stykanimate --env "STYK_MCP_TOKEN=$env:STYK_MCP_TOKEN" --env "STYK_MCP_PORT=47832" -- node C:/path/to/StykAnimate-App/scripts/mcp-server.mjs
```

```powershell
claude mcp add --transport stdio --env "STYK_MCP_TOKEN=$env:STYK_MCP_TOKEN" --env "STYK_MCP_PORT=47832" stykanimate -- node C:/path/to/StykAnimate-App/scripts/mcp-server.mjs
```

These commands store the token in the client's local configuration. Use the
same token on subsequent app launches, or update the client configuration when
rotating it. Restart/reconnect the client after configuration changes. No AI
API key is required by StykAnimate; the chosen client supplies the model.

Try: "Inspect my StykAnimate project, move the first figure 20 canvas units to
the right, and show me a preview."

## Tools

| Tool | Arguments | Behavior |
| --- | --- | --- |
| get_project | none | Canvas dimensions, FPS, tick, dirty state, keyframe count, up to 1000 visible-frame instances |
| get_frame_preview | none | PNG of the app window, including editor overlays, scaled to at most 1600 pixels on the longest edge |
| seek | tick | Seek using zero-based ticks, clamped by the editor |
| move_figure | uid, dx, dy | One undoable translation in canvas units at the current tick; joined figures can move together |
| set_fps | fps | One undoable project FPS change (1–1000) |
| undo | none | Undo the last editor command, including a manual command |
| redo | none | Redo the last undone command |

Commands reject while the editor is starting, playing, showing Home or a dialog,
building a figure, or handling a drag. Close overlays and return to Animate first.
This version does not create rigs, author joint poses or keyframes, save projects,
or export movies; use the app for these operations. A translation is the first
supported pose edit. Saving remains an explicit manual action.

## Implementation and verification

scripts/mcp-server.mjs owns MCP schemas and stdio transport. desktop/automation.cjs
owns the authenticated HTTP endpoint and request/reply IPC. The preload exposes
only the two automation channels. src/automation.mjs validates commands and calls
the existing Editor methods; it never accepts JavaScript or arbitrary IPC names.
src/app.mjs owns interaction gating and render scheduling. Electron captures the
window for previews so WebGPU readback does not depend on canvas preservation.

The bridge accepts one request at a time, caps request bodies at 16 KiB, and
rejects browser Origin headers. Renderer replies must originate from the target
window's main frame. A timed-out edit has an uncertain outcome: inspect the
project before retrying it. This interface assumes one human/AI editor at a time;
it does not provide transactions across multiple tool calls.

```powershell
npm run test:mcp
npm run test:mcp:desktop
npm run check
```

The desktop smoke test launches an isolated profile and a real stdio MCP client,
checks discovery, inspects a live project, translates a figure, verifies undo/redo,
changes FPS and requests a PNG. The app's usual Python and Electron prerequisites
must be installed.
