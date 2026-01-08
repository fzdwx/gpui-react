# RUST/SRC - Rust FFI Library

**Scope:** rust/src only

## OVERVIEW

Rust FFI library for GPUI integration - handles React element tree rendering to GPU-accelerated UI via async_channel
command bus.

## STRUCTURE

```
rust/src/
├── lib.rs              # FFI exports (gpui_init, gpui_create_window, gpui_batch_update_elements)
├── ffi_helpers.rs      # FFI helper functions (ptr_to_u64, read_c_string, etc.)
├── renderer.rs         # RootView, render_element_to_gpui
├── element/            # CSS-capable element implementations
│   ├── mod.rs          # ElementKind, ReactElement, ElementStyle (CSS property structs)
│   ├── div.rs          # ReactDivElement with full CSS support
│   ├── span.rs         # ReactSpanElement with CSS support
│   ├── text.rs         # ReactTextElement for text nodes
│   └── img.rs          # ReactImgElement for images
├── host_command.rs     # async_channel command bus (TriggerRender, UpdateElement, BatchUpdateElements)
├── window.rs           # Window, WindowState, element tree management
├── global_state.rs     # Global state management (lazy_static)
├── ffi_types.rs       # FFI type bindings (serde)
└── logging.rs         # Logging utilities (logforth)
```

## WHERE TO LOOK

| Task               | File            | Notes                                                          |
| ------------------ | --------------- | -------------------------------------------------------------- |
| FFI exports        | lib.rs          | gpui_init, gpui_create_window, gpui_batch_update_elements      |
| FFI helpers        | ffi_helpers.rs  | ptr_to_u64, read_c_string, validate_result_ptr                 |
| GPUI rendering     | renderer.rs     | render_element_to_gpui (div/text/span/img)                     |
| Command bus        | host_command.rs | HostCommand enum, handle_on_app_thread, send_host_command      |
| Window             | window.rs       | Window, WindowState, render_element(), batch_update_elements() |
| Element structures | element/mod.rs  | ReactElement, ElementStyle, ElementKind enum                   |
| CSS styling        | element/\*.rs   | Individual element implementations with CSS property mapping   |

## CONVENTIONS

- **Root tracking:** ROOT_ELEMENT_ID AtomicU64 (HashMap iteration was non-deterministic)
- **Event handlers:** All ReactElement structs require event_handlers: None field
- **Span text:** Collect from child text elements, not span.text directly
- **Arc wrapping:** Elements stored as Arc<ReactElement> for sharing
- **Render trigger:** RENDER_TRIGGER AtomicU64 signals GPUI to re-render
- **Command bus:** async_channel → GPUI App thread → window.refresh()
- **Async handling:** tokio runtime for async command processing
- **Serialization:** serde for JS↔Rust element data transfer
- **Window handle:** Window uses AnyWindowHandle for type-erased GPUI window reference
- **Window ID tracking:** Window struct stores window_id for easier tracking
- **FFI sync:** Call batchElementUpdates() in resetAfterCommit, then renderFrame()
- **CSS caching:** ElementStyle.cached_gpui_style avoids recomputing every frame

## ANTI-PATTERNS (THIS PROJECT)

- Don't iterate HashMap to find root - use ROOT_ELEMENT_ID
- Don't render span.text - collect from child text elements
- Don't create ReactElement without event_handlers: None - required field
- Don't call gpui_render_frame without rebuild_tree first
- Don't forget to wrap elements in Arc before storing
- Don't skip send_host_command(TriggerRender) after updates

## KEY PATTERNS

- **Two-phase rendering:** JS builds tree → Rust updates by ID → GPUI renders
- **Element hierarchy:** div → span → text (text always child of span)
- **Update pipeline:** batch_update_elements JSON → deserialize → update_element_tree → refresh
- **Window refresh:** App::new().set_background_color().run() pattern
- **Window struct:** Holds AnyWindowHandle + WindowState for unified management
- **Command architecture:**
    - FFI functions send HostCommand via send_host_command()
    - host_command.rs processes commands on app thread via handle_on_app_thread()
    - Window methods contain actual element processing logic
- **CSS support:** ElementStyle struct maps CSS properties to GPUI Style (text_color, bg_color, margin, padding, flex,
  etc.)
