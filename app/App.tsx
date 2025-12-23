import { useState, useCallback } from "react";
import { Terminal } from "./components/Terminal";
import "./App.css";

function App() {
  const [sessionId] = useState(() => crypto.randomUUID());
  const [exited, setExited] = useState(false);

  const handleExit = useCallback((code: number) => {
    console.log("Terminal exited with code:", code);
    setExited(true);
  }, []);

  return (
    <main className="h-screen w-screen flex flex-col bg-gray-900">
      <header className="h-8 bg-gray-800 flex items-center px-4 text-gray-300 text-sm shrink-0">
        Orthrus - Terminal
      </header>
      <div className="flex-1 min-h-0">
        {!exited ? (
          <Terminal sessionId={sessionId} onExit={handleExit} />
        ) : (
          <div className="flex items-center justify-center h-full text-gray-400">
            Terminal session ended
          </div>
        )}
      </div>
    </main>
  );
}

export default App;
