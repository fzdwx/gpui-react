import React from "react";
import { createRoot } from "../src/index";
import { DrawCanvasApp } from "./canvas-draw-app";

const root = createRoot({
    windowOption: {
        width: 1000,
        height: 800,
        title: "Canvas Drawing Demo - gpui-react",
    },
});

root.render(React.createElement(DrawCanvasApp));

console.log("Canvas Drawing Demo - GPUI window should be opening...");
console.log("Keeping process alive...");

setTimeout(() => {
    console.log("Done!");
    process.exit(0);
}, 100000);

process.on("SIGINT", () => {
    console.log("\nShutting down...");
    process.exit(0);
});
