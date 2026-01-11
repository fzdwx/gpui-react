import React from "react";
import { createRoot } from "../../src/index";
import { InputApp } from "./app";

const root = createRoot({
    windowOption: {
        width: 700,
        height: 700,
    },
});
root.render(
    React.createElement(
        "div",
        { style: { backgroundColor: "#1e1e1e", padding: 20 } },
        React.createElement(InputApp)
    )
);

console.log("Input demo running...");
console.log("Click on inputs and type to test text input functionality");

setTimeout(() => {
    console.log("Done! The input window should be visible.");
    process.exit(0);
}, 120000);

process.on("SIGINT", () => {
    console.log("\nShutting down...");
    process.exit(0);
});
