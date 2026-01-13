import React from "react";
import { createRoot } from "../../src/index";
import { InputApp } from "./app";

const root = createRoot({
    windowOption: {
        width: 750,
        height: 850,
    },
});

root.render(
    React.createElement(
        "div",
        { style: { backgroundColor: "#1a1a2e", padding: 20, minHeight: 810 } },
        React.createElement(InputApp)
    )
);

console.log("=================================");
console.log("  Input Component Demo Running");
console.log("=================================");
console.log("");
console.log("Features demonstrated:");
console.log("  - Search input with Enter submit");
console.log("  - Text input with character limit");
console.log("  - Password input with strength indicator");
console.log("  - Multi-line input (textarea)");
console.log("  - Disabled input state");
console.log("  - Focus/blur tracking");
console.log("  - Keyboard event tracking");
console.log("");
console.log("Press Ctrl+C to exit");

// Keep alive for 5 minutes
setTimeout(() => {
    console.log("\nDemo timeout. Exiting...");
    process.exit(0);
}, 300000);

process.on("SIGINT", () => {
    console.log("\nShutting down...");
    process.exit(0);
});
