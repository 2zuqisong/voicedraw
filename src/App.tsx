import { useEffect } from "react";
import CanvasView from "./components/canvas/CanvasView";
import PixelCanvas from "./components/canvas/PixelCanvas";
import PixelChatBubble from "./components/status/PixelChatBubble";
import CanvasOverlay from "./components/canvas/CanvasOverlay";
import OperationPreview from "./components/canvas/OperationPreview";
import ChatBubble from "./components/status/ChatBubble";
import TopBar from "./components/layout/TopBar";
import VoiceBar from "./components/layout/VoiceBar";
import ShapePalette from "./components/layout/ShapePalette";
import Toast from "./components/status/Toast";
import { useAppStore } from "./store";
import "./App.css";

function App() {
  const canvasState = useAppStore((s) => s.canvasState);
  const canvasMode = useAppStore((s) => s.canvasMode);
  const initEventListener = useAppStore((s) => s._initEventListener);

  useEffect(() => {
    initEventListener();
  }, []);

  return (
    <div className="app-container">
      <Toast />
      <TopBar />
      <div style={{ flex: 1, position: "relative", overflow: "hidden" }}>
        {canvasMode === "pixel" ? (
          <>
            <PixelCanvas />
            <PixelChatBubble />
            <VoiceBar />
          </>
        ) : (
          <>
            <CanvasView canvasState={canvasState} />
            <CanvasOverlay />
            <ShapePalette />
            <OperationPreview />
            <ChatBubble />
            <VoiceBar />
          </>
        )}
      </div>
    </div>
  );
}

export default App;