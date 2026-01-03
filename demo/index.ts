import React from "react";
import { createRoot } from "../src/renderer/index";

const root = createRoot();
root.render(React.createElement("div", null, "hello world"));

console.log("GPUI window should be opening...");
console.log("Window will stay open until you close it...");

root.run();

console.log("Window closed, exiting...");
process.exit(0);
