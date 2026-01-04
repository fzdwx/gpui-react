# PROJECT KNOWLEDGE BASE

**Generated:** 2026-01-04 09:14:00
**Commit:** 3ac410a
**Branch:** main

## OVERVIEW
React renderer for GPUI (Zed's GPU-accelerated UI) using Bun FFI. Architecture: React → Reconciler → Element Store → Bun FFI → Rust → GPUI → GPU.

## STRUCTURE
```
gpui-react/
├── demo/              # Demo applications (10 files)
├── rust/              # Rust FFI library
│   └── src/        # Rust source (8 files)
├── src/renderer/      # React reconciler + FFI bindings (7 files)
│   └── __tests__/   # Manual tests
└── dist/             # TypeScript build output
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| Reconciler host config | src/renderer/host-config.ts | React-to-GPUI bridge |
| Element management | src/renderer/element-store.ts | JS-side element data |
| FFI bindings | src/renderer/gpui-binding.ts | Bun FFI calls to Rust |
| Rust FFI exports | rust/src/lib.rs | gpui_init, gpui_render_frame, gpui_update_element |
| GPUI rendering | rust/src/app.rs | RootView + render_element_to_gpui |
| Element data model | rust/src/element_store.rs | ReactElement + ElementStyle |
| Run demo | bun run demo/{flex,styled,elements,event}-demo | | |

## CONVENTIONS
- **Rust nested:** Rust code in rust/ subdirectory (not root src/)
- **FFI sync:** After appendChild, call updateElement() to sync children to Rust
- **Root tracking:** ROOT_ELEMENT_ID in lib.rs tracks root element (HashMap iteration is non-deterministic)
- **Manual tests:** Console.log assertions instead of test framework
- **No CI/CD:** Build/test via npm scripts, no GitHub Actions

## ANTI-PATTERNS (THIS PROJECT)
- Don't iterate HashMap to find root - use ROOT_ELEMENT_ID
- Don't call updateElement only on create - call after appendChild too
- Don't render span children - collect text from child text elements

## UNIQUE STYLES
- React reconciler wraps GPUI directly via Bun FFI
- Span elements contain text elements as children (text in child.text, not span.text)
- Two-phase rendering: React builds element tree → Rust tracks by ID → GPUI renders

## COMMANDS
```bash
cd rust && cargo build --release  # Build Rust library (2-5 min first time)
bun run demo                            # Run basic demo
bun run elements-demo                   # Run span/div elements demo
bun run flex-demo                      # Run flexbox demo
bun run styled-demo                    # Run styling demo
bun run event-demo                    # Run event handling demo
```

## NOTES
- GPUI compilation is slow (downloads 3GB+ on first build)
- Root element selection was buggy - now tracked with ROOT_ELEMENT_ID AtomicU64
- Span rendering must collect text from children.text array
- Always add event_handlers: None to ReactElement structs
