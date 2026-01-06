import React from "react";
import { createRoot } from "../src/index";
import { EventApp } from "./event-app";

const root = createRoot({
    windowOption: {
        width: 800,
        height: 800,
    },
});
root.render(
    React.createElement(
        "div",
        { style: { backgroundColor: "#1e1e1e", padding: 40 } },
        React.createElement(EventApp)
    )
);

console.log("Event demo running...");
console.log("Click the button to see console logs");

setTimeout(() => {
    console.log("Done! The event window should be visible.");
    process.exit(0);
}, 10000);

process.on("SIGINT", () => {
    console.log("\nShutting down...");
    process.exit(0);
});
