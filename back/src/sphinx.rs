use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Emitter};

/// sphinx-autobuildプロセス情報
pub struct SphinxProcess {
    child: Child,
    port: u16,
}

/// Sphinxプロセスマネージャ
pub struct SphinxManager {
    processes: HashMap<String, SphinxProcess>,
}

impl SphinxManager {
    pub fn new() -> Self {
        Self {
            processes: HashMap::new(),
        }
    }

    /// 利用可能なポートを検索
    fn find_available_port() -> Result<u16, String> {
        TcpListener::bind("127.0.0.1:0")
            .map_err(|e| format!("ポートの検索に失敗: {}", e))?
            .local_addr()
            .map_err(|e| format!("アドレスの取得に失敗: {}", e))
            .map(|addr| addr.port())
    }

    /// sphinx-autobuildを起動
    pub fn start(
        &mut self,
        session_id: String,
        project_path: String,
        source_dir: String,
        build_dir: String,
        python_path: String,
        requested_port: u16,
        app_handle: AppHandle,
    ) -> Result<u16, String> {
        // 既存セッションがあれば停止
        if self.processes.contains_key(&session_id) {
            self.stop(&session_id)?;
        }

        let port = if requested_port == 0 {
            Self::find_available_port()?
        } else {
            requested_port
        };

        let source_path = std::path::Path::new(&project_path).join(&source_dir);
        let build_path = std::path::Path::new(&project_path).join(&build_dir);

        // sphinx-autobuildを起動
        let mut child = Command::new(&python_path)
            .args([
                "-m",
                "sphinx_autobuild",
                source_path.to_str().unwrap(),
                build_path.to_str().unwrap(),
                "--port",
                &port.to_string(),
                "--host",
                "127.0.0.1",
                "--open-browser=false",
            ])
            .current_dir(&project_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("sphinx-autobuildの起動に失敗: {}", e))?;

        // stderrを監視してビルドイベントを通知
        let stderr = child.stderr.take();
        let sid = session_id.clone();
        let handle = app_handle.clone();

        if let Some(stderr) = stderr {
            thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        // ビルド完了を検出
                        if line.contains("build succeeded") || line.contains("waiting for changes")
                        {
                            let _ = handle.emit("sphinx_built", &sid);
                        }
                        // エラーを検出
                        if line.contains("ERROR") || line.contains("error:") {
                            let _ = handle.emit("sphinx_error", (&sid, &line));
                        }
                    }
                }
            });
        }

        let process = SphinxProcess { child, port };
        self.processes.insert(session_id.clone(), process);

        // サーバー起動を通知
        let _ = app_handle.emit("sphinx_started", (&session_id, port));

        Ok(port)
    }

    /// sphinx-autobuildを停止
    pub fn stop(&mut self, session_id: &str) -> Result<(), String> {
        if let Some(mut process) = self.processes.remove(session_id) {
            process
                .child
                .kill()
                .map_err(|e| format!("プロセスの停止に失敗: {}", e))?;
        }
        Ok(())
    }

    /// ポートを取得
    pub fn get_port(&self, session_id: &str) -> Option<u16> {
        self.processes.get(session_id).map(|p| p.port)
    }

    /// 実行中かどうか
    pub fn is_running(&self, session_id: &str) -> bool {
        self.processes.contains_key(session_id)
    }
}

impl Drop for SphinxManager {
    fn drop(&mut self) {
        // 全プロセスを停止
        for (_, mut process) in self.processes.drain() {
            let _ = process.child.kill();
        }
    }
}

pub type SharedSphinxManager = Arc<Mutex<SphinxManager>>;

pub fn create_sphinx_manager() -> SharedSphinxManager {
    Arc::new(Mutex::new(SphinxManager::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphinx_manager_creation() {
        let manager = SphinxManager::new();
        assert!(!manager.is_running("test"));
    }

    #[test]
    fn test_find_available_port() {
        let port = SphinxManager::find_available_port().unwrap();
        assert!(port > 0);
    }

    #[test]
    fn test_stop_nonexistent_session() {
        let mut manager = SphinxManager::new();
        // 存在しないセッションの停止は成功する
        assert!(manager.stop("nonexistent").is_ok());
    }
}
