import React from "react";
import { createRoot } from "../src/index";

const root = createRoot({
    windowOption: {
        width: 400,
        height: 400,
        title: "Basic Test - gpui-react",
    },
});

function TestApp() {
    return React.createElement(
        "div",
        { style: { padding: 20, backgroundColor: "#2d2d2d" } },
        React.createElement("div", { style: { color: "white" } }, "Basic Test"),
        React.createElement("div", {
            style: { width: 100, height: 100, backgroundColor: "red", marginTop: 10 },
        })
    );
}

root.render(React.createElement(TestApp));

setTimeout(() => {
    process.exit(0);
}, 10000);

process.on("SIGINT", () => {
    process.exit(0);
});
