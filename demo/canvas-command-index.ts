import React from "react";
import { createRoot } from "../src/index";
import { CanvasCommandDemo } from "./canvas-command-app";

const root = createRoot({
    windowOption: {
        width: 1000,
        height: 800,
        title: "Canvas Command API Demo - gpui-react",
    },
});

root.render(React.createElement(CanvasCommandDemo));

console.log("Canvas Command API Demo - GPUI window should be opening...");

setTimeout(() => {
    console.log("Done!");
    process.exit(0);
}, 100000);

process.on("SIGINT", () => {
    console.log("\nShutting down...");
    process.exit(0);
});
