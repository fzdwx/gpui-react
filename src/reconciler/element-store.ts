import { trace, debug } from "../utils/logging";
import { ElementData } from "../core";

const STORE_SYMBOL = Symbol("store");

export class ElementStore {
    private store = new Map<number, ElementData>();
    private nextId = 2;
    private rootId: number | null = null;
    private windowId: number = 0;

    setWindowId(id: number): void {
        this.windowId = id;
    }

    getWindowId(): number {
        return this.windowId;
    }

    reset(): void {
        this.store.clear();
        this.nextId = 2;
        this.rootId = null;
        this.windowId = 0;
    }

    createElement(
        type: string,
        text?: string,
        style?: Record<string, any>,
        eventHandlers?: Record<string, number>
    ): number {
        const globalId = this.nextId++;
        const element: ElementData = {
            globalId,
            type,
            text,
            style,
            children: [],
            eventHandlers,
        };
        Object.defineProperty(element, STORE_SYMBOL, {
            value: this,
            writable: false,
            enumerable: false,
        });
        this.store.set(globalId, element);
        trace("createElement", { type, globalId, style, eventHandlers });
        return globalId;
    }

    getStore(element: ElementData): ElementStore | undefined {
        return (element as any)[STORE_SYMBOL];
    }

    appendChild(parentId: number, childId: number): void {
        const parent = this.store.get(parentId);
        if (!parent) throw new Error(`Parent element ${parentId} not found`);
        parent.children.push(childId);
        trace("appendChild", { parentId, childId });
    }

    removeChild(parentId: number, childId: number): void {
        const parent = this.store.get(parentId);
        if (!parent) throw new Error(`Parent element ${parentId} not found`);
        const index = parent.children.indexOf(childId);
        if (index !== -1) {
            parent.children.splice(index, 1);
            trace("removeChild", { parentId, childId });
        }
    }

    setContainerChild(childId: number): void {
        if (this.rootId === null) {
            this.rootId = childId;
            trace("setContainerChild - rootId", this.rootId);
        }
    }

    getElement(globalId: number): ElementData | undefined {
        return this.store.get(globalId);
    }

    getRoot(): ElementData {
        if (this.rootId === null) {
            throw new Error("No root element created yet");
        }
        return this.store.get(this.rootId)!;
    }
}
