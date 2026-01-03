# React-GPUI Renderer

A React renderer for GPUI (Zed's GPU-accelerated UI framework) using Bun FFI.

### Architecture

```
React Component (JSX)
         ↓
React Reconciler (host config)
         ↓
Element Store (JS)
         ↓
Bun FFI (gpui-binding.ts)
         ↓
Rust FFI (libgpui_renderer.so)
         ↓
GPUI Runtime (Rust)
         ↓
GPU Rendering (GPUI)
```

## To Run Demo

1. Build Rust library (will take time due to GPUI size):
   ```bash
   cd rust && cargo build --release
   ```

2. Run demo:
   ```bash
   bun run demo/index.ts
   ```
