use phpvm_core::PhpManager;
use tokio::sync::Mutex;

pub struct AppState {
    pub manager: Mutex<PhpManager>,
}

impl AppState {
    pub fn new() -> Result<Self, String> {
        let manager = PhpManager::new().map_err(|e| e.to_string())?;
        Ok(AppState {
            manager: Mutex::new(manager),
        })
    }
}
