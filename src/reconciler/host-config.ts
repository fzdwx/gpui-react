import * as ReactReconciler from "react-reconciler";
import {ElementStore, ElementData} from "./element-store";
import {renderFrame, batchElementUpdates} from "./gpui-binding";
import {mapStyleToProps, StyleProps} from "./styles";
import {EventHandler, MouseEvent, Event} from "./events";
import {HostConfig, OpaqueHandle} from "react-reconciler";
import {DefaultEventPriority, NoEventPriority} from "react-reconciler/constants";

// Import ReactContext from react-reconciler namespace
type ReactContext<T> = ReactReconciler.ReactContext<T>;

// Type parameters for HostConfig
type Type = string;              // Element type ("div", "text", etc.)
type Props = any;                // Component props
type Container = ElementStore;   // Root container
type SuspenseInstance = never;   // Not supported
type HydratableInstance = never; // Not supported
type FormInstance = never;       // Not supported
type PublicInstance = Element;   // Public instance (DOM Element for compatibility)
type HostContext = null;         // No context needed
type ChildSet = never;           // Not using persistent mode
type TimeoutHandle = number;     // setTimeout return type
type NoTimeout = -1;             // Invalid timeout constant
type TransitionStatus = any;     // Transition status
type EventPriority = number;     // Event priority type

// Instance type - self-contained object with store reference
export interface Instance {
    id: number;
    type: string;
    text?: string;
    children: Instance[];
    style?: Record<string, any>;
    eventHandlers?: Record<string, number>;
    store: ElementStore;
}

// TextInstance type - same structure for text nodes
export interface TextInstance {
    id: number;
    type: "text";
    text: string;
    children: [];
    store: ElementStore;
}

let nextEventId = 0;
const eventHandlers = new Map<number, EventHandler>();
const pendingUpdates: any[] = [];

function registerEventHandler(handler: EventHandler): number {
    const id = nextEventId++;
    eventHandlers.set(id, handler);
    return id;
}

function extractStyleProps(props: any): StyleProps {
    const styleProps: StyleProps = {};

    if (props.style) {
        Object.assign(styleProps, props.style);
    }

    if (props.className) {
        console.warn("className not yet supported, use style prop instead");
    }

    if (props.onClick) {
        styleProps.onClick = props.onClick;
    }

    if (props.onHover) {
        styleProps.onHover = props.onHover;
    }

    if (props.onMouseEnter) {
        styleProps.onMouseEnter = props.onMouseEnter;
    }

    if (props.onMouseLeave) {
        styleProps.onMouseLeave = props.onMouseLeave;
    }

    return styleProps;
}

function extractEventHandlers(props: any): Record<string, number> {
    const handlers: Record<string, number> = {};

    if (props.onClick) {
        handlers['onClick'] = registerEventHandler(props.onClick);
    }

    if (props.onHover) {
        handlers['onHover'] = registerEventHandler(props.onHover);
    }

    if (props.onMouseEnter) {
        handlers['onMouseEnter'] = registerEventHandler(props.onMouseEnter);
    }

    if (props.onMouseLeave) {
        handlers['onMouseLeave'] = registerEventHandler(props.onMouseLeave);
    }

    return handlers;
}

function queueElementUpdate(element: any): void {
    if (!element) return;

    const existingIndex = pendingUpdates.findIndex((e) => e.globalId === element.globalId);

    if (existingIndex !== -1) {
        pendingUpdates[existingIndex] = element;
    } else {
        pendingUpdates.push(element);
    }
}

let currentUpdatePriority = 0;
export const hostConfig: HostConfig<
    Type,
    Props,
    Container,
    Instance,
    TextInstance,
    SuspenseInstance,
    HydratableInstance,
    FormInstance,
    PublicInstance,
    HostContext,
    ChildSet,
    TimeoutHandle,
    NoTimeout,
    TransitionStatus
> = {
    supportsMutation: true,
    supportsPersistence: false,
    supportsHydration: false,
    isPrimaryRenderer: true,

    getPublicInstance(instance: Instance | TextInstance): PublicInstance {
        return document.createElement("div");
    },

    getRootHostContext(rootContainer: Container): HostContext | null {
        return null;
    },

    getChildHostContext(
        parentHostContext: HostContext,
        type: Type,
        rootContainer: Container,
    ): HostContext {
        return null;
    },

    prepareForCommit(containerInfo: Container): Record<string, any> | null {
        return null;
    },

    resetAfterCommit(containerInfo: Container): void {
        if (pendingUpdates.length > 0) {
            console.log(`=== Processing ${pendingUpdates.length} batched updates ===`);
            batchElementUpdates(pendingUpdates);
            pendingUpdates.length = 0;
        }

        const root = containerInfo.getRoot();
        console.log("resetAfterCommit - root element:", JSON.stringify(root, null, 2));
        renderFrame(root);
    },

    shouldSetTextContent(type: Type, props: Props): boolean {
        return false; // Force React to call createTextInstance for text children
    },

    resetTextContent(instance: Instance): void {
    },

    createTextInstance(
        text: string,
        rootContainer: Container,
        hostContext: HostContext,
        internalHandle: OpaqueHandle,
    ): TextInstance {
        const styleProps = extractStyleProps({style: {}});
        const styles = mapStyleToProps(styleProps);
        const id = rootContainer.createElement("text", String(text), styles);
        console.log("createTextInstance:", {text, id, styles});

        // Return self-contained TextInstance object with store reference
        const instance: TextInstance = {
            id,
            type: "text",
            text: String(text),
            children: [],
            store: rootContainer,
        };
        queueElementUpdate(rootContainer.getElement(id));
        return instance;
    },

    commitTextUpdate(textInstance: TextInstance, oldText: string, newText: string): void {
        textInstance.text = String(newText);
        console.log("commitTextUpdate:", {textInstance, newText});
        const element = textInstance.store.getElement(textInstance.id);
        if (element) {
            queueElementUpdate(element);
        }
    },

    createInstance(
        type: Type,
        props: Props,
        rootContainer: Container,
        hostContext: HostContext,
        internalHandle: OpaqueHandle,
    ): Instance {
        const styleProps = extractStyleProps(props);
        const styles = mapStyleToProps(styleProps);
        const eventHandlers = extractEventHandlers(props);
        const id = rootContainer.createElement(type, undefined, {...styles, eventHandlers});
        console.log("createInstance:", {type, id, styles, eventHandlers});

        // Return self-contained Instance object with store reference
        const instance: Instance = {
            id,
            type,
            children: [],
            style: styles,
            eventHandlers,
            store: rootContainer,
        };
        queueElementUpdate(rootContainer.getElement(id));
        return instance;
    },

    appendInitialChild(parentInstance: Instance, child: Instance | TextInstance): void {
        console.log("appendInitialChild:", {parent: parentInstance, child});
        // Add to local children array
        parentInstance.children.push(child as Instance);
        // Also update the store
        parentInstance.store.appendChild(parentInstance.id, (child as Instance).id);
        queueElementUpdate(parentInstance.store.getElement(parentInstance.id));
    },

    appendChild(parentInstance: Instance, child: Instance | TextInstance): void {
        console.log("appendChild:", {parent: parentInstance, child});
        // Add to local children array
        parentInstance.children.push(child as Instance);
        // Also update the store
        parentInstance.store.appendChild(parentInstance.id, (child as Instance).id);
        queueElementUpdate(parentInstance.store.getElement(parentInstance.id));
    },

    appendChildToContainer(container: Container, child: Instance | TextInstance): void {
        console.log("appendChildToContainer:", {container, child});
        // Track the first child as the root element
        container.setContainerChild((child as Instance).id);
    },

    insertBefore(
        parentInstance: Instance,
        child: Instance | TextInstance,
        beforeChild: Instance | TextInstance,
    ): void {
        const childInstance = child as Instance;
        const beforeChildInstance = beforeChild as Instance;

        const beforeIndex = parentInstance.children.indexOf(beforeChildInstance);
        if (beforeIndex !== -1) {
            parentInstance.children.splice(beforeIndex, 0, childInstance);
        } else {
            parentInstance.children.push(childInstance);
        }
        queueElementUpdate(parentInstance.store.getElement(parentInstance.id));
    },

    insertInContainerBefore(
        container: Container,
        child: Instance | TextInstance,
        beforeChild: Instance | TextInstance,
    ): void {
        // For now, just track that we're inserting before
        console.log("insertInContainerBefore:", {child, beforeChild});
    },

    removeChild(parentInstance: Instance, child: Instance | TextInstance): void {
        const childInstance = child as Instance;
        const index = parentInstance.children.indexOf(childInstance);
        if (index !== -1) {
            parentInstance.children.splice(index, 1);
        }
        parentInstance.store.removeChild(parentInstance.id, childInstance.id);
        queueElementUpdate(parentInstance.store.getElement(parentInstance.id));
    },

    removeChildFromContainer(container: Container, child: Instance | TextInstance): void {
        console.log("removeChildFromContainer:", {container, child});
        const childInstance = child as Instance;
        container.removeChild(container.getRoot().globalId, childInstance.id);
    },

    commitUpdate(
        instance: Instance,
        type: Type,
        prevProps: Props,
        nextProps: Props,
        internalHandle: OpaqueHandle,
    ): void {
        if (nextProps && typeof nextProps.children === "string") {
            instance.text = String(nextProps.children);
            const element = instance.store.getElement(instance.id);
            if (element) {
                queueElementUpdate(element);
            }
        }
    },

    finalizeInitialChildren(
        instance: Instance,
        type: Type,
        props: Props,
        rootContainer: Container,
        hostContext: HostContext,
    ): boolean {
        return false;
    },

    clearContainer(container: Container): void {
    },

    hideInstance(instance: Instance): void {
    },

    unhideInstance(instance: Instance, props: Props): void {
    },

    detachDeletedInstance(instance: Instance): void {
        console.log("detachDeletedInstance:", {instance});
    },

    prepareScopeUpdate(scopeInstance: any, instance: any): void {
    },

    getInstanceFromScope(scopeInstance: any): Instance | null {
        return null;
    },

    scheduleTimeout(fn: (...args: unknown[]) => unknown, delay?: number): TimeoutHandle {
        return setTimeout(fn, delay) as unknown as number;
    },

    cancelTimeout(id: TimeoutHandle): void {
        clearTimeout(id);
    },

    noTimeout: -1,

    preparePortalMount(containerInfo: Container): void {
    },

    getInstanceFromNode(node: any): null {
        return null;
    },

    beforeActiveInstanceBlur(): void {
    },

    afterActiveInstanceBlur(): void {
    },

    NotPendingTransition: null,

    HostTransitionContext: null as any as ReactContext<TransitionStatus>,

    resetFormInstance(form: FormInstance): void {
    },

    requestPostPaintCallback(callback: (time: number) => void): void {
    },

    shouldAttemptEagerTransition(): boolean {
        return false;
    },

    trackSchedulerEvent(): void {
    },

    resolveEventType(): null | string {
        return null;
    },

    resolveEventTimeStamp(): number {
        return Date.now();
    },

    maySuspendCommit(type: Type, props: Props): boolean {
        return false;
    },

    preloadInstance(type: Type, props: Props): boolean {
        return false;
    },

    startSuspendingCommit(): void {
    },

    suspendInstance(type: Type, props: Props): void {
    },

    waitForCommitToBeReady():
        | ((initiateCommit: (...args: unknown[]) => unknown) => (...args: unknown[]) => unknown)
        | null {
        return null;
    },

    setCurrentUpdatePriority(newPriority: number) {
        currentUpdatePriority = newPriority
    },

    getCurrentUpdatePriority: () => currentUpdatePriority,

    resolveUpdatePriority() {
        if (currentUpdatePriority !== NoEventPriority) {
            return currentUpdatePriority
        }

        return DefaultEventPriority
    },
}
