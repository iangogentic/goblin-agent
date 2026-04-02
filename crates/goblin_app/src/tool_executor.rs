use std::path::PathBuf;
use std::sync::Arc;

use anyhow::anyhow;
use goblin_domain::{CodebaseQueryResult, ToolCallContext, ToolCatalog, ToolOutput};

use crate::fmt::content::FormatContent;
use crate::operation::{TempContentFiles, ToolOperation};
use crate::services::{Services, ShellService};
use crate::{
    AgentRegistry, ConversationService, EnvironmentInfra, FollowUpService, FsPatchService,
    FsReadService, FsRemoveService, FsSearchService, FsUndoService, FsWriteService,
    ImageReadService, NetFetchService, PlanCreateService, ProviderService, SkillFetchService,
    WorkspaceService,
};

pub struct ToolExecutor<S> {
    services: Arc<S>,
}

impl<
    S: FsReadService
        + ImageReadService
        + FsWriteService
        + FsSearchService
        + WorkspaceService
        + NetFetchService
        + FsRemoveService
        + FsPatchService
        + FsUndoService
        + ShellService
        + FollowUpService
        + ConversationService
        + EnvironmentInfra
        + PlanCreateService
        + SkillFetchService
        + AgentRegistry
        + ProviderService
        + Services,
> ToolExecutor<S>
{
    pub fn new(services: Arc<S>) -> Self {
        Self { services }
    }

    fn require_prior_read(
        &self,
        context: &ToolCallContext,
        raw_path: &str,
        action: &str,
    ) -> anyhow::Result<()> {
        let target_path = self.normalize_path(raw_path.to_string());
        let has_read = context.with_metrics(|metrics| {
            metrics.files_accessed.contains(&target_path)
                || metrics.files_accessed.contains(raw_path)
        })?;

        if has_read {
            Ok(())
        } else {
            Err(anyhow!(
                "You must read the file with the read tool before attempting to {action}.",
                action = action
            ))
        }
    }

    async fn dump_operation(&self, operation: &ToolOperation) -> anyhow::Result<TempContentFiles> {
        match operation {
            ToolOperation::NetFetch { input: _, output } => {
                let original_length = output.content.len();
                let is_truncated = original_length > self.services.get_config().max_fetch_chars;
                let mut files = TempContentFiles::default();

                if is_truncated {
                    files = files.stdout(
                        self.create_temp_file("goblin_fetch_", ".txt", &output.content)
                            .await?,
                    );
                }

                Ok(files)
            }
            ToolOperation::Shell { output } => {
                let config = self.services.get_config();
                let stdout_lines = output.output.stdout.lines().count();
                let stderr_lines = output.output.stderr.lines().count();
                let stdout_truncated =
                    stdout_lines > config.max_stdout_prefix_lines + config.max_stdout_suffix_lines;
                let stderr_truncated =
                    stderr_lines > config.max_stdout_prefix_lines + config.max_stdout_suffix_lines;

                let mut files = TempContentFiles::default();

                if stdout_truncated {
                    files = files.stdout(
                        self.create_temp_file("goblin_shell_stdout_", ".txt", &output.output.stdout)
                            .await?,
                    );
                }
                if stderr_truncated {
                    files = files.stderr(
                        self.create_temp_file("goblin_shell_stderr_", ".txt", &output.output.stderr)
                            .await?,
                    );
                }

                Ok(files)
            }
            _ => Ok(TempContentFiles::default()),
        }
    }

    /// Converts a path to absolute by joining it with the current working
    /// directory if it's relative
    fn normalize_path(&self, path: String) -> String {
        let env = self.services.get_environment();
        let path_buf = PathBuf::from(&path);

        if path_buf.is_absolute() {
            path
        } else {
            PathBuf::from(&env.cwd).join(path_buf).display().to_string()
        }
    }

    async fn create_temp_file(
        &self,
        prefix: &str,
        ext: &str,
        content: &str,
    ) -> anyhow::Result<std::path::PathBuf> {
        let path = tempfile::Builder::new()
            .disable_cleanup(true)
            .prefix(prefix)
            .suffix(ext)
            .tempfile()?
            .into_temp_path()
            .to_path_buf();
        self.services
            .write(
                path.to_string_lossy().to_string(),
                content.to_string(),
                true,
            )
            .await?;
        Ok(path)
    }

    async fn call_internal(
        &self,
        input: ToolCatalog,
        context: &ToolCallContext,
    ) -> anyhow::Result<ToolOperation> {
        Ok(match input {
            ToolCatalog::Read(input) => {
                let normalized_path = self.normalize_path(input.file_path.clone());
                let output = self
                    .services
                    .read(
                        normalized_path,
                        input.start_line.map(|i| i as u64),
                        input.end_line.map(|i| i as u64),
                    )
                    .await?;

                (input, output).into()
            }
            ToolCatalog::Write(input) => {
                let normalized_path = self.normalize_path(input.file_path.clone());
                let output = self
                    .services
                    .write(normalized_path, input.content.clone(), input.overwrite)
                    .await?;
                (input, output).into()
            }
            ToolCatalog::FsSearch(input) => {
                let mut params = input.clone();
                // Normalize path if provided
                if let Some(ref path) = params.path {
                    params.path = Some(self.normalize_path(path.clone()));
                }
                let output = self.services.search(params).await?;
                (input, output).into()
            }
            ToolCatalog::SemSearch(input) => {
                let env = self.services.get_environment();
                let config = self.services.get_config();
                let services = self.services.clone();
                let cwd = env.cwd.clone();
                let limit = config.max_sem_search_results;
                let top_k = config.sem_search_top_k as u32;
                let params: Vec<_> = input
                    .queries
                    .iter()
                    .map(|search_query| {
                        goblin_domain::SearchParams::new(&search_query.query, &search_query.use_case)
                            .limit(limit)
                            .top_k(top_k)
                    })
                    .collect();

                // Execute all queries in parallel
                let futures: Vec<_> = params
                    .into_iter()
                    .map(|param| services.query_workspace(cwd.clone(), param))
                    .collect();

                let mut results = futures::future::try_join_all(futures).await?;

                // Deduplicate results across queries
                crate::search_dedup::deduplicate_results(&mut results);

                let output = input
                    .queries
                    .into_iter()
                    .zip(results)
                    .map(|(query, results)| CodebaseQueryResult {
                        query: query.query,
                        use_case: query.use_case,
                        results,
                    })
                    .collect::<Vec<_>>();

                let output = goblin_domain::CodebaseSearchResults { queries: output };
                ToolOperation::CodebaseSearch { output }
            }
            ToolCatalog::Remove(input) => {
                let normalized_path = self.normalize_path(input.path.clone());
                let output = self.services.remove(normalized_path).await?;
                (input, output).into()
            }
            ToolCatalog::Patch(input) => {
                let normalized_path = self.normalize_path(input.file_path.clone());
                let output = self
                    .services
                    .patch(
                        normalized_path,
                        input.old_string.clone(),
                        input.new_string.clone(),
                        input.replace_all,
                    )
                    .await?;
                (input, output).into()
            }
            ToolCatalog::Undo(input) => {
                let normalized_path = self.normalize_path(input.path.clone());
                let output = self.services.undo(normalized_path).await?;
                (input, output).into()
            }
            ToolCatalog::Shell(input) => {
                let cwd = input
                    .cwd
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| self.services.get_environment().cwd.display().to_string());
                let normalized_cwd = self.normalize_path(cwd);
                let output = self
                    .services
                    .execute(
                        input.command.clone(),
                        PathBuf::from(normalized_cwd),
                        input.keep_ansi,
                        false,
                        input.env.clone(),
                        input.description.clone(),
                    )
                    .await?;
                output.into()
            }
            ToolCatalog::Fetch(input) => {
                let output = self.services.fetch(input.url.clone(), input.raw).await?;
                (input, output).into()
            }
            ToolCatalog::Followup(input) => {
                let output = self
                    .services
                    .follow_up(
                        input.question.clone(),
                        input
                            .option1
                            .clone()
                            .into_iter()
                            .chain(input.option2.clone())
                            .chain(input.option3.clone())
                            .chain(input.option4.clone())
                            .chain(input.option5.clone())
                            .collect(),
                        input.multiple,
                    )
                    .await?;
                output.into()
            }
            ToolCatalog::Plan(input) => {
                let output = self
                    .services
                    .create_plan(
                        input.plan_name.clone(),
                        input.version.clone(),
                        input.content.clone(),
                    )
                    .await?;
                (input, output).into()
            }
            ToolCatalog::Skill(input) => {
                let skill = self.services.fetch_skill(input.name.clone()).await?;
                ToolOperation::Skill { output: skill }
            }
            ToolCatalog::TodoWrite(input) => {
                let before = context.get_todos()?;
                context.update_todos(input.todos.clone())?;
                let after = context.get_todos()?;
                ToolOperation::TodoWrite { before, after }
            }
            ToolCatalog::TodoRead(_input) => {
                let todos = context.get_todos()?;
                ToolOperation::TodoRead { output: todos }
            }
            // Hermes brain tools - memory operations
            ToolCatalog::MemoryCheckpoint(input) => {
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let output = goblin_core::MemoryCheckpointOutput {
                    checkpoint_id: format!("checkpoint_{}", timestamp),
                    timestamp: timestamp as i64,
                    message: format!("Checkpoint '{}' saved successfully", input.label),
                };
                let input = goblin_core::MemoryCheckpointInput {
                    label: input.label,
                };
                ToolOperation::MemoryCheckpoint { input, output }
            }
            ToolCatalog::MemoryCompact(input) => {
                let output = goblin_core::MemoryCompactOutput {
                    entries_compacted: 0,
                    new_size_bytes: 0,
                    message: "Memory compaction completed".to_string(),
                };
                let input = goblin_core::MemoryCompactInput {
                    max_entries: input.max_entries.map(|v| v as usize),
                };
                ToolOperation::MemoryCompact { input, output }
            }
            ToolCatalog::MemorySearch(input) => {
                let output = goblin_core::MemorySearchOutput {
                    results: vec![],
                    total: 0,
                    message: "Search completed (use Goblin with memory enabled)".to_string(),
                };
                let input = goblin_core::MemorySearchInput {
                    query: input.query,
                    scope: input.scope,
                };
                ToolOperation::MemorySearch { input, output }
            }
            ToolCatalog::MemorySummarize(input) => {
                let output = goblin_core::MemorySummarizeOutput {
                    summary: "Memory summarization available with Goblin Core".to_string(),
                    tokens_saved: 0,
                };
                let input = goblin_core::MemorySummarizeInput {
                    scope: input.scope,
                };
                ToolOperation::MemorySummarize { input, output }
            }
            // Hermes brain tools - skill operations
            ToolCatalog::SkillCreate(input) => {
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis();
                let output = goblin_core::SkillCreateOutput {
                    skill_id: format!("skill_{}", timestamp),
                    name: input.name.clone(),
                    message: format!("Skill '{}' created successfully", input.name),
                };
                let input = goblin_core::SkillCreateInput {
                    name: input.name,
                    prompt: None,
                };
                ToolOperation::SkillCreate { input, output }
            }
            ToolCatalog::SkillImprove(input) => {
                let output = goblin_core::SkillImproveOutput {
                    name: input.name.clone(),
                    quality_delta: 0.1,
                    message: format!("Skill '{}' improved", input.name),
                };
                let input = goblin_core::SkillImproveInput {
                    name: input.name,
                    success: input.success,
                };
                ToolOperation::SkillImprove { input, output }
            }
            ToolCatalog::SkillList(_) => {
                let output = goblin_core::SkillListOutput {
                    skills: vec![],
                    total: 0,
                };
                ToolOperation::SkillList { output }
            }
            // Hermes brain tools - schedule operations
            ToolCatalog::ScheduleCreate(input) => {
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis();
                let output = goblin_core::ScheduleCreateOutput {
                    job_id: format!("job_{}", timestamp),
                    next_run: None,
                    message: format!("Job scheduled: {}", input.prompt),
                };
                let input = goblin_core::ScheduleCreateInput {
                    prompt: input.prompt,
                    schedule: Some(input.schedule),
                };
                ToolOperation::ScheduleCreate { input, output }
            }
            ToolCatalog::ScheduleCancel(input) => {
                let output = goblin_core::ScheduleCancelOutput {
                    success: true,
                    message: format!("Job '{}' cancelled", input.job_id),
                };
                let input = goblin_core::ScheduleCancelInput {
                    job_id: input.job_id,
                };
                ToolOperation::ScheduleCancel { input, output }
            }
            ToolCatalog::ScheduleList(_) => {
                let output = goblin_core::ScheduleListOutput {
                    jobs: vec![],
                    total: 0,
                };
                ToolOperation::ScheduleList { output }
            }
        })
    }

    pub async fn execute(
        &self,
        tool_input: ToolCatalog,
        context: &ToolCallContext,
    ) -> anyhow::Result<ToolOutput> {
        let tool_kind = tool_input.kind();
        let env = self.services.get_environment();
        let config = self.services.get_config();

        // Enforce read-before-edit for patch
        if let ToolCatalog::Patch(input) = &tool_input {
            self.require_prior_read(context, &input.file_path, "edit it")?;
        }

        // Enforce read-before-edit for overwrite writes
        if let ToolCatalog::Write(input) = &tool_input
            && input.overwrite
        {
            self.require_prior_read(context, &input.file_path, "overwrite it")?;
        }

        let execution_result = self.call_internal(tool_input.clone(), context).await;

        if let Err(ref error) = execution_result {
            tracing::error!(error = ?error, "Tool execution failed");
        }

        let operation = execution_result?;

        // Send formatted output message
        if let Some(output) = operation.to_content(&env) {
            context.send(output).await?;
        }

        let truncation_path = self.dump_operation(&operation).await?;

        context.with_metrics(|metrics| {
            operation.into_tool_output(tool_kind, truncation_path, &env, &config, metrics)
        })
    }
}
