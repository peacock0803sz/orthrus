/** sphinx-autobuildサーバー設定 */
export interface ServerConfig {
  port: number;
}

/** Sphinx関連設定 */
export interface SphinxConfig {
  source_dir: string;
  build_dir: string;
  server: ServerConfig;
  extra_args: string[];
}

/** Python環境設定 */
export interface PythonConfig {
  interpreter: string;
}

/** エディタ設定 */
export interface EditorConfig {
  command: string;
}

/** カラースキーム（xterm.js ITheme互換） */
export interface ColorScheme {
  background?: string;
  foreground?: string;
  cursor?: string;
  cursor_accent?: string;
  selection_background?: string;
  selection_foreground?: string;
  // ANSI colors (0-7)
  black?: string;
  red?: string;
  green?: string;
  yellow?: string;
  blue?: string;
  magenta?: string;
  cyan?: string;
  white?: string;
  // Bright ANSI colors (8-15)
  bright_black?: string;
  bright_red?: string;
  bright_green?: string;
  bright_yellow?: string;
  bright_blue?: string;
  bright_magenta?: string;
  bright_cyan?: string;
  bright_white?: string;
}

/** ターミナル設定 */
export interface TerminalConfig {
  shell?: string;
  font_family?: string;
  font_size?: number;
  theme_file?: string;
  color_scheme?: ColorScheme;
}

/** プロジェクト設定全体 */
export interface ProjectConfig {
  sphinx: SphinxConfig;
  python: PythonConfig;
  editor: EditorConfig;
  terminal: TerminalConfig;
}
