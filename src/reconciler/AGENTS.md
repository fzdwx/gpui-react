# SRC/RECONCILER - React Reconciler + FFI Bindings

**Scope:** src/reconciler only

## OVERVIEW
React-to-GPUI bridge: reconciler config, element store, Bun FFI bindings. Manages React element tree → FFI → Rust GPUI.

## STRUCTURE
```
src/reconciler/
├── index.ts            # Public API (createRoot)
├── host-config.ts      # React reconciler host config (431 lines)
├── element-store.ts    # JS-side element data store (79 lines)
├── gpui-binding.ts     # Bun FFI bindings (226 lines)
├── reconciler.ts       # Reconciler instance
├── renderer.ts         # Renderer instance
├── ctx.ts             # Context utilities
├── events.ts          # Event type definitions
├── styles.ts          # Style mapping utilities
└── __tests__/element-store.test.ts  # Manual test file
```

## WHERE TO LOOK
| Task | File | Notes |
|------|------|-------|
| Public API | index.ts | createRoot(container) |
| Reconciler hooks | host-config.ts | appendChild, commitUpdate, resetAfterCommit |
| Element store | element-store.ts | Map<globalId, ElementData>, nextId starts at 2 |
| FFI bindings | gpui-binding.ts | dlopen, liveBuffers, renderFrame, batchElementUpdates |
| Event handlers | events.ts | EventHandler, MouseEvent types |

## CONVENTIONS
- **Sync to Rust:** Call batchElementUpdates() in resetAfterCommit, then renderFrame()
- **Event handler IDs:** registerEventHandler() returns incrementing ID stored in Map
- **Buffer lifetime:** Push all ArrayBuffers to liveBuffers[] before FFI calls
- **Root detection:** First child appended to container becomes root (rootId)
- **Element IDs:** CreateElement returns nextId++, starts at 2
- **Style props:** Extract via extractStyleProps(), map to GPUI format via mapStyleToProps()

## ANTI-PATTERNS (THIS PROJECT)
- Don't skip batchElementUpdates after updates - children won't sync to Rust
- Don't create element with ID 1 - elementStore starts IDs at 2
- Don't let FFI buffers be GC'd - push to liveBuffers array
- Don't use className - use style prop (warns if used)
- Don't return false from shouldSetTextContent for text children
- Don't forget to queueElementUpdate after any DOM mutation

## KEY IMPLEMENTATIONS
- **pendingUpdates array:** Batches DOM mutations, processed in resetAfterCommit
- **extractEventHandlers:** Registers handlers, returns Map of eventName → handlerId
- **resetAfterCommit:** Processes pendingUpdates → batchElementUpdates → renderFrame
- **liveBuffers pattern:** Prevents GC during blocking FFI calls