export interface ElementData {
    globalId: number;
    type: string;
    text?: string;
    children: number[];
    style?: Record<string, any>;
    eventHandlers?: Record<string, number>;
}
export declare class ElementStore {
    private store;
    private nextId;
    private rootId;
    reset(): void;
    createElement(type: string, text?: string, style?: Record<string, any>): number;
    getStore(element: ElementData): ElementStore | undefined;
    appendChild(parentId: number, childId: number): void;
    removeChild(parentId: number, childId: number): void;
    setContainerChild(childId: number): void;
    getElement(globalId: number): ElementData | undefined;
    getRoot(): ElementData;
}
//# sourceMappingURL=element-store.d.ts.map