import React from "react";

let clickCount = 0;

export function EventApp() {
  function handleClick() {
    clickCount++;
    console.log(`Button clicked! Count: ${clickCount}`);
  }

  return (
    <div style={{
      display: "flex",
      flexDirection: "column",
      gap: 20,
      backgroundColor: "#1e1e1e",
      padding: 40,
      alignItems: "center"
    }}>
      <div style={{
        color: "#ffffff",
        fontSize: 24,
        fontWeight: "bold"
      }}>
        Click the button below:
      </div>
      <div style={{
        backgroundColor: "#ff6b6b",
        color: "white",
        padding: "15px 30px",
        borderRadius: 8,
        fontSize: 18,
        cursor: "pointer"
      }}>
        Clicked {clickCount} times
      </div>
    </div>
  );
}
