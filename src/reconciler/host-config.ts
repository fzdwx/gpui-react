import * as ReactReconciler from "react-reconciler";
import { ElementStore } from "./element-store";
import { mapStyleToProps, StyleProps } from "./styles";
import { HostConfig, OpaqueHandle } from "react-reconciler";
import { DefaultEventPriority, NoEventPriority } from "react-reconciler/constants";
import { trace, debug, info, warn } from "./utils/logging";
import { rustLib } from "../core";
import {bindEventToElement, registerEventHandler} from "./event-router";

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
        handlers["onClick"] = registerEventHandler(props.onClick);
    }

    if (props.onHover) {
        handlers["onHover"] = registerEventHandler(props.onHover);
    }

    if (props.onMouseEnter) {
        handlers["onMouseEnter"] = registerEventHandler(props.onMouseEnter);
    }

    if (props.onMouseLeave) {
        handlers["onMouseLeave"] = registerEventHandler(props.onMouseLeave);
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
        const styleProps = extractStyleProps(props);
        const styles = mapStyleToProps(styleProps);
        const eventHandlers = extractEventHandlers(props);
        const id = rootContainer.createElement(type, undefined, styles, eventHandlers);
        trace("createInstance", { type, id, styles, eventHandlers });

        for (const [eventType, handlerId] of Object.entries(eventHandlers || {})) {
            bindEventToElement(id, eventType, handlerId);
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
        parentInstance.children.push(child as Instance);
        parentInstance.store.appendChild(parentInstance.id, (child as Instance).id);
        queueElementUpdate(parentInstance.store.getElement(parentInstance.id));
    },

    appendChild(parentInstance: Instance, child: Instance | TextInstance): void {
        trace("appendChild", { parent: parentInstance, child });
        parentInstance.children.push(child as Instance);
        parentInstance.store.appendChild(parentInstance.id, (child as Instance).id);
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
        queueElementUpdate(parentInstance.store.getElement(parentInstance.id));
    },

    removeChildFromContainer(_container: Container, child: Instance | TextInstance): void {
        trace("removeChildFromContainer", { container: _container, child });
        const childInstance = child as Instance;
        _container.removeChild(_container.getRoot().globalId, childInstance.id);
    },

    commitUpdate(
        instance: Instance,
        _type: Type,
        _prevProps: Props,
        nextProps: Props,
        _internalHandle: OpaqueHandle
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
