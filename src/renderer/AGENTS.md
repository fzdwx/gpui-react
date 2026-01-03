# SRC/RENDERER - React Reconciler + FFI Bindings

**Generated:** 2025-01-03 16:30:00
**Branch:** main

## OVERVIEW
React-to-GPUI bridge: reconciler config, element store, Bun FFI bindings (6 files).

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| Public API | src/renderer/index.ts | createRoot(), init/createWindow calls |
| Reconciler host config | src/renderer/host-config.ts | appendChild/updateElement sync, resetAfterCommit → renderFrame |
| Bun FFI bindings | src/renderer/gpui-binding.ts | renderFrame serializes to pointers, updateElement via JSON |
| Element data store | src/renderer/element-store.ts | Map<id, ElementData>, rootId tracking, IDs start from 2 |
| Style mapping | src/renderer/styles.ts | parseColor/parseSize, mapStyleToProps converts to GPUI format |
| Event types | src/renderer/events.ts | EventHandler, MouseEvent definitions |

## CONVENTIONS
- **Sync children to Rust:** Call updateElement() after appendChild/appendChildToContainer
- **Event handler IDs:** Incrementing counter stored in Map<number, EventHandler>
- **Buffer lifetime:** FFI buffers must stay in liveBuffers[] during calls
- **Root detection:** First child appended to container becomes root (setContainerChild)
- **Reconciler lifecycle:** resetAfterCommit gets root → calls renderFrame → gpui_render_frame

## ANTI-PATTERNS (THIS LAYER)
- Don't skip updateElement after appendChild - children won't sync to Rust
- Don't create root element with ID 1 - elementStore IDs start from 2
- Don't let FFI buffers be GC'd before call returns - push to liveBuffers
- Don't use className prop - warn, must use style prop instead
- Don't render directly to GPU - always go through reconciler → elementStore → renderFrame
