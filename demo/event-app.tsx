import React, { useState } from "react";

export function EventApp() {
    const [clickCount, setClickCount] = useState(0);
    const [mouseState, setMouseState] = useState("idle");
    const [mousePos, setMousePos] = useState({ x: 0, y: 0 });
    const [lastKey, setLastKey] = useState("");
    const [scrollDelta, setScrollDelta] = useState({ x: 0, y: 0 });

    return (
        <div
            style={{
                display: "flex",
                flexDirection: "column",
                gap: 16,
                backgroundColor: "#1e1e1e",
                padding: 40,
                width: 600,
            }}
            onKeyDown={(e) => setLastKey("Down: " + e.key)}
            onKeyUp={(e) => setLastKey("Up: " + e.key)}
        >
            {/* Title */}
            <div
                style={{
                    color: "#1db588",
                    fontSize: 24,
                    fontWeight: 700,
                }}
            >
                Event System Demo
            </div>

            {/* Click Counter */}
            <div
                style={{
                    backgroundColor: "#4ed93b",
                    color: "#ffffff",
                    padding: 20,
                    borderRadius: 8,
                    fontSize: 20,
                    cursor: "pointer",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                }}
                onClick={() => setClickCount(clickCount + 1)}
            >
                {"Click Count: " + clickCount}
            </div>

            {/* Mouse Events Area */}
            <div
                style={{
                    backgroundColor: mouseState === "down" ? "#e74c3c" : "#2c3e50",
                    color: "#ffffff",
                    padding: 20,
                    borderRadius: 8,
                    fontSize: 16,
                    display: "flex",
                    flexDirection: "column",
                    alignItems: "center",
                    justifyContent: "center",
                    gap: 8,
                }}
                onMouseDown={() => setMouseState("down")}
                onMouseUp={() => setMouseState("up")}
                onMouseMove={(e) => setMousePos({ x: Math.round(e.clientX), y: Math.round(e.clientY) })}
            >
                <div>{"Mouse: " + mouseState}</div>
                <div>{"Position: " + mousePos.x + ", " + mousePos.y}</div>
            </div>

            {/* Keyboard Events */}
            <div
                style={{
                    backgroundColor: "#9b59b6",
                    color: "#ffffff",
                    padding: 20,
                    borderRadius: 8,
                    fontSize: 18,
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                }}
            >
                {"Key: " + (lastKey || "(press a key)")}
            </div>

            {/* Scroll/Wheel Events */}
            <div
                style={{
                    backgroundColor: "#e67e22",
                    color: "#ffffff",
                    padding: 20,
                    borderRadius: 8,
                    fontSize: 16,
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                }}
                onWheel={(e) => setScrollDelta({ x: Math.round(e.deltaX), y: Math.round(e.deltaY) })}
            >
                {"Wheel: " + scrollDelta.x + ", " + scrollDelta.y}
            </div>

            {/* Span with click */}
            <span
                style={{
                    color: "#f1c40f",
                    fontSize: 14,
                }}
                onClick={() => console.log("Span clicked!")}
            >
                Click this span (check console)
            </span>
        </div>
    );
}
