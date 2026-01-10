import React from "react";

export function CanvasApp() {
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
            "Canvas Element Demo"
        ),
        React.createElement(
            "div",
            {
                style: { color: "#888888", fontSize: 14 },
            },
            "The canvas element renders using GPUI's native canvas() function"
        ),
        React.createElement(
            "canvas",
            {
                style: {
                    width: 400,
                    height: 300,
                    backgroundColor: "#ffffff",
                    borderRadius: 8,
                    border: "2px solid #4ecdc4",
                },
            },
            "Canvas content would be rendered here"
        ),
        React.createElement(
            "div",
            {
                style: {
                    display: "flex",
                    gap: 20,
                },
            },
            React.createElement(
                "canvas",
                {
                    style: {
                        width: 150,
                        height: 150,
                        backgroundColor: "#ff6b6b",
                        borderRadius: 8,
                    },
                },
                "Small Canvas 1"
            ),
            React.createElement(
                "canvas",
                {
                    style: {
                        width: 150,
                        height: 150,
                        backgroundColor: "#4ecdc4",
                        borderRadius: 8,
                    },
                },
                "Small Canvas 2"
            ),
            React.createElement(
                "canvas",
                {
                    style: {
                        width: 150,
                        height: 150,
                        backgroundColor: "#45b7d1",
                        borderRadius: 8,
                    },
                },
                "Small Canvas 3"
            )
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
                "canvas",
                {
                    style: {
                        width: 100,
                        height: 80,
                        backgroundColor: "#f0f0f0",
                        borderRadius: 4,
                    },
                },
                "Canvas"
            )
        )
    );
}
