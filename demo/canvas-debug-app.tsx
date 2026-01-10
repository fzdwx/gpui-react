import React, { useState, useEffect, useRef, useMemo } from "react";

export function CanvasCommandDemo() {
    const [time, setTime] = useState(0);
    const [colorIndex, setColorIndex] = useState(0);
    const COLORS = ["#ff6b6b", "#4ecdc4", "#45b7d1", "#96ceb4", "#ffeaa7"];

    useEffect(() => {
        const interval = setInterval(() => {
            setTime((t) => t + 1);
            setColorIndex((c) => (c + 1) % COLORS.length);
        }, 100);
        return () => clearInterval(interval);
    }, []);

    const cmd1 = {
        type: "fillRect" as const,
        x: 50,
        y: 50,
        width: 100,
        height: 100,
        color: "#ff0000",
    };
    const cmd2 = {
        type: "fillRect" as const,
        x: 100,
        y: 100,
        width: 100,
        height: 100,
        color: "#00ff00",
    };
    const cmd3 = {
        type: "fillRect" as const,
        x: 150,
        y: 150,
        width: 100,
        height: 100,
        color: "#0000ff",
    };
    const cmd4 = { type: "circle" as const, x: 50, y: 250, radius: 50, color: "#ffff00" };
    const cmd5 = { type: "circle" as const, x: 200, y: 250, radius: 50, color: "#ff00ff" };

    const mainCommands = useMemo(
        () => [{ type: "clear" as const, color: "#ffffff" }, cmd1, cmd2, cmd3, cmd4, cmd5],
        [colorIndex]
    );

    const lightCommands = useMemo(
        () => [
            { type: "clear" as const, color: "#fafafa" },
            { type: "fillRect" as const, x: 20, y: 20, width: 50, height: 50, color: "#ff6b6b" },
            { type: "fillRect" as const, x: 80, y: 20, width: 50, height: 50, color: "#4ecdc4" },
            { type: "fillRect" as const, x: 20, y: 80, width: 50, height: 50, color: "#45b7d1" },
            { type: "fillRect" as const, x: 80, y: 80, width: 50, height: 50, color: "#96ceb4" },
        ],
        []
    );

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
            { style: { color: "#ffffff", fontSize: 24, fontWeight: "bold" } },
            "Canvas Debug - Red/Green/Blue Rectangles + Circles"
        ),
        React.createElement(
            "div",
            { style: { color: "#888888", fontSize: 14 } },
            `Time: ${time} | Color: ${COLORS[colorIndex]}`
        ),
        React.createElement("canvas", {
            style: { width: 300, height: 320, backgroundColor: "#333333" },
            drawCommands: JSON.stringify(mainCommands),
        }),
        React.createElement(
            "div",
            { style: { display: "flex", gap: 20 } },
            React.createElement("canvas", {
                style: { width: 150, height: 150, backgroundColor: "#333333" },
                drawCommands: JSON.stringify(lightCommands),
            }),
            React.createElement("canvas", {
                style: { width: 150, height: 150, backgroundColor: "#333333" },
                drawCommands: JSON.stringify([
                    { type: "clear", color: "#1a1a2e" },
                    { type: "fillRect", x: 20, y: 20, width: 40, height: 40, color: "#ff6b6b" },
                    { type: "fillRect", x: 80, y: 20, width: 40, height: 40, color: "#4ecdc4" },
                ]),
            })
        )
    );
}
