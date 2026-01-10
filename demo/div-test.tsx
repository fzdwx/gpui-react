import React, { useState, useEffect, useMemo } from "react";
import { createRoot } from "../src/index";

function CanvasTest() {
    const [count, setCount] = useState(0);

    useEffect(() => {
        const interval = setInterval(() => {
            setCount((c) => c + 1);
        }, 200);
        return () => clearInterval(interval);
    }, []);

    const bgColor = useMemo(
        () => ["#ff6b6b", "#4ecdc4", "#45b7d1"][count % 3] || "#ff6b6b",
        [count]
    );

    return (
        <div style={{ padding: 20, backgroundColor: "#1a1a1a" }}>
            <div style={{ color: "white", marginBottom: 10 }}>Count: {count}</div>
            <div
                style={{
                    position: "relative",
                    width: 250,
                    height: 250,
                    backgroundColor: bgColor,
                }}
            >
                {/* fillRect at (50, 50) 100x80 */}
                <div
                    style={{
                        position: "absolute",
                        left: 50,
                        top: 50,
                        width: 100,
                        height: 80,
                        backgroundColor: "#4ecdc4",
                    }}
                />
                {/* fillRect at (100, 100) 80x60 */}
                <div
                    style={{
                        position: "absolute",
                        left: 100,
                        top: 100,
                        width: 80,
                        height: 60,
                        backgroundColor: "#45b7d1",
                    }}
                />
            </div>
        </div>
    );
}

const root = createRoot({
    windowOption: {
        width: 400,
        height: 400,
        title: "Div Canvas Test",
    },
});

root.render(<CanvasTest />);

setTimeout(() => {
    process.exit(0);
}, 10000);
process.on("SIGINT", () => {
    process.exit(0);
});
