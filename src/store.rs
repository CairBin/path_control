use crate::error::AppResult;
use crate::model::{Entry, Scope};

pub trait EnvStore {
    fn load_entries(&self) -> AppResult<Vec<Entry>>;
    fn load_path(&self) -> AppResult<String>;
    fn save_path(&self, value: &str) -> AppResult<()>;
    fn save_entries(&self, entries: &[Entry]) -> AppResult<()>;
    fn apply_path(&self, entries: &[Entry]) -> AppResult<()>;
}

#[cfg(windows)]
mod windows;

#[cfg(unix)]
mod unix;

pub fn create_store(scope: Scope) -> Box<dyn EnvStore> {
    #[cfg(windows)]
    {
        Box::new(windows::WindowsRegistryStore::new(scope))
    }

    #[cfg(unix)]
    {
        Box::new(unix::UnixFileStore::new(scope))
    }
}
