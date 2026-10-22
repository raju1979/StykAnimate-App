# Full native Rust rewrite: acceptance matrix

The requested end state is full feature parity with the existing StykAnimate
app, including import compatibility, audio, effects and exports. A native
window or a subset of animation tools does not satisfy that end state.

`native/rust-app` is an independently runnable preview. It does not replace the
Electron app. The optional Rust PIV frame scanner is a separate acceleration
component and also does not constitute a native app rewrite.

## Current status and acceptance gates

| Area | Native preview today | Required before claiming parity |
| --- | --- | --- |
| Native runtime | Rust/egui/OpenGL editor, Windows x64 exercised | Supported platform builds, installers and release checks |
| Scene model | Independent layers with 2D figures and ordered line/circle bones | Native IDs, joins, definitions/instances, all shape/media records, unknown-field preservation |
| Posing and builder | FK rotation, two-bone IK, simple line/circle addition | Full rig constraints, pin/stretch semantics, shared-definition edits, all builder/3D commands |
| History | Validated commands, undo/redo, one record per drag | Affected-state history, dense-scene budgets and all command parity |
| Timeline | Independent layer/Camera/Scene keys, blank layer exposures, locks, hold/linear samples, playback | Folders, focus, property tracks, easing, retiming, range edits and all original animation modes |
| Camera and composition | Keyed camera center/zoom/rotation and held Scene background colors; applied to stage and exports | Original scene transforms, clipping, opacity and complete compositing |
| Rendering | Simple solid primitives, onions and frame preview | Artwork/media, text, outlines, gradients, fills, VFX, 3D and image comparison fixtures |
| Persistence | Atomic `.rustanim.json` v2 saves, v1 preview migration, validation, save-before-close | Native STYK/STKZ/STYKMOV v1/v2/legacy, provenance, unknown metadata, recovery, large-file limits, failure preservation |
| Pivot compatibility | No import/export yet | PIV/STK versions and fixtures, joins, images, text, opaque records, unchanged byte preservation and edited export |
| Stick Nodes | No import/export yet | Supported original Stick Nodes versions/media and cross-app round trips |
| Movieclips | Not implemented | Nested editable clips, timing, playback and native persistence |
| Audio and drawing | Not implemented | Audio tracks, waveform/sync/export; editable drawings and their persistence |
| Exports | Basic PNG/SVG and streaming cancellable GIF | All original image/video/document exports, audio, timing/color/camera fidelity and bounded cancellation |
| Workspace | One document, light styled egui workspace, figure/Camera/Scene properties, viewport pan/zoom and file dialogs | Home/library/folders, project tabs, recovery, preferences, localization, accessibility, complete keyboard/touch behavior and tutorials |
| MCP | 27 native preview tools, independent track operations, authenticated local bridge | Existing tool semantics applied to original documents, cross-client regression and advanced operations as their owning features land |
| Performance | No overall speed claim | Same-scene measurements of interaction latency, memory, preparation and export, with correctness comparisons |

## Order of implementation

1. Establish the native editor, command boundary and MCP test harness (current
   preview). Keep incompatible preview data explicitly separate.
2. Port the authoritative editable report schema and codec contracts. Preserve
   identities and opaque data before exposing existing projects to editing.
   Reuse the existing format fixtures as differential tests. Never flatten an
   original document into preview bones and silently discard unsupported data.
3. Port layer/property evaluation, animation modes, joins, shared definitions,
   constraints, builder and camera behavior. Run the same command scenarios
   against both implementations and compare evaluated records.
4. Port rendering, media, drawings, movieclips, effects, audio and export.
   Compare frame images, timing, saved semantic records and failed/cancelled
   operation behavior on representative documents.
5. Port workspace features, recovery, platform integration, localization and
   accessibility; package and verify supported platforms. Measure identical
   dense workloads before claiming a performance improvement.

## Authoritative contracts

- [Architecture](ARCHITECTURE.md), [scene editing](SCENE-EDITING.md), and
  `src/pivot/`: editable semantics, identities and geometry.
- [Timeline](TIMELINE.md), [animation modes](animation-modes.md),
  [layer folders](LAYER-FOLDERS.md): exact exposure and channel semantics.
- [Import pipeline](IMPORT-PIPELINE.md), [document exports](document-exports.md),
  `backend/` and `codecs/`: original source retention, limits and round trips.
- [Renderer](renderer.md), [adaptive preview](ADAPTIVE-PREVIEW.md): visual and
  lifetime contracts; caches remain derived data.
- [Home](HOME.md), [project tabs](PROJECT-TABS.md), [desktop releases](DESKTOP-RELEASE.md):
  workspace and delivery contracts.
- Existing tests in `tests/`, `backend/` and `scripts/` remain the behavioral
  baseline. New Rust tests must establish equivalent outcomes, not only that
  the new implementation agrees with its own simplified model.

The matrix records substantial unfinished work. It is not a release checklist
whose completion can be inferred from a successful build or one MCP demo.
