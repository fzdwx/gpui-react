import React, { useState, useEffect, useRef, useMemo } from "react";

const COLORS = ["#ff6b6b", "#4ecdc4", "#45b7d1", "#96ceb4", "#ffeaa7"];

interface DrawCommand {
    type: string;
    [key: string]: any;
}

export function CanvasCommandDemo() {
    const [time, setTime] = useState(0);
    const [colorIndex, setColorIndex] = useState(0);

    useEffect(() => {
        const interval = setInterval(() => {
            setTime((t) => t + 1);
            setColorIndex((c) => (c + 1) % COLORS.length);
        }, 50);
        return () => clearInterval(interval);
    }, []);

    const mainCanvasCommands: DrawCommand[] = useMemo(() => {
        const animX = 150 + Math.sin(time * 0.1) * 50;
        const animY = 100 + Math.cos(time * 0.1) * 30;
        const cmds: DrawCommand[] = [];

        console.log("Frame:", time, "animX:", animX, "animY:", animY, "color:", COLORS[colorIndex]);
        cmds.push({ type: "clear", color: "#ffffff" });

        for (let i = 0; i < 10; i++) {
            cmds.push({
                type: "line",
                x1: i * 40,
                y1: 0,
                x2: i * 40,
                y2: 220,
                width: 1,
                color: "#e0e0e0",
            });
        }
        for (let i = 0; i < 6; i++) {
            cmds.push({
                type: "line",
                x1: 0,
                y1: i * 40,
                x2: 400,
                y2: i * 40,
                width: 1,
                color: "#e0e0e0",
            });
        }

        cmds.push({
            type: "circle",
            x: animX + 20,
            y: animY + 20,
            radius: 20,
            color: COLORS[colorIndex],
        });
        cmds.push({ type: "fillRect", x: 30, y: 30, width: 80, height: 60, color: "#ff6b6b" });
        cmds.push({ type: "fillRect", x: 250, y: 50, width: 60, height: 60, color: "#4ecdc4" });
        cmds.push({ type: "circle", x: 315, y: 185, radius: 25, color: "#45b7d1" });
        cmds.push({ type: "fillRect", x: 30, y: 165, width: 180, height: 20, color: "#333333" });

        return cmds;
    }, [time, colorIndex]);

    const lightCanvasCommands: DrawCommand[] = useMemo(
        () => [
            { type: "clear", color: "#fafafa" },
            { type: "fillRect", x: 20, y: 20, width: 100, height: 60, color: "#ff6b6b" },
            { type: "circle", x: 160, y: 50, radius: 20, color: "#4ecdc4" },
        ],
        []
    );

    const darkCanvasCommands: DrawCommand[] = useMemo(
        () => [
            { type: "clear", color: "#1a1a2e" },
            { type: "fillRect", x: 30, y: 30, width: 60, height: 40, color: "#ff6b6b" },
            { type: "circle", x: 145, y: 85, radius: 25, color: "#4ecdc4" },
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
            "Canvas Command API Demo"
        ),
        React.createElement(
            "div",
            { style: { color: "#888888", fontSize: 14 } },
            "Direct canvas commands (clear, fillRect, circle, line)"
        ),
        React.createElement("canvas", {
            style: { width: 400, height: 220 },
            drawCommands: JSON.stringify(mainCanvasCommands),
        }),
        React.createElement(
            "div",
            { style: { display: "flex", gap: 20 } },
            React.createElement("canvas", {
                style: { width: 200, height: 150 },
                drawCommands: JSON.stringify(lightCanvasCommands),
            }),
            React.createElement("canvas", {
                style: { width: 200, height: 150 },
                drawCommands: JSON.stringify(darkCanvasCommands),
            })
        ),
        React.createElement(
            "div",
            { style: { color: "#4ecdc4", fontSize: 12 } },
            `Time: ${time} | Color: ${COLORS[colorIndex]}`
        )
    );
}
