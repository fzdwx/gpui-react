# SRC/RENDERER - React Reconciler + FFI Bindings

**Scope:** src/renderer only

## OVERVIEW
React-to-GPUI bridge: reconciler config, element store, Bun FFI bindings.

## STRUCTURE
```
src/renderer/
├── index.ts            # Public API (createRoot, init, createWindow)
├── host-config.ts      # React reconciler host config
├── element-store.ts     # JS-side element data store
├── gpui-binding.ts      # Bun FFI bindings to Rust
└── events.ts           # Event type definitions
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| Public API | index.ts | createRoot(), sendHostCommand() |
| Reconciler | host-config.ts | appendChild, commitUpdate, resetAfterCommit |
| FFI bindings | gpui-binding.ts | dlopen, FFIType, renderFrame |
| Element store | element-store.ts | Map<id, ElementData>, IDs start from 2 |

## CONVENTIONS
- **Sync children to Rust:** Call sendHostCommand(TriggerRender) after batch updates
- **Event handler IDs:** Incrementing counter in Map<number, EventHandler>
- **Buffer lifetime:** FFI buffers must stay in liveBuffers[] during calls
- **Root detection:** First child appended to container becomes root

## ANTI-PATTERNS (THIS PROJECT)
- Don't skip sendHostCommand after update - children won't sync to Rust
- Don't create root element with ID 1 - elementStore IDs start from 2
- Don't let FFI buffers be GC'd before call returns - push to liveBuffers
- Don't use className prop - use style prop instead