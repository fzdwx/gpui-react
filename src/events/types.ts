/**
 * GPUI Event Type Mappings and Handler Types
 * Provides type-safe event handler definitions
 */

import { GPUIMouseEvent } from "./mouse";
import { GPUIKeyboardEvent } from "./keyboard";
import { GPUIFocusEvent } from "./focus";
import { GPUIScrollEvent, GPUIWheelEvent } from "./scroll";

// Import generated constants for local use and re-export
import {
    EVENT_PROP_TO_TYPE as _EVENT_PROP_TO_TYPE,
    EVENT_TYPE_TO_PROP as _EVENT_TYPE_TO_PROP,
    isEventHandlerProp as _isEventHandlerProp,
    MOUSE_EVENT_TYPES as _MOUSE_EVENT_TYPES,
    KEYBOARD_EVENT_TYPES as _KEYBOARD_EVENT_TYPES,
    FOCUS_EVENT_TYPES as _FOCUS_EVENT_TYPES,
    SCROLL_EVENT_TYPES as _SCROLL_EVENT_TYPES,
} from "./generated";

import type {
    GPUIEventType as _GPUIEventType,
    GPUIEventPropName as _GPUIEventPropName,
} from "./generated";

// Re-export with original names
export const EVENT_PROP_TO_TYPE = _EVENT_PROP_TO_TYPE;
export const EVENT_TYPE_TO_PROP = _EVENT_TYPE_TO_PROP;
export const isEventHandlerProp = _isEventHandlerProp;
export const MOUSE_EVENT_TYPES = _MOUSE_EVENT_TYPES;
export const KEYBOARD_EVENT_TYPES = _KEYBOARD_EVENT_TYPES;
export const FOCUS_EVENT_TYPES = _FOCUS_EVENT_TYPES;
export const SCROLL_EVENT_TYPES = _SCROLL_EVENT_TYPES;
export type GPUIEventType = _GPUIEventType;
export type GPUIEventPropName = _GPUIEventPropName;

/**
 * Union type of all GPUI events
 */
export type GPUIEvent =
    | GPUIMouseEvent
    | GPUIKeyboardEvent
    | GPUIFocusEvent
    | GPUIScrollEvent
    | GPUIWheelEvent;

/**
 * Maps event type string to its corresponding event interface
 */
export interface GPUIEventMap {
    // Mouse events
    click: GPUIMouseEvent;
    dblclick: GPUIMouseEvent;
    mousedown: GPUIMouseEvent;
    mouseup: GPUIMouseEvent;
    mousemove: GPUIMouseEvent;
    mouseenter: GPUIMouseEvent;
    mouseleave: GPUIMouseEvent;
    hover: GPUIMouseEvent;

    // Keyboard events
    keydown: GPUIKeyboardEvent;
    keyup: GPUIKeyboardEvent;
    keypress: GPUIKeyboardEvent;

    // Focus events
    focus: GPUIFocusEvent;
    blur: GPUIFocusEvent;
    focusin: GPUIFocusEvent;
    focusout: GPUIFocusEvent;

    // Scroll events
    scroll: GPUIScrollEvent;
    wheel: GPUIWheelEvent;
}

/**
 * Type-safe event handler
 * Automatically infers the correct event type based on the event name
 */
export type GPUIEventHandler<T extends keyof GPUIEventMap> = (event: GPUIEventMap[T]) => void;

/**
 * Generic event handler for any GPUI event
 */
export type AnyGPUIEventHandler = (event: GPUIEvent) => void;

/**
 * Event handler props for React components
 * Maps React-style prop names (onClick) to handler types
 */
export interface GPUIEventHandlerProps {
    // Mouse event handlers
    onClick?: GPUIEventHandler<"click">;
    onDoubleClick?: GPUIEventHandler<"dblclick">;
    onMouseDown?: GPUIEventHandler<"mousedown">;
    onMouseUp?: GPUIEventHandler<"mouseup">;
    onMouseMove?: GPUIEventHandler<"mousemove">;
    onMouseEnter?: GPUIEventHandler<"mouseenter">;
    onMouseLeave?: GPUIEventHandler<"mouseleave">;
    onHover?: GPUIEventHandler<"hover">;

    // Keyboard event handlers
    onKeyDown?: GPUIEventHandler<"keydown">;
    onKeyUp?: GPUIEventHandler<"keyup">;
    onKeyPress?: GPUIEventHandler<"keypress">;

    // Focus event handlers
    onFocus?: GPUIEventHandler<"focus">;
    onBlur?: GPUIEventHandler<"blur">;

    // Scroll event handlers
    onScroll?: GPUIEventHandler<"scroll">;
    onWheel?: GPUIEventHandler<"wheel">;
}

/**
 * List of all supported event prop names
 */
export const SUPPORTED_EVENT_PROPS = Object.keys(
    EVENT_PROP_TO_TYPE
) as (keyof GPUIEventHandlerProps)[];
