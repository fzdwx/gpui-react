import React from "react";

export function FlexApp() {
  return React.createElement(
    "div",
    {
      style: {
        display: "flex",
        flexDirection: "column",
        backgroundColor: "#2d2d2d",
        padding: 20,
        gap: 10,
      }
    },
    React.createElement(
      "div",
      {
        style: {
          display: "flex",
          justifyContent: "center",
          backgroundColor: "#ff6b6b",
          padding: 15,
          borderRadius: 8,
        }
      },
      React.createElement(
        "div",
        {
          style: {
            display: "flex",
            justifyContent: "center",
            gap: 15,
          }
        },
        React.createElement("div", {
          style: { width: 100, height: 60, backgroundColor: "#4ecdc4", color: "white", fontSize: 16 }
        }, "Box 1"),
        React.createElement("div", {
          style: { width: 100, height: 60, backgroundColor: "#45b7d1", color: "white", fontSize: 16 }
        }, "Box 2")
      )
    )
  );
}
