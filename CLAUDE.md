# CLAUDE.md

This file provides guidance for Claude (Anthropic) when working with this codebase.

## Project Overview

React renderer for GPUI (Zed's GPU-accelerated UI) using Bun native FFI.

Architecture: `React → Reconciler → Element Store → Bun FFI → Rust → GPUI → GPU`

## Key Conventions

### Rust

- Code lives in `rust/src/` (not root `src/`)
- Edition 2024, hard tabs, 2-space indent (see `rustfmt.toml`)
- Build with: `just native`

### TypeScript

- Prettier: printWidth 100, tabWidth 4, useTabs false
- No test framework - manual console.log tests in `__tests__/`
- Use `style` prop for styling, NOT `className`

### FFI Pattern

- Always call `batchElementUpdates()` + `renderFrame()` after batch updates
- Keep FFI buffers alive in `liveBuffers` during calls
- Element IDs start from 2 (ID 1 reserved)

## Critical Anti-Patterns

- ❌ Don't iterate HashMap to find root - use `ROOT_ELEMENT_ID`
- ❌ Don't skip `updateElement` after `appendChild`
- ❌ Don't render `span.text` - collect from child text elements
- ❌ Don't create `ReactElement` without `event_handlers: None`
- ❌ Don't let FFI buffers be GC'd - push to `liveBuffers`
- ❌ Don't return `false` from `shouldSetTextContent` for text children

## File Locations

| Concern         | File                              |
| --------------- | --------------------------------- |
| Public API      | `src/index.ts`                    |
| Reconciler      | `src/reconciler/host-config.ts`   |
| Element Store   | `src/reconciler/element-store.ts` |
| FFI Abstraction | `src/core/rust.ts`                |
| FFI Bindings    | `src/core/ffi.ts`                 |
| Event Routing   | `src/reconciler/event-router.ts`  |
| Rust FFI        | `rust/src/lib.rs`                 |
| GPUI Rendering  | `rust/src/renderer.rs`            |
| Command Bus     | `rust/src/host_command.rs`        |

## Build Commands

```bash
just native           # Build Rust library
bun run demo          # Run demos
bun run format        # Format all code
```

## Notes

- Auto-generated files: `src/events/generated.ts`, `rust/src/element/events.rs` - DO NOT EDIT
- Focus: onFocus, onBlur with automatic tab navigation
- Hover: onMouseEnter, onMouseLeave supported
