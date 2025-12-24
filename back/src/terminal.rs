use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

/// シェルパスを決定する
/// 優先順位: 設定値 > $SHELL環境変数 > /bin/sh
fn detect_shell(config_shell: Option<&str>) -> String {
    // 設定で指定されていれば優先
    if let Some(shell) = config_shell {
        return shell.to_string();
    }

    // $SHELL 環境変数
    if let Ok(shell) = std::env::var("SHELL") {
        return shell;
    }

    // フォールバック
    "/bin/sh".to_string()
}

/// PTYセッションを管理する構造体
pub struct PtySession {
    writer: Box<dyn Write + Send>,
    size: PtySize,
    #[allow(dead_code)]
    child: Box<dyn Child + Send + Sync>,
    #[allow(dead_code)]
    master: Box<dyn MasterPty + Send>,
}

/// 全PTYセッションを管理するマネージャー
pub struct TerminalManager {
    sessions: HashMap<String, PtySession>,
}

impl Default for TerminalManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    /// 新しいPTYセッションを生成
    pub fn spawn(
        &mut self,
        session_id: String,
        cwd: Option<String>,
        shell: Option<String>,
        cols: u16,
        rows: u16,
        app_handle: AppHandle,
    ) -> Result<(), String> {
        // 既に同じセッションが存在する場合はスキップ（React StrictMode対策）
        if self.sessions.contains_key(&session_id) {
            return Ok(());
        }

        let pty_system = native_pty_system();

        let size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };

        let pair = pty_system
            .openpty(size)
            .map_err(|e| format!("Failed to open pty: {}", e))?;

        // シェルを検出してログインシェルとして起動
        let shell_path = detect_shell(shell.as_deref());
        let mut cmd = CommandBuilder::new(&shell_path);
        cmd.arg("-l");

        if let Some(ref dir) = cwd {
            cmd.cwd(dir);
        }

        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");
        cmd.env("SHELL", &shell_path);

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("Failed to spawn command: {}", e))?;

        // macOS: spawn後の短いスリープでレースコンディション回避
        thread::sleep(Duration::from_millis(50));

        // slaveをdrop（親で保持するとEOF問題が発生）
        drop(pair.slave);

        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| format!("Failed to clone reader: {}", e))?;

        let writer = pair
            .master
            .take_writer()
            .map_err(|e| format!("Failed to take writer: {}", e))?;

        let session = PtySession {
            writer,
            size,
            child,
            master: pair.master,
        };
        self.sessions.insert(session_id.clone(), session);

        // 出力読み取りスレッド（即時送信）
        let sid = session_id.clone();

        thread::spawn(move || {
            let mut buffer = [0u8; 4096];

            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => {
                        let _ = app_handle.emit("pty_exit", (&sid, 0));
                        break;
                    }
                    Ok(n) => {
                        // 読み取ったデータを即座に送信
                        let data = String::from_utf8_lossy(&buffer[..n]).to_string();
                        let _ = app_handle.emit("pty_data", (&sid, data));
                    }
                    Err(_) => {
                        let _ = app_handle.emit("pty_exit", (&sid, 1));
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// PTYにデータを書き込む
    pub fn write(&mut self, session_id: &str, data: &[u8]) -> Result<(), String> {
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| format!("Session not found: {}", session_id))?;

        session
            .writer
            .write_all(data)
            .map_err(|e| format!("Failed to write: {}", e))?;

        session
            .writer
            .flush()
            .map_err(|e| format!("Failed to flush: {}", e))?;

        Ok(())
    }

    /// PTYのサイズを変更
    pub fn resize(&mut self, session_id: &str, cols: u16, rows: u16) -> Result<(), String> {
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| format!("Session not found: {}", session_id))?;

        session.size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };

        // Note: portable-ptyではresizeはmasterから行う必要がある
        // 現在の実装ではsizeを保存するのみ

        Ok(())
    }

    /// セッションを終了
    pub fn kill(&mut self, session_id: &str) -> Result<(), String> {
        self.sessions
            .remove(session_id)
            .ok_or_else(|| format!("Session not found: {}", session_id))?;
        Ok(())
    }
}

/// グローバルなTerminalManagerへのアクセス用
pub type SharedTerminalManager = Arc<Mutex<TerminalManager>>;

pub fn create_terminal_manager() -> SharedTerminalManager {
    Arc::new(Mutex::new(TerminalManager::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_manager_creation() {
        let manager = TerminalManager::new();
        assert!(manager.sessions.is_empty());
    }

    #[test]
    fn test_write_to_nonexistent_session() {
        let mut manager = TerminalManager::new();
        let result = manager.write("nonexistent", b"test");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Session not found"));
    }

    #[test]
    fn test_resize_nonexistent_session() {
        let mut manager = TerminalManager::new();
        let result = manager.resize("nonexistent", 80, 24);
        assert!(result.is_err());
    }

    #[test]
    fn test_kill_nonexistent_session() {
        let mut manager = TerminalManager::new();
        let result = manager.kill("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_shell_with_config() {
        // 設定値が優先される
        let shell = detect_shell(Some("/opt/homebrew/bin/fish"));
        assert_eq!(shell, "/opt/homebrew/bin/fish");
    }

    #[test]
    fn test_detect_shell_from_env() {
        // 設定がない場合は $SHELL を使用
        let original = std::env::var("SHELL").ok();
        std::env::set_var("SHELL", "/usr/local/bin/zsh");
        let shell = detect_shell(None);
        assert_eq!(shell, "/usr/local/bin/zsh");

        // 環境変数を元に戻す
        match original {
            Some(v) => std::env::set_var("SHELL", v),
            None => std::env::remove_var("SHELL"),
        }
    }

    #[test]
    fn test_detect_shell_fallback() {
        // $SHELL がない場合は /bin/sh
        let original = std::env::var("SHELL").ok();
        std::env::remove_var("SHELL");
        let shell = detect_shell(None);
        assert_eq!(shell, "/bin/sh");

        // 環境変数を元に戻す
        if let Some(v) = original {
            std::env::set_var("SHELL", v);
        }
    }
}
