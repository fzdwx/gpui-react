import React, { useState, useEffect, useMemo } from "react";
import type { CanvasDrawCommand } from "../../src/canvas";

const COLORS = ["#ff6b6b", "#4ecdc4", "#45b7d1", "#96ceb4", "#ffeaa7"];

export function CanvasSimpleDemo() {
    const [time, setTime] = useState(0);

    useEffect(() => {
        const interval = setInterval(() => {
            setTime((t) => t + 1);
        }, 50); // ~20fps for smooth animation
        return () => clearInterval(interval);
    }, []);

    const colorIndex = time % COLORS.length;
    const animatedColor = COLORS[colorIndex];

    // Animated circle position (figure-8 pattern)
    const animX = 150 + Math.sin(time * 0.1) * 50;
    const animY = 100 + Math.cos(time * 0.1) * 30;

    // Main canvas commands (400x220)
    const mainCommands = useMemo(() => {
        const cmds: CanvasDrawCommand[] = [
            // Clear with white background
            { type: "clear", color: "#ffffff" },
        ];

        // Draw grid - vertical lines
        for (let i = 0; i < 10; i++) {
            cmds.push({
                type: "fillRect",
                x: i * 40,
                y: 0,
                width: 1,
                height: 220,
                color: "#e0e0e0",
            });
        }

        // Draw grid - horizontal lines
        for (let i = 0; i < 6; i++) {
            cmds.push({
                type: "fillRect",
                x: 0,
                y: i * 40,
                width: 400,
                height: 1,
                color: "#e0e0e0",
            });
        }

        // Animated circle
        cmds.push({
            type: "circle",
            x: animX + 20, // center x
            y: animY + 20, // center y
            radius: 20,
            color: animatedColor,
        });

        // Static rectangle 1 (red)
        cmds.push({
            type: "fillRect",
            x: 30,
            y: 30,
            width: 80,
            height: 60,
            color: "#ff6b6b",
        });

        // Static rectangle 2 (cyan)
        cmds.push({
            type: "fillRect",
            x: 250,
            y: 50,
            width: 60,
            height: 60,
            color: "#4ecdc4",
        });

        // Static circle (blue)
        cmds.push({
            type: "circle",
            x: 315, // 290 + 25 (center)
            y: 185, // 160 + 25 (center)
            radius: 25,
            color: "#45b7d1",
        });

        // Text (not yet implemented in canvas, will be skipped)
        cmds.push({
            type: "text",
            text: "Canvas Drawing Demo",
            x: 30,
            y: 180,
            size: 16,
            color: "#333333",
        });

        return cmds;
    }, [time, animX, animY, animatedColor]);

    // Light canvas commands (200x150)
    const lightCommands = useMemo(
        (): CanvasDrawCommand[] => [
            { type: "clear", color: "#fafafa" },
            { type: "fillRect", x: 20, y: 20, width: 100, height: 60, color: "#ff6b6b" },
            { type: "circle", x: 160, y: 100, radius: 20, color: "#4ecdc4" },
        ],
        []
    );

    // Dark canvas commands (200x150)
    const darkCommands = useMemo(
        (): CanvasDrawCommand[] => [
            { type: "clear", color: "#1a1a2e" },
            { type: "fillRect", x: 30, y: 30, width: 60, height: 40, color: "#ff6b6b" },
            { type: "circle", x: 145, y: 85, radius: 25, color: "#4ecdc4" },
        ],
        []
    );

    return (
        <div style={{ backgroundColor: "#1e1e1e", padding: 40, minHeight: 600 }}>
            {/* Title */}
            <div style={{ color: "#ffffff", fontSize: 24, fontWeight: 700, marginBottom: 8 }}>
                Canvas Custom Drawing Demo (gpui-react)
            </div>
            <div style={{ color: "#888888", fontSize: 14, marginBottom: 20 }}>
                This demonstrates the same drawing operations as the HTML reference
            </div>

            {/* Main Canvas */}
            <div style={{ marginBottom: 20 }}>
                <div style={{ color: "#ffffff", fontSize: 14, marginBottom: 10 }}>
                    Main Canvas (400x220)
                </div>
                <canvas
                    style={{
                        width: 400,
                        height: 220,
                        borderRadius: 8,
                    }}
                    width={400}
                    height={220}
                    drawCommands={JSON.stringify(mainCommands)}
                />
            </div>

            {/* Two smaller canvases side by side */}
            <div style={{ display: "flex", flexDirection: "row", gap: 20, marginBottom: 20 }}>
                {/* Light Canvas */}
                <div>
                    <div style={{ color: "#ffffff", fontSize: 14, marginBottom: 10 }}>
                        Light Theme (200x150)
                    </div>
                    <canvas
                        style={{
                            width: 200,
                            height: 150,
                            borderRadius: 8,
                        }}
                        width={200}
                        height={150}
                        drawCommands={JSON.stringify(lightCommands)}
                    />
                </div>

                {/* Dark Canvas */}
                <div>
                    <div style={{ color: "#ffffff", fontSize: 14, marginBottom: 10 }}>
                        Dark Theme (200x150)
                    </div>
                    <canvas
                        style={{
                            width: 200,
                            height: 150,
                            borderRadius: 8,
                        }}
                        width={200}
                        height={150}
                        drawCommands={JSON.stringify(darkCommands)}
                    />
                </div>
            </div>

            {/* Info Panel */}
            <div
                style={{
                    backgroundColor: "#2d2d2d",
                    padding: 15,
                    borderRadius: 8,
                }}
            >
                <div style={{ color: "#4ecdc4", fontSize: 12, marginBottom: 8 }}>
                    Drawing Operations:
                </div>
                <div style={{ color: "#4ecdc4", fontSize: 12 }}>
                    Background grid (vertical and horizontal lines)
                </div>
                <div style={{ color: "#4ecdc4", fontSize: 12 }}>
                    Animated circle (moving in a figure-8 pattern with color cycling)
                </div>
                <div style={{ color: "#4ecdc4", fontSize: 12 }}>
                    Static rectangles (red, cyan colors)
                </div>
                <div style={{ color: "#4ecdc4", fontSize: 12 }}>Static circle (blue color)</div>
                <div style={{ color: "#ffffff", fontSize: 12, marginTop: 10 }}>
                    Time: {time} | Color: {animatedColor}
                </div>
            </div>
        </div>
    );
}
