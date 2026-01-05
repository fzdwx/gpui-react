import * as React from "react";
import { _render, reconciler } from "./reconciler";
import { ElementStore } from "./element-store";
import { AppContext } from "./ctx";
import { rustLib } from "../core";

export type Root = {
    render: (children: React.ReactNode) => void;
    unmount: () => void;
};

export type RootProps = {
    width: number;
    height: number;
    title?: string;
};

export function createRoot({ width, height, title }: RootProps): Root {
    let container: null = null;
    const windowId = rustLib.createWindow(width, height, title);
    console.log("Created window with id:", windowId);

    const elementStore = new ElementStore();
    elementStore.setWindowId(windowId);
    return {
        render(node: React.ReactNode) {
            container = _render(
                React.createElement(AppContext.Provider, { value: { windowId } }, node),
                elementStore
            );
        },
        unmount() {
            reconciler.updateContainer(null, container, null, () => {});
        },
    };
}
