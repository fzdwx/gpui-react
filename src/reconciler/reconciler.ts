import ReactReconciler from "react-reconciler";
import {hostConfig} from "./host-config";
import React from "react";
import {ConcurrentRoot} from "react-reconciler/constants";
import {ElementData, ElementStore} from "./element-store";

export const reconciler = ReactReconciler(hostConfig)


export function _render(element: React.ReactNode, root: ElementStore) {
    const container = reconciler.createContainer(
        root,
        ConcurrentRoot,
        null,
        false,
        null,
        "",
        console.error,
        console.error,
        console.error,
        console.error,
        null,
    )

    reconciler.updateContainer(element, container, null, () => {})

    return container
}