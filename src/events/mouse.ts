/**
 * GPUI Mouse Event Types
 */

import { GPUIBaseEvent, ModifierKeys } from "./base";

/** Mouse button values (matches standard: 0=left, 1=middle, 2=right) */
export type MouseButton = 0 | 1 | 2 | 3 | 4;

/** Mouse button names for convenience */
export const MouseButtons = {
    Left: 0 as MouseButton,
    Middle: 1 as MouseButton,
    Right: 2 as MouseButton,
    Back: 3 as MouseButton,
    Forward: 4 as MouseButton,
} as const;

/** Mouse event type names */
export type MouseEventType =
    | "click"
    | "dblclick"
    | "mousedown"
    | "mouseup"
    | "mousemove"
    | "mouseenter"
    | "mouseleave"
    | "hover";

/**
 * GPUI Mouse Event
 * Contains all mouse-related event information
 */
export interface GPUIMouseEvent extends GPUIBaseEvent {
    readonly type: MouseEventType;

    /** Mouse button that triggered the event (0=left, 1=middle, 2=right) */
    readonly button: MouseButton;

    /** X coordinate relative to the window */
    readonly clientX: number;

    /** Y coordinate relative to the window */
    readonly clientY: number;

    /** X coordinate relative to the target element */
    readonly offsetX: number;

    /** Y coordinate relative to the target element */
    readonly offsetY: number;

    /** Modifier keys state at the time of the event */
    readonly modifiers: ModifierKeys;
}

/**
 * Check if an event is a mouse event
 */
export function isMouseEvent(event: GPUIBaseEvent): event is GPUIMouseEvent {
    return [
        "click",
        "dblclick",
        "mousedown",
        "mouseup",
        "mousemove",
        "mouseenter",
        "mouseleave",
        "hover",
    ].includes(event.type);
}

/**
 * Events that should NOT bubble (per DOM spec)
 */
export const NON_BUBBLING_MOUSE_EVENTS: MouseEventType[] = ["mouseenter", "mouseleave"];

/**
 * Check if a mouse event should bubble
 */
export function shouldMouseEventBubble(type: MouseEventType): boolean {
    return !NON_BUBBLING_MOUSE_EVENTS.includes(type);
}
