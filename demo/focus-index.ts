import React from "react";
import { createRoot } from "../src/index";
import { FocusTestApp } from "./focus-test";

const root = createRoot({
    windowOption: {
        width: 700,
        height: 800,
        title: "Focus & Keyboard Test",
    },
});
root.render(
    React.createElement(
        "div",
        { style: { backgroundColor: "#1e1e1e", padding: 20 } },
        React.createElement(FocusTestApp)
    )
);

console.log("Focus test demo running...");
console.log("Click boxes to focus, use Tab/Shift+Tab to navigate");
console.log("Press keys while focused to see keyboard events");

setTimeout(() => {
    console.log("Done! The focus test window should be visible.");
    process.exit(0);
}, 120000);

process.on("SIGINT", () => {
    console.log("\nShutting down...");
    process.exit(0);
});
