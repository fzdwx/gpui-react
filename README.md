# gpui-react

React renderer for GPUI (Zed's GPU-accelerated UI) using Bun native FFI.

## Quick Start

```bash
# not yet
bun add gpui-react react
```

the code:

```tsx
import { createRoot } from "gpui-react";

createRoot({ width: 800, height: 600 }).render(
    <div style={{ backgroundColor: "#1e1e1e", padding: 40 }}>
        <div style={{ color: "#ffffff", fontSize: 24 }}>Hello World</div>
    </div>
);
```

## Demos

See [demo/](./demo/) for complete examples.

```bash
bun run build:rust && bun run copy:native  # compile native lib
bun run event-demo                         # Event handling
bun run drawing-demo                       # Drawing board
bun run canvas-demo                        # Canvas drawing
```

https://github.com/user-attachments/assets/30871650-e776-4c79-9184-529a55f0f74b

## Requirements

- [Bun](https://bun.sh) ≥ 1.3.5
- [Rust](https://rust-lang.org) ≥ 1.92
