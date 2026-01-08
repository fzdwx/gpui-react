/**
 * GPUI Event Factory
 * Creates properly typed GPUI events from raw Rust event data
 */

import {
    EventPhase,
    ModifierKeys,
    createEventMethods,
} from "./base";
import { GPUIMouseEvent, MouseEventType, MouseButton } from "./mouse";
import { GPUIKeyboardEvent, KeyboardEventType } from "./keyboard";
import { GPUIFocusEvent, FocusEventType } from "./focus";
import { GPUIScrollEvent, GPUIWheelEvent, WheelDeltaMode } from "./scroll";
import { GPUIEvent } from "./types";

/** Raw event data from Rust FFI */
export interface RawEventData {
    windowId: number;
    elementId: number;
    eventType: string;
    // Mouse event data
    clientX?: number;
    clientY?: number;
    offsetX?: number;
    offsetY?: number;
    button?: number;
    // Keyboard event data
    code?: string;
    key?: string;
    repeat?: boolean;
    // Focus event data
    relatedTarget?: number | null;
    // Scroll/Wheel event data
    scrollX?: number;
    scrollY?: number;
    deltaX?: number;
    deltaY?: number;
    deltaZ?: number;
    deltaMode?: number;
    // Modifier keys
    ctrlKey?: boolean;
    shiftKey?: boolean;
    altKey?: boolean;
    metaKey?: boolean;
}

/**
 * Create a GPUIEvent from raw Rust event data
 */
export function createEvent(raw: RawEventData): GPUIEvent {
    const eventType = raw.eventType;
    const methods = createEventMethods();

    const modifiers: ModifierKeys = {
        ctrl: raw.ctrlKey ?? false,
        shift: raw.shiftKey ?? false,
        alt: raw.altKey ?? false,
        meta: raw.metaKey ?? false,
    };

    const baseProps = {
        target: raw.elementId,
        currentTarget: raw.elementId,
        windowId: raw.windowId,
        timestamp: Date.now(),
        phase: "target" as EventPhase,
        get propagationStopped() { return methods.propagationStopped; },
        get defaultPrevented() { return methods.defaultPrevented; },
        stopPropagation: methods.stopPropagation,
        preventDefault: methods.preventDefault,
    };

    // Mouse events
    if (isMouseEventType(eventType)) {
        const mouseEvent: GPUIMouseEvent = {
            ...baseProps,
            type: eventType as MouseEventType,
            button: (raw.button ?? 0) as MouseButton,
            clientX: raw.clientX ?? 0,
            clientY: raw.clientY ?? 0,
            offsetX: raw.offsetX ?? raw.clientX ?? 0,
            offsetY: raw.offsetY ?? raw.clientY ?? 0,
            modifiers,
        };
        return mouseEvent;
    }

    // Keyboard events
    if (isKeyboardEventType(eventType)) {
        const keyboardEvent: GPUIKeyboardEvent = {
            ...baseProps,
            type: eventType as KeyboardEventType,
            code: raw.code ?? "",
            key: raw.key ?? "",
            repeat: raw.repeat ?? false,
            modifiers,
        };
        return keyboardEvent;
    }

    // Focus events
    if (isFocusEventType(eventType)) {
        const focusEvent: GPUIFocusEvent = {
            ...baseProps,
            type: eventType as FocusEventType,
            relatedTarget: raw.relatedTarget ?? null,
        };
        return focusEvent;
    }

    // Scroll event
    if (eventType === "scroll") {
        const scrollEvent: GPUIScrollEvent = {
            ...baseProps,
            type: "scroll",
            scrollX: raw.scrollX ?? 0,
            scrollY: raw.scrollY ?? 0,
        };
        return scrollEvent;
    }

    // Wheel event
    if (eventType === "wheel") {
        const wheelEvent: GPUIWheelEvent = {
            ...baseProps,
            type: "wheel",
            deltaX: raw.deltaX ?? 0,
            deltaY: raw.deltaY ?? 0,
            deltaZ: raw.deltaZ ?? 0,
            deltaMode: (raw.deltaMode ?? 0) as WheelDeltaMode,
            modifiers,
        };
        return wheelEvent;
    }

    // Fallback: create a mouse click event for unknown types
    const fallbackEvent: GPUIMouseEvent = {
        ...baseProps,
        type: "click",
        button: 0,
        clientX: raw.clientX ?? 0,
        clientY: raw.clientY ?? 0,
        offsetX: raw.offsetX ?? 0,
        offsetY: raw.offsetY ?? 0,
        modifiers,
    };
    return fallbackEvent;
}

// Type guards for event types
function isMouseEventType(type: string): type is MouseEventType {
    return [
        "click", "dblclick", "mousedown", "mouseup",
        "mousemove", "mouseenter", "mouseleave", "hover"
    ].includes(type);
}

function isKeyboardEventType(type: string): type is KeyboardEventType {
    return ["keydown", "keyup", "keypress"].includes(type);
}

function isFocusEventType(type: string): type is FocusEventType {
    return ["focus", "blur", "focusin", "focusout"].includes(type);
}
