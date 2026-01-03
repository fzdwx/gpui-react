import React from "react";

export function ElementsApp() {
  return React.createElement(
    "div",
    {
      style: {
        display: "flex",
        flexDirection: "column",
        gap: 20,
        backgroundColor: "#1e1e1e",
        padding: 30,
      }
    },
    React.createElement("span", {
      style: { color: "#ff6b6b", fontSize: 18, fontWeight: "bold" }
    }, "Inline span element! "),
    React.createElement("div", {
      style: {
        display: "flex",
        gap: 20,
        padding: 15,
        backgroundColor: "#2d2d2d",
        borderRadius: 8,
      }
    },
      React.createElement("span", {
        style: { color: "#4ecdc4", fontSize: 16 }
      }, "Second span"),
      React.createElement("span", {
        style: { color: "#45b7d1", fontSize: 16 }
      }, "Third span")
    )
  );
}
