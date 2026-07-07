//! In-memory storage backend (default when `AGENT_RUNTIME_DATABASE_URL` unset).

use std::sync::Mutex;

use crate::approval::ConfirmTaskStore;
use crate::chat::ChatStore;
use crate::run_log::RunLogStore;
use crate::run_mirror::RunMirrorStore;
use crate::runtime::RuntimeRegistry;
use crate::TaskStore;

#[derive(Debug, Default)]
pub struct MemoryStorage {
    pub tasks: Mutex<TaskStore>,
    pub confirms: Mutex<ConfirmTaskStore>,
    pub runtimes: Mutex<RuntimeRegistry>,
    pub mirrors: Mutex<RunMirrorStore>,
    pub logs: Mutex<RunLogStore>,
    pub chat: Mutex<ChatStore>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            tasks: Mutex::new(TaskStore::default()),
            confirms: Mutex::new(ConfirmTaskStore::default()),
            runtimes: Mutex::new(RuntimeRegistry::new()),
            mirrors: Mutex::new(RunMirrorStore::default()),
            logs: Mutex::new(RunLogStore::default()),
            chat: Mutex::new(ChatStore::default()),
        }
    }
}
