use std::io;

use windows_sys::Win32::UI::WindowsAndMessaging::{
    HWND_BROADCAST, SMTO_ABORTIFHUNG, SendMessageTimeoutW, WM_SETTINGCHANGE,
};
use winreg::enums::{
    HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_SET_VALUE, REG_EXPAND_SZ,
};
use winreg::{RegKey, RegValue};

use crate::error::AppResult;
use crate::model::{Entry, Scope};
use crate::path_merge::{PathCase, merge_path};
use crate::store::EnvStore;

const USER_ENV_PATH: &str = "Environment";
const SYSTEM_ENV_PATH: &str = r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment";
const APP_PATH: &str = r"Software\PathControl";
const ENTRIES_VALUE: &str = "Entries";
const PATH_VALUE: &str = "Path";
const PATH_SEPARATOR: char = ';';

pub struct WindowsRegistryStore {
    scope: Scope,
}

impl WindowsRegistryStore {
    pub fn new(scope: Scope) -> Self {
        Self { scope }
    }

    fn env_key(&self, access: u32) -> io::Result<RegKey> {
        match self.scope {
            Scope::User => {
                RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(USER_ENV_PATH, access)
            }
            Scope::System => {
                RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey_with_flags(SYSTEM_ENV_PATH, access)
            }
        }
    }

    fn app_key(&self, access: u32) -> io::Result<RegKey> {
        self.app_root().open_subkey_with_flags(APP_PATH, access)
    }

    fn app_root(&self) -> RegKey {
        match self.scope {
            Scope::User => RegKey::predef(HKEY_CURRENT_USER),
            Scope::System => RegKey::predef(HKEY_LOCAL_MACHINE),
        }
    }

    fn entries_value_name(&self) -> String {
        format!("{}_{}", self.scope.label(), ENTRIES_VALUE)
    }
}

impl EnvStore for WindowsRegistryStore {
    fn load_entries(&self) -> AppResult<Vec<Entry>> {
        let key = match self.app_key(KEY_READ) {
            Ok(key) => key,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(err) => return Err(err.into()),
        };
        let raw = match key.get_value::<String, _>(self.entries_value_name()) {
            Ok(value) => value,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(err) => return Err(err.into()),
        };

        if raw.trim().is_empty() {
            return Ok(Vec::new());
        }

        Ok(serde_json::from_str(&raw)?)
    }

    fn load_path(&self) -> AppResult<String> {
        let env = self.env_key(KEY_READ)?;
        Ok(read_string_value(&env, PATH_VALUE)?.unwrap_or_default())
    }

    fn save_path(&self, value: &str) -> AppResult<()> {
        let env = self.env_key(KEY_SET_VALUE)?;
        write_expand_string_value(&env, PATH_VALUE, value)?;
        broadcast_environment_change();
        Ok(())
    }

    fn save_entries(&self, entries: &[Entry]) -> AppResult<()> {
        let (key, _) = self.app_root().create_subkey(APP_PATH)?;
        key.set_value(self.entries_value_name(), &serde_json::to_string(entries)?)?;
        Ok(())
    }

    fn apply_path(&self, entries: &[Entry]) -> AppResult<()> {
        let current = self.load_path()?;
        let next = merge_path(&current, entries, PATH_SEPARATOR, PathCase::Insensitive);
        self.save_path(&next)
    }
}

fn read_string_value(key: &RegKey, name: &str) -> io::Result<Option<String>> {
    match key.get_raw_value(name) {
        Ok(value) => Ok(Some(decode_registry_string(&value.bytes))),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err),
    }
}

fn write_expand_string_value(key: &RegKey, name: &str, value: &str) -> io::Result<()> {
    let mut bytes = Vec::with_capacity((value.len() + 1) * 2);
    for unit in value.encode_utf16().chain(std::iter::once(0)) {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }

    key.set_raw_value(
        name,
        &RegValue {
            vtype: REG_EXPAND_SZ,
            bytes,
        },
    )
}

fn decode_registry_string(bytes: &[u8]) -> String {
    let units = bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .take_while(|unit| *unit != 0)
        .collect::<Vec<_>>();

    String::from_utf16_lossy(&units)
}

fn broadcast_environment_change() {
    let environment = "Environment"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();

    unsafe {
        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            0,
            environment.as_ptr() as isize,
            SMTO_ABORTIFHUNG,
            5000,
            std::ptr::null_mut(),
        );
    }
}
