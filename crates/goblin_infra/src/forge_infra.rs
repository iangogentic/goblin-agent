use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;
use std::sync::Arc;

use bytes::Bytes;
use goblin_app::{
    CommandInfra, DirectoryReaderInfra, EnvironmentInfra, FileDirectoryInfra, FileInfoInfra,
    FileReaderInfra, FileRemoverInfra, FileWriterInfra, GrpcInfra, HttpInfra, McpServerInfra,
    StrategyFactory, UserInfra, WalkerInfra,
};
use goblin_domain::{
    AuthMethod, CommandOutput, FileInfo as FileInfoData, McpServerConfig, ProviderId, URLParamSpec,
};
use reqwest::header::HeaderMap;
use reqwest::{Response, Url};
use reqwest_eventsource::EventSource;

use crate::auth::{AnyAuthStrategy, GoblinAuthStrategyFactory};
use crate::console::StdConsoleWriter;
use crate::env::GoblinEnvironmentInfra;
use crate::executor::GoblinCommandExecutorService;
use crate::fs_create_dirs::GoblinCreateDirsService;
use crate::fs_meta::GoblinFileMetaService;
use crate::fs_read::GoblinFileReadService;
use crate::fs_read_dir::GoblinDirectoryReaderService;
use crate::fs_remove::GoblinFileRemoveService;
use crate::fs_write::GoblinFileWriteService;
use crate::grpc::GoblinGrpcClient;
use crate::http::GoblinHttpInfra;
use crate::inquire::GoblinInquire;
use crate::mcp_client::GoblinMcpClient;
use crate::mcp_server::GoblinMcpServer;
use crate::walker::GoblinWalkerService;

#[derive(Clone)]
pub struct GoblinInfra {
    // TODO: Drop the "Service" suffix. Use names like GoblinFileReader, GoblinFileWriter,
    // GoblinHttpClient etc.
    file_read_service: Arc<GoblinFileReadService>,
    file_write_service: Arc<GoblinFileWriteService>,
    file_remove_service: Arc<GoblinFileRemoveService>,
    config_infra: Arc<GoblinEnvironmentInfra>,
    file_meta_service: Arc<GoblinFileMetaService>,
    create_dirs_service: Arc<GoblinCreateDirsService>,
    directory_reader_service: Arc<GoblinDirectoryReaderService>,
    command_executor_service: Arc<GoblinCommandExecutorService>,
    inquire_service: Arc<GoblinInquire>,
    mcp_server: GoblinMcpServer,
    walker_service: Arc<GoblinWalkerService>,
    http_service: Arc<GoblinHttpInfra<GoblinFileWriteService>>,
    strategy_factory: Arc<GoblinAuthStrategyFactory>,
    grpc_client: Arc<GoblinGrpcClient>,
    output_printer: Arc<StdConsoleWriter>,
}

impl GoblinInfra {
    pub fn new(cwd: PathBuf) -> Self {
        let config_infra = Arc::new(GoblinEnvironmentInfra::new(cwd));
        let env = config_infra.get_environment();
        let config = config_infra.get_config();

        let file_write_service = Arc::new(GoblinFileWriteService::new());
        let http_service = Arc::new(GoblinHttpInfra::new(
            config.clone(),
            file_write_service.clone(),
        ));
        let file_read_service = Arc::new(GoblinFileReadService::new());
        let file_meta_service = Arc::new(GoblinFileMetaService);
        let directory_reader_service = Arc::new(GoblinDirectoryReaderService::new(
            config.max_parallel_file_reads,
        ));
        let grpc_client = Arc::new(GoblinGrpcClient::new(
            config
                .services_url
                .parse()
                .expect("services_url must be a valid URL"),
        ));
        let output_printer = Arc::new(StdConsoleWriter::default());

        Self {
            file_read_service,
            file_write_service,
            file_remove_service: Arc::new(GoblinFileRemoveService::new()),
            config_infra,
            file_meta_service,
            create_dirs_service: Arc::new(GoblinCreateDirsService),
            directory_reader_service,
            command_executor_service: Arc::new(GoblinCommandExecutorService::new(
                env.clone(),
                output_printer.clone(),
            )),
            inquire_service: Arc::new(GoblinInquire::new()),
            mcp_server: GoblinMcpServer,
            walker_service: Arc::new(GoblinWalkerService::new()),
            strategy_factory: Arc::new(GoblinAuthStrategyFactory::new()),
            http_service,
            grpc_client,
            output_printer,
        }
    }
}

impl EnvironmentInfra for GoblinInfra {
    type Config = goblin_config::GoblinConfig;

    fn get_env_var(&self, key: &str) -> Option<String> {
        self.config_infra.get_env_var(key)
    }

    fn get_env_vars(&self) -> BTreeMap<String, String> {
        self.config_infra.get_env_vars()
    }

    fn get_environment(&self) -> goblin_domain::Environment {
        self.config_infra.get_environment()
    }

    fn get_config(&self) -> goblin_config::GoblinConfig {
        self.config_infra.get_config()
    }

    async fn update_environment(
        &self,
        ops: Vec<goblin_domain::ConfigOperation>,
    ) -> anyhow::Result<()> {
        self.config_infra.update_environment(ops).await
    }
}

#[async_trait::async_trait]
impl FileReaderInfra for GoblinInfra {
    async fn read_utf8(&self, path: &Path) -> anyhow::Result<String> {
        self.file_read_service.read_utf8(path).await
    }

    fn read_batch_utf8(
        &self,
        batch_size: usize,
        paths: Vec<PathBuf>,
    ) -> impl futures::Stream<Item = (PathBuf, anyhow::Result<String>)> + Send {
        self.file_read_service.read_batch_utf8(batch_size, paths)
    }

    async fn read(&self, path: &Path) -> anyhow::Result<Vec<u8>> {
        self.file_read_service.read(path).await
    }

    async fn range_read_utf8(
        &self,
        path: &Path,
        start_line: u64,
        end_line: u64,
    ) -> anyhow::Result<(String, FileInfoData)> {
        self.file_read_service
            .range_read_utf8(path, start_line, end_line)
            .await
    }
}

#[async_trait::async_trait]
impl FileWriterInfra for GoblinInfra {
    async fn write(&self, path: &Path, contents: Bytes) -> anyhow::Result<()> {
        self.file_write_service.write(path, contents).await
    }

    async fn write_temp(&self, prefix: &str, ext: &str, content: &str) -> anyhow::Result<PathBuf> {
        self.file_write_service
            .write_temp(prefix, ext, content)
            .await
    }
}

#[async_trait::async_trait]
impl FileInfoInfra for GoblinInfra {
    async fn is_binary(&self, path: &Path) -> anyhow::Result<bool> {
        self.file_meta_service.is_binary(path).await
    }

    async fn is_file(&self, path: &Path) -> anyhow::Result<bool> {
        self.file_meta_service.is_file(path).await
    }

    async fn exists(&self, path: &Path) -> anyhow::Result<bool> {
        self.file_meta_service.exists(path).await
    }

    async fn file_size(&self, path: &Path) -> anyhow::Result<u64> {
        self.file_meta_service.file_size(path).await
    }
}
#[async_trait::async_trait]
impl FileRemoverInfra for GoblinInfra {
    async fn remove(&self, path: &Path) -> anyhow::Result<()> {
        self.file_remove_service.remove(path).await
    }
}

#[async_trait::async_trait]
impl FileDirectoryInfra for GoblinInfra {
    async fn create_dirs(&self, path: &Path) -> anyhow::Result<()> {
        self.create_dirs_service.create_dirs(path).await
    }
}

#[async_trait::async_trait]
impl CommandInfra for GoblinInfra {
    async fn execute_command(
        &self,
        command: String,
        working_dir: PathBuf,
        silent: bool,
        env_vars: Option<Vec<String>>,
    ) -> anyhow::Result<CommandOutput> {
        self.command_executor_service
            .execute_command(command, working_dir, silent, env_vars)
            .await
    }

    async fn execute_command_raw(
        &self,
        command: &str,
        working_dir: PathBuf,
        env_vars: Option<Vec<String>>,
    ) -> anyhow::Result<ExitStatus> {
        self.command_executor_service
            .execute_command_raw(command, working_dir, env_vars)
            .await
    }
}

#[async_trait::async_trait]
impl UserInfra for GoblinInfra {
    async fn prompt_question(&self, question: &str) -> anyhow::Result<Option<String>> {
        self.inquire_service.prompt_question(question).await
    }

    async fn select_one<T: Clone + std::fmt::Display + Send + 'static>(
        &self,
        message: &str,
        options: Vec<T>,
    ) -> anyhow::Result<Option<T>> {
        self.inquire_service.select_one(message, options).await
    }

    async fn select_many<T: std::fmt::Display + Clone + Send + 'static>(
        &self,
        message: &str,
        options: Vec<T>,
    ) -> anyhow::Result<Option<Vec<T>>> {
        self.inquire_service.select_many(message, options).await
    }
}

#[async_trait::async_trait]
impl McpServerInfra for GoblinInfra {
    type Client = GoblinMcpClient;

    async fn connect(
        &self,
        config: McpServerConfig,
        env_vars: &BTreeMap<String, String>,
    ) -> anyhow::Result<Self::Client> {
        self.mcp_server.connect(config, env_vars).await
    }
}

#[async_trait::async_trait]
impl WalkerInfra for GoblinInfra {
    async fn walk(&self, config: goblin_app::Walker) -> anyhow::Result<Vec<goblin_app::WalkedFile>> {
        self.walker_service.walk(config).await
    }
}

#[async_trait::async_trait]
impl HttpInfra for GoblinInfra {
    async fn http_get(&self, url: &Url, headers: Option<HeaderMap>) -> anyhow::Result<Response> {
        self.http_service.http_get(url, headers).await
    }

    async fn http_post(
        &self,
        url: &Url,
        headers: Option<HeaderMap>,
        body: Bytes,
    ) -> anyhow::Result<Response> {
        self.http_service.http_post(url, headers, body).await
    }

    async fn http_delete(&self, url: &Url) -> anyhow::Result<Response> {
        self.http_service.http_delete(url).await
    }

    async fn http_eventsource(
        &self,
        url: &Url,
        headers: Option<HeaderMap>,
        body: Bytes,
    ) -> anyhow::Result<EventSource> {
        self.http_service.http_eventsource(url, headers, body).await
    }
}
#[async_trait::async_trait]
impl DirectoryReaderInfra for GoblinInfra {
    async fn list_directory_entries(
        &self,
        directory: &Path,
    ) -> anyhow::Result<Vec<(PathBuf, bool)>> {
        self.directory_reader_service
            .list_directory_entries(directory)
            .await
    }

    async fn read_directory_files(
        &self,
        directory: &Path,
        pattern: Option<&str>,
    ) -> anyhow::Result<Vec<(PathBuf, String)>> {
        self.directory_reader_service
            .read_directory_files(directory, pattern)
            .await
    }
}

impl StrategyFactory for GoblinInfra {
    type Strategy = AnyAuthStrategy;
    fn create_auth_strategy(
        &self,
        provider_id: ProviderId,
        method: AuthMethod,
        required_params: Vec<URLParamSpec>,
    ) -> anyhow::Result<Self::Strategy> {
        self.strategy_factory
            .create_auth_strategy(provider_id, method, required_params)
    }
}

impl GrpcInfra for GoblinInfra {
    fn channel(&self) -> tonic::transport::Channel {
        self.grpc_client.channel()
    }

    fn hydrate(&self) {
        self.grpc_client.hydrate();
    }
}

impl goblin_domain::ConsoleWriter for GoblinInfra {
    fn write(&self, buf: &[u8]) -> std::io::Result<usize> {
        self.output_printer.write(buf)
    }

    fn write_err(&self, buf: &[u8]) -> std::io::Result<usize> {
        self.output_printer.write_err(buf)
    }

    fn flush(&self) -> std::io::Result<()> {
        self.output_printer.flush()
    }

    fn flush_err(&self) -> std::io::Result<()> {
        self.output_printer.flush_err()
    }
}
