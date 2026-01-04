# PROJECT KNOWLEDGE BASE

**Generated:** 2026-01-04
**Mode:** --create-new

## OVERVIEW
React renderer for GPUI (Zed's GPU-accelerated UI) using Bun native FFI. Architecture: React → Reconciler → Element Store → Bun FFI → Rust → GPUI → GPU.

## STRUCTURE
```
gpui-react/
├── rust/              # Rust FFI library
│   └── src/        # 9 files: FFI exports, GPUI rendering
├── src/renderer/     # 9 files: React reconciler, FFI bindings
└── demo/             # Demo apps
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| React reconciler | src/renderer/host-config.ts | React-to-GPUI bridge |
| Element management | src/renderer/element-store.ts | JS-side element data |
| Bun FFI bindings | src/renderer/gpui-binding.ts | dlopen, FFIType, pointers |
| Rust FFI exports | rust/src/lib.rs | gpui_init, gpui_trigger_render |
| GPUI rendering | rust/src/renderer.rs | RootView, render_element_to_gpui |
| Commands | rust/src/host_command.rs | async_channel command bus |
| Window state | rust/src/window_state.rs | ElementTree, render_count |

## CONVENTIONS
- **Rust subdir:** Rust code in rust/src/ (not root src/)
- **FFI sync:** Call `send_host_command(TriggerRender)` after batch updates
- **Root tracking:** ROOT_ELEMENT_ID AtomicU64 (HashMap iteration is non-deterministic)
- **Manual tests:** Console.log assertions, no test framework
- **No CI/CD:** Build/test via npm scripts

## ANTI-PATTERNS (THIS PROJECT)
- Don't iterate HashMap to find root - use ROOT_ELEMENT_ID
- Don't skip updateElement after appendChild - children won't sync to Rust
- Don't render span.text - collect from child text elements
- Don't create ReactElement without event_handlers: None - required field
- Don't call gpui_render_frame without rebuild_tree first

## UNIQUE STYLES
- Native Bun FFI (not wasm-bindgen) - no browser compatibility
- Two-phase: React builds tree → Rust tracks by ID → GPUI renders
- Span elements contain text elements as children (text in child.text)
- Command bus: async_channel → GPUI App thread → window.refresh()

## COMMANDS
```bash
cd rust && cargo build --release      # Build Rust library
bun run demo                         # Run basic demo
bun run event-demo                   # Run event handling demo
```

## NOTES
- GPUI compilation downloads 3GB+ on first build
- Root element selection buggy - now tracked with ROOT_ELEMENT_ID