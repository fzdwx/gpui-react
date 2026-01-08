/**
 * Event Router for React-GPUI
 * Routes events based on window_id and element_id to the correct JavaScript handlers
 * Supports event capture/bubble phases, memory management, and type-safe handlers
 */

import type {
    GPUIEvent,
    AnyGPUIEventHandler,
} from "./types";
import {
    GPUIBaseEvent,
    MutableGPUIEvent,
} from "./base";

/** Handler registration options */
export interface HandlerOptions {
    /** Run handler during capture phase */
    capture?: boolean;
    /** Remove handler after first invocation */
    once?: boolean;
}

/** Internal handler entry */
interface HandlerEntry {
    handler: AnyGPUIEventHandler;
    capture: boolean;
    once: boolean;
}

/** Element event data */
interface ElementEventData {
    /** Map of eventType -> handlerIds (multiple handlers per event type) */
    handlers: Map<string, number[]>;
    /** Parent element ID for bubbling */
    parentId: number | null;
}

/** Event router statistics for debugging */
export interface EventRouterStats {
    totalHandlers: number;
    totalElements: number;
    windowRoots: number;
}

/**
 * EventRouter class - manages all event routing for GPUI
 */
export class EventRouter {
    /** Handler storage: handlerId -> HandlerEntry */
    private handlerStore = new Map<number, HandlerEntry>();

    /** Element event data: elementId -> ElementEventData */
    private elementStore = new Map<number, ElementEventData>();

    /** Window root element tracking: windowId -> rootElementId */
    private windowRoots = new Map<number, number>();

    /** Next handler ID */
    private nextHandlerId = 0;

    /**
     * Register an event handler and return its ID
     */
    registerHandler(
        handler: AnyGPUIEventHandler,
        options: HandlerOptions = {}
    ): number {
        const id = this.nextHandlerId++;
        this.handlerStore.set(id, {
            handler,
            capture: options.capture ?? false,
            once: options.once ?? false,
        });
        return id;
    }

    /**
     * Unregister a handler by ID
     */
    unregisterHandler(handlerId: number): boolean {
        return this.handlerStore.delete(handlerId);
    }

    /**
     * Bind an event handler to an element for a specific event type
     */
    bindEvent(elementId: number, eventType: string, handlerId: number): void {
        let elementData = this.elementStore.get(elementId);
        if (!elementData) {
            elementData = {
                handlers: new Map(),
                parentId: null,
            };
            this.elementStore.set(elementId, elementData);
        }

        let handlerIds = elementData.handlers.get(eventType);
        if (!handlerIds) {
            handlerIds = [];
            elementData.handlers.set(eventType, handlerIds);
        }

        // Avoid duplicates
        if (!handlerIds.includes(handlerId)) {
            handlerIds.push(handlerId);
        }
    }

    /**
     * Unbind a specific handler from an element
     */
    unbindEvent(elementId: number, eventType: string, handlerId: number): void {
        const elementData = this.elementStore.get(elementId);
        if (!elementData) return;

        const handlerIds = elementData.handlers.get(eventType);
        if (!handlerIds) return;

        const index = handlerIds.indexOf(handlerId);
        if (index !== -1) {
            handlerIds.splice(index, 1);
        }

        // Clean up empty arrays
        if (handlerIds.length === 0) {
            elementData.handlers.delete(eventType);
        }
    }

    /**
     * Unbind all handlers for a specific event type on an element
     */
    unbindEventType(elementId: number, eventType: string): number[] {
        const elementData = this.elementStore.get(elementId);
        if (!elementData) return [];

        const handlerIds = elementData.handlers.get(eventType) || [];
        elementData.handlers.delete(eventType);
        return handlerIds;
    }

    /**
     * Set parent-child relationship for event bubbling
     */
    setParent(elementId: number, parentId: number | null): void {
        let elementData = this.elementStore.get(elementId);
        if (!elementData) {
            elementData = {
                handlers: new Map(),
                parentId: null,
            };
            this.elementStore.set(elementId, elementData);
        }
        elementData.parentId = parentId;
    }

    /**
     * Get parent element ID
     */
    getParent(elementId: number): number | null {
        return this.elementStore.get(elementId)?.parentId ?? null;
    }

    /**
     * Set root element for a window
     */
    setWindowRoot(windowId: number, rootElementId: number): void {
        this.windowRoots.set(windowId, rootElementId);
    }

    /**
     * Get root element for a window
     */
    getWindowRoot(windowId: number): number | null {
        return this.windowRoots.get(windowId) ?? null;
    }

    /**
     * Clean up all event data for an element
     * Should be called when an element is removed from the tree
     */
    cleanupElement(elementId: number): void {
        const elementData = this.elementStore.get(elementId);
        if (!elementData) return;

        // Unregister all handlers associated with this element
        for (const handlerIds of elementData.handlers.values()) {
            for (const handlerId of handlerIds) {
                this.handlerStore.delete(handlerId);
            }
        }

        // Remove element data
        this.elementStore.delete(elementId);
    }

    /**
     * Clean up an element and all its descendants
     * @param elementId The root element to clean up
     * @param childIds All descendant element IDs
     */
    cleanupElementTree(elementId: number, childIds: number[]): void {
        // Clean up all children first
        for (const childId of childIds) {
            this.cleanupElement(childId);
        }
        // Then clean up the element itself
        this.cleanupElement(elementId);
    }

    /**
     * Get handlers for an element and event type
     */
    getHandlers(
        elementId: number,
        eventType: string,
        capturePhase: boolean
    ): HandlerEntry[] {
        const elementData = this.elementStore.get(elementId);
        if (!elementData) return [];

        const handlerIds = elementData.handlers.get(eventType);
        if (!handlerIds) return [];

        const handlers: HandlerEntry[] = [];
        for (const handlerId of handlerIds) {
            const entry = this.handlerStore.get(handlerId);
            if (entry && entry.capture === capturePhase) {
                handlers.push(entry);
            }
        }
        return handlers;
    }

    /**
     * Build the path from root to target element
     */
    private buildEventPath(targetId: number, _windowId: number): number[] {
        const path: number[] = [];
        let currentId: number | null = targetId;

        while (currentId !== null) {
            path.unshift(currentId);
            currentId = this.getParent(currentId);
        }

        return path;
    }

    /**
     * Check if an event type should bubble
     */
    private shouldBubble(eventType: string): boolean {
        // Mouse events that don't bubble
        if (eventType === "mouseenter" || eventType === "mouseleave") {
            return false;
        }
        // Focus/blur don't bubble (use focusin/focusout for bubbling)
        if (eventType === "focus" || eventType === "blur") {
            return false;
        }
        // Scroll doesn't bubble
        if (eventType === "scroll") {
            return false;
        }
        return true;
    }

    /**
     * Dispatch an event through the capture → target → bubble phases
     */
    dispatchEvent(event: GPUIEvent): void {
        const { target, windowId, type } = event as GPUIBaseEvent;

        // Build path from root to target
        const path = this.buildEventPath(target, windowId);
        if (path.length === 0) return;

        // Create mutable event for phase tracking
        const mutableEvent = event as MutableGPUIEvent;

        // Handlers to remove after dispatch (for once: true)
        const handlersToRemove: { elementId: number; eventType: string; handlerId: number }[] = [];

        // Capture phase: root → target (exclusive)
        mutableEvent.phase = "capture";
        for (let i = 0; i < path.length - 1; i++) {
            const elementId = path[i];
            mutableEvent.currentTarget = elementId;

            const handlers = this.getHandlers(elementId, type, true);
            for (const entry of handlers) {
                entry.handler(event);
                if (entry.once) {
                    const handlerIds = this.elementStore.get(elementId)?.handlers.get(type);
                    if (handlerIds) {
                        const idx = handlerIds.findIndex(id => this.handlerStore.get(id) === entry);
                        if (idx !== -1) {
                            handlersToRemove.push({ elementId, eventType: type, handlerId: handlerIds[idx] });
                        }
                    }
                }
            }

            if ((event as GPUIBaseEvent).propagationStopped) break;
        }

        // Target phase
        if (!(event as GPUIBaseEvent).propagationStopped) {
            mutableEvent.phase = "target";
            const targetId = path[path.length - 1];
            mutableEvent.currentTarget = targetId;

            // Both capture and bubble handlers fire at target
            const captureHandlers = this.getHandlers(targetId, type, true);
            const bubbleHandlers = this.getHandlers(targetId, type, false);

            for (const entry of [...captureHandlers, ...bubbleHandlers]) {
                entry.handler(event);
                if (entry.once) {
                    const handlerIds = this.elementStore.get(targetId)?.handlers.get(type);
                    if (handlerIds) {
                        const idx = handlerIds.findIndex(id => this.handlerStore.get(id) === entry);
                        if (idx !== -1) {
                            handlersToRemove.push({ elementId: targetId, eventType: type, handlerId: handlerIds[idx] });
                        }
                    }
                }
                if ((event as GPUIBaseEvent).propagationStopped) break;
            }
        }

        // Bubble phase: target → root (exclusive), only if event bubbles
        if (!(event as GPUIBaseEvent).propagationStopped && this.shouldBubble(type)) {
            mutableEvent.phase = "bubble";
            for (let i = path.length - 2; i >= 0; i--) {
                const elementId = path[i];
                mutableEvent.currentTarget = elementId;

                const handlers = this.getHandlers(elementId, type, false);
                for (const entry of handlers) {
                    entry.handler(event);
                    if (entry.once) {
                        const handlerIds = this.elementStore.get(elementId)?.handlers.get(type);
                        if (handlerIds) {
                            const idx = handlerIds.findIndex(id => this.handlerStore.get(id) === entry);
                            if (idx !== -1) {
                                handlersToRemove.push({ elementId, eventType: type, handlerId: handlerIds[idx] });
                            }
                        }
                    }
                    if ((event as GPUIBaseEvent).propagationStopped) break;
                }

                if ((event as GPUIBaseEvent).propagationStopped) break;
            }
        }

        // Clean up once handlers
        for (const { elementId, eventType, handlerId } of handlersToRemove) {
            this.unbindEvent(elementId, eventType, handlerId);
            this.unregisterHandler(handlerId);
        }
    }

    /**
     * Simple dispatch for a single handler (backward compatible)
     */
    dispatchToHandler(elementId: number, eventType: string, event: GPUIEvent): boolean {
        const handlers = this.getHandlers(elementId, eventType, false);
        if (handlers.length === 0) return false;

        for (const entry of handlers) {
            entry.handler(event);
        }
        return true;
    }

    /**
     * Get statistics for debugging
     */
    getStats(): EventRouterStats {
        return {
            totalHandlers: this.handlerStore.size,
            totalElements: this.elementStore.size,
            windowRoots: this.windowRoots.size,
        };
    }

    /**
     * Clear all data (for testing/reset)
     */
    clear(): void {
        this.handlerStore.clear();
        this.elementStore.clear();
        this.windowRoots.clear();
        this.nextHandlerId = 0;
    }
}

// Global singleton instance
export const eventRouter = new EventRouter();
