/**
 * Event Types for React-GPUI Renderer
 */

export type EventHandler = (event: Event) => void;

export type MouseEvent = {
    type: "click" | "hover" | "mouseenter" | "mouseleave";
    target: number;
    button?: number;
    clientX?: number;
    clientY?: number;
};

export type Event = MouseEvent;
