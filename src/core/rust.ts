import { lib } from "./ffi";
import { peek, sleep } from "bun";
import { ptr, read, toArrayBuffer, CString } from "bun:ffi";
import { info, trace } from "../utils/logging";
import { decoder, FfiState } from "./ffi-state";
import { EventEmitter } from "events";
import { eventRouter, createEvent, RawEventData } from "../events";

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
    private pollIntervals: Map<number, ReturnType<typeof setInterval>> = new Map();

    public constructor() {
        this.ffiStateMap = new Map();
        const resultBuffer = new Uint8Array(RESULT_SIZE);
        lib.symbols.gpui_init(resultBuffer);
        this.checkResult(resultBuffer);
        this.waitReady();
    }

    public createWindow(options: WindowOptions, pollEventInterval?: number): number {
        const resultBuffer = new Uint8Array(RESULT_SIZE);
        const ffiState = new FfiState();
        const [optionsBuffer, optionsPtr] = ffiState.encodeCString(JSON.stringify(options));
        lib.symbols.gpui_create_window(optionsPtr, resultBuffer);
        const windowId = this.checkWindowCreateResult(resultBuffer);
        info(`Created window with id: ${windowId}`);
        this.ffiStateMap.set(windowId, ffiState);

        // Start event polling for this window
        this.startEventPolling(windowId, pollEventInterval);

        return windowId;
    }

    public batchElementUpdates(windowId: number, elements: ElementData[]): void {
        let ffiState = this.getFfiState(windowId);
        if (!ffiState) {
            return;
        }

        if (elements.length === 0) return;
        trace(`Batching ${elements.length} updates for window ${windowId}`);
        ffiState.clear();
        const [windowIdBuffer, windowIdPtr] = ffiState.createInt64(BigInt(windowId));
        const [countBuffer, countPtr] = ffiState.createInt64(BigInt(elements.length));
        const [elementsBuffer, elementsPtr] = ffiState.encodeCString(JSON.stringify(elements));
        const resultBuffer = new Uint8Array(8);
        lib.symbols.gpui_batch_update_elements(windowIdPtr, countPtr, elementsPtr, resultBuffer);
        trace(`Batch update completed`);
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

    /**
     * Poll events from a window's event queue
     * This is called periodically instead of using callbacks
     */
    public pollEvents(windowId: number): void {
        let ffiState = this.getFfiState(windowId);
        if (!ffiState) {
            return;
        }

        ffiState.clear();
        const [windowIdBuffer, windowIdPtr] = ffiState.createInt64(BigInt(windowId));
        const eventsPtr = lib.symbols.gpui_poll_events(windowIdPtr);

        if (!eventsPtr) {
            return; // No events
        }

        try {
            // Read the JSON string from the pointer
            const cString = new CString(eventsPtr);
            const jsonStr = cString.toString();

            if (!jsonStr || jsonStr === "[]") {
                return;
            }

            const events = JSON.parse(jsonStr) as RawEventData[];

            // Process each event
            for (const rawEvent of events) {
                const { elementId, eventType } = rawEvent;
                const gpuiEvent = createEvent(rawEvent);
                eventRouter.dispatchToHandler(elementId, eventType, gpuiEvent);
            }
        } catch (err) {
            console.error("[JS] Event polling error:", err);
        } finally {
            // Free the string allocated by Rust
            lib.symbols.gpui_free_event_string(eventsPtr);
        }
    }

    /**
     * Start periodic event polling for a window
     */
    private startEventPolling(windowId: number, pollEventInterval?: number): void {
        // Poll every 5ms  for responsive event handling
        pollEventInterval = pollEventInterval ? pollEventInterval : 5;
        const interval = setInterval(() => {
            this.pollEvents(windowId);
        }, pollEventInterval);

        this.pollIntervals.set(windowId, interval);
        console.log(`[JS] Started event polling for window ${windowId}`);
    }

    /**
     * Stop event polling for a window
     */
    public stopEventPolling(windowId: number): void {
        const interval = this.pollIntervals.get(windowId);
        if (interval) {
            clearInterval(interval);
            this.pollIntervals.delete(windowId);
            console.log(`[JS] Stopped event polling for window ${windowId}`);
        }
    }

    /**
     * Get the current value of an input element from Rust
     * This is used to sync React state with Rust state for controlled inputs
     */
    public getInputValue(windowId: number, elementId: number): string {
        let ffiState = this.getFfiState(windowId);
        if (!ffiState) {
            return "";
        }

        ffiState.clear();
        const [windowIdBuffer, windowIdPtr] = ffiState.createInt64(BigInt(windowId));
        const [elementIdBuffer, elementIdPtr] = ffiState.createInt64(BigInt(elementId));
        const valuePtr = lib.symbols.gpui_get_input_value(windowIdPtr, elementIdPtr);

        if (!valuePtr) {
            return "";
        }

        try {
            const cString = new CString(valuePtr);
            const jsonStr = cString.toString();
            const parsed = JSON.parse(jsonStr) as { value: string };
            return parsed.value || "";
        } catch (err) {
            console.error("[JS] getInputValue error:", err);
            return "";
        } finally {
            lib.symbols.gpui_free_event_string(valuePtr);
        }
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
