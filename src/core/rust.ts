import { lib } from "./ffi";
import { peek, sleep } from "bun";
import { JSCallback, Pointer, ptr, read, toArrayBuffer, FFIType, CString } from "bun:ffi";
import { info, trace } from "../reconciler/utils/logging";
import { decoder, FfiState } from "./ffi-state";
import { EventEmitter } from "events";
import { getEventHandler } from "../reconciler/event-router";

export interface ElementData {
    globalId: number;
    type: string;
    text?: string;
    children: number[];
    style?: Record<string, any>;
    eventHandlers?: Record<string, number>;
}

const RESULT_SIZE = 16;

export interface WindowOptions {
    width: number;
    height: number;
    title?: string;
    x?: number;
    y?: number;
    resizable?: boolean;
    fullscreen?: boolean;
}

export class RustLib {
    ffiStateMap: Map<number, FfiState>;
    private _nativeEvents: EventEmitter = new EventEmitter();
    private eventCallbackWrapper: any;

    public constructor() {
        this.ffiStateMap = new Map();
        const resultBuffer = new Uint8Array(RESULT_SIZE);
        lib.symbols.gpui_init(resultBuffer);
        this.checkResult(resultBuffer);
        this.waitReady();
        this.setupEventBus();
    }

    public createWindow(options: WindowOptions): number {
        const resultBuffer = new Uint8Array(RESULT_SIZE);
        const ffiState = new FfiState();
        const [optionsBuffer, optionsPtr] = ffiState.encodeCString(JSON.stringify(options));
        lib.symbols.gpui_create_window(optionsPtr, resultBuffer);
        const windowId = this.checkWindowCreateResult(resultBuffer);
        info(`Created window with id: ${windowId}`);
        this.ffiStateMap.set(windowId, ffiState);
        return windowId;
    }

    public batchElementUpdates(windowId: number, elements: ElementData[]): void {
        let ffiState = this.getFfiState(windowId);
        if (!ffiState) {
            return;
        }

        if (elements.length === 0) return;
        info(`Batching ${elements.length} updates for window ${windowId}`);
        ffiState.clear();
        const [windowIdBuffer, windowIdPtr] = ffiState.createInt64(BigInt(windowId));
        const [countBuffer, countPtr] = ffiState.createInt64(BigInt(elements.length));
        const [elementsBuffer, elementsPtr] = ffiState.encodeCString(JSON.stringify(elements));
        const resultBuffer = new Uint8Array(8);
        lib.symbols.gpui_batch_update_elements(windowIdPtr, countPtr, elementsPtr, resultBuffer);
        info(`Batch update completed`);
    }

    public renderFrame(windowId: number, element: ElementData): void {
        let ffiState = this.getFfiState(windowId);
        if (!ffiState) {
            return;
        }

        trace(`renderFrame for window ${windowId}`);
        ffiState.clear();
        const [windowIdBuffer, windowIdPtr] = ffiState.createInt64(BigInt(windowId));
        const [globalIdBuffer, globalIdPtr] = ffiState.createInt64(BigInt(element.globalId));
        const [typeBuffer, typePtr] = ffiState.encodeCString(element.type);
        const [textBuffer, textPtr] = ffiState.encodeCString(element.text || " ");
        const childrenArray = element.children || [];
        const [childCountBuffer, childCountPtr] = ffiState.createInt64(
            BigInt(childrenArray.length)
        );
        const [childrenBuffer, childrenPtr] = ffiState.createInt64Array(
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

    public triggerRender(windowId: number): void {
        let ffiState = this.getFfiState(windowId);
        if (!ffiState) {
            return;
        }
        trace(`triggerRender for window ${windowId}`);
        ffiState.clear();
        const [windowIdBuffer, windowIdPtr] = ffiState.createInt64(BigInt(windowId));
        lib.symbols.gpui_trigger_render(windowIdPtr, new Uint8Array(8));
    }

    private setupEventBus() {
        if (this.eventCallbackWrapper) {
            return;
        }

        console.log("[JS] Setting up event bus, creating JSCallback...");

        const eventCallback = new JSCallback(
            (jsonPtr: any,jsonLen:number) => {
                try {
                    if (!jsonPtr) {
                        console.log("[JS] Callback: null pointer");
                        return;
                    }

                    const jsonStr = new CString(jsonPtr,0,jsonLen).toString();
                    console.log(`[JS] Callback: json=${jsonStr}`);

                    if (!jsonStr) {
                        console.log("[JS] Empty JSON, skipping");
                        return;
                    }

                    const event = JSON.parse(jsonStr);
                    const { windowId, elementId, eventType } = event;

                    console.log(`[JS] Event: windowId=${windowId}, elementId=${elementId}, type="${eventType}"`);

                    const handler = getEventHandler(elementId, eventType);
                    if (handler) {
                        console.log(`[JS] Found handler, calling...`);
                        handler({
                            type: eventType,
                            target: elementId,
                            windowId: windowId,
                            timestamp: Date.now(),
                        });
                    } else {
                        console.log(`[JS] No handler for elementId=${elementId}, type="${eventType}"`);
                    }
                } catch (err) {
                    console.error("[JS] Error:", err);
                }
            },
            {
                args: [FFIType.ptr,"u32"],
                returns: "void",
                threadsafe: true,
            }
        );

        this.eventCallbackWrapper = eventCallback;

        console.log(`[JS] eventCallback.ptr = ${eventCallback.ptr}`);

        if (!eventCallback.ptr) {
            throw new Error("Failed to create event callback");
        }

        lib.symbols.set_event_callback(eventCallback.ptr);
        console.log("[JS] Event callback registered");
    }

    getFfiState(windowId: number) {
        return this.ffiStateMap.get(windowId);
    }

    waitReady(): void {
        let delay = 1;
        while (Date.now() - Date.now() < 5000) {
            if (lib.symbols.gpui_is_ready()) return;
            sleep(Math.min(delay, 100));
            delay *= 2;
        }
        throw new Error("GPUI failed to become ready");
    }

    checkResult(resultBuffer: Uint8Array): void {
        const status = read.i32(ptr(resultBuffer), 0);
        if (status !== 0) {
            const errorPtr = read.i32(ptr(resultBuffer), 8);
            lib.symbols.gpui_free_result(resultBuffer);
            throw new Error(`GPUI operation failed: error ptr=${errorPtr}`);
        }
        const errorCheck = read.i32(ptr(resultBuffer), 8);
        if (errorCheck !== 0) lib.symbols.gpui_free_result(resultBuffer);
    }

    checkWindowCreateResult(resultBuffer: Uint8Array): number {
        const status = read.i32(ptr(resultBuffer), 0);
        if (status !== 0) {
            const errorPtr = read.i32(ptr(resultBuffer), 8);
            lib.symbols.gpui_free_result(resultBuffer);
            throw new Error(`Window creation failed: ptr=${errorPtr}`);
        }
        return Number(read.u64(ptr(resultBuffer), 8));
    }
}
