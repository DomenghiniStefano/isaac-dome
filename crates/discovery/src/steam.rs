//! Locating the Steam root.

use std::path::PathBuf;

use crate::{Diagnostic, Options, SteamInstall, SteamSource};

pub(crate) fn find_steam(opts: &Options) -> (Option<SteamInstall>, Vec<Diagnostic>) {
    if let Some(root) = &opts.steam_root {
        let libraries = libraries_from_root(root);
        return (
            Some(SteamInstall {
                root: root.clone(),
                source: SteamSource::Override,
                libraries,
            }),
            Vec::new(),
        );
    }

    if let Some(install) = try_steamlocate() {
        return (Some(install), Vec::new());
    }

    if let Some(install) = try_registry() {
        return (Some(install), Vec::new());
    }

    (None, vec![Diagnostic::SteamNotFound])
}

fn try_steamlocate() -> Option<SteamInstall> {
    let steam_dir = steamlocate::SteamDir::locate().ok()?;
    let root = steam_dir.path().to_path_buf();
    let libraries = match steam_dir.library_paths() {
        Ok(paths) => paths,
        Err(_) => vec![root.clone()],
    };
    Some(SteamInstall {
        root,
        source: SteamSource::SteamLocate,
        libraries,
    })
}

#[cfg(windows)]
fn try_registry() -> Option<SteamInstall> {
    use winreg::{
        enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ},
        RegKey,
    };

    // First choice: HKCU\Software\Valve\Steam, value SteamPath.
    // It's the path Steam itself writes and is always up to date.
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(key) = hkcu.open_subkey_with_flags("Software\\Valve\\Steam", KEY_READ) {
        if let Ok(path_str) = key.get_value::<String, _>("SteamPath") {
            let root = PathBuf::from(path_str);
            let libraries = libraries_from_root(&root);
            return Some(SteamInstall {
                root,
                source: SteamSource::Registry,
                libraries,
            });
        }
    }

    // Fallback: HKLM\SOFTWARE\Wow6432Node\Valve\Steam, value InstallPath.
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm
        .open_subkey_with_flags("SOFTWARE\\Wow6432Node\\Valve\\Steam", KEY_READ)
        .or_else(|_| hklm.open_subkey_with_flags("SOFTWARE\\Valve\\Steam", KEY_READ))
        .ok()?;
    let path_str: String = key.get_value("InstallPath").ok()?;
    let root = PathBuf::from(path_str);
    let libraries = libraries_from_root(&root);
    Some(SteamInstall {
        root,
        source: SteamSource::Registry,
        libraries,
    })
}

#[cfg(not(windows))]
fn try_registry() -> Option<SteamInstall> {
    None
}

fn libraries_from_root(root: &std::path::Path) -> Vec<PathBuf> {
    match steamlocate::SteamDir::from_dir(root) {
        Ok(sd) => sd
            .library_paths()
            .unwrap_or_else(|_| vec![root.to_path_buf()]),
        Err(_) => vec![root.to_path_buf()],
    }
}
