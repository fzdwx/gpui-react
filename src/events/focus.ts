/**
 * GPUI Focus Event Types
 */

import { GPUIBaseEvent } from "./base";

/** Focus event type names */
export type FocusEventType = "focus" | "blur" | "focusin" | "focusout";

/**
 * GPUI Focus Event
 * Contains focus-related event information
 */
export interface GPUIFocusEvent extends GPUIBaseEvent {
    readonly type: FocusEventType;

    /**
     * The element that is losing/gaining focus relative to this event
     * - For focus/focusin: the element that lost focus
     * - For blur/focusout: the element that will gain focus
     * - null if focus is entering/leaving the window
     */
    readonly relatedTarget: number | null;
}

/**
 * Check if an event is a focus event
 */
export function isFocusEvent(event: GPUIBaseEvent): event is GPUIFocusEvent {
    return ["focus", "blur", "focusin", "focusout"].includes(event.type);
}

/**
 * Events that should NOT bubble
 * focus and blur don't bubble, use focusin/focusout for bubbling versions
 */
export const NON_BUBBLING_FOCUS_EVENTS: FocusEventType[] = ["focus", "blur"];

/**
 * Check if a focus event should bubble
 */
export function shouldFocusEventBubble(type: FocusEventType): boolean {
    return !NON_BUBBLING_FOCUS_EVENTS.includes(type);
}
