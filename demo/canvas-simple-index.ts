import React from "react";
import { createRoot } from "../src/index";
import { CanvasSimpleDemo } from "./canvas-simple-app";

const root = createRoot({
    windowOption: {
        width: 520,
        height: 700,
        title: "Canvas Drawing Demo - gpui-react",
    },
});

root.render(React.createElement(CanvasSimpleDemo));

setTimeout(() => {
    process.exit(0);
}, 10000);

process.on("SIGINT", () => {
    process.exit(0);
});
