# PROJECT KNOWLEDGE BASE

**Generated:** 2026-01-05
**Branch:** $(git branch --show-current 2>/dev/null || echo "unknown")
**Commit:** $(git rev-parse --short HEAD 2>/dev/null || echo "unknown")

## OVERVIEW

React renderer for GPUI (Zed's GPU-accelerated UI) using Bun native FFI. Architecture: React → Reconciler → Element
Store → Bun FFI → Rust → GPUI → GPU.

## 规则：

1. 每次修改代码后都要调用工具格式化代码(如果没有则可以告诉用户是否要集成格式化工具)
2. 不要轻易放弃任务；在合理范围内尝试不同思路
3. 代码首先是写给人类阅读和维护的，机器执行只是副产品
4. 主动留意并指出以下“坏味道”：
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
│   └── src/        # 8 files: FFI exports, GPUI rendering
├── src/reconciler/   # React reconciler + FFI bindings (10 files)
└── demo/             # Demo apps (5 entry points)
```

## WHERE TO LOOK

| Task               | Location                        | Notes                                       |
|--------------------|---------------------------------|---------------------------------------------|
| Public API         | src/index.ts                    | createRoot(), createWindow()                |
| React reconciler   | src/reconciler/host-config.ts   | appendChild, commitUpdate, resetAfterCommit |
| Element management | src/reconciler/element-store.ts | Map<id, ElementData>, IDs start from 2      |
| Bun FFI bindings   | src/reconciler/gpui-binding.ts  | dlopen, FFIType, liveBuffers                |
| Rust FFI exports   | rust/src/lib.rs                 | gpui_init, gpui_trigger_render              |
| GPUI rendering     | rust/src/renderer.rs            | RootView, render_element_to_gpui            |
| Command bus        | rust/src/host_command.rs        | async_channel command bus                   |
| Window state       | rust/src/window_state.rs        | ElementTree, render_count                   |

## CONVENTIONS

- **Rust subdir:** Rust code in rust/src/ (not root src/)
- **FFI sync:** Call batchElementUpdates() + renderFrame() after batch updates
- **Root tracking:** ROOT_ELEMENT_ID AtomicU64 (HashMap iteration is non-deterministic)
- **Element IDs:** Start from 2 (ID 1 reserved)
- **Buffer lifetime:** FFI buffers must stay in liveBuffers[] during calls
- **Manual tests:** Console.log assertions, no test framework
- **No CI/CD:** Build/test via npm scripts

## ANTI-PATTERNS (THIS PROJECT)

- Don't iterate HashMap to find root - use ROOT_ELEMENT_ID
- Don't skip updateElement after appendChild - children won't sync to Rust
- Don't render span.text - collect from child text elements
- Don't create ReactElement without event_handlers: None - required field
- Don't call gpui_render_frame without rebuild_tree first
- Don't use className prop - use style prop instead
- Don't let FFI buffers be GC'd before call returns - push to liveBuffers

## UNIQUE STYLES

- Native Bun FFI (not wasm-bindgen) - no browser compatibility
- Two-phase: React builds tree → Rust tracks by ID → GPUI renders
- Span elements contain text elements as children (text in child.text)
- Command bus: async_channel → GPUI App thread → window.refresh()
- Event handlers registered in JS, passed as IDs to Rust

## COMMANDS

```bash
cd rust && cargo build --release      # Build Rust library (3GB+ first time)
bun run demo                         # Basic demo
bun run event-demo                   # Event handling demo
bun run src/reconciler/__tests__/element-store.test.ts  # Run tests
```

## NOTES

- Bun FFI uses suffix() to load platform-specific .so/.dylib/.dll
- Element IDs start from 2 to reserve ID 1 for special purposes
- liveBuffers array prevents GC from collecting FFI buffers during calls