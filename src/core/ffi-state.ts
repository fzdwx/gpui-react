import {ptr, read} from "bun:ffi";
import {sleep} from "bun";
import {lib} from "./ffi";

export class Gpui {
    liveBuffers: ArrayBuffer[] = [];

    keep(buffer: ArrayBuffer): void {
        this.liveBuffers.push(buffer);
    }

    clear(): void {
        this.liveBuffers.length = 0;
    }

    encodeCString(str: string): [ArrayBuffer, ReturnType<typeof ptr>] {
        const buffer = new TextEncoder().encode(str + "\0");
        this.keep(buffer.buffer);
        return [buffer.buffer, ptr(buffer)];
    }

    createInt64(value: bigint): [ArrayBuffer, ReturnType<typeof ptr>] {
        const buffer = new ArrayBuffer(8);
        new DataView(buffer).setBigInt64(0, value, true);
        this.keep(buffer);
        return [buffer, ptr(buffer)];
    }

    createInt64Array(values: bigint[]): [ArrayBuffer, ReturnType<typeof ptr>] {
        const byteLength = Math.max(values.length * 8, 8);
        const buffer = new ArrayBuffer(byteLength);
        if (values.length > 0) {
            const view = new BigInt64Array(buffer);
            for (let i = 0; i < values.length; i++) view[i] = values[i];
        }
        this.keep(buffer);
        return [buffer, ptr(buffer)];
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

    waitReady(): void {
        let delay = 1;
        while (Date.now() - Date.now() < 5000) {
            if (lib.symbols.gpui_is_ready()) return;
            sleep(Math.min(delay, 100));
            delay *= 2;
        }
        throw new Error("GPUI failed to become ready");
    }
}

export const gpui = new Gpui();
