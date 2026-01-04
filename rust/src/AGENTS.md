# RUST/SRC - Rust FFI Library

**Scope:** rust/src only

## OVERVIEW
Rust FFI library for GPUI integration - handles React element tree rendering to GPU-accelerated UI.

## STRUCTURE
```
rust/src/
├── lib.rs              # FFI exports (gpui_init, gpui_trigger_render)
├── renderer.rs         # RootView, render_element_to_gpui
├── element.rs          # ReactElement, ElementStyle
├── host_command.rs     # async_channel command bus
└── window_state.rs    # ElementTree, render_count
```

## WHERE TO LOOK
| Task | File | Notes |
|------|------|-------|
| FFI exports | lib.rs | gpui_init, gpui_trigger_render, gpui_batch_update_elements |
| GPUI rendering | renderer.rs | render_element_to_gpui (div/text/span/img) |
| Command bus | host_command.rs | init(cx), send_host_command(TriggerRender) |
| Window state | window_state.rs | update_element_tree(), render_count |

## CONVENTIONS
- **Root tracking:** ROOT_ELEMENT_ID AtomicU64 (HashMap iteration was non-deterministic)
- **Event handlers:** All ReactElement structs require event_handlers: None field
- **Span text:** Collect from child text elements, not span.text directly
- **Arc wrapping:** Elements stored as Arc<ReactElement>
- **Render trigger:** RENDER_TRIGGER AtomicU64 signals GPUI to re-render

## ANTI-PATTERNS (THIS PROJECT)
- Don't iterate HashMap to find root - use ROOT_ELEMENT_ID
- Don't render span.text - collect from child text elements
- Don't create ReactElement without event_handlers: None - required field
- Don't call gpui_render_frame without rebuild_tree first