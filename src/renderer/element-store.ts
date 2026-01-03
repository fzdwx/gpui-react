export interface ElementData {
  globalId: number;
  type: string;
  text?: string;
  children: number[];
}

class ElementStore {
  private store = new Map<number, ElementData>();
  private nextId = 2; // Start at 2 to avoid collision with container tag 1
  private rootId: number | null = null;

  createElement(type: string, text?: string): number {
    const globalId = this.nextId++;
    this.store.set(globalId, {
      globalId,
      type,
      text,
      children: [],
    });
    console.log("createElement:", { type, globalId });
    return globalId;
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

export const elementStore = new ElementStore();
