import { invoke } from "@tauri-apps/api/tauri";
import { Button } from "primereact/button";
import { InputText } from "primereact/inputtext";
import { useState } from "react";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    setGreetMsg(await invoke("greet", { name }));
  }

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        greet();
      }}
    >
      <InputText
        id="greet-input"
        onChange={(e) => setName(e.currentTarget.value)}
        placeholder="Enter a name..."
      />
      <Button type="submit">Greet</Button>
      <p>{greetMsg}</p>
    </form>
  );
}

export default App;
