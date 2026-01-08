# PROJECT KNOWLEDGE BASE

**Generated:** 2026-01-08 **Branch:** main **Commit:** 1b52f4e

## OVERVIEW

React renderer for GPUI (Zed's GPU-accelerated UI) using Bun native FFI. Architecture: React → Reconciler → Element
Store → Bun FFI → Rust → GPUI → GPU.

## 规则：

1. 每次修改代码后都要调用工具格式化代码(如果没有则可以告诉用户是否要集成格式化工具)
2. 不要轻易放弃任务；在合理范围内尝试不同思路
3. 代码首先是写给人类阅读和维护的，机器执行只是副产品
4. 主动留意并指出以下"坏味道"：
    - 重复逻辑 / 复制粘贴代码；
    - 模块间耦合过紧或循环依赖；
    - 改动一处导致大量无关部分破坏的脆弱设计；
    - 意图不清晰、抽象混乱、命名含糊；
    - 没有实际收益的过度设计与不必要复杂度。
5. 当识别到坏味道时：
    - 用简洁自然语言说明问题；
    - 给出 1–2 个可行的重构方向，并简要说明优缺点与影响范围。

## STRUCTURE

```
gpui-react/
├── rust/              # Rust FFI library (cdylib)
│   ├── src/         # 11 files: FFI exports, GPUI rendering, command bus
│   │   └── element/ # CSS-capable elements (div, span, img, text)
│   └── Cargo.toml   # cdylib output, GPUI dependency
├── src/              # TypeScript source
│   ├── core/        # FFI abstraction (RustLib, FFI state, bindings)
│   └── reconciler/ # React reconciler + FFI bindings + event router
└── demo/            # Demo apps (5 entry points)
```

## WHERE TO LOOK

| Task               | Location                        | Notes                                                     |
| ------------------ | ------------------------------- | --------------------------------------------------------- |
| Public API         | src/index.ts                    | createRoot()                                              |
| React reconciler   | src/reconciler/host-config.ts   | appendChild, commitUpdate, resetAfterCommit               |
| Element management | src/reconciler/element-store.ts | Map<id, ElementData>, IDs start from 2                    |
| FFI abstraction    | src/core/rust.ts                | RustLib class, batchElementUpdates, renderFrame           |
| FFI bindings       | src/core/ffi.ts                 | Bun FFI function signatures                               |
| FFI state          | src/core/ffi-state.ts           | FfiState buffer management                                |
| Event routing      | src/reconciler/event-router.ts  | registerEventHandler, bindEventToElement, getEventHandler |
| Rust FFI exports   | rust/src/lib.rs                 | gpui_init, gpui_create_window, gpui_batch_update_elements |
| GPUI rendering     | rust/src/renderer.rs            | RootView, render_element_to_gpui                          |
| Element styles     | rust/src/element/mod.rs         | ElementStyle struct, CSS property mapping                 |
| Command bus        | rust/src/host_command.rs        | HostCommand enum, async_channel                           |
| Window             | rust/src/window.rs              | Window (holds AnyWindowHandle + WindowState)              |

## CONVENTIONS

- **Rust subdir:** Rust code in rust/src/ (not root src/)
- **Rust edition:** 2021, formatting with Rust 2024 rules (hard tabs, 2 spaces)
- **Build command:** Run `just native` to compile Rust library after changes
- **FFI sync:** Call batchElementUpdates() + renderFrame() after batch updates
- **Root tracking:** ROOT_ELEMENT_ID AtomicU64 (HashMap iteration is non-deterministic)
- **Element IDs:** Start from 2 (ID 1 reserved)
- **Buffer lifetime:** FFI buffers must stay in FfiState.liveBuffers during calls
- **Manual tests:** Console.log assertions, no test framework
- **Prettier:** printWidth 100, tabWidth 4, useTabs false (see prettier.config.js)
- **CSS styling:** Use style prop for all styling (className unsupported at render level)
- **Event handling:** Register handlers via event-router.ts, pass IDs to Rust

## ANTI-PATTERNS (THIS PROJECT)

- Don't iterate HashMap to find root - use ROOT_ELEMENT_ID
- Don't skip updateElement after appendChild - children won't sync to Rust
- Don't render span.text - collect from child text elements
- Don't create ReactElement without event_handlers: None - required field
- Don't call gpui_render_frame without rebuild_tree first
- Don't use className prop - use style prop instead (handled by reconciler)
- Don't let FFI buffers be GC'd before call returns - push to liveBuffers
- Don't create element with ID 1 - elementStore starts IDs at 2

## UNIQUE STYLES

- Native Bun FFI (not wasm-bindgen) - no browser compatibility
- Command-based architecture: FFI sends commands → host_command.rs processes on app thread
- Two-phase: React builds tree → Rust tracks by ID → GPUI renders
- Span elements contain text elements as children (text in child.text)
- Event bus: async_channel → GPUI App thread → window.refresh()
- Event handlers registered in JS, passed as IDs to Rust via event-router
- Isolated Rust crate in subdirectory with cdylib output
- Window struct: holds AnyWindowHandle (type-erased) + WindowState
- ElementStyle: CSS property struct with caching (cached_gpui_style)

## COMMANDS

```bash
just native                              # Build Rust native library
bun run demo                            # Basic demo
bun run styled-demo                     # CSS styling demo
bun run flex-demo                       # Flexbox layout demo
bun run elements-demo                   # Element types demo
bun run event-demo                      # Event handling demo
bun run src/reconciler/__tests__/element-store.test.ts  # Run tests
bun run format                          # Format all code (staged files)
bun run format:ts                       # Format TypeScript only
bun run format:rust                     # Format Rust only
```

## NOTES

- Bun FFI uses suffix() to load platform-specific .so/.dylib/.dll
- Element IDs start from 2 to reserve ID 1 for special purposes
- FfiState.liveBuffers array prevents GC from collecting FFI buffers during calls
- Rust crate uses cdylib for native library output, not WebAssembly
- HostCommand: TriggerRender, UpdateElement, BatchUpdateElements
- ElementStyle supports: text properties, sizing, margin, padding, position, overflow, background
- Event router uses Map<number, Map<string, number>> for element → eventType → handlerId
