import React from "react";
import { createRoot } from "../src/index";
import { CanvasCommandDemo } from "./canvas-debug-app";

const root = createRoot({
    windowOption: {
        width: 800,
        height: 600,
        title: "Canvas Debug - gpui-react",
    },
});

root.render(React.createElement(CanvasCommandDemo));

setTimeout(() => {
    process.exit(0);
}, 100000);

process.on("SIGINT", () => {
    process.exit(0);
});
