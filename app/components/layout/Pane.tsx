import { ReactNode } from "react";

interface PaneProps {
  children: ReactNode;
  className?: string;
}

/** 汎用ペインコンテナ */
export function Pane({ children, className = "" }: PaneProps) {
  return <div className={`h-full w-full overflow-hidden ${className}`}>{children}</div>;
}
