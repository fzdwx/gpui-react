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
        </div>
    );
}
