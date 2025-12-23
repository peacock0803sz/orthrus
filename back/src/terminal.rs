use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

/// PTYセッションを管理する構造体
pub struct PtySession {
    writer: Box<dyn Write + Send>,
    size: PtySize,
}

/// 全PTYセッションを管理するマネージャー
pub struct TerminalManager {
    sessions: HashMap<String, PtySession>,
    /// 出力バッチング用のバッファ（16-33ms周期で送信）
    batch_interval_ms: u64,
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
            batch_interval_ms: 16, // 約60fps
        }
    }

    /// 新しいPTYセッションを生成
    pub fn spawn(
        &mut self,
        session_id: String,
        cwd: Option<String>,
        cols: u16,
        rows: u16,
        app_handle: AppHandle,
    ) -> Result<(), String> {
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

        let mut cmd = CommandBuilder::new_default_prog();
        if let Some(dir) = cwd {
            cmd.cwd(dir);
        }

        pair.slave
            .spawn_command(cmd)
            .map_err(|e| format!("Failed to spawn command: {}", e))?;

        let writer = pair
            .master
            .take_writer()
            .map_err(|e| format!("Failed to take writer: {}", e))?;

        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| format!("Failed to clone reader: {}", e))?;

        let session = PtySession { writer, size };
        self.sessions.insert(session_id.clone(), session);

        // 出力を読み取るスレッドを起動（バッチング付き）
        let sid = session_id.clone();
        let batch_interval = Duration::from_millis(self.batch_interval_ms);

        thread::spawn(move || {
            let mut buffer = [0u8; 4096];
            let mut batch_buffer = Vec::new();
            let mut last_emit = Instant::now();

            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => {
                        // EOF - プロセス終了
                        let _ = app_handle.emit("pty_exit", (&sid, 0));
                        break;
                    }
                    Ok(n) => {
                        batch_buffer.extend_from_slice(&buffer[..n]);

                        // バッチング: 一定間隔で送信
                        if last_emit.elapsed() >= batch_interval {
                            if !batch_buffer.is_empty() {
                                let data = String::from_utf8_lossy(&batch_buffer).to_string();
                                let _ = app_handle.emit("pty_data", (&sid, data));
                                batch_buffer.clear();
                            }
                            last_emit = Instant::now();
                        }
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

    /// PTYのサイズを変更（resize間引き用のラッパーはフロント側で実装）
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

        // Note: portable-ptyではresizeはPtyPairで行う必要がある
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
        assert_eq!(manager.batch_interval_ms, 16);
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
}
