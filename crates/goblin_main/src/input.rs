use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use goblin_api::Environment;

use crate::editor::{GoblinEditor, ReadResult};
use crate::model::{GoblinCommandManager, SlashCommand};
use crate::prompt::GoblinPrompt;
use crate::tracker;

/// Console implementation for handling user input via command line.
pub struct Console {
    command: Arc<GoblinCommandManager>,
    editor: Mutex<GoblinEditor>,
}

impl Console {
    /// Creates a new instance of `Console`.
    pub fn new(
        env: Environment,
        custom_history_path: Option<PathBuf>,
        command: Arc<GoblinCommandManager>,
    ) -> Self {
        let editor = Mutex::new(GoblinEditor::new(env, custom_history_path, command.clone()));
        Self { command, editor }
    }
}

impl Console {
    pub async fn prompt(&self, prompt: GoblinPrompt) -> anyhow::Result<SlashCommand> {
        loop {
            let mut goblin_editor = self.editor.lock().unwrap();
            let user_input = goblin_editor.prompt(&prompt)?;
            drop(goblin_editor);
            match user_input {
                ReadResult::Continue => continue,
                ReadResult::Exit => return Ok(SlashCommand::Exit),
                ReadResult::Empty => continue,
                ReadResult::Success(text) => {
                    tracker::prompt(text.clone());
                    return self.command.parse(&text);
                }
            }
        }
    }

    /// Sets the buffer content for the next prompt
    pub fn set_buffer(&self, content: String) {
        let mut editor = self.editor.lock().unwrap();
        editor.set_buffer(content);
    }
}
