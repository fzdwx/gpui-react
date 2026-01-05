import {dlopen, FFIType, ptr, read} from "bun:ffi";
import {info, debug, trace} from "../reconciler/utils/logging";
import {getNativeLibPath} from "../reconciler/utils/plat";
import {sleep} from "bun";

// ============================================================================
// GPUI FFI Binding
// ============================================================================

const libPath = getNativeLibPath(import.meta.dir);
info(`Loading GPUI library from: ${libPath}`);

const lib = dlopen(libPath, {
    gpui_init: {args: [FFIType.ptr], returns: FFIType.void},
    gpui_create_window: {
        args: [FFIType.f32, FFIType.f32, FFIType.ptr, FFIType.ptr],
        returns: FFIType.void,
    },
    gpui_is_ready: {args: [], returns: FFIType.bool},
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
    gpui_trigger_render: {args: [FFIType.ptr, FFIType.ptr], returns: FFIType.void},
    gpui_free_result: {args: [FFIType.ptr], returns: FFIType.void},
    gpui_batch_update_elements: {
        args: [FFIType.ptr, FFIType.ptr, FFIType.ptr, FFIType.ptr],
        returns: FFIType.void,
    },
});

export const gpui = {
    liveBuffers: [] as ArrayBuffer[],
    RESULT_SIZE: 16,


    keep(buffer: ArrayBuffer): void {
        this.liveBuffers.push(buffer);
    },

    clear(): void {
        this.liveBuffers.length = 0;
    },

    encodeCString(str: string): [ArrayBuffer, ReturnType<typeof ptr>] {
        const buffer = new TextEncoder().encode(str + "\0");
        this.keep(buffer.buffer);
        return [buffer.buffer, ptr(buffer)];
    },

    createInt64(value: bigint): [ArrayBuffer, ReturnType<typeof ptr>] {
        const buffer = new ArrayBuffer(8);
        new DataView(buffer).setBigInt64(0, value, true);
        this.keep(buffer);
        return [buffer, ptr(buffer)];
    },

    createInt64Array(values: bigint[]): [ArrayBuffer, ReturnType<typeof ptr>] {
        const byteLength = Math.max(values.length * 8, 8);
        const buffer = new ArrayBuffer(byteLength);
        if (values.length > 0) {
            const view = new BigInt64Array(buffer);
            for (let i = 0; i < values.length; i++) view[i] = values[i];
        }
        this.keep(buffer);
        return [buffer, ptr(buffer)];
    },

    checkResult(resultBuffer: Uint8Array): void {
        const status = read.i32(ptr(resultBuffer), 0);
        if (status !== 0) {
            const errorPtr = read.i32(ptr(resultBuffer), 8);
            lib.symbols.gpui_free_result(resultBuffer);
            throw new Error(`GPUI operation failed: error ptr=${errorPtr}`);
        }
        const errorCheck = read.i32(ptr(resultBuffer), 8);
        if (errorCheck !== 0) lib.symbols.gpui_free_result(resultBuffer);
    },

    waitReady(): void {
        let delay = 1;
        while (Date.now() - Date.now() < 5000) {
            if (lib.symbols.gpui_is_ready()) return;
            sleep(Math.min(delay, 100));
            delay *= 2;
        }
        throw new Error("GPUI failed to become ready");
    },
};

// ============================================================================
// Public API
// ============================================================================

export function init(): void {
    const resultBuffer = new Uint8Array(gpui.RESULT_SIZE);
    lib.symbols.gpui_init(resultBuffer);
    gpui.checkResult(resultBuffer);
    gpui.waitReady();
}

export function createWindow(width: number, height: number, title?: string): number {
    gpui.clear();
    const resultBuffer = new Uint8Array(gpui.RESULT_SIZE);
    const [titleBuffer, titlePtr] = gpui.encodeCString(title || "React-GPUI");
    lib.symbols.gpui_create_window(width, height, titlePtr, resultBuffer);
    const windowId = checkWindowCreateResult(resultBuffer);
    info(`Created window with id: ${windowId}`);
    return windowId;
}

export function renderFrame(windowId: number, element: ElementData): void {
    trace(`renderFrame for window ${windowId}`);
    gpui.clear();
    const [windowIdBuffer, windowIdPtr] = gpui.createInt64(BigInt(windowId));
    const [globalIdBuffer, globalIdPtr] = gpui.createInt64(BigInt(element.globalId));
    const [typeBuffer, typePtr] = gpui.encodeCString(element.type);
    const [textBuffer, textPtr] = gpui.encodeCString(element.text || " ");
    const childrenArray = element.children || [];
    const [childCountBuffer, childCountPtr] = gpui.createInt64(BigInt(childrenArray.length));
    const [childrenBuffer, childrenPtr] = gpui.createInt64Array(
        childrenArray.map((c) => BigInt(c))
    );
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
    if (new DataView(resultBuffer.buffer).getInt32(0, true) !== 0) {
        throw new Error("GPUI render failed");
    }
}

export function triggerRender(windowId: number): void {
    trace(`triggerRender for window ${windowId}`);
    gpui.clear();
    const [windowIdBuffer, windowIdPtr] = gpui.createInt64(BigInt(windowId));
    lib.symbols.gpui_trigger_render(windowIdPtr, new Uint8Array(8));
}

export function batchElementUpdates(windowId: number, elements: ElementData[]): void {
    if (elements.length === 0) return;
    info(`Batching ${elements.length} updates for window ${windowId}`);
    gpui.clear();
    const [windowIdBuffer, windowIdPtr] = gpui.createInt64(BigInt(windowId));
    const [countBuffer, countPtr] = gpui.createInt64(BigInt(elements.length));
    const [elementsBuffer, elementsPtr] = gpui.encodeCString(JSON.stringify(elements));
    const resultBuffer = new Uint8Array(8);
    lib.symbols.gpui_batch_update_elements(windowIdPtr, countPtr, elementsPtr, resultBuffer);
    lib.symbols.gpui_trigger_render(windowIdPtr, new Uint8Array(8));
    info(`Batch update completed`);
}

function checkWindowCreateResult(resultBuffer: Uint8Array): number {
    const status = read.i32(ptr(resultBuffer), 0);
    if (status !== 0) {
        const errorPtr = read.i32(ptr(resultBuffer), 8);
        lib.symbols.gpui_free_result(resultBuffer);
        throw new Error(`Window creation failed: ptr=${errorPtr}`);
    }
    return Number(read.u64(ptr(resultBuffer), 8));
}

export interface ElementData {
    globalId: number;
    type: string;
    text?: string;
    children?: number[];

    [key: string]: unknown;
}

export type EventCallback = (event: EventData) => void;

export interface EventData {
    type: string;
    target: number;
    x?: number;
    y?: number;
    key?: string;

    [key: string]: unknown;
}
