/**
 * GPUI Keyboard Event Types
 */

import { GPUIBaseEvent, ModifierKeys } from "./base";

/** Keyboard event type names */
export type KeyboardEventType = "keydown" | "keyup" | "keypress";

/**
 * GPUI Keyboard Event
 * Contains all keyboard-related event information
 */
export interface GPUIKeyboardEvent extends GPUIBaseEvent {
    readonly type: KeyboardEventType;

    /**
     * Physical key code (e.g., "KeyA", "Enter", "Space")
     * This represents the physical key position, not the character
     */
    readonly code: string;

    /**
     * Character value considering modifiers (e.g., "a", "A", "Enter")
     * This is the logical key value
     */
    readonly key: string;

    /** Whether this is a repeat event (key held down) */
    readonly repeat: boolean;

    /** Modifier keys state at the time of the event */
    readonly modifiers: ModifierKeys;
}

/**
 * Check if an event is a keyboard event
 */
export function isKeyboardEvent(event: GPUIBaseEvent): event is GPUIKeyboardEvent {
    return ["keydown", "keyup", "keypress"].includes(event.type);
}

/**
 * Common key codes for convenience
 */
export const KeyCodes = {
    // Letters
    KeyA: "KeyA",
    KeyB: "KeyB",
    KeyC: "KeyC",
    // ... etc

    // Numbers
    Digit0: "Digit0",
    Digit1: "Digit1",
    // ... etc

    // Special keys
    Enter: "Enter",
    Space: "Space",
    Escape: "Escape",
    Tab: "Tab",
    Backspace: "Backspace",
    Delete: "Delete",

    // Arrow keys
    ArrowUp: "ArrowUp",
    ArrowDown: "ArrowDown",
    ArrowLeft: "ArrowLeft",
    ArrowRight: "ArrowRight",

    // Modifiers
    ShiftLeft: "ShiftLeft",
    ShiftRight: "ShiftRight",
    ControlLeft: "ControlLeft",
    ControlRight: "ControlRight",
    AltLeft: "AltLeft",
    AltRight: "AltRight",
    MetaLeft: "MetaLeft",
    MetaRight: "MetaRight",
} as const;
