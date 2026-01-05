import React from "react";

export function FlexApp() {
    return (
        <div
            style={{
                display: "flex",
                flexDirection: "column",
                backgroundColor: "#2d2d2d",
                padding: 20,
                gap: 10,
            }}
        >
            <div
                style={{
                    display: "flex",
                    justifyContent: "center",
                    backgroundColor: "#ff6b6b",
                    padding: 15,
                    borderRadius: 8,
                }}
            >
                <div
                    style={{
                        display: "flex",
                        justifyContent: "center",
                        gap: 15,
                    }}
                >
                    <div
                        style={{
                            width: 100,
                            height: 60,
                            backgroundColor: "#4ecdc4",
                            color: "white",
                            fontSize: 16,
                        }}
                    >
                        Box 1
                    </div>
                    <div
                        style={{
                            width: 100,
                            height: 60,
                            backgroundColor: "#45b7d1",
                            color: "white",
                            fontSize: 16,
                        }}
                    >
                        Box 2
                    </div>
                </div>
            </div>
        </div>
    );
}
