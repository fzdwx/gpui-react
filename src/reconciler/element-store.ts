export interface ElementData {
  globalId: number;
  type: string;
  text?: string;
  children: number[];
  style?: Record<string, any>;
  eventHandlers?: Record<string, number>;
  // Note: store is non-enumerable to avoid serialization issues
}

// Symbol for private store reference
const STORE_SYMBOL = Symbol('store');

export class ElementStore {
  private store = new Map<number, ElementData>();
  private nextId = 2;
  private rootId: number | null = null;

  reset(): void {
    this.store.clear();
    this.nextId = 2;
    this.rootId = null;
  }

  createElement(type: string, text?: string, style?: Record<string, any>): number {
    const globalId = this.nextId++;
    const element: ElementData = {
      globalId,
      type,
      text,
      style,
      children: [],
    };
    // Store reference using non-enumerable property to avoid serialization
    Object.defineProperty(element, STORE_SYMBOL, { value: this, writable: false, enumerable: false });
    this.store.set(globalId, element);
    console.log("createElement:", { type, globalId, style });
    return globalId;
  }

  // Get store reference from element (non-enumerable)
  getStore(element: ElementData): ElementStore | undefined {
    return (element as any)[STORE_SYMBOL];
  }

  appendChild(parentId: number, childId: number): void {
    const parent = this.store.get(parentId);
    if (!parent) throw new Error(`Parent element ${parentId} not found`);
    parent.children.push(childId);
    console.log("appendChild:", { parentId, childId });
  }

  removeChild(parentId: number, childId: number): void {
    const parent = this.store.get(parentId);
    if (!parent) throw new Error(`Parent element ${parentId} not found`);
    const index = parent.children.indexOf(childId);
    if (index !== -1) {
      parent.children.splice(index, 1);
      console.log("removeChild:", { parentId, childId });
    }
  }

  setContainerChild(childId: number): void {
    // First child appended to container is the root
    if (this.rootId === null) {
      this.rootId = childId;
      console.log("setContainerChild - rootId:", this.rootId);
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
