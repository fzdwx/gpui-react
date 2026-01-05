import { dlopen, FFIType, ptr, read } from "bun:ffi";
import { info, debug, trace } from "./utils/logging";
import { getNativeLibPath } from "./util";
import { sleep } from "bun";

const libPath = getNativeLibPath(import.meta.dir);

info(`Loading GPUI library from: ${libPath}`);

const liveBuffers: ArrayBuffer[] = [];

const lib = dlopen(libPath, {
    gpui_init: {
        args: [FFIType.ptr],
        returns: FFIType.void,
    },
    gpui_create_window: {
        args: [FFIType.f32, FFIType.f32, FFIType.ptr, FFIType.ptr],
        returns: FFIType.void,
    },
    gpui_is_ready: {
        args: [],
        returns: FFIType.bool,
    },
    gpui_render_frame: {
        args: [
            FFIType.ptr,
            FFIType.ptr,
            FFIType.ptr,
            FFIType.ptr,
            FFIType.ptr,
            FFIType.ptr,
            FFIType.ptr,
        ],
        returns: FFIType.void,
    },
    gpui_trigger_render: {
        args: [FFIType.ptr, FFIType.ptr],
        returns: FFIType.void,
    },
    gpui_free_result: {
        args: [FFIType.ptr],
        returns: FFIType.void,
    },
    gpui_batch_update_elements: {
        args: [FFIType.ptr, FFIType.ptr, FFIType.ptr, FFIType.ptr],
        returns: FFIType.void,
    },
});

if (!lib.symbols) {
    throw new Error("Failed to load GPUI library");
}

const FFI_RESULT_SIZE = 16;

function checkResult(resultBuffer: Uint8Array): void {
    // Use read() for faster pointer access (bun:ffi best practice for short-lived pointers)
    const status = read.i32(ptr(resultBuffer), 0);

    if (status !== 0) {
        const errorPtr = read.i32(ptr(resultBuffer), 8);
        lib.symbols.gpui_free_result(resultBuffer);
        throw new Error(`GPUI operation failed: error ptr=${errorPtr}`);
    }

    const errorCheck = read.i32(ptr(resultBuffer), 8);
    if (errorCheck !== 0) {
        lib.symbols.gpui_free_result(resultBuffer);
    }
}

export function checkWindowCreateResult(resultBuffer: Uint8Array): number {
    // Use read() for faster pointer access (bun:ffi best practice for short-lived pointers)
    const status = read.i32(ptr(resultBuffer), 0);

    if (status !== 0) {
        const errorPtr = read.i32(ptr(resultBuffer), 8);
        lib.symbols.gpui_free_result(resultBuffer);
        throw new Error(`GPUI window creation failed: error ptr=${errorPtr}`);
    }

    const windowId = read.u64(ptr(resultBuffer), 8);
    return Number(windowId);
}

export function init(): void {
    const resultBuffer = new Uint8Array(FFI_RESULT_SIZE);
    lib.symbols.gpui_init(resultBuffer);
    checkResult(resultBuffer);

    waitGpuiReady();
}

export function createWindow(width: number, height: number, title?: string): number {
    const resultBuffer = new Uint8Array(FFI_RESULT_SIZE);

    // 编码 title 字符串，添加 null 终止符
    const titleStr = title || "React-GPUI";
    const titleBuffer = new TextEncoder().encode(titleStr + "\0");
    liveBuffers.push(titleBuffer.buffer); // 保持引用防止 GC
    const titlePtr = ptr(titleBuffer);

    lib.symbols.gpui_create_window(width, height, titlePtr, resultBuffer);
    const windowId = checkWindowCreateResult(resultBuffer);
    info(`Created window with id: ${windowId}`);
    return windowId;
}

export function renderFrame(windowId: number, element: any): void {
    trace("renderFrame called");
    debug("Element", element);

    liveBuffers.length = 0;

    const windowIdBuffer = new ArrayBuffer(8);
    new DataView(windowIdBuffer).setBigInt64(0, BigInt(windowId), true);
    liveBuffers.push(windowIdBuffer);
    const windowIdPtr = ptr(windowIdBuffer);

    const globalIdBuffer = new ArrayBuffer(8);
    new DataView(globalIdBuffer).setBigInt64(0, BigInt(element.globalId), true);
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
    new DataView(childCountBuffer).setBigInt64(0, BigInt(childrenArray.length), true);
    liveBuffers.push(childCountBuffer);
    const childCountPtr = ptr(childCountBuffer);

    const childrenByteLength = Math.max(childrenArray.length * 8, 8);
    const childrenBuffer = new ArrayBuffer(childrenByteLength);
    if (childrenArray.length > 0) {
        const childrenView = new BigInt64Array(childrenBuffer);
        for (let i = 0; i < childrenArray.length; i++) {
            childrenView[i] = BigInt(childrenArray[i]);
        }
    }
    liveBuffers.push(childrenBuffer);
    const childrenPtr = ptr(childrenBuffer);

    debug("FFI params", {
        windowId,
        globalId: element.globalId,
        type: element.type,
        text: textContent,
        childCount: childrenArray.length,
        children: childrenArray,
    });

    const resultBuffer = new Uint8Array(8);

    lib.symbols.gpui_render_frame(
        windowIdPtr,
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

    trace("renderFrame completed");
}

export function triggerRender(windowId: number): void {
    // Clear liveBuffers to prevent memory leak - bun:ffi does not manage memory
    liveBuffers.length = 0;

    const windowIdBuffer = new ArrayBuffer(8);
    new DataView(windowIdBuffer).setBigInt64(0, BigInt(windowId), true);
    liveBuffers.push(windowIdBuffer);
    const windowIdPtr = ptr(windowIdBuffer);

    const triggerBuffer = new Uint8Array(8);
    lib.symbols.gpui_trigger_render(windowIdPtr, triggerBuffer);
}

export function batchElementUpdates(windowId: number, elements: any[]): void {
    if (elements.length === 0) {
        return;
    }

    info(`Batching ${elements.length} element updates for window ${windowId}`);

    liveBuffers.length = 0;

    const windowIdBuffer = new ArrayBuffer(8);
    new DataView(windowIdBuffer).setBigInt64(0, BigInt(windowId), true);
    liveBuffers.push(windowIdBuffer);
    const windowIdPtr = ptr(windowIdBuffer);

    const countBuffer = new ArrayBuffer(8);
    new DataView(countBuffer).setBigInt64(0, BigInt(elements.length), true);
    liveBuffers.push(countBuffer);
    const countPtr = ptr(countBuffer);

    const elementsJsonString = JSON.stringify(elements);
    debug("batchElementUpdates - elements JSON", elementsJsonString.substring(0, 500));
    const elementsBuffer = new TextEncoder().encode(elementsJsonString + "\0");
    liveBuffers.push(elementsBuffer.buffer);
    const elementsPtr = ptr(elementsBuffer);

    const resultBuffer = new Uint8Array(8);

    lib.symbols.gpui_batch_update_elements(windowIdPtr, countPtr, elementsPtr, resultBuffer);

    const triggerBuffer = new Uint8Array(8);
    lib.symbols.gpui_trigger_render(windowIdPtr, triggerBuffer);

    info(`Batch update completed for ${elements.length} elements`);
}

function waitGpuiReady() {
    let delay = 1;
    const maxDelay = 100;
    const maxTotalWait = 5000;

    const startTime = Date.now();
    while (Date.now() - startTime < maxTotalWait) {
        if (lib.symbols.gpui_is_ready()) {
            break;
        }
        sleep(Math.min(delay, maxDelay));
        delay *= 2;
    }
}
