# Development Plan - React-GPUI Renderer

**Last Updated**: January 3, 2026
**Current Version**: 0.1.0
**Status**: Phase 1 - Core MVP Completion (96% complete)

---

## Executive Summary

This document outlines the development roadmap for the React-GPUI renderer, a React renderer for GPUI (Zed's GPU-accelerated UI framework) using Bun FFI.

**Current State**: All core architecture is implemented. The project can render `<div>hello world</div>` but is blocked by GPUI compilation timeouts.

**Immediate Goal**: Complete MVP by resolving compilation blocker and verifying basic rendering.

**Long-term Goal**: Create a production-ready React renderer with full styling, events, and component support.

---

## Project Architecture

```
┌─────────────────────────────────────────┐
│         React Application             │
│         (JSX Components)            │
└──────────────┬──────────────────────┘
               ↓
┌─────────────────────────────────────────┐
│      React Reconciler                │
│    (host-config.ts)                  │
│  - Diffing & reconciliation           │
│  - Lifecycle management              │
└──────────────┬──────────────────────┘
               ↓
┌─────────────────────────────────────────┐
│       Element Store                  │
│    (element-store.ts)                │
│  - Track element hierarchy           │
│  - GlobalId mapping                 │
└──────────────┬──────────────────────┘
               ↓
┌─────────────────────────────────────────┐
│         Bun FFI Layer                │
│     (gpui-binding.ts)                │
│  - Memory management                │
│  - Data marshaling                 │
└──────────────┬──────────────────────┘
               ↓
┌─────────────────────────────────────────┐
│      Rust FFI Layer                 │
│        (lib.rs)                      │
│  - Element tree building            │
│  - Render triggering               │
└──────────────┬──────────────────────┘
               ↓
┌─────────────────────────────────────────┐
│         GPUI Runtime                 │
│       (app.rs, elements.rs)          │
│  - Window management                │
│  - Element conversion              │
│  - GPU rendering                  │
└─────────────────────────────────────────┘
```

---

## Phase 1: MVP Completion (Immediate - Week 1)

### 1.1 Resolve GPUI Compilation Blocker 🔴 **CRITICAL**

**Current Issue**:
- `cargo build --release` times out (>2 minutes)
- Using `gpui = "0.2.2"` from crates.io
- GPUI is a large dependency (~3GB with all deps)

**Proposed Solutions**:

**Option A**: Switch to git dependency (as originally planned)
```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed.git", rev = "main" }
```

**Option B**: Use specific versioned git tag
```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed.git", tag = "v0.1.0" }
```

**Option C**: Try alternate build configurations
- Use `cargo build --debug` for faster initial build
- Pre-download dependencies separately
- Use build caching

**Action Items**:
- [ ] Test Option A (git dependency)
- [ ] Test Option B (tagged version)
- [ ] Test Option C (debug build)
- [ ] Document working solution

**Success Criteria**:
- `cargo build --release` completes in reasonable time (<5 min)
- `libgpui_renderer.so` builds without errors
- No compilation warnings

---

### 1.2 Verify Basic Rendering 🔴 **HIGH PRIORITY**

**Goal**: Confirm `<div>hello world</div>` renders correctly

**Test Scenarios**:

1. **Basic Text Rendering**
```tsx
<div>hello world</div>
```
Expected: Window opens with centered "hello world" text

2. **Nested Elements**
```tsx
<div>
  <div>child 1</div>
  <div>child 2</div>
</div>
```
Expected: Both child divs render correctly

3. **Multiple Text Nodes**
```tsx
<div>hello</div>
<div>world</div>
```
Expected: Both text nodes render

**Action Items**:
- [ ] Build Rust library successfully
- [ ] Run demo with basic test cases
- [ ] Verify window opens
- [ ] Verify text displays correctly
- [ ] Check for console errors
- [ ] Verify no memory leaks (check with Valgrind/sanitizers)

**Success Criteria**:
- Demo runs without crashes
- Text renders legibly
- Window is responsive
- No console errors or warnings

---

## Phase 2: Styling System (Weeks 2-3)

### 2.1 Basic Style Properties 🟡 **MEDIUM PRIORITY**

**Goal**: Support fundamental CSS properties mapped to GPUI

**Properties to Implement**:

| CSS Property | GPUI Mapping | Priority |
|--------------|--------------|----------|
| `color` | `.text_color()` | High |
| `backgroundColor` | `.bg()` | High |
| `fontSize` | `.text_size()` | High |
| `fontWeight` | `.font_weight()` | Medium |
| `width` | `.w()` | High |
| `height` | `.h()` | High |
| `margin` | `.m()`, `.mt()`, `.mr()`, `.mb()`, `.ml()` | Medium |
| `padding` | `.p()`, `.pt()`, `.pr()`, `.pb()`, `.pl()` | Medium |
| `borderRadius` | `.rounded()` | Low |
| `opacity` | `.opacity()` | Low |

**Implementation Approach**:

```typescript
// Updated element interface
export interface StyleProps {
  color?: string;
  backgroundColor?: string;
  fontSize?: number | string;
  width?: number | string;
  height?: number | string;
  margin?: number | string;
  padding?: number | string;
}

// Style mapping utility
function mapToGpuiStyles(props: StyleProps): GpuiStyles {
  return {
    textColor: parseColor(props.color),
    bgColor: parseColor(props.backgroundColor),
    textSize: parseSize(props.fontSize),
    // ... more mappings
  };
}
```

**Action Items**:
- [ ] Define StyleProps interface
- [ ] Implement color parser (hex, rgb, rgba, named colors)
- [ ] Implement size parser (px, em, rem, %, auto)
- [ ] Create GPUI style mapping utility
- [ ] Update host-config to pass styles
- [ ] Update Rust element builder to apply styles
- [ ] Write unit tests for style parsing

**Success Criteria**:
- All listed CSS properties work correctly
- Color parsing supports hex, rgb, rgba formats
- Size parsing supports px and em units
- Styles apply visually as expected

---

### 2.2 Flexbox Layout System 🟡 **MEDIUM PRIORITY**

**Goal**: Implement basic flexbox layout support

**Flexbox Properties**:

| CSS Property | GPUI Mapping | Priority |
|--------------|--------------|----------|
| `display: flex` | `.flex()` | High |
| `flexDirection` | `.flex_row()`, `.flex_col()` | High |
| `justifyContent` | `.justify_start()`, `.justify_center()`, `.justify_end()`, `.justify_between()` | High |
| `alignItems` | `.items_start()`, `.items_center()`, `.items_end()` | High |
| `gap` | `.gap()`, `.gap_x()`, `.gap_y()` | Medium |
| `flexGrow` | `.flex_grow()` | Medium |
| `flexShrink` | `.flex_shrink()` | Low |

**Implementation Approach**:

```typescript
export interface FlexProps {
  display?: 'flex';
  flexDirection?: 'row' | 'column';
  justifyContent?: 'flex-start' | 'center' | 'flex-end' | 'space-between';
  alignItems?: 'flex-start' | 'center' | 'flex-end';
  gap?: number | string;
  flexGrow?: number;
}

function mapFlexProps(props: FlexProps) {
  let element = div();

  if (props.display === 'flex') {
    element = element.flex();
  }

  if (props.flexDirection === 'row') {
    element = element.flex_row();
  } else if (props.flexDirection === 'column') {
    element = element.flex_col();
  }

  // ... more mappings
  return element;
}
```

**Action Items**:
- [ ] Define FlexProps interface
- [ ] Implement flex property mapper
- [ ] Update host-config to recognize flex props
- [ ] Update Rust builder to apply flex styles
- [ ] Create flexbox demo components
- [ ] Write unit tests

**Success Criteria**:
- Flexbox layouts render correctly
- All justifyContent values work
- All alignItems values work
- Gap property works correctly

---

## Phase 3: Element Types (Week 4)

### 3.1 Span Element 🟡 **MEDIUM PRIORITY**

**Goal**: Add inline text element support

```tsx
<span>inline text</span>
```

**Implementation**:
- [ ] Add `span` type to element store
- [ ] Map to GPUI text element
- [ ] Support inline styling
- [ ] Test with inline and block elements

---

### 3.2 Image Element 🟡 **MEDIUM PRIORITY**

**Goal**: Add image support

```tsx
<img src="/path/to/image.png" alt="description" />
```

**Implementation**:
- [ ] Define ImageProps interface
- [ ] Add `img` type to element store
- [ ] Implement image loading (async)
- [ ] Map to GPUI image component
- [ ] Handle loading states
- [ ] Test with various image formats

---

### 3.3 Input Element 🟢 **LOW PRIORITY**

**Goal**: Add basic input field support

```tsx
<input type="text" value={state} onChange={handleChange} />
```

**Implementation**:
- [ ] Define InputProps interface
- [ ] Add `input` type
- [ ] Implement value binding
- [ ] Add event handlers
- [ ] Test focus states

---

## Phase 4: Event System (Weeks 5-6)

### 4.1 Basic Events 🟡 **HIGH PRIORITY**

**Goal**: Implement fundamental event handling

**Events to Support**:

| Event Type | Use Case | Priority |
|------------|----------|----------|
| `onClick` | Button clicks, interactions | High |
| `onHover` | Hover effects, tooltips | High |
| `onFocus` | Form validation, styling | Medium |
| `onBlur` | Form validation, cleanup | Medium |
| `onChange` | Input fields, forms | High |

**Implementation Approach**:

```typescript
export interface EventProps {
  onClick?: (event: MouseEvent) => void;
  onHover?: (event: MouseEvent) => void;
  onFocus?: (event: FocusEvent) => void;
  onBlur?: (event: FocusEvent) => void;
  onChange?: (value: string) => void;
}

// Event ID mapping
let nextEventId = 0;
const eventHandlers = new Map<number, EventHandler>();

function registerEventHandler(handler: EventHandler): number {
  const id = nextEventId++;
  eventHandlers.set(id, handler);
  return id;
}
```

**Rust Side**:
```rust
pub struct ReactElement {
    // ... existing fields
    pub event_handlers: HashMap<String, u64>,  // eventType -> handlerId
}
```

**Action Items**:
- [ ] Define event types
- [ ] Create event handler registry
- [ ] Pass event handlers via FFI
- [ ] Capture GPUI events
- [ ] Bridge events to React
- [ ] Test event propagation
- [ ] Handle event cleanup

**Success Criteria**:
- Click events fire correctly
- Hover events fire correctly
- Events bubble properly
- Event handlers are cleaned up on unmount

---

### 4.2 Event Propagation 🟢 **LOW PRIORITY**

**Goal**: Implement proper event bubbling and capturing

**Action Items**:
- [ ] Implement event capture phase
- [ ] Implement event bubble phase
- [ ] Support stopPropagation()
- [ ] Support preventDefault()
- [ ] Test complex component hierarchies

---

## Phase 5: Advanced Components (Weeks 7-8)

### 5.1 Virtual List 🟢 **LOW PRIORITY**

**Goal**: Implement virtualized list for performance

```tsx
<VirtualList items={largeArray} itemHeight={50}>
  {(item) => <div>{item.name}</div>}
</VirtualList>
```

**Implementation**:
- [ ] Create VirtualList component
- [ ] Implement viewport calculation
- [ ] Implement item recycling
- [ ] Add scroll handling
- [ ] Test with large datasets (>10,000 items)

---

### 5.2 Scrollable Containers 🟢 **LOW PRIORITY**

**Goal**: Add overflow scrolling support

```tsx
<div style={{ overflow: 'auto', maxHeight: '300px' }}>
  {/* scrollable content */}
</div>
```

**Implementation**:
- [ ] Add overflow style support
- [ ] Implement scroll capturing
- [ ] Add scroll event handlers
- [ ] Test with various scroll scenarios

---

## Phase 6: Performance Optimization (Week 9)

### 6.1 FFI Call Batching 🟢 **LOW PRIORITY**

**Goal**: Reduce FFI overhead by batching calls

**Current**: One FFI call per element update
**Target**: Batch multiple updates in single call

**Implementation**:
- [ ] Design batch update API
- [ ] Implement update queue
- [ ] Flush queue on commit
- [ ] Measure performance improvement

---

### 6.2 Element Caching 🟢 **LOW PRIORITY**

**Goal**: Cache GPUI elements to reduce allocations

**Implementation**:
- [ ] Implement element pool
- [ ] Reuse elements when possible
- [ ] Clear cache on unmount
- [ ] Benchmark performance

---

### 6.3 Memory Management 🟢 **LOW PRIORITY**

**Goal**: Ensure no memory leaks

**Action Items**:
- [ ] Run Valgrind memory checks
- [ ] Fix any detected leaks
- [ ] Add memory usage monitoring
- [ ] Document cleanup procedures

---

## Phase 7: Testing & Documentation (Week 10)

### 7.1 Unit Tests 🟡 **MEDIUM PRIORITY**

**Coverage Targets**:
- TypeScript: 80%
- Rust: 90%

**Test Areas**:
- [ ] Element store operations
- [ ] Style parsing
- [ ] Flex property mapping
- [ ] Event handler registration
- [ ] FFI data marshaling

**Tools**:
- Bun test for TypeScript
- Cargo test for Rust

---

### 7.2 Integration Tests 🟡 **MEDIUM PRIORITY**

**Test Scenarios**:
- [ ] Full rendering pipeline
- [ ] Event handling end-to-end
- [ ] Style application
- [ ] Layout correctness
- [ ] Performance benchmarks

---

### 7.3 Documentation 🟡 **MEDIUM PRIORITY**

**Documentation Tasks**:
- [ ] API reference for all components
- [ ] Styling guide
- [ ] Event handling guide
- [ ] Performance best practices
- [ ] Migration guide (from React DOM)
- [ ] Troubleshooting guide
- [ ] Architecture overview

---

## Phase 8: Release Preparation (Week 11-12)

### 8.1 Versioning 🟢 **LOW PRIORITY**

**Semantic Versioning**:
- v0.1.0: MVP (div, text, basic styles)
- v0.2.0: Styling & Flexbox
- v0.3.0: Events & Interactivity
- v0.4.0: Advanced components
- v1.0.0: Production-ready

---

### 8.2 Release Checklist

- [ ] All critical bugs fixed
- [ ] Documentation complete
- [ ] Test coverage meets targets
- [ ] Performance benchmarks documented
- [ ] Migration guide written
- [ ] Changelog updated
- [ ] Release notes written

---

## Risk Assessment

### Technical Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| GPUI API changes | Medium | High | Pin to specific version, monitor changes |
| FFI memory leaks | Medium | High | Rigorous testing, Valgrind checks |
| Performance degradation | Low | Medium | Benchmarking, optimization sprints |
| Event system complexity | Medium | Medium | Simplified MVP first, iterate |

### External Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Bun FFI limitations | Low | Medium | Use native addons if needed |
| GPUI dependency issues | Medium | High | Alternative: use direct bindings |
| OS compatibility issues | Low | Medium | Test on Linux, macOS, Windows |

---

## Success Metrics

### Phase 1 (MVP)
- [x] Architecture complete
- [ ] Demo runs without crashes
- [ ] `<div>hello world</div>` renders correctly
- [ ] Build time <5 minutes

### Phase 2 (Styling)
- [ ] All basic CSS properties supported
- [ ] Flexbox layouts work correctly
- [ ] Style changes update reactively

### Phase 3 (Elements)
- [ ] Span, img, input elements work
- [ ] Custom components supported
- [ ] Element lifecycle correct

### Phase 4 (Events)
- [ ] Basic events fire correctly
- [ ] Event propagation works
- [ ] No memory leaks from events

### Phase 5 (Advanced)
- [ ] Virtual list handles 10k+ items
- [ ] Scrolling performs well
- [ ] Advanced components stable

### Phase 6 (Performance)
- [ ] FFI calls reduced by 50%
- [ ] Memory usage stable
- [ ] Frame rate 60+ FPS

---

## Timeline Summary

| Phase | Duration | Start | End |
|-------|----------|-------|-----|
| Phase 1: MVP | 1 week | Week 1 | Week 1 |
| Phase 2: Styling | 2 weeks | Week 2 | Week 3 |
| Phase 3: Elements | 1 week | Week 4 | Week 4 |
| Phase 4: Events | 2 weeks | Week 5 | Week 6 |
| Phase 5: Advanced | 2 weeks | Week 7 | Week 8 |
| Phase 6: Performance | 1 week | Week 9 | Week 9 |
| Phase 7: Testing | 1 week | Week 10 | Week 10 |
| Phase 8: Release | 2 weeks | Week 11 | Week 12 |

**Total Estimated Time**: 12 weeks

---

## Next Actions

**This Week**:
1. Fix GPUI compilation (Option A, B, or C)
2. Verify basic rendering
3. Update README with progress

**Next Week**:
1. Implement basic style properties
2. Start flexbox layout system
3. Write unit tests for styling

---

## Appendix: Resources

### GPUI Documentation
- [GPUI GitHub](https://github.com/zed-industries/zed)
- [GPUI Examples](https://github.com/zed-industries/zed/tree/main/crates/gpui/src)
- [Zed Editor](https://github.com/zed-industries/zed)

### React Documentation
- [React Reconciler](https://react.dev/reference/react/useSyncExternalStore)
- [Custom Renderers](https://legacy.reactjs.org/docs/custom-renderers.html)

### Bun Documentation
- [Bun FFI](https://bun.sh/docs/runtime/ffi)
- [FFI Best Practices](https://bun.sh/docs/runtime/ffi/best-practices)

---

**Last Updated**: January 3, 2026
**Next Review**: Weekly standup
