# Roadmap

**Last Updated:** 2025-01-03
**Current Version:** 0.1.0
**Overall Progress:** 87.5% (7/8 phases complete)

---

## Overview

This roadmap outlines the development path for React-GPUI Renderer, a React renderer for GPUI (Zed's GPU-accelerated UI framework) using Bun FFI. The project combines product-level feature planning with detailed technical implementation steps.

**Architecture:**
```
React Components (JSX)
    ↓
React Reconciler (host-config.ts)
    ↓
Element Store (element-store.ts)
    ↓
Bun FFI (gpui-binding.ts)
    ↓
Rust FFI (lib.rs)
    ↓
GPUI Runtime (app.rs)
    ↓
GPU Rendering
```

---

## Phase 1: MVP Completion ✅ COMPLETE

**Status:** ✅ 100% Complete
**Duration:** Week 1

### Goals
- Create project structure for Bun + Rust
- Implement Bun FFI bridge
- Integrate GPUI framework
- Create minimal React reconciler
- Build basic demo

### Completed Tasks

#### 1.1 Project Initialization ✅
- [x] Initialize Bun project with package.json
- [x] Initialize Rust library with Cargo.toml
- [x] Configure TypeScript (tsconfig.json)
- [x] Create directory structure
- [x] Verify build setup

#### 1.2 Bun FFI Setup ✅
- [x] Implement simple Rust FFI function (`gpui_greet`)
- [x] Build Rust shared library
- [x] Create Bun FFI binding (gpui-binding.ts)
- [x] Test FFI integration
- [x] Verify memory management

#### 1.3 Rust GPUI Integration ✅
- [x] Add GPUI dependency (git = "https://github.com/zed-industries/zed.git")
- [x] Implement FFI types (FfiResult, ElementData)
- [x] Create app wrapper (app.rs)
- [x] Implement window management
- [x] Export FFI functions (gpui_init, gpui_render_frame, gpui_free_result)

#### 1.4 React Renderer Host Config ✅
- [x] Install React and react-reconciler
- [x] Create ElementStore class
- [x] Implement host config for React Reconciler
- [x] Connect reconciler to FFI layer
- [x] Implement createRoot API

#### 1.5 React-to-GPUI Bridge ✅
- [x] Simplify ElementStore for MVP
- [x] Update host config for div + text only
- [x] Implement renderFrame binding
- [x] Connect ElementStore to FFI
- [x] Add element ID tracking (IDs start from 2)

#### 1.6 Minimal Demo ✅
- [x] Create demo app component
- [x] Create demo entry point
- [x] Add build scripts to package.json
- [x] Test basic rendering
- [x] Verify `<div>hello world</div>` renders

### Deliverables
- ✅ Rust library (libgpui_renderer.so)
- ✅ TypeScript modules (6 files)
- ✅ Demo applications (11 files)
- ✅ FFI integration tested
- ✅ Documentation (AGENTS.md, README.md)

### Known Issues
- 🔴 **Critical:** GPUI compilation timeout (>2 minutes on first build)
  - Cause: GPUI is a large dependency (~3GB)
  - Workaround: Use cached builds after first compilation
- 🔴 **Critical:** Elements rendering bug (fixed in commit 21b71c68)
  - Cause: Wrong root element selection, missing child hierarchy sync
  - Fix: Added ROOT_ELEMENT_ID tracking, updateElement after appendChild

---

## Phase 2: Styling System ✅ COMPLETE

**Status:** ✅ 100% Complete
**Duration:** Weeks 2-3

### Goals
- Support fundamental CSS properties mapped to GPUI
- Implement flexbox layout system
- Style parsing and validation

### Completed Tasks

#### 2.1 Basic Style Properties ✅
- [x] Define StyleProps interface
- [x] Implement color parser (hex, rgb, rgba, named colors)
- [x] Implement size parser (px, em, rem, %, auto)
- [x] Create GPUI style mapping utility (styles.ts)
- [x] Update host-config to pass styles
- [x] Update Rust element builder to apply styles
- [x] Write style tests (flex-app, styled-app demos)

#### 2.2 Flexbox Layout System ✅
- [x] Define FlexProps interface
- [x] Implement flex property mapper
- [x] Update host-config to recognize flex props
- [x] Update Rust builder to apply flex styles
- [x] Create flexbox demo components (flex-app.tsx)
- [x] Support flex-direction (row, column)
- [x] Support justify-content (start, center, end, space-between)
- [x] Support align-items (start, center, end)
- [x] Support gap property

### Deliverables
- ✅ Style mapping system (styles.ts)
- ✅ Flexbox layout support
- ✅ Demo applications (flex-app, styled-app)
- ✅ Color and size parsers
- ✅ GPUI style integration

### Supported Properties

| CSS Property | GPUI Mapping | Status |
|--------------|--------------|--------|
| `color` | `.text_color()` | ✅ |
| `backgroundColor` | `.bg()` | ✅ |
| `fontSize` | `.text_size()` | ✅ |
| `fontWeight` | `.font_weight()` | ✅ |
| `width` | `.w()` | ✅ |
| `height` | `.h()` | ✅ |
| `margin` | `.m()`, `.mt()`, `.mr()`, `.mb()`, `.ml()` | ✅ |
| `padding` | `.p()`, `.pt()`, `.pr()`, `.pb()`, `.pl()` | ✅ |
| `borderRadius` | `.rounded()` | ✅ |
| `opacity` | `.opacity()` | ✅ |
| `display: flex` | `.flex()` | ✅ |
| `flexDirection` | `.flex_row()`, `.flex_col()` | ✅ |
| `justifyContent` | `.justify_*()` | ✅ |
| `alignItems` | `.items_*()` | ✅ |
| `gap` | `.gap()` | ✅ |

---

## Phase 3: Element Types ✅ COMPLETE

**Status:** ✅ 100% Complete
**Duration:** Week 4

### Goals
- Add inline text element support (span)
- Add image support
- Support custom components
- Element lifecycle management

### Completed Tasks

#### 3.1 Span Element ✅
- [x] Add `span` type to element store
- [x] Map to GPUI text element
- [x] Support inline styling
- [x] Test with inline and block elements (elements-app.tsx)
- [x] Fix span text rendering (collect from child text elements)

#### 3.2 Image Element ✅
- [x] Define ImageProps interface
- [x] Add `img` type to element store
- [x] Implement image placeholder rendering
- [x] Map to GPUI image component
- [x] Handle loading states (placeholder)
- [x] Create image demo (image-app.tsx)

#### 3.3 Custom Components ✅
- [x] Support component rendering via React Reconciler
- [x] Component lifecycle management
- [x] Props passing and state management
- [x] Event handling for custom components

#### 3.4 Element Lifecycle ✅
- [x] Mount/unmount handling
- [x] Update batching
- [x] Parent-child relationship management
- [x] Element cleanup on unmount

### Deliverables
- ✅ Span element support
- ✅ Image element support (placeholder)
- ✅ Custom component rendering
- ✅ Element lifecycle management
- ✅ Demo applications (elements-app, image-app)

---

## Phase 4: Event System ✅ COMPLETE

**Status:** ✅ 100% Complete
**Duration:** Weeks 5-6

### Goals
- Implement fundamental event handling
- Support event types (onClick, onHover, etc.)
- Event handler registration and cleanup

### Completed Tasks

#### 4.1 Basic Events ✅
- [x] Define event types (events.ts)
- [x] Create event handler registry
- [x] Implement onClick support
- [x] Implement onHover support
- [x] Implement onFocus/onBlur support
- [x] Implement onChange support
- [x] Pass event handlers via FFI
- [x] Create event demo (event-app.tsx)

#### 4.2 Event Infrastructure ✅
- [x] Event ID mapping (nextEventId counter)
- [x] Event handler storage (Map<number, EventHandler>)
- [x] Event handler cleanup on unmount
- [x] Event bubbling support (GPUI events)
- [x] Event propagation handling

### Deliverables
- ✅ Event type definitions
- ✅ Event handler registration system
- ✅ onClick, onHover, onFocus, onBlur support
- ✅ Event cleanup mechanism
- ✅ Demo application (event-app)

### Supported Events

| Event Type | Use Case | Status |
|------------|-----------|--------|
| `onClick` | Button clicks, interactions | ✅ |
| `onHover` | Hover effects, tooltips | ✅ |
| `onFocus` | Form validation, styling | ✅ |
| `onBlur` | Form validation, cleanup | ✅ |
| `onChange` | Input fields, forms | ✅ |

---

## Phase 5: Advanced Components ✅ COMPLETE

**Status:** ✅ 100% Complete
**Duration:** Weeks 7-8
**Completed:** 2025-01-03

### Goals
- Implement virtualized list for performance
- Add scrollable containers
- Component composition patterns

### Completed Tasks

#### 5.1 Virtual List ✅
- [x] Design VirtualList component architecture
- [x] Implement VirtualList React component
- [x] Implement viewport calculation logic
- [x] Implement item recycling mechanism
- [x] Add scroll handling
- [x] Create VirtualList demo (virtual-list-app.tsx, virtual-list-index.ts)
- [x] Test basic functionality
- [x] Optimize for 60+ FPS (visible items only)

**Deliverables:**
- ✅ `src/components/VirtualList.tsx` - Virtualized list component with viewport calculation
- ✅ `demo/virtual-list-app.tsx` - Demo with 10,000 items
- ✅ `demo/virtual-list-index.ts` - Demo entry point
- ✅ Supports: viewport calculation, item recycling, scroll handling
- ✅ Performance: Only renders visible items (typically 10-30)

#### 5.2 Scrollable Containers ✅
- [x] Add overflow style support (implemented in VirtualList)
- [x] Implement scroll capturing
- [x] Add scroll event handlers (implemented in VirtualList)
- [x] Test with various scroll scenarios

**Note:** Scrollable containers are integrated into VirtualList component

#### 5.3 Component Composition ✅
- [x] Support component rendering (VirtualList accepts any renderItem)
- [x] Props passing and state management
- [x] Event handling support (can pass events through renderItem)

**Note:** Component composition is supported through the renderItem prop

### Deliverables
- ✅ VirtualList component with 10,000+ item support
- ✅ Scroll handling and optimization
- ✅ Component composition patterns
- ✅ Demo applications (virtual-list-app, virtual-list-index)

---

## Phase 6: Performance Optimization ✅ COMPLETE

**Status:** ✅ 100% Complete
**Duration:** Weeks 7-8
**Completed:** 2025-01-03

### Goals
- Reduce FFI overhead by batching calls
- Cache GPUI elements to reduce allocations
- Ensure no memory leaks

### Completed Tasks

#### 6.1 FFI Call Batching ✅
- [x] Design batched FFI update mechanism
- [x] Implement batchElementUpdates function in gpui-binding.ts
- [x] Update host-config to use batch updates during commit phase
- [x] Add Rust FFI function gpui_batch_update_elements
- [x] Add Rust module batch-updates.rs
- [x] Process JSON array of elements in single FFI call
- [x] Reduce N individual updateElement calls during tree construction

#### 6.2 Memory Management ✅
- [x] Reduce Arc cloning with interior mutability
- [x] Lazy tree rebuild (only when GPUI render pending)
- [x] Add ELEMENT_STYLE module for style caching
- [x] Reduce mutex contention with scoped locks

**Deliverables:**
- ✅ Batched FFI updates (batchElementUpdates function)
- ✅ Rust batch update handler (gpui_batch_update_elements)
- ✅ Reduced FFI overhead for element creation
- ✅ Performance improvements measured

---

## Phase 7: Testing & Documentation ✅ COMPLETE

**Status:** ✅ 100% Complete
**Duration:** Week 10

### Goals
- Unit test coverage
- Integration test framework
- Documentation completeness

### Completed Tasks

#### 7.1 Unit Tests ✅
- [x] Create test framework (manual console.log tests)
- [x] Test element store operations (element-store.test.ts)
- [x] Test createElement, appendChild, removeChild
- [x] Test getRoot functionality

#### 7.2 Integration Tests ✅
- [x] Test full rendering pipeline
- [x] Test event handling end-to-end
- [x] Test style application
- [x] Test layout correctness
- [x] Create demo applications for each feature

#### 7.3 Documentation ✅
- [x] Create comprehensive README.md
- [x] Add AGENTS.md for knowledge base (root + subdirectories)
- [x] Document FFI integration patterns
- [x] Document styling system
- [x] Document event handling
- [x] Document anti-patterns
- [x] Document unique project conventions

### Deliverables
- ✅ Test suite (element-store.test.ts)
- ✅ Demo applications (7 apps including virtual-list-app)
- ✅ README.md (184 lines)
- ✅ AGENTS.md files (3 files, 127 lines total)
- ✅ API documentation
- ✅ Architecture documentation

---

## Phase 8: Release Preparation ⚪ FUTURE

**Status:** ⚪ Not Started
**Duration:** Weeks 11-12
**Estimated Start:** Week 11

### Goals
- Prepare for v0.1.0 release
- Release notes and changelog
- Migration guide

### Tasks

#### 8.1 Versioning ⚪
- [ ] Tag version 0.1.0 (MVP release)
- [ ] Update package.json version
- [ ] Update Cargo.toml version
- [ ] Create git tag

#### 8.2 Release Checklist ⚪
- [ ] All critical bugs fixed (including elements rendering bug ✅)
- [ ] Documentation complete
- [ ] Test coverage meets targets
- [ ] Performance benchmarks documented
- [ ] Migration guide written
- [ ] Release notes written
- [ ] Changelog updated

### Deliverables
- ⚪ v0.1.0 release
- ⚪ Release notes
- ⚪ Changelog
- ⚪ Migration guide from React DOM
- ⚪ GitHub release

### Semantic Versioning

- **v0.1.0:** MVP (div, text, basic styles, flexbox, events)
- **v0.2.0:** Advanced Components (Virtual List)
- **v0.3.0:** Performance Optimizations
- **v1.0.0:** Production-ready with all features

---

## Phase 7: Testing & Documentation ✅ COMPLETE

**Status:** ✅ 100% Complete
**Duration:** Week 10

### Goals
- Unit test coverage
- Integration test framework
- Documentation completeness

### Completed Tasks

#### 7.1 Unit Tests ✅
- [x] Create test framework (manual console.log tests)
- [x] Test element store operations (element-store.test.ts)
- [x] Test createElement, appendChild, removeChild
- [x] Test getRoot functionality
- [x] Add test scripts to package.json

#### 7.2 Integration Tests ✅
- [x] Test full rendering pipeline
- [x] Test event handling end-to-end
- [x] Test style application
- [x] Test layout correctness
- [x] Create demo applications for each feature

#### 7.3 Documentation ✅
- [x] Create comprehensive README.md
- [x] Add AGENTS.md for knowledge base
- [x] Document FFI integration patterns
- [x] Document styling system
- [x] Document event handling
- [x] Document anti-patterns
- [x] Document unique project conventions

### Deliverables
- ✅ Test suite (element-store.test.ts)
- ✅ Demo applications (6 apps)
- ✅ README.md (184 lines)
- ✅ AGENTS.md files (3 files, 127 lines total)
- ✅ API documentation
- ✅ Architecture documentation

---

## Phase 8: Release Preparation ⚪ FUTURE

**Status:** ⚪ Not Started
**Duration:** Weeks 11-12
**Estimated Start:** Week 11

### Goals
- Prepare for v0.1.0 release
- Release notes and changelog
- Migration guide

### Tasks

#### 8.1 Versioning ⚪
- [ ] Tag version 0.1.0 (MVP release)
- [ ] Update package.json version
- [ ] Update Cargo.toml version
- [ ] Create git tag

#### 8.2 Release Checklist ⚪
- [ ] All critical bugs fixed (including elements rendering bug ✅)
- [ ] Documentation complete
- [ ] Test coverage meets targets
- [ ] Performance benchmarks documented
- [ ] Migration guide written
- [ ] Release notes written
- [ ] Changelog updated

### Deliverables
- ⚪ v0.1.0 release
- ⚪ Release notes
- ⚪ Changelog
- ⚪ Migration guide from React DOM
- ⚪ GitHub release

### Semantic Versioning

- **v0.1.0:** MVP (div, text, basic styles, flexbox)
- **v0.2.0:** Styling & Events
- **v0.3.0:** Elements & Advanced Components
- **v0.4.0:** Performance Optimizations
- **v1.0.0:** Production-ready with all features

---

## Risk Assessment

### Technical Risks

| Risk | Probability | Impact | Status | Mitigation |
|------|-------------|--------|---------|------------|
| GPUI API changes | Medium | High | ⚠️ Active | Pin to specific version, monitor changes |
| FFI memory leaks | Medium | High | ✅ Managed | Rigorous testing, Arc wrapping |
| Performance degradation | Low | Medium | ⚪ Active | Benchmarking, optimization sprints |
| Event system complexity | Medium | Medium | ✅ Managed | Simplified MVP first, iterate |
| HashMap non-determinism | Low | High | ✅ Fixed | Use ROOT_ELEMENT_ID tracking |

### External Risks

| Risk | Probability | Impact | Status | Mitigation |
|------|-------------|--------|---------|------------|
| Bun FFI limitations | Low | Medium | ✅ Managed | Use native addons if needed |
| GPUI dependency issues | Medium | High | ✅ Managed | Cached builds after first compilation |
| OS compatibility issues | Low | Medium | ⚪ Active | Test on Linux, macOS, Windows |

---

## Success Metrics

### Phase 1 (MVP) - ✅ 100%
- [x] Architecture complete
- [x] Demo runs without crashes
- [x] `<div>hello world</div>` renders correctly
- [x] Build time acceptable (after first build)
- [x] Bun FFI integration tested

### Phase 2 (Styling) - ✅ 100%
- [x] All basic CSS properties supported
- [x] Flexbox layouts work correctly
- [x] Style changes reactively update
- [x] Multiple demo apps work

### Phase 3 (Elements) - ✅ 100%
- [x] Span elements work (with child text)
- [x] Image elements work (placeholder)
- [x] Custom components supported
- [x] Element lifecycle correct

### Phase 4 (Events) - ✅ 100%
- [x] Basic events fire correctly
- [x] Event types defined
- [x] Event handler registration works
- [x] Event cleanup on unmount

### Phase 5 (Advanced) - ⚪ 0%
- [ ] Virtual list handles 10k+ items
- [ ] Scrolling performs well
- [ ] Advanced components stable

### Phase 6 (Performance) - ⚪ 0%
- [ ] FFI calls reduced by 50%
- [ ] Memory usage stable
- [ ] Frame rate 60+ FPS

### Phase 7 (Testing) - ✅ 100%
- [x] Unit tests for element store
- [x] Integration tests (demo apps)
- [x] Documentation complete
- [x] Test scripts in package.json

### Phase 8 (Release) - ⚪ 0%
- [ ] Version 0.1.0 released
- [ ] Release notes documented
- [ ] Changelog updated
- [ ] Migration guide written

---

## Timeline Summary

| Phase | Duration | Start | End | Status |
|-------|----------|-------|-----|--------|
| Phase 1: MVP | 1 week | Week 1 | Week 1 | ✅ Complete |
| Phase 2: Styling | 2 weeks | Week 2 | Week 3 | ✅ Complete |
| Phase 3: Elements | 1 week | Week 4 | Week 4 | ✅ Complete |
| Phase 4: Events | 2 weeks | Week 5 | Week 6 | ✅ Complete |
| Phase 5: Advanced | 2 weeks | Week 7 | Week 8 | ⚪ Future |
| Phase 6: Performance | 1 week | Week 9 | Week 9 | ⚪ Future |
| Phase 7: Testing | 1 week | Week 10 | Week 10 | ✅ Complete |
| Phase 8: Release | 2 weeks | Week 11 | Week 12 | ⚪ Future |

**Total Estimated Time:** 12 weeks
**Current Progress:** 7/8 phases complete (87.5%)

---

## Recent Changes

### 2025-01-03 - Bug Fixes
- **Commit:** 21b71c68
- **Fixed:** Elements rendering issue
  - Wrong root element selection (HashMap iteration non-deterministic)
  - Missing child hierarchy synchronization
  - Empty span text rendering
- **Impact:** Elements demo now renders correctly with nested structure

---

## Next Actions

### Immediate (This Week)
1. Resolve any remaining GPUI compilation issues
2. Verify all demo applications work correctly
3. Update ROADMAP.md after any fixes

### Next Week (If starting Phase 5)
1. Implement VirtualList component
2. Add scrollable container support
3. Start performance profiling

---

## Appendix: Resources

### Project Documentation
- [README.md](./README.md) - Project overview and getting started
- [AGENTS.md](./AGENTS.md) - Root knowledge base (63 lines)
- [src/renderer/AGENTS.md](src/reconciler/AGENTS.md) - React renderer layer (32 lines)
- [rust/src/AGENTS.md](./rust/src/AGENTS.md) - Rust FFI layer (32 lines)
- [DEVELOPMENT_PLAN.md](./DEVELOPMENT_PLAN.md) - Development roadmap (archived)
- [IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md) - Implementation details (archived)

### External Resources
- [GPUI GitHub](https://github.com/zed-industries/zed) - Zed's GPUI framework
- [GPUI Examples](https://github.com/zed-industries/zed/tree/main/crates/gpui/src) - GPUI component examples
- [React Documentation](https://react.dev) - Official React docs
- [React Reconciler](https://github.com/facebook/react/tree/main/packages/react-reconciler) - Reconciler API
- [Bun FFI](https://bun.sh/docs/runtime/ffi) - Bun FFI documentation

---

**Last Updated:** 2025-01-03
**Next Review:** Weekly standup or after major changes
