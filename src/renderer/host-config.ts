import * as ReactReconciler from "react-reconciler";
import { elementStore } from "./element-store";
import { renderFrame, updateElement } from "./gpui-binding";

const config = {
  supportsMutation: true,

  getPublicInstance(instance: number): Element {
    return document.createElement("div");
  },

  getRootHostContext(): null {
    return null;
  },

  getChildHostContext(): null {
    return null;
  },

  prepareForCommit(): any {
    return null;
  },

  resetAfterCommit(): void {
    const root = elementStore.getRoot();
    console.log("resetAfterCommit - root element:", JSON.stringify(root, null, 2));
    renderFrame(root);
  },

  shouldSetTextContent(type: string, props: any): boolean {
    return false; // Force React to call createTextInstance for text children
  },

  resetTextContent(instance: number): void {},

  createTextInstance(text: string): number {
    const id = elementStore.createElement("text", String(text));
    console.log("createTextInstance:", { text, id });
    updateElement(elementStore.getElement(id));
    return id;
  },

  commitTextUpdate(textInstance: number, oldText: string, newText: string): void {
    const element = elementStore.getElement(textInstance);
    if (element) {
      element.text = String(newText);
      console.log("commitTextUpdate:", { textInstance, newText });
      updateElement(element);
    }
  },

  createInstance(type: string, props: any): number {
    const id = elementStore.createElement(type);
    console.log("createInstance:", { type, id });
    updateElement(elementStore.getElement(id));
    return id;
  },

  appendInitialChild(parent: number, child: number): void {
    console.log("appendInitialChild:", { parent, child });
    elementStore.appendChild(parent, child);
  },

  appendChild(parent: number, child: number): void {
    console.log("appendChild:", { parent, child });
    elementStore.appendChild(parent, child);
  },

  appendChildToContainer(container: any, child: number): void {
    console.log("appendChildToContainer:", { container, child });
    // Track the first child as the root element
    elementStore.setContainerChild(child);
    // Don't append to a fake element 1 - just track the root
  },

  insertBefore(parent: number, child: number, beforeChild: number): void {
    const parentEl = elementStore.getElement(parent);
    if (!parentEl) return;

    const beforeIndex = parentEl.children.indexOf(beforeChild);
    if (beforeIndex !== -1) {
      parentEl.children.splice(beforeIndex, 0, child);
    } else {
      parentEl.children.push(child);
    }
  },

  insertInContainerBefore(container: any, child: number, beforeChild: number): void {
    // For now, just track that we're inserting before
    console.log("insertInContainerBefore:", { child, beforeChild });
  },

  removeChild(parent: number, child: number): void {
    const parentEl = elementStore.getElement(parent);
    if (!parentEl) return;

    const index = parentEl.children.indexOf(child);
    if (index !== -1) {
      parentEl.children.splice(index, 1);
    }
  },

  removeChildFromContainer(container: any, child: number): void {
    console.log("removeChildFromContainer:", { container, child });
    // The child is being removed from the container - it should be the root element
    // For now, just log it since we don't have a parent-child relationship at container level
  },

  commitUpdate(instance: number, updatePayload: any, type: string, oldProps: any, newProps: any): void {
    const element = elementStore.getElement(instance);
    if (element && newProps && typeof newProps.children === "string") {
      element.text = newProps.children;
    }
  },

  finalizeInitialChildren(instance: number, type: string, props: any): boolean {
    return false;
  },

  prepareUpdate(instance: number, type: string, oldProps: any, newProps: any, rootContainerInstance: any, currentHostContext: null): any {
    return newProps;
  },

  clearContainer(container: any): void {},

  hideInstance(instance: number): void {},

  unhideInstance(instance: number, props: any): void {},

  detachDeletedInstance(instance: number): void {
    console.log("detachDeletedInstance:", { instance });
  },

  prepareScopeUpdate(scopeInstance: any, instance: any): void {},

  getInstanceFromScope(scopeInstance: any): number {
    return 0;
  },

  scheduleTimeout(fn: any, delay?: number): any {
    return setTimeout(fn, delay);
  },

  cancelTimeout(id: number): void {
    clearTimeout(id);
  },

  noTimeout: -1,
};

const reconciler = (ReactReconciler as any).default(config);

export { reconciler };
