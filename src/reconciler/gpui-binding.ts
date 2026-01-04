import { dlopen, FFIType, suffix, ptr } from "bun:ffi";
import { join } from "path";

const libName = `libgpui_renderer.${suffix}`;
const libPath = join(import.meta.dir, "../../rust/target/release", libName);

console.log(`Loading GPUI library from: ${libPath}`);

// Keep buffers alive - they must not be garbage collected while FFI call is in flight
const liveBuffers: ArrayBuffer[] = [];

const lib = dlopen(libPath, {
  gpui_init: {
    args: [FFIType.f32, FFIType.f32, FFIType.ptr],
    returns: FFIType.void,
  },
  gpui_create_window: {
    args: [FFIType.f32, FFIType.f32, FFIType.ptr],
    returns: FFIType.void,
  },
  gpui_render_frame: {
    // Pass all args as pointers to avoid u64 issues
    args: [FFIType.ptr, FFIType.ptr, FFIType.ptr, FFIType.ptr, FFIType.ptr, FFIType.ptr],
    returns: FFIType.void,
  },
  gpui_trigger_render: {
    args: [FFIType.ptr],
    returns: FFIType.void,
  },
  gpui_free_result: {
    args: [FFIType.ptr],
    returns: FFIType.void,
  },
  gpui_batch_update_elements: {
    args: [FFIType.ptr, FFIType.ptr, FFIType.ptr],
    returns: FFIType.void,
  },
});

if (!lib.symbols) {
  throw new Error("Failed to load GPUI library");
}

const FFI_RESULT_SIZE = 16;

function checkResult(resultBuffer: Uint8Array): void {
  const status = new DataView(resultBuffer.buffer).getInt32(0, true);

  if (status !== 0) {
    const errorPtr = new DataView(resultBuffer.buffer).getInt32(8, true);
    lib.symbols.gpui_free_result(resultBuffer);
    throw new Error(`GPUI operation failed: error ptr=${errorPtr}`);
  }

  const errorCheck = new DataView(resultBuffer.buffer).getInt32(8, true);
  if (errorCheck !== 0) {
    lib.symbols.gpui_free_result(resultBuffer);
  }
}

export function init(width: number, height: number): void {
  const resultBuffer = new Uint8Array(FFI_RESULT_SIZE);
  lib.symbols.gpui_init(width, height, resultBuffer);
  checkResult(resultBuffer);
}

export function createWindow(width: number, height: number): void {
  const resultBuffer = new Uint8Array(FFI_RESULT_SIZE);
  lib.symbols.gpui_create_window(width, height, resultBuffer);
  checkResult(resultBuffer);
}

export function renderFrame(element: any): void {
  console.log("=== renderFrame called ===");
  console.log("Element:", JSON.stringify(element, null, 2));

  // Clear old buffers (they're only needed during the FFI call)
  liveBuffers.length = 0;

  // Create buffers for each parameter
  const globalIdBuffer = new ArrayBuffer(8);
  new DataView(globalIdBuffer).setBigUint64(0, BigInt(element.globalId), true); // little-endian
  liveBuffers.push(globalIdBuffer);
  const globalIdPtr = ptr(globalIdBuffer);

  const typeBuffer = new TextEncoder().encode(element.type + "\0"); // null-terminate
  liveBuffers.push(typeBuffer.buffer);
  const typePtr = ptr(typeBuffer);

  const textContent = element.text || " ";
  const textBuffer = new TextEncoder().encode(textContent + "\0"); // null-terminate
  liveBuffers.push(textBuffer.buffer);
  const textPtr = ptr(textBuffer);

  const childrenArray = element.children || [];

  // Create child count buffer (8 bytes)
  const childCountBuffer = new ArrayBuffer(8);
  new DataView(childCountBuffer).setBigUint64(0, BigInt(childrenArray.length), true); // little-endian
  liveBuffers.push(childCountBuffer);
  const childCountPtr = ptr(childCountBuffer);

  // Create children buffer with child IDs (use min 8 bytes to avoid empty buffer issue)
  const childrenByteLength = Math.max(childrenArray.length * 8, 8);
  const childrenBuffer = new ArrayBuffer(childrenByteLength);
  if (childrenArray.length > 0) {
    const childrenView = new BigUint64Array(childrenBuffer);
    for (let i = 0; i < childrenArray.length; i++) {
      childrenView[i] = BigInt(childrenArray[i]);
    }
  }
  liveBuffers.push(childrenBuffer);
  const childrenPtr = ptr(childrenBuffer);

  console.log("FFI params:", {
    globalId: element.globalId,
    type: element.type,
    text: textContent,
    childCount: childrenArray.length,
    children: childrenArray
  });

  // Create result buffer
  const resultBuffer = new Uint8Array(8);

  // Call FFI with all parameters as pointers
  lib.symbols.gpui_render_frame(
      globalIdPtr,
      typePtr,
      textPtr,
      childCountPtr,
      childrenPtr,
      resultBuffer
  );

  // Check result (first 4 bytes are status)
  const status = new DataView(resultBuffer.buffer).getInt32(0, true);
  if (status !== 0) {
    throw new Error(`GPUI render failed with status: ${status}`);
  }

  const triggerBuffer = new Uint8Array(8);
  lib.symbols.gpui_trigger_render(triggerBuffer);

  console.log("=== renderFrame completed ===");
}

export function batchElementUpdates(elements: any[]): void {
  if (elements.length === 0) {
    return;
  }

  console.log(`=== Batching ${elements.length} element updates ===`);

  liveBuffers.length = 0;

  const countBuffer = new ArrayBuffer(8);
  new DataView(countBuffer).setBigUint64(0, BigInt(elements.length), true);
  liveBuffers.push(countBuffer);
  const countPtr = ptr(countBuffer);

  const elementsJsonString = JSON.stringify(elements);
  console.log("batchElementUpdates - elements JSON:", elementsJsonString.substring(0, 500));
  const elementsBuffer = new TextEncoder().encode(elementsJsonString + "\0");
  liveBuffers.push(elementsBuffer.buffer);
  const elementsPtr = ptr(elementsBuffer);

  const resultBuffer = new Uint8Array(8);

  lib.symbols.gpui_batch_update_elements(countPtr, elementsPtr, resultBuffer);

  // Send trigger_render command to force GPUI to pick up state changes
  const triggerBuffer = new Uint8Array(8);
  lib.symbols.gpui_trigger_render(triggerBuffer);

  console.log(`=== Batch update completed for ${elements.length} elements ===`);
}
