import { dlopen, FFIType, ptr, read } from "bun:ffi";
import { info, debug, trace } from "../reconciler/utils/logging";
import { getNativeLibPath } from "../reconciler/utils/plat";
import { sleep } from "bun";

// ============================================================================
// GPUI FFI Binding
// ============================================================================

const libPath = getNativeLibPath(import.meta.dir);
info(`Loading GPUI library from: ${libPath}`);

export const lib = dlopen(libPath, {
    gpui_init: { args: [FFIType.ptr], returns: FFIType.void },
    set_event_callback: {
        args: [FFIType.ptr],
        returns: FFIType.void,
    },
    gpui_free_event_string: {
        args: [FFIType.ptr],
        returns: FFIType.void,
    },
    gpui_create_window: {
        args: [FFIType.ptr, FFIType.ptr],
        returns: FFIType.void,
    },
    gpui_is_ready: { args: [], returns: FFIType.bool },
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
    gpui_trigger_render: { args: [FFIType.ptr, FFIType.ptr], returns: FFIType.void },
    gpui_free_result: { args: [FFIType.ptr], returns: FFIType.void },
    gpui_batch_update_elements: {
        args: [FFIType.ptr, FFIType.ptr, FFIType.ptr, FFIType.ptr],
        returns: FFIType.void,
    },
});
