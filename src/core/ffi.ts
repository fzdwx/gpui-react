import { dlopen, FFIType } from "bun:ffi";
import { info } from "../utils/logging";
import { getNativeLibPath } from "../utils/plat";

// ============================================================================
// GPUI FFI Binding
// ============================================================================

const libPath = getNativeLibPath(import.meta.dir);
info(`Loading GPUI library from: ${libPath}`);

export const lib = dlopen(libPath, {
    gpui_init: { args: [FFIType.ptr], returns: FFIType.void },
    gpui_free_event_string: {
        args: [FFIType.ptr],
        returns: FFIType.void,
    },
    gpui_poll_events: {
        args: [FFIType.ptr],
        returns: FFIType.ptr,
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
    gpui_get_input_value: {
        args: [FFIType.ptr, FFIType.ptr],
        returns: FFIType.ptr,
    },
});
