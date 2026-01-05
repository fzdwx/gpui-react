import React from "react";
import {createRoot} from "../src/index";

const root = createRoot({
    width: 800, height: 800, title: "hello world"
});
root.render(React.createElement("div", null, "hello world"));

// Keep the process alive to allow GPUI window to render
// GPUI runs in a background thread, so we need to wait
console.log("GPUI window should be opening...");
console.log("Keeping process alive for 10 seconds...");

setTimeout(() => {
    console.log("Done! The GPUI window should now be visible.");
    console.log("If no window appeared, check for GPUI compilation issues.");
    process.exit(0);
}, 100000);

// Also handle Ctrl+C gracefully
process.on("SIGINT", () => {
    console.log("\nShutting down...");
    process.exit(0);
});
