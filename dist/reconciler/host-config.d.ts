import { ElementStore } from "./element-store";
import { HostConfig } from "react-reconciler";
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
export declare const hostConfig: HostConfig<Type, Props, Container, Instance, TextInstance, SuspenseInstance, HydratableInstance, FormInstance, PublicInstance, HostContext, ChildSet, TimeoutHandle, NoTimeout, TransitionStatus>;
export {};
//# sourceMappingURL=host-config.d.ts.map