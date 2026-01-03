import { dlopen, FFIType, ptr, toArrayBuffer } from "bun:ffi";
import { join } from "path";

const libName = `libgpui_renderer.${suffix}`;
const libPath = join(import.meta.dir, "../../rust/target/release", libName);

const lib = dlopen(libPath, {
  gpui_init: {
    args: [FFIType.f32, FFIType.f32],
    returns: FFIType.ptr,
  },
  gpui_create_window: {
    args: [FFIType.f32, FFIType.f32],
    returns: FFIType.ptr,
  },
  gpui_render_frame: {
    args: [FFIType.ptr, FFIType.ptr, FFIType.ptr, FFIType.ptr, FFIType.ptr, FFIType.ptr],
    returns: FFIType.ptr,
  },
  gpui_trigger_render: {
    args: [FFIType.ptr],
    returns: FFIType.void,
  },
  gpui_update_element: {
    args: [FFIType.ptr, FFIType.ptr],
    returns: FFIType.ptr,
  },
  gpui_free_result: {
    args: [FFIType.ptr],
    returns: FFIType.void,
  },
  gpui_batch_update_elements: {
    args: [FFIType.ptr, FFIType.ptr, FFIType.ptr],
    returns: FFIType.ptr,
  },
});

if (!lib.symbols) {
  throw new Error("Failed to load GPUI library");
}

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
  const resultBuffer = new Uint8Array(8);
  lib.symbols.gpui_init(width, height, resultBuffer);
  checkResult(resultBuffer);
}

export function createWindow(width: number, height: number): void {
  const resultBuffer = new Uint8Array(8);
  lib.symbols.gpui_create_window(width, height, resultBuffer);
  checkResult(resultBuffer);
}

export function renderFrame(element: any): void {
  console.log("=== renderFrame called ===");
  console.log("Element:", JSON.stringify(element, null, 2));

  const liveBuffers: ArrayBuffer[] = [];

  const globalIdBuffer = new ArrayBuffer(8);
  new DataView(globalIdBuffer).setBigUint64(0, BigInt(element.globalId), true);
  liveBuffers.push(globalIdBuffer);
  const globalIdPtr = ptr(globalIdBuffer);

  const typeBuffer = new TextEncoder().encode(element.type + "\0");
  liveBuffers.push(typeBuffer.buffer);
  const typePtr = ptr(typeBuffer);

  const textContent = element.text || " ";
  const textBuffer = new TextEncoder().encode(textContent + "\0");
  liveBuffers.push(textBuffer.buffer);
  const textPtr = ptr(textBuffer);

  const childrenArray = element.children || [];
  const childCountBuffer = new ArrayBuffer(8);
  new DataView(childCountBuffer).setBigUint64(0, BigInt(childrenArray.length), true);
  liveBuffers.push(childCountBuffer);
  const childCountPtr = ptr(childCountBuffer);

  const childrenByteLength = childrenArray.length * 8;
  const childrenBuffer = new ArrayBuffer(childrenByteLength);
  const childrenView = new BigUint64Array(childrenBuffer);
  for (let i = 0; i < childrenArray.length; i++) {
    childrenView[i] = BigInt(childrenArray[i]);
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

  const resultBuffer = new Uint8Array(8);

  lib.symbols.gpui_render_frame(
    globalIdPtr,
    typePtr,
    textPtr,
    childCountPtr,
    childrenPtr,
    resultBuffer
  );

  const status = new DataView(resultBuffer.buffer).getInt32(0, true);
  if (status !== 0) {
    throw new Error(`GPUI render failed with status: ${status}`);
  }

  const triggerBuffer = new Uint8Array(8);
  lib.symbols.gpui_trigger_render(ptr(triggerBuffer));

  console.log("=== renderFrame completed ===");

  liveBuffers.length = 0;
}

export function updateElement(element: any): void {
  const jsonString = JSON.stringify(element);
  const jsonBuffer = new TextEncoder().encode(jsonString + "\0");
  lib.symbols.gpui_update_element(ptr(jsonBuffer), resultBuffer);

  const status = new DataView(resultBuffer.buffer).getInt32(0, true);
  if (status !== 0) {
    const errorPtr = new DataView(resultBuffer.buffer).getInt32(8, true);
    lib.symbols.gpui_free_result(resultBuffer);
    throw new Error(`GPUI update element failed: ${errorPtr}`);
  }
}

export function batchElementUpdates(elements: any[], callback?: () => void): void {
  if (elements.length === 0) {
    if (callback) callback();
    return;
  }

  console.log(`=== Batching ${elements.length} element updates ===`);

  const liveBuffers: ArrayBuffer[] = [];

  const countBuffer = new ArrayBuffer(8);
  new DataView(countBuffer).setBigUint64(0, BigInt(elements.length), true);
  liveBuffers.push(countBuffer);
  const countPtr = ptr(countBuffer);

  const elementsJsonString = JSON.stringify(elements);
  const elementsBuffer = new TextEncoder().encode(elementsJsonString + "\0");
  liveBuffers.push(elementsBuffer.buffer);
  const elementsPtr = ptr(elementsBuffer);

  const resultBuffer = new Uint8Array(8);

  lib.symbols.gpui_batch_update_elements(countPtr, elementsPtr, resultBuffer);

  const status = new DataView(resultBuffer.buffer).getInt32(0, true);
  if (status !== 0) {
    const errorPtr = new DataView(resultBuffer.buffer).getInt32(8, true);
    lib.symbols.gpui_free_result(resultBuffer);
    throw new Error(`GPUI batch update failed: error ptr=${errorPtr}`);
  }

  console.log(`=== Batch update completed for ${elements.length} elements ===`);

  liveBuffers.length = 0;

  if (callback) callback();
}

export function triggerRender(): void {
  const buffer = new Uint8Array(8);
  lib.symbols.gpui_trigger_render(ptr(buffer));
}
