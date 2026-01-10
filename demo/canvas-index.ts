import React from "react";
import { createRoot } from "../src/index";
import { CanvasApp } from "./canvas-app";

const root = createRoot({
    windowOption: {
        width: 900,
        height: 800,
        title: "Canvas Demo - gpui-react",
    },
});

root.render(React.createElement(CanvasApp));

console.log("Canvas Demo - GPUI window should be opening...");
console.log("Keeping process alive...");

setTimeout(() => {
    console.log("Done!");
    process.exit(0);
}, 100000);

process.on("SIGINT", () => {
    console.log("\nShutting down...");
    process.exit(0);
});
