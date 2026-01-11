/**
 * GPUI Input Event Type
 * For text input elements (input, textarea)
 */

import type { GPUIBaseEvent } from "./base";

/** Input event types */
export type InputEventType = "input" | "change" | "beforeinput";

/**
 * GPUI Input Event
 * Fired when the value of an input element changes
 */
export interface GPUIInputEvent extends GPUIBaseEvent {
    readonly type: InputEventType;

    /** Current value of the input */
    readonly value: string;

    /** The data being inserted (for beforeinput/input) */
    readonly data: string | null;

    /** Type of input operation (insertText, deleteContentBackward, etc.) */
    readonly inputType: string;

    /** Whether the event is part of an IME composition */
    readonly isComposing: boolean;
}

/**
 * Input type for inputType field
 * Based on InputEvent.inputType specification
 */
export type InputTypeValue =
    | "insertText"
    | "insertCompositionText"
    | "insertFromComposition"
    | "insertFromPaste"
    | "insertFromDrop"
    | "deleteContentBackward"
    | "deleteContentForward"
    | "deleteWordBackward"
    | "deleteWordForward"
    | "deleteByCut"
    | "deleteByDrag"
    | "historyUndo"
    | "historyRedo";

/**
 * Type guard: Check if event is an input event
 */
export function isInputEvent(event: GPUIBaseEvent): event is GPUIInputEvent {
    return event.type === "input" || event.type === "change" || event.type === "beforeinput";
}
