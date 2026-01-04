import React from "react";
import {createRoot} from "../src";
import {FlexApp} from "./flex-app";

const root = createRoot();
root.render(
    React.createElement("div", {style: {backgroundColor: "#1e1e1e", padding: 40}},
        React.createElement(FlexApp)
    )
);

console.log("Flexbox demo running...");
console.log("Expecting window with flex layouts");

setTimeout(() => {
    console.log("Done! The flexbox window should be visible.");
    process.exit(0);
}, 10000);

process.on("SIGINT", () => {
    console.log("\nShutting down...");
    process.exit(0);
});
