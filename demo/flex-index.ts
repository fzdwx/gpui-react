import React from "react";
import { createRoot } from "../src/renderer/index";
import { FlexApp } from "./flex-app";

const root = createRoot();
root.render(
  React.createElement("div", { style: { backgroundColor: "#1e1e1e", padding: 40 } },
    React.createElement(FlexApp)
  )
);

console.log("Flexbox demo running...");
console.log("Expecting window with flex layouts");
console.log("Window will stay open until you close it...");

root.run();

console.log("Window closed, exiting...");
process.exit(0);
