# Roadmap

**Last Updated:** 2026-01-05
**Current Version:** 0.1.0
**Overall Progress:** 75% (6/8 phases complete)

---

## Overview

Development roadmap for React-GPUI Renderer, a React renderer for GPUI (Zed's GPU-accelerated UI framework) using Bun native FFI.

**Architecture:**
```
React Components (JSX)
    ↓
React Reconciler (src/reconciler/host-config.ts)
    ↓
Element Store (src/reconciler/element-store.ts)
    ↓
Bun FFI (src/reconciler/gpui-binding.ts)
    ↓
Rust FFI (rust/src/lib.rs)
    ↓
GPUI Runtime
    ↓
GPU Rendering
```

---

## Phase 1: MVP ✅ COMPLETE

**Status:** ✅ 100% Complete

### Completed Tasks
- [x] Bun + Rust project structure
- [x] Bun FFI bridge implementation
- [x] GPUI framework integration
- [x] Minimal React reconciler
- [x] Basic demo (div + text)

### Known Issues (Resolved)
- **GPUI compilation timeout:** First build downloads ~3GB
- **Root element selection bug:** Fixed with ROOT_ELEMENT_ID tracking

---

## Phase 2: Styling System ✅ COMPLETE

**Status:** ✅ 100% Complete

### Supported CSS Properties

| CSS Property | GPUI Mapping |
|--------------|--------------|
| `color` | `.text_color()` |
| `backgroundColor` | `.bg()` |
| `fontSize` | `.text_size()` |
| `fontWeight` | `.font_weight()` |
| `width` / `height` | `.w()` / `.h()` |
| `margin` / `padding` | `.m()` / `.p()` (directional support) |
| `borderRadius` | `.rounded()` |
| `opacity` | `.opacity()` |
| `display: flex` | `.flex()` |
| `flexDirection` | `.flex_row()` / `.flex_col()` |
| `justifyContent` | `.justify_*()` |
| `alignItems` | `.items_*()` |
| `gap` | `.gap()` |

---

## Phase 3: Element Types ✅ COMPLETE

**Status:** ✅ 100% Complete

### Elements
- **div:** Block container
- **span:** Inline container (contains text children)
- **text:** Text nodes (always child of span)

### Demo: `bun run elements-demo`

---

## Phase 4: Event System 🔵 IN PROGRESS

**Status:** 🔵 30% Complete

### Supported Events

| Event | Status | Notes |
|-------|--------|-------|
| `onClick` | 🔵 In Progress | FFI callback registered, GPUI event binding pending |
| `onHover` | 🔵 In Progress | Same as onClick |
| `onMouseEnter` | 🔵 In Progress | Same as onClick |
| `onMouseLeave` | 🔵 In Progress | Same as onClick |

### Required Work
- [ ] Add FFI event callback registration function
- [ ] Bind GPUI click/hover events to element handlers
- [ ] Pass event data (elementId, coords) back to JS via FFI
- [ ] Test with event-demo

---

## Phase 5: Advanced Components ⚪ FUTURE

**Status:** ⚪ Not Started

### Goals
- Virtualized list component
- Scrollable containers
- Component composition patterns

---

## Phase 6: Performance Optimization ✅ COMPLETE

**Status:** ✅ 100% Complete

### Optimizations
- **FFI batching:** `batchElementUpdates()` sends multiple elements in single call
- **Memory management:** Arc wrapping, lazy tree rebuilds
- **Style caching:** Reduced allocations

---

## Timeline Summary

| Phase | Status | Duration |
|-------|--------|----------|
| Phase 1: MVP | ✅ Complete | 1 week |
| Phase 2: Styling | ✅ Complete | 2 weeks |
| Phase 3: Elements | ✅ Complete | 1 week |
| Phase 4: Events | 🔵 In Progress | 2 weeks |
| Phase 5: Advanced | ⚪ Future | 2 weeks |
| Phase 6: Performance | ✅ Complete | 1 week |

**Total:** 9 weeks | **Progress:** 67% (4/6 complete)

---

## Demo Applications

| Demo | Command | Status | Purpose |
|------|---------|--------|---------|
| Basic | `bun run demo` | ✅ Working | Core rendering |
| Events | `bun run event-demo` | 🔵 In Progress | Click/hover handling (not yet functional) |
| Styled | `bun run styled-demo` | ✅ Working | CSS properties |
| Flex | `bun run flex-demo` | ✅ Working | Flexbox layout |
| Elements | `bun run elements-demo` | ✅ Working | Span/image elements |

---

## Documentation

| File | Purpose |
|------|---------|
| [README.md](./README.md) | Getting started guide |
| [AGENTS.md](./AGENTS.md) | Root knowledge base |
| [src/reconciler/AGENTS.md](./src/reconciler/AGENTS.md) | React reconciler layer |
| [rust/src/AGENTS.md](./rust/src/AGENTS.md) | Rust FFI layer |

---

## Resources

- [GPUI GitHub](https://github.com/zed-industries/zed)
- [React Reconciler](https://github.com/facebook/react/tree/main/packages/react-reconciler)
- [Bun FFI](https://bun.sh/docs/runtime/ffi)

---

**Last Updated:** 2026-01-05
**Next Review:** After Phase 5 implementation