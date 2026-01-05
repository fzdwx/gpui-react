import { renderFrame, batchElementUpdates, currentWindowId } from "./gpui-binding";
import { mapStyleToProps } from "./styles";
import { DefaultEventPriority, NoEventPriority } from "react-reconciler/constants";
import { trace, info, warn } from "./logging";
let nextEventId = 0;
const eventHandlers = new Map();
const pendingUpdates = [];
function registerEventHandler(handler) {
    const id = nextEventId++;
    eventHandlers.set(id, handler);
    return id;
}
function extractStyleProps(props) {
    const styleProps = {};
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
function extractEventHandlers(props) {
    const handlers = {};
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
function queueElementUpdate(element) {
    if (!element)
        return;
    const existingIndex = pendingUpdates.findIndex((e) => e.globalId === element.globalId);
    if (existingIndex !== -1) {
        pendingUpdates[existingIndex] = element;
    }
    else {
        pendingUpdates.push(element);
    }
}
let currentUpdatePriority = 0;
export const hostConfig = {
    supportsMutation: true,
    supportsPersistence: false,
    supportsHydration: false,
    isPrimaryRenderer: true,
    getPublicInstance(_instance) {
        return document.createElement("div");
    },
    getRootHostContext(_rootContainer) {
        return {};
    },
    getChildHostContext(_parentHostContext, _type, _rootContainer) {
        return {};
    },
    prepareForCommit(_containerInfo) {
        return {};
    },
    resetAfterCommit(containerInfo) {
        if (pendingUpdates.length > 0) {
            info(`Processing ${pendingUpdates.length} batched updates`);
            batchElementUpdates(currentWindowId, pendingUpdates);
            pendingUpdates.length = 0;
        }
        const root = containerInfo.getRoot();
        trace("resetAfterCommit - root element", root);
        renderFrame(currentWindowId, root);
        setImmediate(() => {
            if (typeof window !== 'undefined' && window.__gpuiTrigger) {
                window.__gpuiTrigger();
            }
        });
    },
    shouldSetTextContent(_type, _props) {
        return false;
    },
    resetTextContent(_instance) {
    },
    createTextInstance(text, rootContainer, _hostContext, _internalHandle) {
        const styleProps = extractStyleProps({ style: {} });
        const styles = mapStyleToProps(styleProps);
        const id = rootContainer.createElement("text", String(text), styles);
        trace("createTextInstance", { text, id, styles });
        const instance = {
            id,
            type: "text",
            text: String(text),
            children: [],
            store: rootContainer,
        };
        queueElementUpdate(rootContainer.getElement(id));
        return instance;
    },
    commitTextUpdate(textInstance, oldText, newText) {
        textInstance.text = String(newText);
        trace("commitTextUpdate", { oldText, newText });
        const element = textInstance.store.getElement(textInstance.id);
        if (element) {
            element.text = String(newText);
            queueElementUpdate(element);
        }
    },
    createInstance(type, props, rootContainer, _hostContext, _internalHandle) {
        const styleProps = extractStyleProps(props);
        const styles = mapStyleToProps(styleProps);
        const eventHandlers = extractEventHandlers(props);
        const id = rootContainer.createElement(type, undefined, { ...styles, eventHandlers });
        trace("createInstance", { type, id, styles, eventHandlers });
        const instance = {
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
    appendInitialChild(parentInstance, child) {
        trace("appendInitialChild", { parent: parentInstance, child });
        parentInstance.children.push(child);
        parentInstance.store.appendChild(parentInstance.id, child.id);
        queueElementUpdate(parentInstance.store.getElement(parentInstance.id));
    },
    appendChild(parentInstance, child) {
        trace("appendChild", { parent: parentInstance, child });
        parentInstance.children.push(child);
        parentInstance.store.appendChild(parentInstance.id, child.id);
        queueElementUpdate(parentInstance.store.getElement(parentInstance.id));
    },
    appendChildToContainer(container, child) {
        trace("appendChildToContainer", { container, child });
        const childInstance = child;
        container.setContainerChild(childInstance.id);
        queueElementUpdate(container.getElement(childInstance.id));
    },
    insertBefore(parentInstance, child, beforeChild) {
        const childInstance = child;
        const beforeChildInstance = beforeChild;
        const beforeIndex = parentInstance.children.indexOf(beforeChildInstance);
        if (beforeIndex !== -1) {
            parentInstance.children.splice(beforeIndex, 0, childInstance);
        }
        else {
            parentInstance.children.push(childInstance);
        }
        queueElementUpdate(parentInstance.store.getElement(parentInstance.id));
    },
    insertInContainerBefore(_container, child, beforeChild) {
        trace("insertInContainerBefore", { child, beforeChild });
    },
    removeChild(parentInstance, child) {
        const childInstance = child;
        const index = parentInstance.children.indexOf(childInstance);
        if (index !== -1) {
            parentInstance.children.splice(index, 1);
        }
        parentInstance.store.removeChild(parentInstance.id, childInstance.id);
        queueElementUpdate(parentInstance.store.getElement(parentInstance.id));
    },
    removeChildFromContainer(_container, child) {
        trace("removeChildFromContainer", { container: _container, child });
        const childInstance = child;
        _container.removeChild(_container.getRoot().globalId, childInstance.id);
    },
    commitUpdate(instance, _type, _prevProps, nextProps, _internalHandle) {
        if (nextProps && typeof nextProps.children === "string") {
            instance.text = String(nextProps.children);
            const element = instance.store.getElement(instance.id);
            if (element) {
                queueElementUpdate(element);
            }
        }
    },
    finalizeInitialChildren(_instance, _type, _props, _rootContainer, _hostContext) {
        return false;
    },
    clearContainer(_container) {
    },
    hideInstance(_instance) {
    },
    unhideInstance(_instance, _props) {
    },
    detachDeletedInstance(instance) {
        trace("detachDeletedInstance", { instance });
    },
    prepareScopeUpdate(_scopeInstance, _instance) {
    },
    getInstanceFromScope(_scopeInstance) {
        return null;
    },
    scheduleTimeout(fn, delay) {
        return setTimeout(fn, delay);
    },
    cancelTimeout(id) {
        clearTimeout(id);
    },
    noTimeout: -1,
    preparePortalMount(_containerInfo) {
    },
    getInstanceFromNode(_node) {
        return null;
    },
    beforeActiveInstanceBlur() {
    },
    afterActiveInstanceBlur() {
    },
    NotPendingTransition: null,
    HostTransitionContext: null,
    resetFormInstance(_form) {
    },
    requestPostPaintCallback(_callback) {
    },
    shouldAttemptEagerTransition() {
        return false;
    },
    trackSchedulerEvent() {
    },
    resolveEventType() {
        return null;
    },
    resolveEventTimeStamp() {
        return Date.now();
    },
    maySuspendCommit(_type, _props) {
        return false;
    },
    preloadInstance(_type, _props) {
        return false;
    },
    startSuspendingCommit() {
    },
    suspendInstance(_type, _props) {
    },
    waitForCommitToBeReady() {
        return null;
    },
    setCurrentUpdatePriority(newPriority) {
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
