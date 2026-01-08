/**
 * GPUI Events Module
 * Re-exports all event types and utilities
 */

// Base types
export type {
    GPUIBaseEvent,
    GPUIEventType,
    EventPhase,
    ModifierKeys,
    MutableGPUIEvent,
} from "./base";

export { DEFAULT_MODIFIERS, createEventMethods } from "./base";

// Mouse events
export type { GPUIMouseEvent, MouseEventType, MouseButton } from "./mouse";

export {
    MouseButtons,
    isMouseEvent,
    shouldMouseEventBubble,
    NON_BUBBLING_MOUSE_EVENTS,
} from "./mouse";

// Keyboard events
export type { GPUIKeyboardEvent, KeyboardEventType } from "./keyboard";

export { isKeyboardEvent, KeyCodes } from "./keyboard";

// Focus events
export type { GPUIFocusEvent, FocusEventType } from "./focus";

export { isFocusEvent, shouldFocusEventBubble, NON_BUBBLING_FOCUS_EVENTS } from "./focus";

// Scroll events
export type { GPUIScrollEvent, GPUIWheelEvent, ScrollEventType, WheelDeltaMode } from "./scroll";

export { isScrollEvent, isWheelEvent } from "./scroll";

// Type mappings and handlers
export type {
    GPUIEvent,
    GPUIEventMap,
    GPUIEventHandler,
    AnyGPUIEventHandler,
    GPUIEventHandlerProps,
} from "./types";

export {
    EVENT_PROP_TO_TYPE,
    EVENT_TYPE_TO_PROP,
    SUPPORTED_EVENT_PROPS,
    isEventHandlerProp,
} from "./types";

// Event factory
export type { RawEventData } from "./factory";
export { createEvent } from "./factory";

// Event router
export type { HandlerOptions, EventRouterStats } from "./router";
export { EventRouter, eventRouter } from "./router";
