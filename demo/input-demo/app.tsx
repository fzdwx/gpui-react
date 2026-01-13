/// <reference path="../../src/jsx.d.ts" />
import { useState } from "react";

export function InputApp() {
    const [textValue, setTextValue] = useState("");
    const [passwordValue, setPasswordValue] = useState("");
    const [multiLineValue, setMultiLineValue] = useState("");
    const [searchValue, setSearchValue] = useState("");
    const [focusedInput, setFocusedInput] = useState<string | null>(null);
    const [lastKeyPressed, setLastKeyPressed] = useState("");
    const [submitCount, setSubmitCount] = useState(0);

    const handleSubmit = () => {
        setSubmitCount(submitCount + 1);
        console.log("Form submitted!", { textValue, searchValue });
    };

    return (
        <div
            style={{
                display: "flex",
                flexDirection: "column",
                gap: 20,
                backgroundColor: "#1a1a2e",
                padding: 32,
                width: 650,
                minHeight: 700,
            }}
        >
            {/* Title */}
            <div
                style={{
                    color: "#00d9ff",
                    fontSize: 28,
                    fontWeight: 700,
                }}
            >
                Input Component Demo
            </div>

            {/* Status Bar */}
            <div
                style={{
                    display: "flex",
                    gap: 16,
                    backgroundColor: "#16213e",
                    padding: 12,
                    borderRadius: 8,
                }}
            >
                <div style={{ color: "#888888", fontSize: 13 }}>
                    {"Focus: " + (focusedInput || "none")}
                </div>
                <div style={{ color: "#888888", fontSize: 13 }}>
                    {"Last Key: " + (lastKeyPressed || "-")}
                </div>
                <div style={{ color: "#888888", fontSize: 13 }}>
                    {"Submits: " + submitCount}
                </div>
            </div>

            {/* Search Input with Icon */}
            <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
                <div style={{ color: "#aaaaaa", fontSize: 14, fontWeight: 600 }}>
                    Search Input
                </div>
                <div
                    style={{
                        display: "flex",
                        alignItems: "center",
                        backgroundColor: "#0f3460",
                        borderRadius: 8,
                        padding: 4,
                    }}
                >
                    <div style={{ color: "#666666", padding: 8, fontSize: 16 }}>
                        Q
                    </div>
                    <input
                        type="text"
                        value={searchValue}
                        placeholder="Search..."
                        style={{
                            backgroundColor: "transparent",
                            color: "#ffffff",
                            padding: 10,
                            fontSize: 15,
                            width: 500,
                            height: 36,
                        }}
                        onInput={(e: any) => setSearchValue(e.value)}
                        onFocus={() => setFocusedInput("search")}
                        onBlur={() => setFocusedInput(null)}
                        onKeyDown={(e: any) => {
                            setLastKeyPressed(e.key);
                            if (e.key === "Enter") {
                                handleSubmit();
                            }
                        }}
                        tabIndex={0}
                    />
                </div>
                {searchValue && (
                    <div style={{ color: "#666666", fontSize: 12 }}>
                        {"Searching for: \"" + searchValue + "\""}
                    </div>
                )}
            </div>

            {/* Basic Text Input with Character Count */}
            <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
                <div
                    style={{
                        display: "flex",
                        justifyContent: "space-between",
                        alignItems: "center",
                    }}
                >
                    <div style={{ color: "#aaaaaa", fontSize: 14, fontWeight: 600 }}>
                        Text Input
                    </div>
                    <div style={{ color: "#666666", fontSize: 12 }}>
                        {textValue.length + "/100"}
                    </div>
                </div>
                <input
                    type="text"
                    value={textValue}
                    placeholder="Type something here..."
                    style={{
                        backgroundColor: focusedInput === "text" ? "#1a3a5c" : "#0f3460",
                        color: "#ffffff",
                        padding: 14,
                        borderRadius: 8,
                        fontSize: 15,
                        width: 550,
                        height: 44,
                    }}
                    onInput={(e: any) => {
                        if (e.value.length <= 100) {
                            setTextValue(e.value);
                        }
                    }}
                    onFocus={() => setFocusedInput("text")}
                    onBlur={() => setFocusedInput(null)}
                    onKeyDown={(e: any) => setLastKeyPressed(e.key)}
                    tabIndex={0}
                />
            </div>

            {/* Password Input */}
            <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
                <div style={{ color: "#aaaaaa", fontSize: 14, fontWeight: 600 }}>
                    Password Input
                </div>
                <input
                    type="password"
                    value={passwordValue}
                    placeholder="Enter your password..."
                    style={{
                        backgroundColor: focusedInput === "password" ? "#1a3a5c" : "#0f3460",
                        color: "#ffffff",
                        padding: 14,
                        borderRadius: 8,
                        fontSize: 15,
                        width: 550,
                        height: 44,
                    }}
                    onInput={(e: any) => setPasswordValue(e.value)}
                    onFocus={() => setFocusedInput("password")}
                    onBlur={() => setFocusedInput(null)}
                    onKeyDown={(e: any) => setLastKeyPressed(e.key)}
                    tabIndex={0}
                />
                {/* Password Strength Indicator */}
                <div style={{ display: "flex", gap: 4 }}>
                    <div
                        style={{
                            width: 60,
                            height: 4,
                            borderRadius: 2,
                            backgroundColor:
                                passwordValue.length > 0 ? "#e74c3c" : "#333333",
                        }}
                    />
                    <div
                        style={{
                            width: 60,
                            height: 4,
                            borderRadius: 2,
                            backgroundColor:
                                passwordValue.length >= 6 ? "#f39c12" : "#333333",
                        }}
                    />
                    <div
                        style={{
                            width: 60,
                            height: 4,
                            borderRadius: 2,
                            backgroundColor:
                                passwordValue.length >= 10 ? "#2ecc71" : "#333333",
                        }}
                    />
                </div>
            </div>

            {/* Multi-line Input */}
            <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
                <div
                    style={{
                        display: "flex",
                        justifyContent: "space-between",
                        alignItems: "center",
                    }}
                >
                    <div style={{ color: "#aaaaaa", fontSize: 14, fontWeight: 600 }}>
                        Multi-line Input
                    </div>
                    <div style={{ color: "#666666", fontSize: 12 }}>
                        {"Lines: " + multiLineValue.split("\n").length + " | Chars: " + multiLineValue.length}
                    </div>
                </div>
                <input
                    multiLine
                    value={multiLineValue}
                    placeholder="Enter multiple lines of text...&#10;Press Enter to create new lines."
                    style={{
                        backgroundColor: focusedInput === "multiline" ? "#1a3a5c" : "#0f3460",
                        color: "#ffffff",
                        padding: 14,
                        borderRadius: 8,
                        fontSize: 15,
                        width: 550,
                        height: 100,
                    }}
                    onInput={(e: any) => setMultiLineValue(e.value)}
                    onFocus={() => setFocusedInput("multiline")}
                    onBlur={() => setFocusedInput(null)}
                    onKeyDown={(e: any) => setLastKeyPressed(e.key)}
                    tabIndex={0}
                />
            </div>

            {/* Disabled Input */}
            <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
                <div style={{ color: "#aaaaaa", fontSize: 14, fontWeight: 600 }}>
                    Disabled Input
                </div>
                <input
                    type="text"
                    value="This input is disabled"
                    disabled
                    style={{
                        backgroundColor: "#1a1a2e",
                        color: "#555555",
                        padding: 14,
                        borderRadius: 8,
                        fontSize: 15,
                        width: 550,
                        height: 44,
                    }}
                />
            </div>

            {/* Submit Button */}
            <div
                style={{
                    backgroundColor: "#e94560",
                    color: "#ffffff",
                    padding: 14,
                    borderRadius: 8,
                    fontSize: 16,
                    fontWeight: 600,
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    cursor: "pointer",
                    width: 578,
                }}
                onClick={handleSubmit}
            >
                Submit Form
            </div>

            {/* Instructions */}
            <div
                style={{
                    backgroundColor: "#16213e",
                    padding: 12,
                    borderRadius: 8,
                    color: "#666666",
                    fontSize: 12,
                }}
            >
                <div>Tips:</div>
                <div>- Tab to navigate between inputs</div>
                <div>- Type in search and press Enter to submit</div>
                <div>- Password shows strength indicator</div>
                <div>- Text input has 100 char limit</div>
            </div>
        </div>
    );
}
