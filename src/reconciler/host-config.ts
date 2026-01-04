import * as ReactReconciler from "react-reconciler";
import {ElementStore} from "./element-store";
import {renderFrame, batchElementUpdates} from "./gpui-binding";
import {mapStyleToProps, StyleProps} from "./styles";
import {EventHandler, MouseEvent, Event} from "./events";
import {HostConfig, OpaqueHandle} from "react-reconciler";

// Import ReactContext from react-reconciler namespace
type ReactContext<T> = ReactReconciler.ReactContext<T>;

// Type parameters for HostConfig
type Type = string;              // Element type ("div", "text", etc.)
type Props = any;                // Component props
type Container = ElementStore;            // Root container
type Instance = number;          // Element instance ID
type TextInstance = number;      // Text element instance ID
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
        queueElementUpdate(rootContainer.getElement(id));
        return id;
    },

    commitTextUpdate(textInstance: TextInstance, oldText: string, newText: string): void {
        const element = rootContainer.getElement(textInstance);
        if (element) {
            element.text = String(newText);
            console.log("commitTextUpdate:", {textInstance, newText});
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
        const element = {...styles, eventHandlers};
        const id = rootContainer.createElement(type, undefined, element);
        console.log("createInstance:", {type, id, styles, eventHandlers});
        queueElementUpdate(rootContainer.getElement(id));
        return id;
    },

    appendInitialChild(parentInstance: Instance, child: Instance | TextInstance): void {
        console.log("appendInitialChild:", {parent: parentInstance, child});
        rootContainer.appendChild(parentInstance, child);
        queueElementUpdate(rootContainer.getElement(parentInstance));
    },

    appendChild(parentInstance: Instance, child: Instance | TextInstance): void {
        console.log("appendChild:", {parent: parentInstance, child});
        rootContainer.appendChild(parentInstance, child);
        queueElementUpdate(rootContainer.getElement(parentInstance));
    },

    appendChildToContainer(container: Container, child: Instance | TextInstance): void {
        console.log("appendChildToContainer:", {container, child});
        // Track the first child as the root element
        rootContainer.setContainerChild(child);
        // Don't append to a fake element 1 - just track the root
    },

    insertBefore(
        parentInstance: Instance,
        child: Instance | TextInstance,
        beforeChild: Instance | TextInstance,
    ): void {
        const parentEl = rootContainer.getElement(parentInstance);
        if (!parentEl) return;

        const beforeIndex = parentEl.children.indexOf(beforeChild);
        if (beforeIndex !== -1) {
            parentEl.children.splice(beforeIndex, 0, child);
        } else {
            parentEl.children.push(child);
        }
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
        const parentEl = rootContainer.getElement(parentInstance);
        if (!parentEl) return;

        const index = parentEl.children.indexOf(child);
        if (index !== -1) {
            parentEl.children.splice(index, 1);
        }
    },

    removeChildFromContainer(container: Container, child: Instance | TextInstance): void {
        console.log("removeChildFromContainer:", {container, child});
        // The child is being removed from the container - it should be the root element
        // For now, just log it since we don't have a parent-child relationship at container level
    },

    commitUpdate(
        instance: Instance,
        type: Type,
        prevProps: Props,
        nextProps: Props,
        internalHandle: OpaqueHandle,
    ): void {
        const element = rootContainer.getElement(instance);
        if (element && nextProps && typeof nextProps.children === "string") {
            element.text = nextProps.children;
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

    setCurrentUpdatePriority(newPriority: EventPriority): void {
    },

    getCurrentUpdatePriority(): EventPriority {
        return 0;
    },

    resolveUpdatePriority(): EventPriority {
        return 0;
    },

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
};
