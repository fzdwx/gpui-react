import React from "react";
import { createRoot } from "../src/renderer/index";

const root = createRoot();
root.render(React.createElement("div", null, "hello world"));
