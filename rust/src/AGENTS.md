# PROJECT KNOWLEDGE BASE

**Generated:** 2026-01-03
**Commit:** main
**Scope:** rust/src only

## OVERVIEW
Rust FFI library for GPUI integration - handles React element tree rendering to GPU-accelerated UI.

## WHERE TO LOOK
| Task | File | Notes |
|------|------|-------|
| FFI exports | lib.rs | gpui_init, gpui_render_frame, gpui_update_element, gpui_trigger_render |
| GPUI rendering | app.rs | RootView, render_element_to_gpui (div/text/span/img) |
| Element data | element_store.rs | ReactElement struct, ElementStyle, ELEMENT_TREE global |
| Span rendering | app.rs | Collect text from child text elements (not span.text) |
| Tree rebuild | lib.rs | rebuild_tree() updates element.children from child refs |

## CONVENTIONS
- **Root tracking**: ROOT_ELEMENT_ID AtomicU64 (HashMap iteration was non-deterministic)
- **Event handlers**: All ReactElement structs require event_handlers: None field
- **Tree rebuild**: After appendChild, call rebuild_tree() to sync children to ELEMENT_MAP
- **Span text**: Collect from child text elements, not span.text directly
- **Arc wrapping**: Elements stored as Arc<ReactElement> in ELEMENT_MAP and ELEMENT_TREE
- **Render trigger**: RENDER_TRIGGER AtomicU64 signals GPUI to re-render

## ANTI-PATTERNS (THIS PROJECT)
- Don't iterate HashMap to find root - use ROOT_ELEMENT_ID (commit: fixed root selection bug)
- Don't render span.text - collect from child text elements (span contains text children)
- Don't create ReactElement without event_handlers: None - required field
- Don't call gpui_render_frame without calling rebuild_tree first - children won't sync
