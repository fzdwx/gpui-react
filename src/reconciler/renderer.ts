import * as React from "react";
import {_render, reconciler} from "./reconciler"
import {init, createWindow} from "./gpui-binding";
import {ElementStore} from "./element-store";
import {AppContext} from "./ctx";

export type Root = {
    render: (children: React.ReactNode) => void;
    unmount: () => void;
};

export function createRoot(): Root {
    let container: null = null
    init(800, 600);
    const windowId = createWindow(800, 600);
    console.log("Created window with id:", windowId);

    const elementStore = new ElementStore();
    return {
        render(node: React.ReactNode) {
            container = _render(
                React.createElement(
                    AppContext.Provider,
                    {value: {windowId}},
                    node,
                ),
                elementStore,
            )
        },
        unmount() {
            reconciler.updateContainer(null, container, null, () => {
            })
        },
    };
}
