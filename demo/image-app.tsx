import React from "react";

export function ImageApp() {
    return React.createElement(
        "div",
        {
            style: {
                display: "flex",
                flexDirection: "column",
                gap: 20,
                backgroundColor: "#1e1e1e",
                padding: 40,
            },
        },
        React.createElement(
            "div",
            {
                style: { color: "#ffffff", fontSize: 24, fontWeight: "bold" },
            },
            "Element Types Demo"
        ),
        React.createElement(
            "span",
            {
                style: { color: "#4ecdc4", fontSize: 18 },
            },
            "This is a span element"
        ),
        React.createElement(
            "div",
            {
                style: {
                    display: "flex",
                    gap: 10,
                    padding: 15,
                    backgroundColor: "#2d2d2d",
                    borderRadius: 8,
                },
            },
            React.createElement(
                "div",
                {
                    style: {
                        width: 100,
                        height: 80,
                        backgroundColor: "#ff6b6b",
                        borderRadius: 4,
                    },
                },
                "Placeholder Image 1"
            ),
            React.createElement(
                "div",
                {
                    style: {
                        width: 100,
                        height: 80,
                        backgroundColor: "#4ecdc4",
                        borderRadius: 4,
                    },
                },
                "Placeholder Image 2"
            ),
            React.createElement(
                "div",
                {
                    style: {
                        width: 100,
                        height: 80,
                        backgroundColor: "#45b7d1",
                        borderRadius: 4,
                    },
                },
                "Placeholder Image 3"
            )
        )
    );
}
