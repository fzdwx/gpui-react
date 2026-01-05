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

### Install Dependency

```bash
bun add gpui-react react
```

### Usage

Create a GPUI window and render React components:

```tsx
// index.ts
import React from "react";
import { createRoot } from "gpui-react";
import { App } from "./app";

const root = createRoot({
  width: 800,
  height: 800,
});

root.render(
  <div style={{ backgroundColor: "#1e1e1e", padding: 40 }}>
    <App />
  </div>
);
```

```tsx
// app.tsx
import React, { useState, useEffect } from "react";

export function App() {
  const [text, setText] = useState("Hello GPUI!");

  useEffect(() => {
    const timer = setTimeout(() => {
      setText(`Updated: ${new Date().toLocaleTimeString()}`);
    }, 1000);
    return () => clearTimeout(timer);
  }, []);

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        gap: 20,
        backgroundColor: "#1e1e1e",
        padding: 40,
        alignItems: "center",
      }}
    >
      <div
        style={{
          color: "#1db588",
          fontSize: 25,
          fontWeight: "bold",
        }}
      >
        {text}
      </div>
      <div
        style={{
          backgroundColor: "#4ed93b",
          color: "white",
          padding: "15px 30px",
          borderRadius: 8,
          fontSize: 30,
          cursor: "pointer",
        }}
      >
        Click me!
      </div>
    </div>
  );
}
```

### Supported Elements

| Element | GPUI Mapping | Notes |
|---------|--------------|-------|
| `div` | `div()` | Block container |
| `span` | `span()` | Inline container, contains text children |
| `text` | Text nodes | Always child of span |


## Documentation

- **[AGENTS.md](./AGENTS.md)** - Root knowledge base
- **[src/reconciler/AGENTS.md](./src/reconciler/AGENTS.md)** - React reconciler layer
- **[rust/src/AGENTS.md](./rust/src/AGENTS.md)** - Rust FFI layer
- **[ROADMAP.md](./ROADMAP.md)** - Development roadmap

## Requirements

- [Bun](https://bun.sh) ≥ 1.0
- [Rust](https://rust-lang.org) ≥ 1.70
- GPUI dependencies (downloaded automatically on first cargo build)