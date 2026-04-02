//! Goblin Terminal - Docker Backend
//!
//! Run Goblin in an isolated Docker container.

use anyhow::{Context, Result};
use bollard::container::{Config, CreateContainerOptions, LogsOptions, RemoveContainerOptions};
use bollard::image::CreateImageOptions;
use bollard::Docker;
use futures::StreamExt;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::sync::Mutex;

/// Docker terminal backend
pub struct DockerTerminal {
    docker: Docker,
    container_id: Arc<Mutex<Option<String>>>,
    workspace_dir: PathBuf,
}

impl DockerTerminal {
    /// Create a new Docker terminal
    pub async fn new(image: &str) -> Result<Self> {
        let docker = Docker::connect_with_local_defaults()
            .context("Failed to connect to Docker daemon")?;
        
        // Verify Docker is running
        docker.ping().await?;
        
        let terminal = Self {
            docker,
            container_id: Arc::new(Mutex::new(None)),
            workspace_dir: PathBuf::from("/workspace"),
        };
        
        // Create container
        terminal.create_container(image).await?;
        
        Ok(terminal)
    }

    /// Create and start a container
    async fn create_container(&self, image: &str) -> Result<()> {
        // Pull image if needed
        self.docker
            .create_image(
                Some(CreateImageOptions {
                    from_image: image,
                    ..Default::default()
                }),
                None,
                None,
            )
            .collect::<Vec<_>>()
            .await;

        // Create container
        let config = Config {
            image: Some(image),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            tty: Some(true),
            env: Some(vec![
                "TERM=xterm-256color",
                "GOBLIN_MODE=docker",
            ]),
            mounts: Some(vec![bollard::container::Mount {
                target: Some("/workspace".to_string()),
                source: Some("/tmp/goblin".to_string()),
                mount_type: Some(bollard::models::MountType::Bind),
                ..Default::default()
            }]),
            ..Default::default()
        };

        let response = self.docker
            .create_container::<String, _>(None, config)
            .await?;

        // Store container ID
        let mut id = self.container_id.lock().await;
        *id = Some(response.id);

        // Start container
        self.docker
            .start_container::<String>(&response.id, None)
            .await?;

        Ok(())
    }

    /// Execute a command
    pub async fn exec(&self, command: &str) -> Result<CommandOutput> {
        let container_id = self.container_id.lock().await;
        let id = container_id
            .as_ref()
            .context("Container not created")?
            .clone();
        drop(container_id);

        // Execute command
        let exec = self.docker
            .create_exec(
                &id,
                bollard::container::ExecCreateParams {
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    cmd: Some(vec!["sh", "-c", command]),
                    ..Default::default()
                },
            )
            .await?;

        let output = self.docker
            .start_exec(&exec.id, None)
            .await?;

        let mut stdout = String::new();
        let mut stderr = String::new();
        let mut exit_code = 0;

        match output {
            bollard::exec::StartExecResults::Attached { output, .. } => {
                let mut stream = output;
                while let Some(Ok(msg)) = stream.next().await {
                    match msg {
                        bollard::exec::ExecResults::StdOut { message } => {
                            stdout.push_str(&String::from_utf8_lossy(&message));
                        }
                        bollard::exec::ExecResults::StdErr { message } => {
                            stderr.push_str(&String::from_utf8_lossy(&message));
                        }
                        bollard::exec::ExecResults::ExitCode { exit_code: code } => {
                            exit_code = code.unwrap_or(0);
                        }
                        _ => {}
                    }
                }
            }
            bollard::exec::StartExecResults::Detached => {
                // Command started but we're detached
            }
        }

        Ok(CommandOutput {
            stdout,
            stderr,
            exit_code,
        })
    }

    /// Copy files to container
    pub async fn copy_to_container(&self, src: &PathBuf, dest: &str) -> Result<()> {
        let container_id = self.container_id.lock().await;
        let id = container_id
            .as_ref()
            .context("Container not created")?;
        
        // Read file content
        let content = tokio::fs::read(src).await?;
        
        // Create a tar archive in memory
        let mut archive = Vec::new();
        {
            let mut tar = tar::Builder::new(&mut archive);
            let mut header = tar::Header::new_gnu();
            header.set_path(src.file_name().unwrap_or_default().to_str().unwrap_or("file"))?;
            header.set_size(content.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            tar.append(&header, content.as_slice())?;
        }
        
        // Copy to container (simplified - would need proper tar transfer)
        // For now, just use exec with cat
        let file_name = src.file_name().unwrap_or_default().to_str().unwrap_or("file");
        let dest_path = format!("{}/{}", dest, file_name);
        
        // Encode content as base64 and decode in container
        let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &content);
        self.exec(&format!("echo '{}' | base64 -d > {}", encoded, dest_path)).await?;
        
        Ok(())
    }

    /// Copy files from container
    pub async fn copy_from_container(&self, src: &str, dest: &PathBuf) -> Result<()> {
        let container_id = self.container_id.lock().await;
        let id = container_id
            .as_ref()
            .context("Container not created")?;
        
        // Read file from container
        let output = self.exec(&format!("cat {}", src)).await?;
        
        if output.exit_code == 0 {
            tokio::fs::write(dest, output.stdout).await?;
        }
        
        Ok(())
    }

    /// Get container status
    pub async fn status(&self) -> Result<ContainerStatus> {
        let container_id = self.container_id.lock().await;
        let id = container_id
            .as_ref()
            .context("Container not created")?;
        
        let info = self.docker.inspect_container(id, None).await?;
        
        let state = info
            .state
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        
        Ok(ContainerStatus {
            container_id: id.clone(),
            state,
            started_at: info.created.map(|c| c.to_string()),
        })
    }

    /// Cleanup container
    pub async fn cleanup(&self) -> Result<()> {
        let mut container_id = self.container_id.lock().await;
        
        if let Some(id) = container_id.take() {
            // Stop container
            let _ = self.docker.stop_container(&id, None).await;
            
            // Remove container
            self.docker
                .remove_container(
                    &id,
                    Some(RemoveContainerOptions {
                        force: true,
                        ..Default::default()
                    }),
                )
                .await?;
        }
        
        Ok(())
    }
}

impl Drop for DockerTerminal {
    fn drop(&mut self) {
        // Note: This is synchronous, but cleanup is async
        // In production, you'd want to handle this properly
        // For now, container will be orphaned
    }
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

/// Container status
#[derive(Debug, Clone)]
pub struct ContainerStatus {
    /// Container ID
    pub container_id: String,
    /// Current state
    pub state: String,
    /// When container started
    pub started_at: Option<String>,
}
