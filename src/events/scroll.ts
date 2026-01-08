/**
 * GPUI Scroll Event Types
 */

import { GPUIBaseEvent, ModifierKeys } from "./base";

/** Scroll event type names */
export type ScrollEventType = "scroll" | "wheel";

/**
 * GPUI Scroll Event
 * Triggered when an element is scrolled
 */
export interface GPUIScrollEvent extends GPUIBaseEvent {
    readonly type: "scroll";

    /** Current horizontal scroll position */
    readonly scrollX: number;

    /** Current vertical scroll position */
    readonly scrollY: number;
}

/** Wheel delta mode values */
export type WheelDeltaMode =
    | 0 // Pixels
    | 1 // Lines
    | 2; // Pages

/**
 * GPUI Wheel Event
 * Triggered by mouse wheel or trackpad
 */
export interface GPUIWheelEvent extends GPUIBaseEvent {
    readonly type: "wheel";

    /** Horizontal scroll amount */
    readonly deltaX: number;

    /** Vertical scroll amount */
    readonly deltaY: number;

    /** Depth scroll amount (rarely used) */
    readonly deltaZ: number;

    /**
     * Delta mode:
     * 0 = pixels
     * 1 = lines
     * 2 = pages
     */
    readonly deltaMode: WheelDeltaMode;

    /** Modifier keys state at the time of the event */
    readonly modifiers: ModifierKeys;
}

/**
 * Check if an event is a scroll event
 */
export function isScrollEvent(event: GPUIBaseEvent): event is GPUIScrollEvent {
    return event.type === "scroll";
}

/**
 * Check if an event is a wheel event
 */
export function isWheelEvent(event: GPUIBaseEvent): event is GPUIWheelEvent {
    return event.type === "wheel";
}
