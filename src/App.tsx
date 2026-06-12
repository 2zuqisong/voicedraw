import { useEffect } from "react";
import CanvasView from "./components/canvas/CanvasView";
import TopBar from "./components/layout/TopBar";
import VoiceBar from "./components/layout/VoiceBar";
import { useAppStore } from "./store";
import "./App.css";

function App() {
  const canvasState = useAppStore((s) => s.canvasState);
  const initEventListener = useAppStore((s) => s._initEventListener);

  useEffect(() => {
    initEventListener();
  }, []);

  return (
    <div className="app-container">
      <TopBar />
      <CanvasView canvasState={canvasState} />
      <VoiceBar />
    </div>
  );
}

export default App;