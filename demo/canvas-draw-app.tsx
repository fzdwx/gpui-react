import React, { useState, useEffect, useRef } from "react";

interface DrawCanvasProps {
    width?: number;
    height?: number;
    backgroundColor?: string;
}

export function DrawCanvasApp() {
    const [time, setTime] = useState(0);
    const [colorIndex, setColorIndex] = useState(0);

    const colors = ["#ff6b6b", "#4ecdc4", "#45b7d1", "#96ceb4", "#ffeaa7"];

    useEffect(() => {
        const interval = setInterval(() => {
            setTime((t) => t + 1);
            setColorIndex((c) => (c + 1) % colors.length);
        }, 100);
        return () => clearInterval(interval);
    }, []);

    // 生成背景网格线
    const vLines = [];
    for (let i = 0; i < 10; i++) {
        vLines.push({ x: i * 40, y: 0, width: 1, height: 200 });
    }
    const hLines = [];
    for (let i = 0; i < 6; i++) {
        hLines.push({ x: 0, y: i * 40, width: 360, height: 1 });
    }

    const animatedCircleX = 150 + Math.sin(time * 0.1) * 50;
    const animatedCircleY = 100 + Math.cos(time * 0.1) * 30;

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
            "Canvas Custom Drawing Demo"
        ),
        React.createElement(
            "canvas",
            {
                style: {
                    width: 400,
                    height: 220,
                    backgroundColor: "#ffffff",
                    position: "relative",
                },
            },
            // Background grid - vertical lines
            ...vLines.map((line) =>
                React.createElement("line", {
                    key: `v-${line.x}`,
                    x: line.x,
                    y: line.y,
                    width: line.width,
                    height: line.height,
                    color: "#e0e0e0",
                    borderTopWidth: 1,
                })
            ),
            // Background grid - horizontal lines
            ...hLines.map((line) =>
                React.createElement("line", {
                    key: `h-${line.y}`,
                    x: line.x,
                    y: line.y,
                    width: line.width,
                    height: line.height,
                    color: "#e0e0e0",
                    borderTopWidth: 1,
                })
            ),
            // Animated circle
            React.createElement("circle", {
                x: animatedCircleX,
                y: animatedCircleY,
                width: 40,
                height: 40,
                color: colors[colorIndex],
            }),
            // Static rectangle
            React.createElement("rect", {
                x: 30,
                y: 30,
                width: 80,
                height: 60,
                color: "#ff6b6b",
            }),
            // Another rectangle
            React.createElement("rect", {
                x: 250,
                y: 50,
                width: 60,
                height: 60,
                color: "#4ecdc4",
            }),
            // Circle
            React.createElement("circle", {
                x: 290,
                y: 160,
                width: 50,
                height: 50,
                color: "#45b7d1",
            }),
            // Text
            React.createElement("text", {
                x: 30,
                y: 180,
                text: "Canvas Drawing Demo",
                textSize: 16,
                textColor: "#333333",
            })
        ),
        React.createElement(
            "div",
            { style: { display: "flex", gap: 20 } },
            React.createElement("canvas", {
                style: { width: 200, height: 150, backgroundColor: "#fafafa" },
                children: [
                    React.createElement("rect", {
                        x: 20,
                        y: 20,
                        width: 100,
                        height: 60,
                        color: "#ff6b6b",
                    }),
                    React.createElement("circle", {
                        x: 140,
                        y: 80,
                        width: 40,
                        height: 40,
                        color: "#4ecdc4",
                    }),
                ],
            }),
            React.createElement("canvas", {
                style: { width: 200, height: 150, backgroundColor: "#1a1a2e" },
                children: [
                    React.createElement("rect", {
                        x: 30,
                        y: 30,
                        width: 60,
                        height: 40,
                        color: "#ff6b6b",
                    }),
                    React.createElement("circle", {
                        x: 120,
                        y: 60,
                        width: 50,
                        height: 50,
                        color: "#4ecdc4",
                    }),
                ],
            })
        ),
        React.createElement(
            "div",
            { style: { color: "#4ecdc4", fontSize: 12 } },
            `Time: ${time} | Color: ${colors[colorIndex]}`
        )
    );
}
