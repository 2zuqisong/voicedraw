import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";

function App() {
  const [msg, setMsg] = useState("");

  const testInvoke = async () => {
    const response: string = await invoke("greet", { name: "世界" });
    setMsg(response);
  };

  return (
    <div>
      <button onClick={testInvoke}>测试 Rust 通信</button>
      <p>{msg}</p>
    </div>
  );
}

export default App;