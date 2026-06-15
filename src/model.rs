use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Scope {
    User,
    System,
}

impl Scope {
    pub fn label(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::System => "system",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryStatus {
    Enabled,
    Disabled,
}

impl EntryStatus {
    pub fn from_disabled(disabled: bool) -> Self {
        if disabled {
            Self::Disabled
        } else {
            Self::Enabled
        }
    }
}

impl fmt::Display for EntryStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Enabled => f.write_str("enabled"),
            Self::Disabled => f.write_str("disabled"),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Entry {
    #[serde(default = "generate_id")]
    pub id: String,
    pub name: String,
    pub value: String,
    pub tips: Option<String>,
    pub status: EntryStatus,
    pub created_at: u64,
}

impl Entry {
    pub fn new(name: String, value: String, tips: Option<String>, status: EntryStatus) -> Self {
        Self {
            id: generate_id(),
            name,
            value,
            tips,
            status,
            created_at: now_unix_seconds(),
        }
    }

    pub fn short_id(&self) -> &str {
        self.id.get(..12).unwrap_or(&self.id)
    }
}

fn now_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn generate_id() -> String {
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let pid = u128::from(std::process::id());
    let sequence = u128::from(SEQUENCE.fetch_add(1, Ordering::Relaxed));
    let mixed = nanos ^ (pid << 32) ^ sequence;

    format!("{mixed:032x}")
}
