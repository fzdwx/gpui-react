import React, { useState, useEffect } from "react";

let clickCount = 0;

export function EventApp() {
    function handleClick() {
        clickCount++;
        console.log(`Button clicked! Count: ${clickCount}`);
    }
    let [text, setText] = useState("123123123123");

    useEffect(() => {
        const timer = setTimeout(() => {
            setText(`hello world 4${new Date()}`);
        }, 1000);

        return () => {
            clearTimeout(timer);
        };
    }, []);

    useEffect(() => {
        const timer = setTimeout(() => {
            setText(`hello world 4${new Date()}`);
        }, 3000);

        return () => {
            clearTimeout(timer);
        };
    }, []);

    useEffect(() => {
        const timer = setTimeout(() => {
            setText(`222222hello world 4${new Date()}`);
        }, 4000);

        return () => {
            clearTimeout(timer);
        };
    }, []);

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
                {text}
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
                onClick={handleClick}
            >
                Clicked {clickCount} times
            </div>
        </div>
    );
}
