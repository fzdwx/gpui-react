import React from "react";
import { createRoot } from "../src/renderer/index";
import { StyledApp } from "./styled-app";

const root = createRoot();
root.render(
  React.createElement("div", { style: { backgroundColor: "#1e1e1e", padding: 20 } },
    React.createElement(StyledApp)
  )
);

console.log("Styled GPUI demo running...");
console.log("Expecting a window with styled elements");
console.log("Window will stay open until you close it...");

root.run();

console.log("Window closed, exiting...");
process.exit(0);
