import React, { useState, useCallback, useMemo } from "react";
import type { CanvasDrawCommand } from "../../src/canvas";

interface Point {
    x: number;
    y: number;
}

interface Stroke {
    points: Point[];
    color: string;
    width: number;
}

const COLORS = [
    "#000000",
    "#ff6b6b",
    "#4ecdc4",
    "#45b7d1",
    "#96ceb4",
    "#ffeaa7",
    "#dfe6e9",
    "#ffffff",
];
const BRUSH_SIZES = [2, 4, 8, 12, 20];

export function DrawingBoardApp() {
    const [strokes, setStrokes] = useState<Stroke[]>([]);
    const [currentStroke, setCurrentStroke] = useState<Stroke | null>(null);
    const [selectedColor, setSelectedColor] = useState("#000000");
    const [brushSize, setBrushSize] = useState(4);
    const [isDrawing, setIsDrawing] = useState(false);

    const canvasWidth = 600;
    const canvasHeight = 400;

    const handleMouseDown = useCallback(
        (e: any) => {
            const x = e.offsetX;
            const y = e.offsetY;
            setIsDrawing(true);
            setCurrentStroke({
                points: [{ x, y }],
                color: selectedColor,
                width: brushSize,
            });
        },
        [selectedColor, brushSize]
    );

    const handleMouseMove = useCallback(
        (e: any) => {
            if (!isDrawing || !currentStroke) return;
            const x = e.offsetX;
            const y = e.offsetY;
            setCurrentStroke((prev) => {
                if (!prev) return null;
                return {
                    ...prev,
                    points: [...prev.points, { x, y }],
                };
            });
        },
        [isDrawing, currentStroke]
    );

    const handleMouseUp = useCallback(() => {
        if (currentStroke && currentStroke.points.length > 0) {
            setStrokes((prev) => [...prev, currentStroke]);
        }
        setCurrentStroke(null);
        setIsDrawing(false);
    }, [currentStroke]);

    const handleClear = useCallback(() => {
        setStrokes([]);
        setCurrentStroke(null);
    }, []);

    const handleUndo = useCallback(() => {
        setStrokes((prev) => prev.slice(0, -1));
    }, []);

    // Generate draw commands from strokes
    const drawCommands = useMemo(() => {
        const cmds: CanvasDrawCommand[] = [
            // White background
            { type: "clear", color: "#ffffff" },
        ];

        // Draw all completed strokes
        const allStrokes = currentStroke ? [...strokes, currentStroke] : strokes;

        for (const stroke of allStrokes) {
            if (stroke.points.length === 1) {
                // Single point - draw a circle
                cmds.push({
                    type: "circle",
                    x: stroke.points[0].x,
                    y: stroke.points[0].y,
                    radius: stroke.width / 2,
                    color: stroke.color,
                });
            } else if (stroke.points.length >= 2) {
                // Multiple points - draw circles at each point for smooth lines
                for (const point of stroke.points) {
                    cmds.push({
                        type: "circle",
                        x: point.x,
                        y: point.y,
                        radius: stroke.width / 2,
                        color: stroke.color,
                    });
                }
                // Also draw rectangles between points for continuity
                for (let i = 1; i < stroke.points.length; i++) {
                    const p1 = stroke.points[i - 1];
                    const p2 = stroke.points[i];
                    // Draw a line segment as small rectangles
                    const dx = p2.x - p1.x;
                    const dy = p2.y - p1.y;
                    const dist = Math.sqrt(dx * dx + dy * dy);
                    if (dist > 0) {
                        const steps = Math.ceil(dist / 2);
                        for (let j = 0; j <= steps; j++) {
                            const t = j / steps;
                            cmds.push({
                                type: "circle",
                                x: p1.x + dx * t,
                                y: p1.y + dy * t,
                                radius: stroke.width / 2,
                                color: stroke.color,
                            });
                        }
                    }
                }
            }
        }

        return cmds;
    }, [strokes, currentStroke]);

    return (
        <div style={{ backgroundColor: "#2d3436", padding: 20, minHeight: 600 }}>
            {/* Title */}
            <div style={{ color: "#ffffff", fontSize: 24, fontWeight: 700, marginBottom: 20 }}>
                Drawing Board
            </div>

            {/* Toolbar */}
            <div
                style={{
                    display: "flex",
                    flexDirection: "row",
                    gap: 20,
                    marginBottom: 15,
                    alignItems: "center",
                }}
            >
                {/* Color Palette */}
                <div style={{ display: "flex", flexDirection: "row", gap: 8 }}>
                    {COLORS.map((color) => (
                        <div
                            key={color}
                            onClick={() => setSelectedColor(color)}
                            style={{
                                width: 32,
                                height: 32,
                                backgroundColor: color,
                                borderRadius: 4,
                                border:
                                    selectedColor === color
                                        ? "3px solid #74b9ff"
                                        : "2px solid #636e72",
                                cursor: "pointer",
                            }}
                        />
                    ))}
                </div>

                {/* Brush Size */}
                <div
                    style={{ display: "flex", flexDirection: "row", gap: 8, alignItems: "center" }}
                >
                    <div style={{ color: "#b2bec3", fontSize: 12 }}>Brush:</div>
                    {BRUSH_SIZES.map((size) => (
                        <div
                            key={size}
                            onClick={() => setBrushSize(size)}
                            style={{
                                width: 32,
                                height: 32,
                                backgroundColor: brushSize === size ? "#74b9ff" : "#636e72",
                                borderRadius: 4,
                                display: "flex",
                                justifyContent: "center",
                                alignItems: "center",
                                cursor: "pointer",
                            }}
                        >
                            <div
                                style={{
                                    width: size,
                                    height: size,
                                    backgroundColor: "#ffffff",
                                    borderRadius: size / 2,
                                }}
                            />
                        </div>
                    ))}
                </div>

                {/* Buttons */}
                <div
                    onClick={handleUndo}
                    style={{
                        backgroundColor: "#636e72",
                        color: "#ffffff",
                        padding: 8,
                        paddingLeft: 16,
                        paddingRight: 16,
                        borderRadius: 4,
                        fontSize: 14,
                        cursor: "pointer",
                    }}
                >
                    Undo
                </div>
                <div
                    onClick={handleClear}
                    style={{
                        backgroundColor: "#d63031",
                        color: "#ffffff",
                        padding: 8,
                        paddingLeft: 16,
                        paddingRight: 16,
                        borderRadius: 4,
                        fontSize: 14,
                        cursor: "pointer",
                    }}
                >
                    Clear
                </div>
            </div>

            {/* Canvas */}
            <div
                style={{
                    border: "2px solid #636e72",
                    borderRadius: 8,
                    overflow: "hidden",
                    width: canvasWidth + 4,
                }}
            >
                <canvas
                    style={{
                        width: canvasWidth,
                        height: canvasHeight,
                    }}
                    width={canvasWidth}
                    height={canvasHeight}
                    drawCommands={JSON.stringify(drawCommands)}
                    onMouseDown={handleMouseDown}
                    onMouseMove={handleMouseMove}
                    onMouseUp={handleMouseUp}
                />
            </div>

            {/* Status */}
            <div
                style={{
                    marginTop: 15,
                    color: "#b2bec3",
                    fontSize: 12,
                }}
            >
                Strokes: {strokes.length} | Color: {selectedColor} | Brush Size: {brushSize}px |{" "}
                {isDrawing ? "Drawing..." : "Ready"}
            </div>

            {/* Instructions */}
            <div
                style={{
                    marginTop: 10,
                    backgroundColor: "#1e272e",
                    padding: 15,
                    borderRadius: 8,
                }}
            >
                <div style={{ color: "#74b9ff", fontSize: 14, marginBottom: 8 }}>Instructions:</div>
                <div style={{ color: "#b2bec3", fontSize: 12 }}>
                    Click and drag on the canvas to draw
                </div>
                <div style={{ color: "#b2bec3", fontSize: 12 }}>
                    Select colors from the palette above
                </div>
                <div style={{ color: "#b2bec3", fontSize: 12 }}>
                    Choose brush size for different stroke widths
                </div>
                <div style={{ color: "#b2bec3", fontSize: 12 }}>
                    Use Undo to remove the last stroke, Clear to start over
                </div>
            </div>
        </div>
    );
}
