import { useState } from "react";

export function InputApp() {
    // Controlled input state
    const [controlledValue, setControlledValue] = useState("");

    // Uncontrolled input just shows events
    const [lastInputEvent, setLastInputEvent] = useState<string>("(none)");
    const [lastChangeEvent, setLastChangeEvent] = useState<string>("(none)");

    // Focus state
    const [focusedInput, setFocusedInput] = useState<string | null>(null);

    return (
        <div
            style={{
                display: "flex",
                flexDirection: "column",
                gap: 20,
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
                Input Element Demo
            </div>

            {/* Controlled Input */}
            <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
                <div style={{ color: "#888", fontSize: 14 }}>
                    Controlled Input (value synced with React state)
                </div>
                <input
                    value={controlledValue}
                    placeholder="Type something..."
                    style={{
                        backgroundColor: "#333",
                        color: "#fff",
                        padding: 12,
                        borderRadius: 4,
                        fontSize: 16,
                        width: 400,
                    }}
                    onInput={(e) => {
                        setControlledValue(e.value);
                        setLastInputEvent(`value: "${e.value}", type: ${e.inputType}`);
                    }}
                    onFocus={() => setFocusedInput("controlled")}
                    onBlur={() => setFocusedInput(null)}
                />
                <div style={{ color: "#666", fontSize: 12 }}>
                    {'Current value: "' + controlledValue + '"'}
                </div>
            </div>

            {/* Uncontrolled Input */}
            <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
                <div style={{ color: "#888", fontSize: 14 }}>
                    Uncontrolled Input (Rust manages state)
                </div>
                <input
                    defaultValue="Initial text"
                    placeholder="Uncontrolled input..."
                    style={{
                        backgroundColor: "#333",
                        color: "#fff",
                        padding: 12,
                        borderRadius: 4,
                        fontSize: 16,
                        width: 400,
                    }}
                    onInput={(e) => {
                        setLastInputEvent(`value: "${e.value}", type: ${e.inputType}`);
                    }}
                    onChange={(e) => {
                        setLastChangeEvent(`value: "${e.value}"`);
                    }}
                    onFocus={() => setFocusedInput("uncontrolled")}
                    onBlur={() => setFocusedInput(null)}
                />
            </div>

            {/* Password Input */}
            <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
                <div style={{ color: "#888", fontSize: 14 }}>Password Input (masked)</div>
                <input
                    type="password"
                    placeholder="Enter password..."
                    style={{
                        backgroundColor: "#333",
                        color: "#fff",
                        padding: 12,
                        borderRadius: 4,
                        fontSize: 16,
                        width: 400,
                    }}
                    onFocus={() => setFocusedInput("password")}
                    onBlur={() => setFocusedInput(null)}
                />
            </div>

            {/* Multi-line Input */}
            <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
                <div style={{ color: "#888", fontSize: 14 }}>Multi-line Input (textarea)</div>
                <input
                    multiLine={true}
                    rows={4}
                    placeholder="Enter multiple lines of text..."
                    style={{
                        backgroundColor: "#333",
                        color: "#fff",
                        padding: 12,
                        borderRadius: 4,
                        fontSize: 16,
                        width: 400,
                        height: 100,
                    }}
                    onInput={(e) => {
                        setLastInputEvent(`value: "${e.value}", type: ${e.inputType}`);
                    }}
                    onFocus={() => setFocusedInput("multiline")}
                    onBlur={() => setFocusedInput(null)}
                />
            </div>

            {/* Event Display */}
            <div
                style={{
                    backgroundColor: "#2c3e50",
                    padding: 16,
                    borderRadius: 8,
                    display: "flex",
                    flexDirection: "column",
                    gap: 8,
                }}
            >
                <div style={{ color: "#fff", fontSize: 14, fontWeight: 700 }}>Event Log</div>
                <div style={{ color: "#aaa", fontSize: 12 }}>
                    {"Last onInput: " + lastInputEvent}
                </div>
                <div style={{ color: "#aaa", fontSize: 12 }}>
                    {"Last onChange: " + lastChangeEvent}
                </div>
                <div style={{ color: "#aaa", fontSize: 12 }}>
                    {"Focused input: " + (focusedInput || "(none)")}
                </div>
            </div>

            {/* Tab Navigation Test */}
            <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
                <div style={{ color: "#888", fontSize: 14 }}>
                    Tab Navigation Test (press Tab to move between inputs)
                </div>
                <div style={{ display: "flex", gap: 12 }}>
                    <input
                        tabIndex={1}
                        placeholder="Tab 1"
                        style={{
                            backgroundColor: "#333",
                            color: "#fff",
                            padding: 8,
                            borderRadius: 4,
                            fontSize: 14,
                            width: 120,
                        }}
                    />
                    <input
                        tabIndex={2}
                        placeholder="Tab 2"
                        style={{
                            backgroundColor: "#333",
                            color: "#fff",
                            padding: 8,
                            borderRadius: 4,
                            fontSize: 14,
                            width: 120,
                        }}
                    />
                    <input
                        tabIndex={3}
                        placeholder="Tab 3"
                        style={{
                            backgroundColor: "#333",
                            color: "#fff",
                            padding: 8,
                            borderRadius: 4,
                            fontSize: 14,
                            width: 120,
                        }}
                    />
                </div>
            </div>

            {/* Instructions */}
            <div style={{ color: "#666", fontSize: 12 }}>
                Instructions: Click on input to focus, type to test text input, use Tab to navigate
                between fields.
            </div>
        </div>
    );
}
