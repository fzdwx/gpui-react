import { trace } from "./logging";
const STORE_SYMBOL = Symbol('store');
export class ElementStore {
    store = new Map();
    nextId = 2;
    rootId = null;
    reset() {
        this.store.clear();
        this.nextId = 2;
        this.rootId = null;
    }
    createElement(type, text, style) {
        const globalId = this.nextId++;
        const element = {
            globalId,
            type,
            text,
            style,
            children: [],
        };
        Object.defineProperty(element, STORE_SYMBOL, { value: this, writable: false, enumerable: false });
        this.store.set(globalId, element);
        trace("createElement", { type, globalId, style });
        return globalId;
    }
    getStore(element) {
        return element[STORE_SYMBOL];
    }
    appendChild(parentId, childId) {
        const parent = this.store.get(parentId);
        if (!parent)
            throw new Error(`Parent element ${parentId} not found`);
        parent.children.push(childId);
        trace("appendChild", { parentId, childId });
    }
    removeChild(parentId, childId) {
        const parent = this.store.get(parentId);
        if (!parent)
            throw new Error(`Parent element ${parentId} not found`);
        const index = parent.children.indexOf(childId);
        if (index !== -1) {
            parent.children.splice(index, 1);
            trace("removeChild", { parentId, childId });
        }
    }
    setContainerChild(childId) {
        if (this.rootId === null) {
            this.rootId = childId;
            trace("setContainerChild - rootId", this.rootId);
        }
    }
    getElement(globalId) {
        return this.store.get(globalId);
    }
    getRoot() {
        if (this.rootId === null) {
            throw new Error("No root element created yet");
        }
        return this.store.get(this.rootId);
    }
}
