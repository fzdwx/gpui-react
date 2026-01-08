# SRC/RECONCILER - React Reconciler + FFI Bindings

**Scope:** src/reconciler only

## OVERVIEW

React-to-GPUI bridge: reconciler config, element store, Bun FFI bindings, event router. Manages React element tree → FFI
→ Rust GPUI.

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
├── event-router.ts    # Event handler routing (Map<elementId, Map<eventType, handlerId>>)
├── utils/
│   ├── logging.ts     # info, debug, trace utilities
│   └── plat.ts        # Platform detection for native lib path
└── __tests__/element-store.test.ts  # Manual test file
```

## WHERE TO LOOK

| Task             | File             | Notes                                                     |
| ---------------- | ---------------- | --------------------------------------------------------- |
| Public API       | index.ts         | createRoot(container)                                     |
| Reconciler hooks | host-config.ts   | appendChild, commitUpdate, resetAfterCommit               |
| Element store    | element-store.ts | Map<globalId, ElementData>, nextId starts at 2            |
| FFI bindings     | gpui-binding.ts  | dlopen, liveBuffers, renderFrame, batchElementUpdates     |
| Event handlers   | events.ts        | EventHandler, MouseEvent types                            |
| Event routing    | event-router.ts  | registerEventHandler, bindEventToElement, getEventHandler |

## CONVENTIONS

- **Sync to Rust:** Call batchElementUpdates() in resetAfterCommit, then renderFrame()
- **Event handler IDs:** registerEventHandler() returns incrementing ID stored in eventHandlerMap
- **Buffer lifetime:** Push all ArrayBuffers to liveBuffers[] before FFI calls
- **Root detection:** First child appended to container becomes root (rootId)
- **Element IDs:** CreateElement returns nextId++, starts at 2
- **Style props:** Extract via extractStyleProps(), map to GPUI format via mapStyleToProps()
- **Event binding:** bindEventToElement(elementId, eventType, handlerId) stores mapping in elementEventMap

## ANTI-PATTERNS (THIS PROJECT)

- Don't skip batchElementUpdates after updates - children won't sync to Rust
- Don't create element with ID 1 - elementStore starts IDs at 2
- Don't let FFI buffers be GC'd - push to liveBuffers array
- Don't use className - use style prop (warns if used)
- Don't return false from shouldSetTextContent for text children
- Don't forget to queueElementUpdate after any DOM mutation
- Don't skip bindEventToElement after registering handlers - handlers won't fire

## KEY IMPLEMENTATIONS

- **pendingUpdates array:** Batches DOM mutations, processed in resetAfterCommit
- **extractEventHandlers:** Registers handlers, returns Map of eventName → handlerId
- **resetAfterCommit:** Processes pendingUpdates → batchElementUpdates → renderFrame
- **liveBuffers pattern:** Prevents GC during blocking FFI calls
- **Event router:** Two-level Map for element → eventType → handlerId routing
