import * as React from "react";
import {_render, reconciler} from "./reconciler";
import {ElementStore} from "./element-store";
import {AppContext} from "./ctx";
import {rustLib, WindowOptions} from "../core";

export type Root = {
    render: (children: React.ReactNode) => void;
    unmount: () => void;
};

export type RootProps = {
    windowOption: WindowOptions
};

export function createRoot(props: RootProps): Root {
    let container: null = null;
    const windowId = rustLib.createWindow(props.windowOption);
    console.log("Created window with id:", windowId);

    const elementStore = new ElementStore();
    elementStore.setWindowId(windowId);
    return {
        render(node: React.ReactNode) {
            container = _render(
                React.createElement(AppContext.Provider, {value: {windowId}}, node),
                elementStore
            );
        },
        unmount() {
            reconciler.updateContainer(null, container, null, () => {
            });
        },
    };
}
