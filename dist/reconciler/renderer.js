import * as React from "react";
import { _render, reconciler } from "./reconciler";
import { init, createWindow } from "./gpui-binding";
import { ElementStore } from "./element-store";
import { AppContext } from "./ctx";
export function createRoot({ width, height, title }) {
    let container = null;
    init();
    const windowId = createWindow(width, height, title);
    console.log("Created window with id:", windowId);
    const elementStore = new ElementStore();
    return {
        render(node) {
            container = _render(React.createElement(AppContext.Provider, { value: { windowId } }, node), elementStore);
        },
        unmount() {
            reconciler.updateContainer(null, container, null, () => {
            });
        },
    };
}
