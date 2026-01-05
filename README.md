# React-GPUI Renderer

A React renderer for GPUI (Zed's GPU-accelerated UI framework) using Bun native FFI.

## Architecture

```
React Component (JSX)
         ↓
React Reconciler (src/reconciler/host-config.ts)
         ↓
Element Store (src/reconciler/element-store.ts)
         ↓
Bun FFI (src/reconciler/gpui-binding.ts)
         ↓
Rust FFI (rust/src/lib.rs)
         ↓
GPUI Runtime (rust/src/renderer.rs)
         ↓
GPU Rendering (GPUI)
```

## Project Structure

```
gpui-react/
├── rust/              # Rust FFI library (cdylib)
│   └── src/        # FFI exports, GPUI rendering, command bus
├── src/reconciler/   # React reconciler, FFI bindings, element store
└── demo/             # Demo applications (5 entry points)
```

## Quick Start

### 1. Build Rust Library

```bash
cd rust && cargo build --release
```

### 2. Run Demos

```bash
bun run demo               # Basic elements
bun run event-demo         # Event handling
bun run styled-demo        # Styled components
bun run flex-demo          # Flexbox layout
bun run elements-demo      # Span, image elements
```

## API Reference

### createRoot(container)

Create a GPUI rendering root for a container element.

```typescript
import { createRoot } from 'gpui-react';

const container = document.getElementById('root');
const root = createRoot(container);
root.render(<App />);
```

### Supported Elements

| Element | GPUI Mapping | Notes |
|---------|--------------|-------|
| `div` | `div()` | Block container |
| `span` | `span()` | Inline container, contains text children |
| `text` | Text nodes | Always child of span |

### Event Handlers

```jsx
<div onClick={handleClick} onHover={handleHover}>
  <span>Click me</span>
</div>
```
的恩托斯
**Supported:** `onClick`, `onHover`, `onMouseEnter`, `onMouseLeave`

### Styles

```jsx
<div style={{
  color: '#fff',
  backgroundColor: '#333',
  fontSize: 16,
  width: 200,
  height: 100,
  margin: 10,
  padding: 20,
  borderRadius: 8,
  display: 'flex',
  flexDirection: 'row',
  justifyContent: 'center',
  alignItems: 'center',
  gap: 8
}}>
  Content
</div>
```

## Documentation

- **[AGENTS.md](./AGENTS.md)** - Root knowledge base
- **[src/reconciler/AGENTS.md](./src/reconciler/AGENTS.md)** - React reconciler layer
- **[rust/src/AGENTS.md](./rust/src/AGENTS.md)** - Rust FFI layer
- **[ROADMAP.md](./ROADMAP.md)** - Development roadmap

## Requirements

- [Bun](https://bun.sh) ≥ 1.0
- [Rust](https://rust-lang.org) ≥ 1.70
- GPUI dependencies (downloaded automatically on first cargo build)