/**
 * GPUI Event Base Types
 * Completely independent from DOM Event - custom event system for GPUI
 */

// Import generated event type
import type { GPUIEventType } from "./generated";

// Re-export for backward compatibility
export type { GPUIEventType };

/** Event propagation phase */
export type EventPhase = "capture" | "target" | "bubble";

/** Modifier keys state */
export interface ModifierKeys {
    readonly ctrl: boolean;
    readonly shift: boolean;
    readonly alt: boolean;
    readonly meta: boolean; // Cmd on Mac, Win on Windows
}

/** Default modifier keys (all false) */
export const DEFAULT_MODIFIERS: ModifierKeys = {
    ctrl: false,
    shift: false,
    alt: false,
    meta: false,
};

/**
 * Base interface for all GPUI events
 * All specific event types extend this interface
 */
export interface GPUIBaseEvent {
    /** Event type identifier */
    readonly type: GPUIEventType;

    /** Element ID that originally triggered the event */
    readonly target: number;

    /** Element ID currently handling the event (changes during bubbling) */
    readonly currentTarget: number;

    /** Window ID this event belongs to */
    readonly windowId: number;

    /** Event timestamp in milliseconds (Unix epoch) */
    readonly timestamp: number;

    /** Current propagation phase */
    readonly phase: EventPhase;

    /** Whether stopPropagation() has been called */
    readonly propagationStopped: boolean;

    /** Whether preventDefault() has been called */
    readonly defaultPrevented: boolean;

    /** Stop event from propagating to parent elements */
    stopPropagation(): void;

    /** Prevent default behavior (if any) */
    preventDefault(): void;
}

/**
 * Create a mutable event wrapper for internal use
 * Allows modifying phase and currentTarget during dispatch
 */
export interface MutableGPUIEvent extends Omit<GPUIBaseEvent, "phase" | "currentTarget"> {
    phase: EventPhase;
    currentTarget: number;
}

/**
 * Create default event methods
 */
export function createEventMethods() {
    let propagationStopped = false;
    let defaultPrevented = false;

    return {
        get propagationStopped() {
            return propagationStopped;
        },
        get defaultPrevented() {
            return defaultPrevented;
        },
        stopPropagation() {
            propagationStopped = true;
        },
        preventDefault() {
            defaultPrevented = true;
        },
    };
}
