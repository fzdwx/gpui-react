import React from "react";

export function StyledApp() {
  return (
    <div style={{
      width: 400,
      height: 300,
      backgroundColor: "#ff6b6b",
      padding: 20,
      borderRadius: 8
    }}>
      <div style={{
        fontSize: 24,
        color: "#ffffff",
        fontWeight: "bold",
        marginTop: 10
      }}>
        Styled Text
      </div>
    </div>
  );
}
