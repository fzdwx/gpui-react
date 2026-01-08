/**
 * GPUI Event Type Mappings and Handler Types
 * Provides type-safe event handler definitions
 */

import { GPUIMouseEvent } from "./mouse";
import { GPUIKeyboardEvent } from "./keyboard";
import { GPUIFocusEvent } from "./focus";
import { GPUIScrollEvent, GPUIWheelEvent } from "./scroll";

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
 * Maps React prop names to internal event type names
 */
export const EVENT_PROP_TO_TYPE: Record<keyof GPUIEventHandlerProps, keyof GPUIEventMap> = {
    onClick: "click",
    onDoubleClick: "dblclick",
    onMouseDown: "mousedown",
    onMouseUp: "mouseup",
    onMouseMove: "mousemove",
    onMouseEnter: "mouseenter",
    onMouseLeave: "mouseleave",
    onHover: "hover",
    onKeyDown: "keydown",
    onKeyUp: "keyup",
    onKeyPress: "keypress",
    onFocus: "focus",
    onBlur: "blur",
    onScroll: "scroll",
    onWheel: "wheel",
};

/**
 * Maps internal event type names to React prop names
 */
export const EVENT_TYPE_TO_PROP: Record<keyof GPUIEventMap, keyof GPUIEventHandlerProps> = {
    click: "onClick",
    dblclick: "onDoubleClick",
    mousedown: "onMouseDown",
    mouseup: "onMouseUp",
    mousemove: "onMouseMove",
    mouseenter: "onMouseEnter",
    mouseleave: "onMouseLeave",
    hover: "onHover",
    keydown: "onKeyDown",
    keyup: "onKeyUp",
    keypress: "onKeyPress",
    focus: "onFocus",
    blur: "onBlur",
    focusin: "onFocus", // focusin maps to onFocus (bubbling version)
    focusout: "onBlur", // focusout maps to onBlur (bubbling version)
    scroll: "onScroll",
    wheel: "onWheel",
};

/**
 * List of all supported event prop names
 */
export const SUPPORTED_EVENT_PROPS = Object.keys(EVENT_PROP_TO_TYPE) as (keyof GPUIEventHandlerProps)[];

/**
 * Check if a prop name is an event handler prop
 */
export function isEventHandlerProp(propName: string): propName is keyof GPUIEventHandlerProps {
    return propName in EVENT_PROP_TO_TYPE;
}
