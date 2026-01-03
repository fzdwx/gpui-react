# React-GPUI Renderer Implementation Plan

## Executive Summary

This plan creates a React renderer for GPUI (Zed's GPU-accelerated UI framework) using Bun FFI. The minimal demo will render `<div>hello world</div>` using React components, with GPUI handling actual GPU-accelerated rendering.

**Architectural Approach**: Direct mapping with GlobalElementIds - React's persistent fiber tree maps to GPUI's ephemeral element tree via GlobalElementIds for state tracking.

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Project Structure](#project-structure)
3. [Phase 1: Project Initialization](#phase-1-project-initialization)
4. [Phase 2: Bun FFI Setup](#phase-2-bun-ffi-setup)
5. [Phase 3: Rust GPUI Integration](#phase-3-rust-gpui-integration)
6. [Phase 4: React Renderer Host Config](#phase-4-react-renderer-host-config)
7. [Phase 5: React-to-GPUI Bridge](#phase-5-react-to-gpui-bridge)
8. [Phase 6: Minimal Demo](#phase-6-minimal-demo)
9. [Build Commands](#build-commands)
10. [Testing Strategy](#testing-strategy)
11. [Troubleshooting](#troubleshooting)

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    React Application                          │
│                (JavaScript/TypeScript)                        │
│                                                             │
│  <div>hello world</div>                                    │
│       ↓                                                      │
│  React Reconciler (scheduler, fiber tree)                     │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     ↓ React Reconciler API
┌─────────────────────────────────────────────────────────────────┐
│              Host Config (Bun FFI Layer)                     │
│                                                             │
│  - createInstance()                                          │
│  - appendChild()                                             │
│  - commitUpdate()                                            │
│  - commitTextUpdate()                                        │
│  └──► Bun FFI calls → Rust functions                        │
└────────────────────┬────────────────────────────────────────────┘
                     │ Bun FFI Boundary
                     ↓
┌─────────────────────────────────────────────────────────────────┐
│              Rust GPUI Runtime (libgpui_renderer.so)          │
│                                                             │
│  GlobalElementId ↔ React Fiber mapping                         │
│  ElementData store (type, props, children)                   │
│                                                             │
│  extern "C" functions:                                      │
│  - register_element()                                         │
│  - update_props()                                            │
│  - render_frame()                                            │
│  - destroy_app()                                             │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     ↓
┌─────────────────────────────────────────────────────────────────┐
│                   GPUI Framework                              │
│                                                             │
│  Application::new().run()                                     │
│  Window::open()                                              │
│  Entity<T: Render>                                           │
│                                                             │
│  Render phase (per frame):                                    │
│  1. Build element tree (ephemeral)                            │
│  2. request_layout() (Taffy flexbox)                         │
│  3. prepaint()                                              │
│  4. paint() (blade-graphics)                                 │
│  5. Elements dropped                                         │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     ↓
┌─────────────────────────────────────────────────────────────────┐
│                   GPU Rendering                              │
│                                                             │
│  120 FPS target                                             │
│  Subpixel anti-aliasing                                      │
│  GPU texture atlas caching                                     │
└─────────────────────────────────────────────────────────────────┘
```

### Key Design Decisions

1. **GlobalElementId as Bridge**: Use GPUI's built-in state tracking mechanism
   - React fibers map to GlobalElementIds
   - State persisted across frame rebuilds
   - Matches GPUI's ephemeral design

2. **Pure Reconstruction per Frame**: Build entire GPUI element tree each frame
   - React's fiber tree tracks structure
   - GPUI elements rebuilt fresh each frame
   - No virtual tree - just data marshaling

3. **Minimal MVP First**: Start with div + text only
   - No layout engine initially (fixed positioning)
   - No styling (default black on white)
   - Expand after basic rendering works

---

## Project Structure

```
gpui-react/
├── package.json                      # Bun project config
├── tsconfig.json                     # TypeScript config
├── bun.lockb                        # Lockfile (auto-generated)
├── Cargo.toml                        # Rust project config
├── src/
│   └── renderer/
│       ├── index.ts                  # Main export, React.createRoot()
│       ├── host-config.ts            # React Reconciler host config
│       ├── element-store.ts           # Element data store (JS side)
│       └── gpui-binding.ts          # Bun FFI bindings
├── rust/
│   ├── src/
│   │   ├── lib.rs                  # FFI exports
│   │   ├── app.rs                  # GPUI App wrapper
│   │   ├── window.rs               # Window management
│   │   ├── elements.rs             # Element tree builder
│   │   └── ffi_types.rs           # FFI-compatible types
│   └── target/
│       ├── release/
│       │   └── libgpui_renderer.so  # Compiled shared library
│       └── debug/
│           └── libgpui_renderer.so
├── demo/
│   ├── index.ts                    # Demo entry point
│   ├── app.tsx                     # React demo app
│   └── styles.css                  # (optional)
└── README.md
```

---

## Phase 1: Project Initialization

### Goal
Set up empty project with build tooling for both Bun (TypeScript) and Rust.

### Steps

#### 1.1 Initialize Bun Project
```bash
# Already in /home/like/workspaces/gpui-react
# Initialize package.json
bun init -y

# Expected output: package.json with default config
```

**package.json** (initial):
```json
{
  "name": "gpui-react",
  "version": "0.1.0",
  "type": "module",
  "module": "src/renderer/index.ts",
  "devDependencies": {
    "bun-types": "latest",
    "typescript": "latest"
  }
}
```

#### 1.2 Initialize Rust Project
```bash
# Initialize Cargo.toml
cargo init --lib

# Expected output: Cargo.toml + src/lib.rs
```

**Cargo.toml** (initial):
```toml
[package]
name = "gpui-renderer"
version = "0.1.0"
edition = "2021"

[lib]
name = "gpui_renderer"
crate-type = ["cdylib"]  # Dynamic library for Bun FFI

[dependencies]
# Dependencies will be added in Phase 3
```

#### 1.3 Configure TypeScript
Create **tsconfig.json**:
```json
{
  "compilerOptions": {
    "target": "ESNext",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "lib": ["ESNext", "DOM"],
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "declaration": true,
    "declarationMap": true,
    "outDir": "./dist"
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "rust/target", "dist"]
}
```

#### 1.4 Create Directory Structure
```bash
mkdir -p src/renderer
mkdir -p rust/src
mkdir -p demo

# Verify structure
tree -L 2
```

**Expected output**:
```
.
├── Cargo.toml
├── package.json
├── src/
│   └── renderer/
├── rust/
│   └── src/
└── demo/
```

### Verification Criteria
- ✅ `bun install` works without errors
- ✅ `cargo check` works without errors
- ✅ Directory structure matches plan

---

## Phase 2: Bun FFI Setup

### Goal
Create minimal working Bun FFI integration with a simple Rust function.

### Steps

#### 2.1 Implement Simple Rust FFI Function

**rust/src/lib.rs**:
```rust
use std::ffi::{CString, CStr};
use std::os::raw::c_char;

/// Returns a greeting message
#[no_mangle]
pub extern "C" fn gpui_greet(name: *const c_char) -> *mut c_char {
    unsafe {
        if name.is_null() {
            return CString::new("Hello, Stranger!").unwrap().into_raw();
        }

        let name_str = CStr::from_ptr(name).to_string_lossy();
        let greeting = format!("Hello, {}!", name_str);
        CString::new(greeting).unwrap().into_raw()
    }
}

/// Frees a string allocated by Rust
#[no_mangle]
pub extern "C" fn gpui_free_string(s: *mut c_char) {
    unsafe {
        if !s.is_null() {
            let _ = CString::from_raw(s);
        }
    }
}
```

#### 2.2 Build Rust Shared Library
```bash
cd rust
cargo build --release
cd ..

# Expected output: rust/target/release/libgpui_renderer.so (Linux)
# or: rust/target/release/libgpui_renderer.dylib (macOS)
# or: rust/target/release/gpui_renderer.dll (Windows)
```

#### 2.3 Create Bun FFI Binding

**src/renderer/gpui-binding.ts**:
```typescript
import { dlopen, FFIType, CString, suffix } from "bun:ffi";
import { join } from "path";

// Platform-specific library name
const libName = `libgpui_renderer.${suffix}`;
const libPath = join(import.meta.dir, "../../rust/target/release", libName);

console.log(`Loading GPUI library from: ${libPath}`);

const lib = dlopen(libPath, {
  gpui_greet: {
    args: [FFIType.cstring],
    returns: FFIType.cstring,
  },
  gpui_free_string: {
    args: [FFIType.ptr],
    returns: FFIType.ptr,
  },
});

if (!lib.symbols) {
  throw new Error("Failed to load GPUI library");
}

export function greet(name: string): string {
  const result = lib.symbols.gpui_greet(name);
  const greeting = new CString(result);
  lib.symbols.gpui_free_string(result);
  return greeting.toString();
}
```

#### 2.4 Test Bun FFI Integration

**demo/test-ffi.ts**:
```typescript
import { greet } from "../src/renderer/gpui-binding.js";

console.log(greet("GPUI"));
// Expected output: "Hello, GPUI!"
```

```bash
bun run demo/test-ffi.ts
```

### Verification Criteria
- ✅ Rust compiles without errors
- ✅ Bun loads library successfully
- ✅ `greet("GPUI")` outputs "Hello, GPUI!"
- ✅ No memory leaks (all freed)

---

## Phase 3: Rust GPUI Integration

### Goal
Integrate GPUI framework and implement window creation + basic rendering.

### Steps

#### 3.1 Add GPUI Dependencies

**rust/Cargo.toml**:
```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed.git", rev = "main" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

#### 3.2 Implement FFI Types

**rust/src/ffi_types.rs**:
```rust
use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_void};

#[repr(C)]
pub struct FfiResult {
    pub status: i32,
    pub error_msg: *mut c_char,
}

impl FfiResult {
    pub fn success() -> Self {
        FfiResult {
            status: 0,
            error_msg: std::ptr::null_mut(),
        }
    }

    pub fn error(message: &str) -> Self {
        FfiResult {
            status: 1,
            error_msg: CString::new(message).unwrap().into_raw(),
        }
    }
}

#[repr(C)]
pub struct ElementData {
    pub global_id: u64,
    pub type_ptr: *const c_char,
    pub text_ptr: *const c_char,
    pub child_count: u32,
    pub children_ptr: *const u64,
}

#[repr(C)]
pub struct AppState {
    pub app_ptr: *mut c_void,
    pub window_ptr: *mut c_void,
}
```

#### 3.3 Implement App Wrapper

**rust/src/app.rs**:
```rust
use gpui::{div, prelude::*, px, rgb, App, Application, Bounds, Context, Window, WindowBounds, WindowOptions};
use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use std::sync::Mutex;

use crate::ffi_types::{AppState, ElementData, FfiResult};

// Global app state (simplified - single window for MVP)
lazy_static::lazy_static! {
    static ref GLOBAL_APP: Mutex<Option<Application>> = Mutex::new(None);
}

pub struct GpuiApp {
    app: Application,
}

impl GpuiApp {
    pub fn new() -> Self {
        let app = Application::new();
        Self { app }
    }

    pub fn initialize_window(&mut self, width: f32, height: f32) -> Result<(*mut c_void, *mut c_void), String> {
        let bounds = Bounds::origin(px(0.0), px(0.0), px(width), px(height));

        let window_handle = self.app.run(|cx: &mut App| {
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |window, cx| {
                    // Create root entity for rendering
                    cx.new(|_| RootView { text: "Ready".into() })
                },
            )
        });

        match window_handle {
            Ok(handle) => {
                // In real implementation, store handle for later use
                Ok((std::ptr::null_mut(), std::ptr::null_mut())) // Simplified
            }
            Err(e) => Err(format!("Failed to create window: {}", e)),
        }
    }
}

struct RootView {
    text: SharedString,
}

impl Render for RootView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .justify_center()
            .items_center()
            .size(px(800.0), px(600.0))
            .bg(rgb(0xffffff))
            .text_color(rgb(0x000000))
            .child(self.text.clone())
    }
}
```

#### 3.4 Implement Element Tree Builder

**rust/src/elements.rs**:
```rust
use gpui::{div, prelude::*, px, rgb, SharedString, IntoElement};
use std::ffi::CStr;
use std::os::raw::c_char;

use crate::ffi_types::ElementData;

pub fn build_element_tree(data: &ElementData) -> Result<impl IntoElement, String> {
    let type_str = unsafe {
        if data.type_ptr.is_null() {
            return Err("type_ptr is null".to_string());
        }
        CStr::from_ptr(data.type_ptr).to_str()
            .map_err(|e| format!("Invalid UTF-8 in type: {}", e))?
    };

    let text_str = if !data.text_ptr.is_null() {
        Some(unsafe {
            CStr::from_ptr(data.text_ptr).to_str()
                .map_err(|e| format!("Invalid UTF-8 in text: {}", e))?
        })
    } else {
        None
    };

    match type_str {
        "div" => {
            let mut element = div()
                .flex()
                .justify_center()
                .items_center()
                .size(px(800.0), px(600.0))
                .bg(rgb(0xffffff))
                .text_color(rgb(0x000000));

            if let Some(text) = text_str {
                element = element.child(SharedString::from(text));
            }

            // TODO: Add children processing
            Ok(element)
        }
        "text" => {
            let text = text_str.ok_or_else(|| "text element requires text content".to_string())?;
            Ok(div().child(SharedString::from(text)))
        }
        _ => Err(format!("Unknown element type: {}", type_str)),
    }
}
```

#### 3.5 Export FFI Functions

**rust/src/lib.rs**:
```rust
mod app;
mod elements;
mod ffi_types;

use std::ffi::CString;
use std::os::raw::{c_char, c_void};
use std::sync::Mutex;
use lazy_static::lazy_static;

use crate::app::GpuiApp;
use crate::ffi_types::{FfiResult};

lazy_static::lazy_static! {
    static ref GPUI_APP: Mutex<Option<GpuiApp>> = Mutex::new(None);
}

/// Initialize GPUI application and window
#[no_mangle]
pub extern "C" fn gpui_init(width: f32, height: f32) -> FfiResult {
    let mut app_lock = GPUI_APP.lock().unwrap();
    *app_lock = Some(GpuiApp::new());

    if let Some(ref mut app) = *app_lock {
        match app.initialize_window(width, height) {
            Ok(_) => FfiResult::success(),
            Err(msg) => FfiResult::error(&msg),
        }
    } else {
        FfiResult::error("Failed to create GPUI app")
    }
}

/// Render a frame from element data
#[no_mangle]
pub extern "C" fn gpui_render_frame(
    element_data: *const crate::ffi_types::ElementData,
) -> FfiResult {
    unsafe {
        if element_data.is_null() {
            return FfiResult::error("element_data is null");
        }

        let data = &*element_data;
        match crate::elements::build_element_tree(data) {
            Ok(_element) => {
                // TODO: Actually render to GPUI window
                FfiResult::success()
            }
            Err(msg) => FfiResult::error(&msg),
        }
    }
}

/// Free error messages
#[no_mangle]
pub extern "C" fn gpui_free_result(result: FfiResult) {
    if !result.error_msg.is_null() {
        let _ = CString::from_raw(result.error_msg);
    }
}
```

#### 3.6 Update Cargo.toml for Missing Dependencies

**rust/Cargo.toml**:
```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed.git", rev = "main" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
lazy_static = "1.4"
```

#### 3.7 Build and Test

```bash
cd rust
cargo build --release
cd ..

# Expected: libgpui_renderer.so built successfully
```

### Verification Criteria
- ✅ Cargo compiles without errors
- ✅ `gpui_init(800, 600)` returns success
- ✅ Window opens (visual verification)
- ✅ `gpui_render_frame()` processes element data

---

## Phase 4: React Renderer Host Config

### Goal
Implement React Reconciler host config for GPUI rendering.

### Steps

#### 4.1 Install React Dependencies

```bash
bun add react react-reconciler
bun add -d @types/react
```

#### 4.2 Create Element Store

**src/renderer/element-store.ts**:
```typescript
export interface ElementData {
  globalId: number;
  type: string;
  text?: string;
  props?: Record<string, any>;
  children: number[];
}

class ElementStore {
  private store = new Map<number, ElementData>();
  private nextId = 1;

  createElement(type: string, props?: Record<string, any>): number {
    const globalId = this.nextId++;
    this.store.set(globalId, {
      globalId,
      type,
      props,
      text: props?.children as string,
      children: [],
    });
    return globalId;
  }

  updateElement(globalId: number, props?: Record<string, any>): void {
    const element = this.store.get(globalId);
    if (!element) throw new Error(`Element ${globalId} not found`);

    if (props) {
      element.props = props;
      element.text = props?.children as string;
    }
  }

  appendChild(parentId: number, childId: number): void {
    const parent = this.store.get(parentId);
    const child = this.store.get(childId);
    if (!parent || !child) throw new Error("Element not found");

    parent.children.push(childId);
  }

  removeChild(parentId: number, childId: number): void {
    const parent = this.store.get(parentId);
    if (!parent) throw new Error("Element not found");

    const index = parent.children.indexOf(childId);
    if (index !== -1) parent.children.splice(index, 1);
  }

  getElement(globalId: number): ElementData | undefined {
    return this.store.get(globalId);
  }

  getTree(rootId: number): ElementData {
    return this.store.get(rootId)!;
  }
}

export const elementStore = new ElementStore();
```

#### 4.3 Implement Host Config

**src/renderer/host-config.ts**:
```typescript
import * as ReactReconciler from "react-reconciler";
import { elementStore } from "./element-store";
import { renderFrame } from "./gpui-binding";

export type Fiber = {
  element: any;
  stateNode: number;
};

const hostConfig = ReactReconciler.HostConfig<
  string,
  Element,
  null,
  Fiber,
  never,
  never,
  never,
  never,
  {}
> & {
  getPublicInstance(instance: number): Element;
  prepareForCommit(): void;
  resetAfterCommit(): void;
  getRootHostContext(): null;
  getChildHostContext(): null;
  shouldSetTextContent(type: string, props: any): boolean;
};

const config: hostConfig = {
  // Lifecycle methods
  getPublicInstance(instance: number) {
    return document.createElement("div"); // Dummy for now
  },

  getRootHostContext() {
    return null;
  },

  getChildHostContext() {
    return null;
  },

  prepareForCommit() {
    // Called before committing changes
  },

  resetAfterCommit() {
    // Called after commit - trigger render
    const rootElement = elementStore.getTree(1);
    renderFrame(rootElement);
  },

  shouldSetTextContent(type: string, props: any) {
    return typeof props.children === "string";
  },

  // Mutation methods
  createInstance(type: string, props: any) {
    const globalId = elementStore.createElement(type, props);
    return globalId;
  },

  appendInitialChild(parent: number, child: number) {
    elementStore.appendChild(parent, child);
  },

  appendChild(parent: number, child: number) {
    elementStore.appendChild(parent, child);
  },

  insertBefore(parent: number, child: number, beforeChild: number) {
    const parentEl = elementStore.getElement(parent)!;
    const beforeIndex = parentEl.children.indexOf(beforeChild);
    parentEl.children.splice(beforeIndex, 0, child);
  },

  removeChild(parent: number, child: number) {
    elementStore.removeChild(parent, child);
  },

  // Update methods
  commitUpdate(
    fiber: Fiber,
    updatePayload: any,
    type: string,
    oldProps: any,
    newProps: any
  ) {
    elementStore.updateElement(fiber.stateNode, newProps);
  },

  commitTextUpdate(textInstance: number, oldText: string, newText: string) {
    elementStore.updateElement(textInstance, { children: newText });
  },

  // Text
  createTextInstance(text: string) {
    const globalId = elementStore.createElement("text", { children: text });
    return globalId;
  },

  // Finalization
  finalizeInitialChildren(instance: number, type: string, props: any) {
    return false; // No auto-focus needed
  },

  prepareUpdate(
    instance: number,
    type: string,
    oldProps: any,
    newProps: any,
    rootContainerInstance: number,
    currentHostContext: null
  ) {
    return newProps; // Return payload for commitUpdate
  },

  // Other required methods (dummy implementations)
  appendChildToContainer(container: number, child: number) {
    // Root container - child is always index 1
  },

  insertInContainerBefore(container: number, child: number, beforeChild: number) {
    // Not used in MVP
  },

  removeChildFromContainer(container: number, child: number) {
    // Not used in MVP
  },

  hideInstance(instance: number) {
    // Not used in MVP
  },

  unhideInstance(instance: number, props: any) {
    // Not used in MVP
  },

  clearContainer(container: number) {
    // Not used in MVP
  },

  prepareScopeUpdate(scopeInstance: any, instance: any) {
    // Not used in MVP
  },

  getInstanceFromScope(scopeInstance: any): number {
    return 0; // Not used in MVP
  },

  createPortalChild(container: any, child: any): never {
    throw new Error("Portals not supported");
  },

  scheduleTimeout(fn: Function, delay: number): void {
    // Not used in MVP
  },

  cancelTimeout(id: number): void {
    // Not used in MVP
  },

  noTimeout: -1,

  isActive(): boolean {
    return true;
  },
};

export const reconciler = ReactReconciler(config);
```

#### 4.4 Implement renderFrame Binding

Update **src/renderer/gpui-binding.ts**:
```typescript
import { dlopen, FFIType, CString, suffix, ptr, toArrayBuffer } from "bun:ffi";
import { join } from "path";

const libName = `libgpui_renderer.${suffix}`;
const libPath = join(import.meta.dir, "../../rust/target/release", libName);

const lib = dlopen(libPath, {
  gpui_init: {
    args: [FFIType.f32, FFIType.f32],
    returns: FFIType.ptr,
  },

  gpui_render_frame: {
    args: [FFIType.ptr],
    returns: FFIType.ptr,
  },

  gpui_free_result: {
    args: [FFIType.ptr],
    returns: FFIType.ptr,
  },
});

// Element data structure matching Rust
interface ElementDataFFI {
  globalId: number;
  typePtr: number;
  textPtr: number;
  childCount: number;
  childrenPtr: number;
}

export function init(width: number, height: number): void {
  const resultPtr = lib.symbols.gpui_init(width, height);
  const status = new Int32Array(toArrayBuffer(resultPtr, 0, 4))[0];

  if (status !== 0) {
    const errorPtr = new Int32Array(toArrayBuffer(resultPtr, 8, 8))[0];
    const error = new CString(errorPtr);
    lib.symbols.gpui_free_result(resultPtr);
    throw new Error(`GPUI init failed: ${error}`);
  }

  lib.symbols.gpui_free_result(resultPtr);
}

export function renderFrame(element: any): void {
  // Convert element to FFI structure
  const typeCString = new CString(element.type);
  const textCString = element.text ? new CString(element.text) : new CString("");

  const childrenBuffer = new Uint64Array(element.children);
  const childrenPtr = ptr(childrenBuffer);

  const elementData: ElementDataFFI = {
    globalId: element.globalId,
    typePtr: typeCString.ptr,
    textPtr: textCString.ptr,
    childCount: element.children.length,
    childrenPtr: childrenPtr,
  };

  const resultPtr = lib.symbols.gpui_render_frame(ptr(elementData));
  const status = new Int32Array(toArrayBuffer(resultPtr, 0, 4))[0];

  if (status !== 0) {
    const errorPtr = new Int32Array(toArrayBuffer(resultPtr, 8, 8))[0];
    const error = new CString(errorPtr);
    lib.symbols.gpui_free_result(resultPtr);
    throw new Error(`GPUI render failed: ${error}`);
  }

  lib.symbols.gpui_free_result(resultPtr);
}
```

#### 4.5 Create createRoot API

**src/renderer/index.ts**:
```typescript
import * as React from "react";
import { reconciler } from "./host-config";
import { init } from "./gpui-binding";

export type Root = {
  render: (children: React.ReactNode) => void;
  unmount: () => void;
};

export function createRoot(): Root {
  // Initialize GPUI
  init(800, 600);

  // Create root container
  const rootContainer = reconciler.createContainer(
    1, // Root ID
    null,
    false,
    null,
    "",
    () => {},
    null
  );

  return {
    render(children: React.ReactNode) {
      reconciler.updateContainer(
        children,
        rootContainer,
        null,
        null,
        () => {
          console.log("Render complete");
        }
      );
    },
    unmount() {
      reconciler.updateContainer(null, rootContainer, null, null, () => {});
    },
  };
}
```

### Verification Criteria
- ✅ TypeScript compiles without errors
- ✅ Reconciler creates host instances
- ✅ Element store tracks hierarchy correctly
- ✅ `renderFrame()` is called after commits

---

## Phase 5: React-to-GPUI Bridge

### Goal
Connect React reconciler to GPUI rendering via element tree marshaling.

### Steps

#### 5.1 Simplify Element Store for MVP

Update **src/renderer/element-store.ts**:
```typescript
export interface ElementData {
  globalId: number;
  type: string;
  text?: string;
  children: number[];
}

class ElementStore {
  private store = new Map<number, ElementData>();
  private nextId = 1;

  createElement(type: string, text?: string): number {
    const globalId = this.nextId++;
    this.store.set(globalId, {
      globalId,
      type,
      text,
      children: [],
    });
    return globalId;
  }

  appendChild(parentId: number, childId: number): void {
    const parent = this.store.get(parentId);
    if (!parent) throw new Error(`Parent element ${parentId} not found`);
    parent.children.push(childId);
  }

  getElement(globalId: number): ElementData | undefined {
    return this.store.get(globalId);
  }

  getRoot(): ElementData {
    return this.store.get(1)!;
  }
}

export const elementStore = new ElementStore();
```

#### 5.2 Update Host Config for MVP

Update **src/renderer/host-config.ts** - simplify to only handle div + text:
```typescript
import * as ReactReconciler from "react-reconciler";
import { elementStore } from "./element-store";
import { renderFrame } from "./gpui-binding";

const config: ReactReconciler.HostConfig<string, any, any, number, any, any, any, any, any> & {
  commitUpdate(instance: number, updatePayload: any, type: string, oldProps: any, newProps: any): void;
  getPublicInstance(instance: number): Element;
  prepareForCommit(): void;
  resetAfterCommit(): void;
  getRootHostContext(): null;
  getChildHostContext(): null;
  shouldSetTextContent(type: string, props: any): boolean;
  resetTextContent(instance: number): void;
  createTextInstance(text: string): number;
  commitTextUpdate(textInstance: number, oldText: string, newText: string): void;
  appendInitialChild(parent: number, child: number): void;
  appendChild(parent: number, child: number): void;
  appendChildToContainer(container: number, child: number): void;
  insertBefore(parent: number, child: number, beforeChild: number): void;
  removeChild(parent: number, child: number): void;
  removeChildFromContainer(container: number, child: number): void;
  insertInContainerBefore(container: number, child: number, beforeChild: number): void;
  finalizeInitialChildren(instance: number, type: string, props: any): boolean;
  prepareUpdate(instance: number, type: string, oldProps: any, newProps: any, rootContainerInstance: any, currentHostContext: null): any;
} = {
  getPublicInstance(instance) {
    return document.createElement("div");
  },

  getRootHostContext() {
    return null;
  },

  getChildHostContext() {
    return null;
  },

  prepareForCommit() {},

  resetAfterCommit() {
    const root = elementStore.getRoot();
    renderFrame(root);
  },

  shouldSetTextContent(type, props) {
    return typeof props.children === "string" || typeof props.children === "number";
  },

  resetTextContent(instance) {
    // Not needed for MVP
  },

  createTextInstance(text) {
    return elementStore.createElement("text", String(text));
  },

  commitTextUpdate(textInstance, oldText, newText) {
    const element = elementStore.getElement(textInstance);
    if (element) element.text = String(newText);
  },

  createInstance(type, props) {
    if (type === "div") {
      return elementStore.createElement("div");
    } else if (typeof type === "string") {
      return elementStore.createElement("text", type);
    }
    return elementStore.createElement("div");
  },

  appendInitialChild(parent, child) {
    elementStore.appendChild(parent, child);
  },

  appendChild(parent, child) {
    elementStore.appendChild(parent, child);
  },

  appendChildToContainer(container, child) {
    // Root is always ID 1
    elementStore.appendChild(1, child);
  },

  insertBefore(parent, child, beforeChild) {
    const parentEl = elementStore.getElement(parent);
    if (!parentEl) return;

    const beforeIndex = parentEl.children.indexOf(beforeChild);
    if (beforeIndex !== -1) {
      parentEl.children.splice(beforeIndex, 0, child);
    } else {
      parentEl.children.push(child);
    }
  },

  insertInContainerBefore(container, child, beforeChild) {
    this.insertBefore(1, child, beforeChild);
  },

  removeChild(parent, child) {
    const parentEl = elementStore.getElement(parent);
    if (!parentEl) return;

    const index = parentEl.children.indexOf(child);
    if (index !== -1) {
      parentEl.children.splice(index, 1);
    }
  },

  removeChildFromContainer(container, child) {
    this.removeChild(1, child);
  },

  commitUpdate(instance, updatePayload, type, oldProps, newProps) {
    const element = elementStore.getElement(instance);
    if (element) {
      element.text = newProps.children;
    }
  },

  finalizeInitialChildren(instance, type, props) {
    return false;
  },

  prepareUpdate(instance, type, oldProps, newProps, rootContainerInstance, currentHostContext) {
    return newProps;
  },

  clearContainer(container) {},

  hideInstance(instance) {},

  unhideInstance(instance, props) {},

  prepareScopeUpdate(scopeInstance, instance) {},

  getInstanceFromScope(scopeInstance): number {
    return 0;
  },

  createPortalChild(container, child) {
    throw new Error("Portals not supported");
  },

  scheduleTimeout(fn, delay) {
    setTimeout(fn, delay);
  },

  cancelTimeout(id) {
    clearTimeout(id);
  },

  noTimeout: -1,

  isActive() {
    return true;
  },
};

export const reconciler = ReactReconciler(config);
```

### Verification Criteria
- ✅ React component renders to element store
- ✅ Text content is captured correctly
- ✅ `renderFrame()` receives correct element tree

---

## Phase 6: Minimal Demo

### Goal
Create demo that renders `<div>hello world</div>`.

### Steps

#### 6.1 Create Demo App

**demo/app.tsx**:
```typescript
import React from "react";

export function App() {
  return <div>hello world</div>;
}
```

#### 6.2 Create Demo Entry Point

**demo/index.ts**:
```typescript
import React from "react";
import { createRoot } from "../src/renderer/index.js";
import { App } from "./app.js";

const root = createRoot();
root.render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
```

#### 6.3 Update package.json Scripts

**package.json**:
```json
{
  "name": "gpui-react",
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "build:rust": "cd rust && cargo build --release",
    "demo": "bun run demo/index.ts",
    "dev": "bun run build:rust && bun run demo"
  },
  "dependencies": {
    "react": "^18.2.0",
    "react-reconciler": "^0.29.0"
  },
  "devDependencies": {
    "@types/react": "^18.2.0",
    "bun-types": "latest",
    "typescript": "^5.3.0"
  }
}
```

#### 6.4 Run Demo

```bash
# Build Rust library
bun run build:rust

# Run demo
bun run demo
```

### Verification Criteria
- ✅ GPUI window opens (800x600)
- ✅ Window shows "hello world" text
- ✅ Background is white, text is black
- ✅ Text is centered
- ✅ No console errors

---

## Build Commands

### Development Build
```bash
# Build Rust in debug mode (faster compilation)
cd rust && cargo build && cd ..

# Run demo
bun run demo
```

### Release Build
```bash
# Build Rust in release mode (optimized)
cd rust && cargo build --release && cd ..

# Run demo
bun run demo
```

### Clean Build
```bash
# Clean Rust build artifacts
cd rust && cargo clean && cd ..

# Rebuild and run
bun run build:rust && bun run demo
```

---

## Testing Strategy

### Unit Tests (Rust)
```bash
cd rust
cargo test
```

### Integration Tests (Bun)
Create **src/renderer/__tests__/element-store.test.ts**:
```typescript
import { elementStore } from "../element-store";

test("createElement", () => {
  const id = elementStore.createElement("div", "test");
  const element = elementStore.getElement(id);
  expect(element?.type).toBe("div");
  expect(element?.text).toBe("test");
});

test("appendChild", () => {
  const parent = elementStore.createElement("div");
  const child = elementStore.createElement("text", "child");
  elementStore.appendChild(parent, child);

  const parentEl = elementStore.getElement(parent);
  expect(parentEl?.children).toContain(child);
});
```

### End-to-End Tests
```bash
# Run demo and verify window opens
bun run demo

# Expected: GPUI window with "hello world" text
```

---

## Troubleshooting

### Common Issues

#### Issue: "Failed to load library"
**Cause**: Library path incorrect or not built
**Solution**:
```bash
# Rebuild library
cd rust && cargo build --release && cd ..

# Verify path in gpui-binding.ts matches actual location
ls -la rust/target/release/
```

#### Issue: "Element not found"
**Cause**: Element ID mismatch between React reconciler and store
**Solution**: Add logging to element store operations:
```typescript
createElement(type: string, text?: string): number {
  const globalId = this.nextId++;
  console.log(`Created element ${globalId}: ${type}`);
  // ...
}
```

#### Issue: GPUI window doesn't appear
**Cause**: GPUI event loop not running or render not triggered
**Solution**:
- Verify `gpui_init()` returns success
- Check that `renderFrame()` is called
- Add logging in Rust render functions

#### Issue: Text doesn't display
**Cause**: Text content not passed to FFI or not applied in GPUI
**Solution**:
- Log text content in host config `commitUpdate`
- Verify FFI `text_ptr` is not null
- Check GPUI element has text child

#### Issue: Memory leak detected
**Cause**: CString not freed after use
**Solution**:
```typescript
const typeCString = new CString("div");
try {
  lib.symbols.someFunction(typeCString.ptr);
} finally {
  //CString automatically freed when out of scope
}
```

### Debugging Tips

1. **Add Logging**:
```typescript
// TypeScript
console.log("Rendering element:", element);

// Rust
eprintln!("Building GPUI element: {:?}", data);
```

2. **Check FFI Calls**:
```typescript
// Verify function pointers
console.log("Library symbols:", lib.symbols);
```

3. **Validate Data Structures**:
```typescript
// Check element store
console.log("Element store:", Array.from(elementStore.store.entries()));
```

---

## Next Steps (Beyond MVP)

After minimal demo works, consider:

1. **Full Styling Support**
   - Map CSS-in-JS to GPUI style properties
   - Support colors, fonts, sizes
   - Implement flexbox layout

2. **Event Handling**
   - Capture GPUI events
   - Bridge to React event system
   - Support onClick, onHover, etc.

3. **Advanced Components**
   - Lists (virtualized)
   - Images
   - Custom components

4. **Performance Optimization**
   - Batch FFI calls
   - Cache GPUI elements
   - Reduce memory allocations

---

## Conclusion

This plan provides a step-by-step path to building a minimal React-GPUI renderer using Bun FFI. The architecture leverages:

- **React Reconciler**: For state management and component lifecycle
- **Bun FFI**: For efficient Rust-JavaScript interop
- **GPUI**: For GPU-accelerated rendering (120 FPS)

Start with Phase 1 (Project Initialization) and proceed sequentially. Each phase builds on the previous one, with clear verification criteria to ensure correctness before moving forward.

**Key Success Metrics**:
- ✅ Renders `<div>hello world</div>` correctly
- ✅ Performance meets target (60+ FPS)
- ✅ Clean build without errors
- ✅ No memory leaks

Good luck! 🚀
