import { useState, useCallback, useEffect } from "react";
import { Terminal } from "./components/Terminal";
import { Preview } from "./components/Preview";
import { SplitView, Pane } from "./components/layout";
import { useProjectDialog } from "./hooks/useProjectDialog";
import { useProjectConfig } from "./hooks/useProjectConfig";
import "./App.css";

function App() {
  const [sessionId] = useState(() => crypto.randomUUID());
  const [exited, setExited] = useState(false);

  // プロジェクト選択
  const { projectPath, showDialog } = useProjectDialog();
  const { config, loading: configLoading } = useProjectConfig(projectPath);

  // TODO: Phase 3でsphinx-autobuildのURLを設定
  const [previewUrl] = useState<string | null>(null);

  const handleExit = useCallback((code: number) => {
    console.log("Terminal exited with code:", code);
    setExited(true);
  }, []);

  // 起動時にプロジェクト選択ダイアログを表示
  useEffect(() => {
    if (!projectPath) {
      showDialog();
    }
  }, []);

  return (
    <main className="h-screen w-screen flex flex-col bg-gray-900">
      <header className="h-8 bg-gray-800 flex items-center justify-between px-4 text-gray-300 text-sm shrink-0">
        <span className="flex items-center gap-2">
          Orthrus
          {projectPath && (
            <span className="text-gray-500 text-xs truncate max-w-md">{projectPath}</span>
          )}
        </span>
        <div className="flex items-center gap-4">
          {configLoading && <span className="text-yellow-400 text-xs">Loading config...</span>}
          {config && <span className="text-green-400 text-xs">Project loaded</span>}
          <button
            onClick={showDialog}
            className="px-2 py-0.5 bg-gray-700 hover:bg-gray-600 rounded text-xs transition-colors"
          >
            Open Project
          </button>
        </div>
      </header>
      <div className="flex-1 min-h-0">
        <SplitView
          left={
            <Pane>
              <Preview url={previewUrl} />
            </Pane>
          }
          right={
            <Pane>
              {!exited ? (
                <Terminal sessionId={sessionId} cwd={projectPath ?? undefined} onExit={handleExit} />
              ) : (
                <div className="flex items-center justify-center h-full text-gray-400">
                  Terminal session ended
                </div>
              )}
            </Pane>
          }
        />
      </div>
    </main>
  );
}

export default App;
