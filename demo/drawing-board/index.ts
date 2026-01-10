import React from "react";
import { createRoot } from "../../src/index";
import { DrawingBoardApp } from "./app";

const root = createRoot({
    windowOption: {
        width: 680,
        height: 650,
        title: "Drawing Board - gpui-react",
    },
});

root.render(React.createElement(DrawingBoardApp));

process.on("SIGINT", () => {
    process.exit(0);
});
