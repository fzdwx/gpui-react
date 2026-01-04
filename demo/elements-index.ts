import React from "react";
import {createRoot} from "../src";
import { ElementsApp } from "./elements-app";

const root = createRoot();
root.render(
  React.createElement("div", { style: { backgroundColor: "#1e1e1e", padding: 40 } },
    React.createElement(ElementsApp)
  )
);

console.log("Elements demo running...");
console.log("Expecting window with span and div elements");

setTimeout(() => {
  console.log("Done! The elements window should be visible.");
  process.exit(0);
}, 10000);

process.on("SIGINT", () => {
  console.log("\nShutting down...");
  process.exit(0);
});
