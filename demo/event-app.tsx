import React, {useState, useEffect} from "react";

let clickCount = 0;

export function EventApp() {
  function handleClick() {
    clickCount++;
    console.log(`Button clicked! Count: ${clickCount}`);
  }
  let [text,setText] = useState("123123123123");

  useEffect(() => {
    const timer = setTimeout(()=>{
      console.log("测试定时1111111112111111111111111111111");
      setText(`hello world 4${new Date()}`)
      console.log("测试定时222222222222222222222")
    },1000)

    return () => {
      clearTimeout(timer)
    }
  }, [])

  useEffect(() => {
    const timer = setTimeout(()=>{
      console.log("测试定时1111111112111111111111111111111");
      setText(`hello world 4${new Date()}`)
      console.log("测试定时222222222222222222222")
    },3000)

    return () => {
      clearTimeout(timer)
    }
  }, [])

  useEffect(() => {
    const timer = setTimeout(()=>{
      console.log("测试定时1111111112111111111111111111111");
      setText(`hello world 4${new Date()}`)
      console.log("测试定时222222222222222222222")
    },4000)

    return () => {
      clearTimeout(timer)
    }
  }, [])

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
        {text}
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
