use std::collections::HashMap;

use crate::env::{EnvManager, EnvScope};
use crate::error::{Error, Result};

#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

pub struct WindowsEnvManager;

impl WindowsEnvManager {
    pub fn new() -> Self {
        Self
    }

    #[cfg(target_os = "windows")]
    fn get_registry_key(&self, scope: EnvScope) -> Result<RegKey> {
        match scope {
            EnvScope::User => {
                let hkcu = RegKey::predef(HKEY_CURRENT_USER);
                hkcu.open_subkey("Environment")
                    .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))
            }
            EnvScope::System => {
                let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
                hklm.open_subkey("SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment")
                    .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))
            }
        }
    }

    #[cfg(target_os = "windows")]
    fn get_registry_key_writable(&self, scope: EnvScope) -> Result<RegKey> {
        match scope {
            EnvScope::User => {
                let hkcu = RegKey::predef(HKEY_CURRENT_USER);
                hkcu.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
                    .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))
            }
            EnvScope::System => {
                let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
                hklm.open_subkey_with_flags(
                    "SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment",
                    KEY_READ | KEY_WRITE,
                )
                .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))
            }
        }
    }
}

impl EnvManager for WindowsEnvManager {
    #[cfg(target_os = "windows")]
    fn list(&self, scope: EnvScope) -> Result<HashMap<String, String>> {
        let key = self.get_registry_key(scope)?;
        let mut vars = HashMap::new();

        for (name, value) in key.enum_values().filter_map(|r| r.ok()) {
            // RegValue.to_string() returns String directly, not Result
            let s = value.to_string();
            vars.insert(name, s);
        }

        Ok(vars)
    }

    #[cfg(not(target_os = "windows"))]
    fn list(&self, _scope: EnvScope) -> Result<HashMap<String, String>> {
        Err(Error::Unsupported(
            "Windows registry access is only available on Windows".into(),
        ))
    }

    #[cfg(target_os = "windows")]
    fn get(&self, scope: EnvScope, key: &str) -> Result<Option<String>> {
        let reg_key = self.get_registry_key(scope)?;
        match reg_key.get_value::<String, _>(key) {
            Ok(value) => Ok(Some(value)),
            Err(_) => Ok(None),
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn get(&self, _scope: EnvScope, _key: &str) -> Result<Option<String>> {
        Err(Error::Unsupported(
            "Windows registry access is only available on Windows".into(),
        ))
    }

    #[cfg(target_os = "windows")]
    fn set(&self, scope: EnvScope, key: &str, value: &str) -> Result<()> {
        let reg_key = self.get_registry_key_writable(scope)?;
        reg_key
            .set_value(key, &value)
            .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        // Broadcast WM_SETTINGCHANGE to notify other applications
        #[cfg(target_os = "windows")]
        unsafe {
            use std::ptr;
            use winapi::um::winuser::{SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE};
            use winapi::shared::minwindef::LPARAM;

            let env_str = "Environment\0".encode_utf16().collect::<Vec<u16>>();
            SendMessageTimeoutW(
                HWND_BROADCAST,
                WM_SETTINGCHANGE,
                0,
                env_str.as_ptr() as LPARAM,
                SMTO_ABORTIFHUNG,
                5000,
                ptr::null_mut(),
            );
        }

        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    fn set(&self, _scope: EnvScope, _key: &str, _value: &str) -> Result<()> {
        Err(Error::Unsupported(
            "Windows registry access is only available on Windows".into(),
        ))
    }

    #[cfg(target_os = "windows")]
    fn unset(&self, scope: EnvScope, key: &str) -> Result<()> {
        let reg_key = self.get_registry_key_writable(scope)?;
        reg_key
            .delete_value(key)
            .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        // Broadcast WM_SETTINGCHANGE
        #[cfg(target_os = "windows")]
        unsafe {
            use std::ptr;
            use winapi::um::winuser::{SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE};
            use winapi::shared::minwindef::LPARAM;

            let env_str = "Environment\0".encode_utf16().collect::<Vec<u16>>();
            SendMessageTimeoutW(
                HWND_BROADCAST,
                WM_SETTINGCHANGE,
                0,
                env_str.as_ptr() as LPARAM,
                SMTO_ABORTIFHUNG,
                5000,
                ptr::null_mut(),
            );
        }

        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    fn unset(&self, _scope: EnvScope, _key: &str) -> Result<()> {
        Err(Error::Unsupported(
            "Windows registry access is only available on Windows".into(),
        ))
    }
}
