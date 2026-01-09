# SRC/CORE - FFI Abstraction Layer

**Scope:** src/core only

## OVERVIEW

Bun FFI abstraction for GPUI native library: RustLib class manages window creation, element updates, and event
callbacks. FfiState handles buffer lifetime management.

## STRUCTURE

```
src/core/
├── ffi.ts            # Bun FFI function signatures (dlopen, symbols)
├── ffi-state.ts      # FfiState class (liveBuffers, encodeCString, createInt64)
├── rust.ts           # RustLib class (createWindow, batchElementUpdates, renderFrame)
└── index.ts          # Public exports (rustLib singleton)
```

## WHERE TO LOOK

| Task          | File         | Notes                                                                        |
| ------------- | ------------ | ---------------------------------------------------------------------------- |
| FFI bindings  | ffi.ts       | dlopen, FFIType, function signatures for all GPUI exports                    |
| Buffer state  | ffi-state.ts | FfiState.liveBuffers prevents GC, encodeCString/createInt64                  |
| Main API      | rust.ts      | RustLib class: createWindow, batchElementUpdates, renderFrame, triggerRender |
| Public export | index.ts     | rustLib singleton export                                                     |

## CONVENTIONS

- **Buffer lifetime:** FfiState.keep() before FFI calls, clear() after
- **Window management:** RustLib.ffiStateMap stores FfiState per windowId
- **Event setup:** JSCallback with threadsafe:true for Rust→JS events
- **Error handling:** checkResult() and checkWindowCreateResult() throw on failure
- **Ready check:** waitReady() polls gpui_is_ready() with exponential backoff
- **Focus/hover:** Event callback receives JSON, parses, routes via event-router

## ANTI-PATTERNS (THIS PROJECT)

- Don't skip FfiState.keep() before FFI calls - buffers get GC'd
- Don't forget to call clear() after each operation - memory leak
- Don't use gpui_render_frame for batch updates - use batchElementUpdates

## KEY PATTERNS

- **Result buffer:** 16-byte status+error_ptr structure for FFI results
- **Window creation:** options JSON → encodeCString → gpui_create_window → extract windowId
- **Batch updates:** elements JSON → encodeCString → gpui_batch_update_elements
- **Event callback:** JSCallback(threadsaf:true) receives JSON from Rust, parses, routes via event-router
- **Focus/hover routing:** Events dispatched via event-router based on element ID and event type
