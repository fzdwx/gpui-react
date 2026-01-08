import React, { useState } from "react";
import { createRoot } from "../src/index";

// Test component for new CSS features
function CSSTestApp() {
    const [count, setCount] = useState(0);

    return (
        <div
            style={{
                display: "flex",
                flexDirection: "column",
                gap: 20,
                padding: 20,
                backgroundColor: "#1a1a2e",
                width: 760,
                height: 760,
            }}
        >
            {/* Title */}
            <div
                style={{
                    fontSize: 24,
                    fontWeight: "bold",
                    color: "#eee",
                    textAlign: "center",
                    marginBottom: 10,
                }}
            >
                CSS Features Test
            </div>

            {/* Row 1: Flexbox features */}
            <div
                style={{
                    display: "flex",
                    flexDirection: "row",
                    gap: 15,
                    flexWrap: "wrap",
                }}
            >
                {/* Box Shadow */}
                <div
                    style={{
                        width: 150,
                        height: 80,
                        backgroundColor: "#16213e",
                        borderRadius: 8,
                        boxShadow: "4px 4px 10px #000",
                        display: "flex",
                        justifyContent: "center",
                        alignItems: "center",
                        color: "#fff",
                        fontSize: 14,
                    }}
                >
                    Box Shadow
                </div>

                {/* Border Complete */}
                <div
                    style={{
                        width: 150,
                        height: 80,
                        backgroundColor: "#16213e",
                        border: "2px solid #e94560",
                        borderRadius: 8,
                        display: "flex",
                        justifyContent: "center",
                        alignItems: "center",
                        color: "#fff",
                        fontSize: 14,
                    }}
                >
                    Border
                </div>

                {/* Min/Max Width */}
                <div
                    style={{
                        minWidth: 100,
                        maxWidth: 200,
                        height: 80,
                        backgroundColor: "#0f3460",
                        borderRadius: 8,
                        display: "flex",
                        justifyContent: "center",
                        alignItems: "center",
                        color: "#fff",
                        fontSize: 14,
                        padding: "0 20px",
                    }}
                >
                    Min/Max Width
                </div>
            </div>

            {/* Row 2: Position & Overflow */}
            <div
                style={{
                    display: "flex",
                    flexDirection: "row",
                    gap: 15,
                }}
            >
                {/* Position Relative/Absolute */}
                <div
                    style={{
                        width: 200,
                        height: 120,
                        backgroundColor: "#16213e",
                        borderRadius: 8,
                        position: "relative",
                    }}
                >
                    <div
                        style={{
                            position: "absolute",
                            top: 10,
                            left: 10,
                            backgroundColor: "#e94560",
                            padding: 8,
                            borderRadius: 4,
                            color: "#fff",
                            fontSize: 12,
                        }}
                    >
                        Top-Left
                    </div>
                    <div
                        style={{
                            position: "absolute",
                            bottom: 10,
                            right: 10,
                            backgroundColor: "#0f3460",
                            padding: 8,
                            borderRadius: 4,
                            color: "#fff",
                            fontSize: 12,
                        }}
                    >
                        Bottom-Right
                    </div>
                </div>

                {/* Overflow Hidden */}
                <div
                    style={{
                        width: 200,
                        height: 120,
                        backgroundColor: "#16213e",
                        borderRadius: 8,
                        overflow: "hidden",
                        padding: 10,
                    }}
                >
                    <div
                        style={{
                            color: "#fff",
                            fontSize: 12,
                            marginBottom: 5,
                        }}
                    >
                        Overflow Hidden:
                    </div>
                    <div
                        style={{
                            width: 300,
                            height: 100,
                            backgroundColor: "#e94560",
                            borderRadius: 4,
                        }}
                    />
                </div>

                {/* Aspect Ratio */}
                <div
                    style={{
                        width: 120,
                        aspectRatio: 1,
                        backgroundColor: "#0f3460",
                        borderRadius: 8,
                        display: "flex",
                        justifyContent: "center",
                        alignItems: "center",
                        color: "#fff",
                        fontSize: 12,
                    }}
                >
                    1:1 Ratio
                </div>
            </div>

            {/* Row 3: Flex grow/shrink */}
            <div
                style={{
                    display: "flex",
                    flexDirection: "row",
                    gap: 10,
                    width: 720,
                }}
            >
                <div
                    style={{
                        flexGrow: 1,
                        height: 60,
                        backgroundColor: "#e94560",
                        borderRadius: 8,
                        display: "flex",
                        justifyContent: "center",
                        alignItems: "center",
                        color: "#fff",
                        fontSize: 12,
                    }}
                >
                    flexGrow: 1
                </div>
                <div
                    style={{
                        flexGrow: 2,
                        height: 60,
                        backgroundColor: "#0f3460",
                        borderRadius: 8,
                        display: "flex",
                        justifyContent: "center",
                        alignItems: "center",
                        color: "#fff",
                        fontSize: 12,
                    }}
                >
                    flexGrow: 2
                </div>
                <div
                    style={{
                        flexGrow: 1,
                        height: 60,
                        backgroundColor: "#16213e",
                        borderRadius: 8,
                        display: "flex",
                        justifyContent: "center",
                        alignItems: "center",
                        color: "#fff",
                        fontSize: 12,
                    }}
                >
                    flexGrow: 1
                </div>
            </div>

            {/* Row 4: Align Content / Self */}
            <div
                style={{
                    display: "flex",
                    flexDirection: "row",
                    gap: 15,
                    alignItems: "stretch",
                    height: 100,
                }}
            >
                <div
                    style={{
                        width: 100,
                        backgroundColor: "#16213e",
                        borderRadius: 8,
                        alignSelf: "flex-start",
                        padding: 10,
                        color: "#fff",
                        fontSize: 11,
                    }}
                >
                    align-self: start
                </div>
                <div
                    style={{
                        width: 100,
                        backgroundColor: "#0f3460",
                        borderRadius: 8,
                        alignSelf: "center",
                        padding: 10,
                        color: "#fff",
                        fontSize: 11,
                    }}
                >
                    align-self: center
                </div>
                <div
                    style={{
                        width: 100,
                        backgroundColor: "#e94560",
                        borderRadius: 8,
                        alignSelf: "flex-end",
                        padding: 10,
                        color: "#fff",
                        fontSize: 11,
                    }}
                >
                    align-self: end
                </div>
                <div
                    style={{
                        width: 100,
                        backgroundColor: "#533483",
                        borderRadius: 8,
                        alignSelf: "stretch",
                        padding: 10,
                        color: "#fff",
                        fontSize: 11,
                    }}
                >
                    align-self: stretch
                </div>
            </div>

            {/* Row 5: Margin shorthand test */}
            <div
                style={{
                    display: "flex",
                    flexDirection: "row",
                    gap: 10,
                    backgroundColor: "#16213e",
                    padding: 15,
                    borderRadius: 8,
                }}
            >
                <div
                    style={{
                        margin: 10,
                        padding: 15,
                        backgroundColor: "#e94560",
                        borderRadius: 4,
                        color: "#fff",
                        fontSize: 12,
                    }}
                >
                    margin: 10
                </div>
                <div
                    style={{
                        margin: "5px 20px",
                        padding: 15,
                        backgroundColor: "#0f3460",
                        borderRadius: 4,
                        color: "#fff",
                        fontSize: 12,
                    }}
                >
                    margin: 5 20
                </div>
                <div
                    style={{
                        marginTop: 5,
                        marginBottom: 15,
                        padding: 15,
                        backgroundColor: "#533483",
                        borderRadius: 4,
                        color: "#fff",
                        fontSize: 12,
                    }}
                >
                    marginTop/Bottom
                </div>
            </div>

            {/* Row 6: Interactive - Click counter */}
            <div
                style={{
                    display: "flex",
                    flexDirection: "row",
                    gap: 15,
                    justifyContent: "center",
                    alignItems: "center",
                }}
            >
                <div
                    style={{
                        padding: "12px 24px",
                        backgroundColor: "#e94560",
                        borderRadius: 8,
                        color: "#fff",
                        fontSize: 16,
                        fontWeight: "bold",
                        cursor: "pointer",
                        boxShadow: "2px 2px 8px #000",
                    }}
                    onClick={() => setCount(count + 1)}
                >
                    Click Me!
                </div>
                <div
                    style={{
                        padding: "12px 24px",
                        backgroundColor: "#16213e",
                        border: "2px solid #e94560",
                        borderRadius: 8,
                        color: "#fff",
                        fontSize: 16,
                    }}
                >
                    Count: {count}
                </div>
            </div>

            {/* Row 7: Row/Column Gap */}
            <div
                style={{
                    display: "flex",
                    flexDirection: "row",
                    flexWrap: "wrap",
                    rowGap: 10,
                    columnGap: 20,
                    backgroundColor: "#16213e",
                    padding: 15,
                    borderRadius: 8,
                    width: 400,
                }}
            >
                {[1, 2, 3, 4, 5, 6].map((n) => (
                    <div
                        key={n}
                        style={{
                            width: 80,
                            height: 40,
                            backgroundColor: n % 2 === 0 ? "#e94560" : "#0f3460",
                            borderRadius: 4,
                            display: "flex",
                            justifyContent: "center",
                            alignItems: "center",
                            color: "#fff",
                            fontSize: 12,
                        }}
                    >
                        Item {n}
                    </div>
                ))}
            </div>
        </div>
    );
}

// Create root and render
const root = createRoot({
    windowOption: {
        width: 800,
        height: 800,
        title: "CSS Features Test",
    },
});

root.render(<CSSTestApp />);

console.log("CSS Test Demo - Window should be opening...");
console.log("Testing: box-shadow, border, position, overflow, flex, margin shorthand, etc.");

setTimeout(() => {
    process.exit(0);
}, 100000);

process.on("SIGINT", () => {
    console.log("\nShutting down...");
    process.exit(0);
});
