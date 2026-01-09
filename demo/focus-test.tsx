import React, { useState } from "react";

export function FocusTestApp() {
    const [focusedElement, setFocusedElement] = useState<string | null>(null);
    const [lastKey, setLastKey] = useState("");
    const [hoverElement, setHoverElement] = useState<string | null>(null);
    const [keyHistory, setKeyHistory] = useState<string[]>([]);

    const handleFocus = (name: string) => {
        setFocusedElement(name);
        console.log(`[Focus] ${name}`);
    };

    const handleBlur = (name: string) => {
        if (focusedElement === name) {
            setFocusedElement(null);
        }
        console.log(`[Blur] ${name}`);
    };

    const handleKeyDown = (name: string, key: string) => {
        const msg = `${name}: KeyDown "${key}"`;
        setLastKey(msg);
        setKeyHistory((prev) => [...prev.slice(-4), msg]);
        console.log(`[KeyDown] ${name}: ${key}`);
    };

    const handleKeyUp = (name: string, key: string) => {
        const msg = `${name}: KeyUp "${key}"`;
        setLastKey(msg);
        console.log(`[KeyUp] ${name}: ${key}`);
    };

    const handleMouseEnter = (name: string) => {
        setHoverElement(name);
        console.log(`[MouseEnter] ${name}`);
    };

    const handleMouseLeave = (name: string) => {
        if (hoverElement === name) {
            setHoverElement(null);
        }
        console.log(`[MouseLeave] ${name}`);
    };

    const boxStyle = (name: string, baseColor: string) => ({
        backgroundColor: focusedElement === name ? "#4a90d9" : hoverElement === name ? "#5a6a7a" : baseColor,
        color: "#ffffff",
        padding: 20,
        borderRadius: 8,
        fontSize: 16,
        cursor: "pointer",
        border: focusedElement === name ? "3px solid #ffcc00" : "3px solid transparent",
        display: "flex",
        flexDirection: "column" as const,
        alignItems: "center",
        justifyContent: "center",
        gap: 4,
        transition: "all 0.1s",
    });

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
        >
            {/* Title */}
            <div
                style={{
                    color: "#1db588",
                    fontSize: 24,
                    fontWeight: 700,
                }}
            >
                Focus & Keyboard Test
            </div>

            {/* Instructions */}
            <div style={{ color: "#888888", fontSize: 14 }}>
                Click boxes to focus. Use Tab/Shift+Tab to navigate. Press keys when focused.
            </div>

            {/* Focusable Box 1 */}
            <div
                tabIndex={0}
                style={boxStyle("Box1", "#2c3e50")}
                onFocus={() => handleFocus("Box1")}
                onBlur={() => handleBlur("Box1")}
                onKeyDown={(e) => handleKeyDown("Box1", e.key)}
                onKeyUp={(e) => handleKeyUp("Box1", e.key)}
                onMouseEnter={() => handleMouseEnter("Box1")}
                onMouseLeave={() => handleMouseLeave("Box1")}
            >
                <div>Box 1 (tabIndex=0)</div>
                <div style={{ fontSize: 12, color: "#aaaaaa" }}>
                    {focusedElement === "Box1" ? "FOCUSED" : "Click to focus"}
                </div>
            </div>

            {/* Focusable Box 2 */}
            <div
                tabIndex={1}
                style={boxStyle("Box2", "#34495e")}
                onFocus={() => handleFocus("Box2")}
                onBlur={() => handleBlur("Box2")}
                onKeyDown={(e) => handleKeyDown("Box2", e.key)}
                onKeyUp={(e) => handleKeyUp("Box2", e.key)}
                onMouseEnter={() => handleMouseEnter("Box2")}
                onMouseLeave={() => handleMouseLeave("Box2")}
            >
                <div>Box 2 (tabIndex=1)</div>
                <div style={{ fontSize: 12, color: "#aaaaaa" }}>
                    {focusedElement === "Box2" ? "FOCUSED" : "Click to focus"}
                </div>
            </div>

            {/* Focusable Box 3 */}
            <div
                tabIndex={2}
                style={boxStyle("Box3", "#4a5568")}
                onFocus={() => handleFocus("Box3")}
                onBlur={() => handleBlur("Box3")}
                onKeyDown={(e) => handleKeyDown("Box3", e.key)}
                onKeyUp={(e) => handleKeyUp("Box3", e.key)}
                onMouseEnter={() => handleMouseEnter("Box3")}
                onMouseLeave={() => handleMouseLeave("Box3")}
            >
                <div>Box 3 (tabIndex=2)</div>
                <div style={{ fontSize: 12, color: "#aaaaaa" }}>
                    {focusedElement === "Box3" ? "FOCUSED" : "Click to focus"}
                </div>
            </div>

            {/* Hover-only Box (no tabIndex) */}
            <div
                style={{
                    ...boxStyle("HoverBox", "#6b7280"),
                    border: "3px solid transparent",
                }}
                onMouseEnter={() => handleMouseEnter("HoverBox")}
                onMouseLeave={() => handleMouseLeave("HoverBox")}
            >
                <div>Hover Only (no tabIndex)</div>
                <div style={{ fontSize: 12, color: "#aaaaaa" }}>
                    {hoverElement === "HoverBox" ? "HOVERED" : "Hover over me"}
                </div>
            </div>

            {/* Status Display */}
            <div
                style={{
                    backgroundColor: "#1a1a2e",
                    color: "#ffffff",
                    padding: 16,
                    borderRadius: 8,
                    fontSize: 14,
                    display: "flex",
                    flexDirection: "column",
                    gap: 8,
                }}
            >
                <div style={{ color: "#4ade80" }}>
                    {"Focused: " + (focusedElement || "none")}
                </div>
                <div style={{ color: "#60a5fa" }}>
                    {"Hovered: " + (hoverElement || "none")}
                </div>
                <div style={{ color: "#fbbf24" }}>
                    {"Last Key: " + (lastKey || "(none)")}
                </div>
                <div style={{ color: "#888888", fontSize: 12 }}>
                    {"History: " + keyHistory.join(" → ")}
                </div>
            </div>
        </div>
    );
}
