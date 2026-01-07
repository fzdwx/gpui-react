import React, {useState, useEffect} from "react";


export function EventApp() {
    const [count, setCount] = useState(0)

    return (
        <div
            style={{
                display: "flex",
                flexDirection: "column",
                gap: 20,
                backgroundColor: "#1e1e1e",
                padding: 40,
                alignItems: "center",
            }}
        >
            <div
                style={{
                    color: "#1db588",
                    fontSize: 25,
                    fontWeight: "bold",
                }}
            >
                Click Test
            </div>
            <div
                style={{
                    backgroundColor: "#4ed93b",
                    color: "white",
                    padding: "15px 30px",
                    borderRadius: 8,
                    fontSize: 30,
                    cursor: "pointer",
                    width: 300,
                    height: 80,
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                }}
                onClick={() => setCount(count + 1)}
            >
                Clicked {count} times
            </div>
        </div>
    );
}
