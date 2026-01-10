import * as ReactReconciler from "react-reconciler";
import { ElementStore } from "./element-store";
import { mapStyleToProps, StyleProps } from "./styles";
import { HostConfig, OpaqueHandle } from "react-reconciler";
import { DefaultEventPriority, NoEventPriority } from "react-reconciler/constants";
import { trace, info, warn } from "../utils/logging";
import { rustLib } from "../core";
import { eventRouter, EVENT_PROP_TO_TYPE, isEventHandlerProp } from "../events";

type ReactContext<T> = ReactReconciler.ReactContext<T>;

type Type = string;
type Props = any;
type Container = ElementStore;
type SuspenseInstance = never;
type HydratableInstance = never;
type FormInstance = never;
type PublicInstance = Element;
type HostContext = object;
type ChildSet = never;
type TimeoutHandle = number;
type NoTimeout = -1;
type TransitionStatus = any;
type EventPriority = number;

export interface Instance {
    id: number;
    type: string;
    text?: string;
    children: Instance[];
    style?: Record<string, any>;
    eventHandlers?: Record<string, number>;
    store: ElementStore;
}

export interface TextInstance {
    id: number;
    type: "text";
    text: string;
    children: [];
    store: ElementStore;
}

const pendingUpdates: any[] = [];

function extractStyleProps(props: any): StyleProps {
    const styleProps: StyleProps = {};

    if (props.style) {
        Object.assign(styleProps, props.style);
    }

    if (props.className) {
        warn("className not yet supported, use style prop instead");
    }

    const canvasProps = ["x", "y", "color", "src", "text", "textSize", "textColor", "drawCommands"];
    for (const prop of canvasProps) {
        if (props[prop] !== undefined) {
            (styleProps as any)[prop] = props[prop];
        }
    }

    for (const propName of Object.keys(props)) {
        if (isEventHandlerProp(propName) && typeof props[propName] === "function") {
            (styleProps as any)[propName] = props[propName];
        }
    }

    if (props.tabIndex !== undefined) {
        styleProps.tabIndex = props.tabIndex;
    }

    return styleProps;
}

function extractEventHandlers(props: any): Record<string, number> {
    const handlers: Record<string, number> = {};

    for (const propName of Object.keys(props)) {
        if (isEventHandlerProp(propName) && typeof props[propName] === "function") {
            const handlerId = eventRouter.registerHandler(props[propName]);
            handlers[propName] = handlerId;
        }
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

    getPublicInstance(_instance: Instance | TextInstance): PublicInstance {
        return document.createElement("div");
    },

    getRootHostContext(_rootContainer: Container): HostContext | null {
        return {};
    },

    getChildHostContext(
        _parentHostContext: HostContext,
        _type: Type,
        _rootContainer: Container
    ): HostContext {
        return {};
    },

    prepareForCommit(_containerInfo: Container): Record<string, any> | null {
        return {};
    },

    resetAfterCommit(containerInfo: Container): void {
        if (pendingUpdates.length > 0) {
            info(`Processing ${pendingUpdates.length} batched updates`);
            rustLib.batchElementUpdates(containerInfo.getWindowId(), pendingUpdates);
            pendingUpdates.length = 0;
        }

        const root = containerInfo.getRoot();
        trace("resetAfterCommit - root element", root);
        rustLib.renderFrame(containerInfo.getWindowId(), root);

        setImmediate(() => {
            if (typeof window !== "undefined" && (window as any).__gpuiTrigger) {
                (window as any).__gpuiTrigger();
            }
        });
    },

    shouldSetTextContent(_type: Type, _props: Props): boolean {
        return false;
    },

    resetTextContent(_instance: Instance): void {},

    createTextInstance(
        text: string,
        rootContainer: Container,
        _hostContext: HostContext,
        _internalHandle: OpaqueHandle
    ): TextInstance {
        const styleProps = extractStyleProps({ style: {} });
        const styles = mapStyleToProps(styleProps);
        const id = rootContainer.createElement("text", String(text), styles);
        trace("createTextInstance", { text, id, styles });

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
        trace("commitTextUpdate", { oldText, newText });
        const element = textInstance.store.getElement(textInstance.id);
        if (element) {
            element.text = String(newText);
            queueElementUpdate(element);
        }
    },

    createInstance(
        type: Type,
        props: Props,
        rootContainer: Container,
        _hostContext: HostContext,
        _internalHandle: OpaqueHandle
    ): Instance {
        if (type === "canvas") {
            console.log(
                "createInstance canvas: drawCommands=" +
                    (props.drawCommands ? props.drawCommands.substring(0, 100) : "UNDEFINED")
            );
        }

        const styleProps = extractStyleProps(props);
        const styles = mapStyleToProps(styleProps);
        const eventHandlers = extractEventHandlers(props);
        const id = rootContainer.createElement(type, undefined, styles, eventHandlers);
        trace("createInstance", { type, id, styles, eventHandlers });

        // Bind event handlers to the element
        for (const [propName, handlerId] of Object.entries(eventHandlers || {})) {
            const eventType = EVENT_PROP_TO_TYPE[propName as keyof typeof EVENT_PROP_TO_TYPE];
            if (eventType) {
                eventRouter.bindEvent(id, eventType, handlerId);
            }
        }

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
        trace("appendInitialChild", { parent: parentInstance, child });
        const childInstance = child as Instance;
        parentInstance.children.push(childInstance);
        parentInstance.store.appendChild(parentInstance.id, childInstance.id);
        // Track parent-child relationship for event bubbling
        eventRouter.setParent(childInstance.id, parentInstance.id);
        queueElementUpdate(parentInstance.store.getElement(parentInstance.id));
    },

    appendChild(parentInstance: Instance, child: Instance | TextInstance): void {
        trace("appendChild", { parent: parentInstance, child });
        const childInstance = child as Instance;
        parentInstance.children.push(childInstance);
        parentInstance.store.appendChild(parentInstance.id, childInstance.id);
        // Track parent-child relationship for event bubbling
        eventRouter.setParent(childInstance.id, parentInstance.id);
        queueElementUpdate(parentInstance.store.getElement(parentInstance.id));
    },

    appendChildToContainer(container: Container, child: Instance | TextInstance): void {
        trace("appendChildToContainer", { container, child });
        const childInstance = child as Instance;
        container.setContainerChild(childInstance.id);
        queueElementUpdate(container.getElement(childInstance.id));
    },

    insertBefore(
        parentInstance: Instance,
        child: Instance | TextInstance,
        beforeChild: Instance | TextInstance
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
        _container: Container,
        child: Instance | TextInstance,
        beforeChild: Instance | TextInstance
    ): void {
        trace("insertInContainerBefore", { child, beforeChild });
    },

    removeChild(parentInstance: Instance, child: Instance | TextInstance): void {
        const childInstance = child as Instance;
        const index = parentInstance.children.indexOf(childInstance);
        if (index !== -1) {
            parentInstance.children.splice(index, 1);
        }
        parentInstance.store.removeChild(parentInstance.id, childInstance.id);
        // Clean up event handlers for the removed element
        eventRouter.cleanupElement(childInstance.id);
        queueElementUpdate(parentInstance.store.getElement(parentInstance.id));
    },

    removeChildFromContainer(_container: Container, child: Instance | TextInstance): void {
        trace("removeChildFromContainer", { container: _container, child });
        const childInstance = child as Instance;
        _container.removeChild(_container.getRoot().globalId, childInstance.id);
        // Clean up event handlers for the removed element
        eventRouter.cleanupElement(childInstance.id);
    },

    commitUpdate(
        instance: Instance,
        _type: Type,
        _prevProps: Props,
        nextProps: Props,
        _internalHandle: OpaqueHandle
    ): void {
        let needsUpdate = false;

        // Update text content
        if (nextProps && typeof nextProps.children === "string") {
            instance.text = String(nextProps.children);
            needsUpdate = true;
        }

        // Clean up old handlers and register new ones
        const oldHandlers = instance.eventHandlers || {};
        const newHandlers = extractEventHandlers(nextProps);

        // Remove handlers that no longer exist
        for (const [propName, handlerId] of Object.entries(oldHandlers)) {
            if (!(propName in newHandlers)) {
                const eventType = EVENT_PROP_TO_TYPE[propName as keyof typeof EVENT_PROP_TO_TYPE];
                if (eventType) {
                    eventRouter.unbindEvent(instance.id, eventType, handlerId);
                    eventRouter.unregisterHandler(handlerId);
                }
            }
        }

        // Bind new handlers (this will update existing ones due to the closure re-creation)
        for (const [propName, handlerId] of Object.entries(newHandlers)) {
            const eventType = EVENT_PROP_TO_TYPE[propName as keyof typeof EVENT_PROP_TO_TYPE];
            if (eventType) {
                // Unbind old handler for this event type first
                const oldHandlerId = oldHandlers[propName];
                if (oldHandlerId !== undefined) {
                    eventRouter.unbindEvent(instance.id, eventType, oldHandlerId);
                    eventRouter.unregisterHandler(oldHandlerId);
                }
                eventRouter.bindEvent(instance.id, eventType, handlerId);
            }
        }
        instance.eventHandlers = newHandlers;

        if (needsUpdate) {
            const element = instance.store.getElement(instance.id);
            if (element) {
                queueElementUpdate(element);
            }
        }
    },

    finalizeInitialChildren(
        _instance: Instance,
        _type: Type,
        _props: Props,
        _rootContainer: Container,
        _hostContext: HostContext
    ): boolean {
        return false;
    },

    clearContainer(_container: Container): void {},

    hideInstance(_instance: Instance): void {},

    unhideInstance(_instance: Instance, _props: Props): void {},

    detachDeletedInstance(instance: Instance): void {
        trace("detachDeletedInstance", { instance });
        // Recursively clean up event handlers for the instance and all descendants
        const collectChildIds = (inst: Instance): number[] => {
            const ids: number[] = [];
            for (const child of inst.children) {
                ids.push(child.id);
                ids.push(...collectChildIds(child));
            }
            return ids;
        };
        const childIds = collectChildIds(instance);
        eventRouter.cleanupElementTree(instance.id, childIds);
    },

    prepareScopeUpdate(_scopeInstance: any, _instance: any): void {},

    getInstanceFromScope(_scopeInstance: any): Instance | null {
        return null;
    },

    scheduleTimeout(fn: (...args: unknown[]) => unknown, delay?: number): TimeoutHandle {
        return setTimeout(fn, delay) as unknown as number;
    },

    cancelTimeout(id: TimeoutHandle): void {
        clearTimeout(id);
    },

    noTimeout: -1,

    preparePortalMount(_containerInfo: Container): void {},

    getInstanceFromNode(_node: any): null {
        return null;
    },

    beforeActiveInstanceBlur(): void {},

    afterActiveInstanceBlur(): void {},

    NotPendingTransition: null,

    HostTransitionContext: null as any as ReactContext<TransitionStatus>,

    resetFormInstance(_form: FormInstance): void {},

    requestPostPaintCallback(_callback: (time: number) => void): void {},

    shouldAttemptEagerTransition(): boolean {
        return false;
    },

    trackSchedulerEvent(): void {},

    resolveEventType(): null | string {
        return null;
    },

    resolveEventTimeStamp(): number {
        return Date.now();
    },

    maySuspendCommit(_type: Type, _props: Props): boolean {
        return false;
    },

    preloadInstance(_type: Type, _props: Props): boolean {
        return false;
    },

    startSuspendingCommit(): void {},

    suspendInstance(_type: Type, _props: Props): void {},

    waitForCommitToBeReady():
        | ((initiateCommit: (...args: unknown[]) => unknown) => (...args: unknown[]) => unknown)
        | null {
        return null;
    },

    setCurrentUpdatePriority(newPriority: number) {
        currentUpdatePriority = newPriority;
    },

    getCurrentUpdatePriority: () => currentUpdatePriority,

    resolveUpdatePriority() {
        if (currentUpdatePriority !== NoEventPriority) {
            return currentUpdatePriority;
        }

        return DefaultEventPriority;
    },
};
