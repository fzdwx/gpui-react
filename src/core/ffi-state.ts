import { ptr, read } from "bun:ffi";
import { sleep } from "bun";
import { lib } from "./ffi";

export const encoder: TextEncoder = new TextEncoder();
export const decoder: TextDecoder = new TextDecoder();

export class FfiState {
    liveBuffers: ArrayBuffer[] = [];

    keep(buffer: ArrayBuffer): void {
        this.liveBuffers.push(buffer);
    }

    clear(): void {
        this.liveBuffers.length = 0;
    }

    encodeCString(str: string): [ArrayBuffer, ReturnType<typeof ptr>] {
        const buffer = encoder.encode(str + "\0");
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
}
