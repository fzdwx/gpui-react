/**
 * Event Router for React-GPUI
 * Routes events based on window_id and element_id to the correct JavaScript handlers
 */

import { JSCallback, Pointer, toArrayBuffer } from "bun:ffi";
import { decoder } from "../core/ffi-state";
import { info, trace, debug } from "./utils/logging";

// Event handler storage: handlerId -> EventHandler
const eventHandlerMap = new Map<number, (event: GPUIEvent) => void>();

// Element event handlers: globalId -> { eventType: handlerId }
const elementEventMap = new Map<number, Map<string, number>>();

let nextHandlerId = 0;

/**
 * Register an event handler and return its ID
 */
export function registerEventHandler(handler: (event: GPUIEvent) => void): number {
    const id = nextHandlerId++;
    eventHandlerMap.set(id, handler);
    return id;
}

/**
 * Bind an event handler to an element for a specific event type
 */
export function bindEventToElement(elementId: number, eventType: string, handlerId: number): void {
    if (!elementEventMap.has(elementId)) {
        console.log(`bind element handle ${elementId} ${eventType}`)
        elementEventMap.set(elementId, new Map());
    }
    elementEventMap.get(elementId)!.set(eventType, handlerId);
}

/**
 * Get the event handler for an element and event type
 */
export function getEventHandler(
    elementId: number,
    eventType: string
): ((event: GPUIEvent) => void) | null {
    const elementHandlers = elementEventMap.get(elementId);
    if (!elementHandlers) {
        return null;
    }
    const handlerId = elementHandlers.get(eventType);
    if (handlerId === undefined) {
        return null;
    }
    return eventHandlerMap.get(handlerId) || null;
}

/**
 * GPUI Event structure
 */
export interface GPUIEvent {
    type: string;
    target: number;
    windowId: number;
    button?: number;
    clientX?: number;
    clientY?: number;
    timestamp: number;
}
