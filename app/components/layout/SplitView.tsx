import { useState, useRef, useCallback, useEffect, ReactNode } from "react";

interface SplitViewProps {
  left: ReactNode;
  right: ReactNode;
  defaultRatio?: number; // 0-1, デフォルト 0.5
  minWidth?: number; // 最小ペイン幅 (px)
}

/** 水平分割ビュー（ドラッグでリサイズ可能） */
export function SplitView({ left, right, defaultRatio = 0.5, minWidth = 200 }: SplitViewProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [ratio, setRatio] = useState(defaultRatio);
  const [isDragging, setIsDragging] = useState(false);

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    setIsDragging(true);
  }, []);

  const handleMouseMove = useCallback(
    (e: MouseEvent) => {
      if (!isDragging || !containerRef.current) return;

      const rect = containerRef.current.getBoundingClientRect();
      const newRatio = (e.clientX - rect.left) / rect.width;

      // 最小幅を確保するための制約
      const minRatio = minWidth / rect.width;
      const maxRatio = 1 - minRatio;
      setRatio(Math.max(minRatio, Math.min(maxRatio, newRatio)));
    },
    [isDragging, minWidth]
  );

  const handleMouseUp = useCallback(() => {
    setIsDragging(false);
  }, []);

  // グローバルマウスイベントの登録
  useEffect(() => {
    if (isDragging) {
      document.addEventListener("mousemove", handleMouseMove);
      document.addEventListener("mouseup", handleMouseUp);
      // ドラッグ中はテキスト選択を無効化
      document.body.style.userSelect = "none";
      document.body.style.cursor = "col-resize";
    }

    return () => {
      document.removeEventListener("mousemove", handleMouseMove);
      document.removeEventListener("mouseup", handleMouseUp);
      document.body.style.userSelect = "";
      document.body.style.cursor = "";
    };
  }, [isDragging, handleMouseMove, handleMouseUp]);

  return (
    <div ref={containerRef} className="flex h-full w-full">
      {/* 左ペイン */}
      <div style={{ width: `${ratio * 100}%` }} className="h-full overflow-hidden">
        {left}
      </div>

      {/* スプリッター */}
      <div
        className="w-1 bg-gray-700 cursor-col-resize hover:bg-blue-500 active:bg-blue-600 transition-colors flex-shrink-0"
        onMouseDown={handleMouseDown}
      />

      {/* 右ペイン */}
      <div style={{ width: `${(1 - ratio) * 100}%` }} className="h-full overflow-hidden">
        {right}
      </div>
    </div>
  );
}
