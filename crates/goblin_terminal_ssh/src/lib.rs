//! Goblin Terminal - SSH Backend
//!
//! Run Goblin on a remote server via SSH.

use anyhow::{Context, Result};
use ssh2::Session;
use std::io::{Read, Write as IoWrite};
use std::net::TcpStream;
use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::net::TcpStream as TokioTcpStream;
use tokio::sync::Mutex;

/// SSH terminal backend
pub struct SshTerminal {
    session: Arc<Mutex<Option<SshSession>>>,
    host: String,
    user: String,
}

impl SshTerminal {
    /// Connect to remote server
    pub async fn connect(host: &str, port: u16, user: &str, password: Option<&str>, key_path: Option<&Path>) -> Result<Self> {
        let terminal = Self {
            session: Arc::new(Mutex::new(None)),
            host: host.to_string(),
            user: user.to_string(),
        };
        
        terminal.establish_connection(port, password, key_path).await?;
        
        Ok(terminal)
    }

    /// Establish SSH connection
    async fn establish_connection(&self, port: u16, password: Option<&str>, key_path: Option<&Path>) -> Result<()> {
        // Connect to TCP socket
        let addr = format!("{}:{}", self.host, port);
        let tcp = TcpStream::connect(&addr).context("Failed to connect to SSH server")?;
        
        // Create SSH session
        let mut session = Session::new().context("Failed to create SSH session")?;
        session.set_tcp_stream(tcp);
        session.handshake().context("SSH handshake failed")?;
        
        // Authenticate
        if let Some(key_path) = key_path {
            // Key-based authentication
            session.userauth_pubkey_file(&self.user, None, key_path, None)
                .context("SSH key authentication failed")?;
        } else if let Some(password) = password {
            // Password authentication
            session.userauth_password(&self.user, password)
                .context("SSH password authentication failed")?;
        } else {
            anyhow::bail!("Either password or key_path must be provided");
        }
        
        if !session.authenticated() {
            anyhow::bail!("SSH authentication failed");
        }
        
        // Store session
        let ssh_session = SshSession {
            session,
            _port: port,
        };
        
        let mut session_lock = self.session.lock().await;
        *session_lock = Some(ssh_session);
        
        Ok(())
    }

    /// Execute a command
    pub async fn exec(&self, command: &str) -> Result<CommandOutput> {
        let session_lock = self.session.lock().await;
        let ssh_session = session_lock
            .as_ref()
            .context("Not connected")?;
        
        // Execute command
        let mut channel = ssh_session.session.channel_session()
            .context("Failed to open channel")?;
        
        channel.exec(command).context("Failed to execute command")?;
        
        // Read stdout
        let mut stdout = String::new();
        channel.read_to_string(&mut stdout).await?;
        
        // Read stderr
        let mut stderr = String::new();
        channel.stderr().read_to_string(&mut stderr).await?;
        
        // Get exit code
        channel.wait_close().await;
        let exit_code = channel.exit_status().unwrap_or(-1);
        
        Ok(CommandOutput {
            stdout,
            stderr,
            exit_code,
        })
    }

    /// Execute a command and stream output
    pub async fn exec_stream<F>(&self, command: &str, mut callback: F) -> Result<CommandOutput>
    where
        F: FnMut(String) + Send,
    {
        let session_lock = self.session.lock().await;
        let ssh_session = session_lock
            .as_ref()
            .context("Not connected")?;
        
        // Execute command
        let mut channel = ssh_session.session.channel_session()
            .context("Failed to open channel")?;
        
        channel.exec(command).context("Failed to execute command")?;
        
        // Read stdout line by line
        let mut reader = tokio::io::BufReader::new(channel);
        let mut line = String::new();
        let mut stdout = String::new();
        
        while reader.read_line(&mut line).await? > 0 {
            callback(line.clone());
            stdout.push_str(&line);
            line.clear();
        }
        
        // Get exit code
        let exit_code = channel.exit_status().unwrap_or(-1);
        
        Ok(CommandOutput {
            stdout,
            stderr: String::new(),
            exit_code,
        })
    }

    /// Copy local file to remote
    pub async fn copy_to_remote(&self, local_path: &Path, remote_path: &str) -> Result<()> {
        let session_lock = self.session.lock().await;
        let ssh_session = session_lock
            .as_ref()
            .context("Not connected")?;
        
        // Read local file
        let content = tokio::fs::read(local_path).await?;
        
        // Create remote file via SFTP
        let sftp = ssh_session.session.sftp()
            .context("Failed to start SFTP session")?;
        
        let mut remote_file = sftp.create(remote_path)
            .context("Failed to create remote file")?;
        
        remote_file.write_all(&content)
            .context("Failed to write to remote file")?;
        
        Ok(())
    }

    /// Copy remote file to local
    pub async fn copy_from_remote(&self, remote_path: &str, local_path: &Path) -> Result<()> {
        let session_lock = self.session.lock().await;
        let ssh_session = session_lock
            .as_ref()
            .context("Not connected")?;
        
        // Open remote file via SFTP
        let sftp = ssh_session.session.sftp()
            .context("Failed to start SFTP session")?;
        
        let mut remote_file = sftp.open(remote_path)
            .context("Failed to open remote file")?;
        
        // Read content
        let mut content = Vec::new();
        remote_file.read_to_end(&mut content)
            .context("Failed to read remote file")?;
        
        // Write to local
        tokio::fs::write(local_path, content).await?;
        
        Ok(())
    }

    /// Check if still connected
    pub async fn is_connected(&self) -> bool {
        let session_lock = self.session.lock().await;
        session_lock
            .as_ref()
            .map(|s| s.session.authenticated())
            .unwrap_or(false)
    }

    /// Get server info
    pub async fn server_info(&self) -> Result<ServerInfo> {
        let session_lock = self.session.lock().await;
        let ssh_session = session_lock
            .as_ref()
            .context("Not connected")?;
        
        // Get hostname
        let mut channel = ssh_session.session.channel_session()?;
        channel.exec("hostname")?;
        let mut hostname = String::new();
        channel.read_to_string(&mut hostname).await?;
        hostname = hostname.trim().to_string();
        
        // Get OS info
        let mut channel = ssh_session.session.channel_session()?;
        channel.exec("uname -a")?;
        let mut os_info = String::new();
        channel.read_to_string(&mut os_info).await?;
        os_info = os_info.trim().to_string();
        
        // Get working directory
        let mut channel = ssh_session.session.channel_session()?;
        channel.exec("pwd")?;
        let mut pwd = String::new();
        channel.read_to_string(&mut pwd).await?;
        pwd = pwd.trim().to_string();
        
        Ok(ServerInfo {
            host: self.host.clone(),
            user: self.user.clone(),
            hostname,
            os_info,
            working_directory: pwd,
        })
    }

    /// Disconnect
    pub async fn disconnect(&self) -> Result<()> {
        let mut session_lock = self.session.lock().await;
        if let Some(ssh_session) = session_lock.take() {
            ssh_session.session.disconnect(None, "Goodbye", None)?;
        }
        Ok(())
    }
}

impl Drop for SshTerminal {
    fn drop(&mut self) {
        // Note: This is synchronous, proper cleanup should be async
    }
}

/// Internal SSH session wrapper
struct SshSession {
    session: Session,
    _port: u16,
}

/// Command output
#[derive(Debug, Clone)]
pub struct CommandOutput {
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Exit code
    pub exit_code: i32,
}

/// Server information
#[derive(Debug, Clone)]
pub struct ServerInfo {
    /// Host address
    pub host: String,
    /// Username
    pub user: String,
    /// Hostname
    pub hostname: String,
    /// OS information
    pub os_info: String,
    /// Working directory
    pub working_directory: String,
}
