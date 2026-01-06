import React from "react";
import { createRoot } from "../src";
import { StyledApp } from "./styled-app";

const root = createRoot({
    windowOption: {
        width: 800,
        height: 600,
    },
});
root.render(
    React.createElement(
        "div",
        { style: { backgroundColor: "#1e1e1e", padding: 20 } },
        React.createElement(StyledApp)
    )
);

console.log("Styled GPUI demo running...");
console.log("Expecting a window with styled elements");

setTimeout(() => {
    console.log("Done! The styled GPUI window should be visible.");
    process.exit(0);
}, 10000);

process.on("SIGINT", () => {
    console.log("\nShutting down...");
    process.exit(0);
});
