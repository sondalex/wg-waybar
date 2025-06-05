use crate::error;
use libc::{EPERM, ESRCH, kill};
use std::ffi::OsString;
use uzers::os::unix::UserExt;
use uzers::{get_current_uid, get_user_by_name, get_user_by_uid};

pub fn find_waybar_pid() -> Option<i32> {
    for process in procfs::process::all_processes().ok()?.flatten() {
        if let Ok(stat) = process.stat() {
            if stat.comm.contains("waybar") {
                return Some(process.pid);
            }
        }
    }
    None
}

pub fn send_signal_to_waybar(signal_num: i32, debug: bool) -> Result<(), error::SignalError> {
    let sigrtmin: i32 = libc::SIGRTMIN();
    let sigrtmax: i32 = libc::SIGRTMAX();
    if signal_num < 0 || signal_num > (sigrtmax - sigrtmin) {
        return Err(error::SignalError::OutOfRange(
            error::SignalOutOfRangeError(
                "Invalid signal number: must be between 0 and SIGRTMAX - SIGRTMIN".to_string(),
            ),
        ));
    }

    let pid = find_waybar_pid().ok_or(error::SignalError::ProcessNotFound(
        error::ProcessNotFoundError("Could not find Waybar process".to_string()),
    ))?;

    let signal = sigrtmin + signal_num;

    let result = unsafe { kill(pid, signal) };
    if debug {
        println!("Sent SIGRTMIN+{} to Waybar (PID: {})", signal_num, pid);
    }

    if result == 0 {
        Ok(())
    } else {
        let err = std::io::Error::last_os_error();
        match err.raw_os_error() {
            Some(ESRCH) => Err(error::SignalError::OS("Process does not exist".to_string())),
            Some(EPERM) => Err(error::SignalError::OS("Permission denied".to_string())),
            _ => Err(error::SignalError::OS("other error".to_string())),
        }
    }
}

fn to_pathbuf(path: OsString) -> Option<std::path::PathBuf> {
    let path: std::path::PathBuf = std::path::PathBuf::from(path);
    if path.is_absolute() { Some(path) } else { None }
}

#[derive(Debug)]
pub struct HomeDirNotFoundError {}

impl std::error::Error for HomeDirNotFoundError {}

impl std::fmt::Display for HomeDirNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HomeDirNotFound")
    }
}

fn get_home_dir_impl(
    get_envvar: impl Fn(&str) -> Option<OsString>,
    get_user_by_uid: impl Fn(u32) -> Option<uzers::User>,
    get_user_by_name: impl Fn(&OsString) -> Option<uzers::User>,
    get_uid: impl Fn() -> u32,
) -> Result<std::path::PathBuf, HomeDirNotFoundError> {
    let username = get_envvar("SUDO_USER").or_else(|| {
        let uid = get_uid();
        get_user_by_uid(uid).map(|u| u.name().into())
    });

    if let Some(user) = username {
        if let Some(user) = get_user_by_name(&user) {
            return Ok(user.home_dir().into());
        }
    }

    Err(HomeDirNotFoundError {})
}

fn get_home_dir() -> Result<std::path::PathBuf, HomeDirNotFoundError> {
    get_home_dir_impl(
        get_environ,
        get_user_by_uid,
        get_user_by_name,
        get_current_uid,
    )
}

pub fn get_environ(key: &str) -> Option<OsString> {
    std::env::var_os(key)
}

fn get_state_home_impl(
    app_name: &str,
    get_envvar: impl Fn(&str) -> Option<OsString>,
    get_home_dir_fn: impl Fn() -> Result<std::path::PathBuf, HomeDirNotFoundError>,
) -> Result<std::path::PathBuf, HomeDirNotFoundError> {
    let default_share_folder = get_home_dir_fn()?.join(".local/state");
    let state_home = get_envvar("XDG_STATE_HOME")
        .and_then(to_pathbuf)
        .unwrap_or(default_share_folder);
    Ok(state_home.join(app_name))
}

pub fn get_state_home(app_name: &str) -> Result<std::path::PathBuf, HomeDirNotFoundError> {
    get_state_home_impl(app_name, get_environ, get_home_dir)
}

pub fn fs_create_dir(path: std::path::PathBuf) -> Result<(), error::Error> {
    std::fs::create_dir(path.clone())?;
    if let Some(username) = get_environ("SUDO_USER") {
        let username_str = username.to_str().ok_or(error::UnCaughtError(
            "Failed to convert username to str".to_string(),
        ))?;

        let user = uzers::get_user_by_name(username_str)
            .ok_or(error::Error::UserNotFound(username_str.to_string()))?;

        let uid = user.uid();
        let gid = user.primary_group_id();
        std::os::unix::fs::chown(path, Some(uid), Some(gid))?
    }
    Ok(())
}

pub fn fs_write<C: AsRef<[u8]>>(path: std::path::PathBuf, content: C) -> Result<(), error::Error> {
    std::fs::write(path.clone(), content)?;
    if let Some(username) = get_environ("SUDO_USER") {
        let username_str = username.to_str().ok_or(error::UnCaughtError(
            "Failed to convert username to str".to_string(),
        ))?;
        let user = uzers::get_user_by_name(username_str)
            .ok_or(error::Error::UserNotFound(username_str.to_string()))?;
        let uid = user.uid();
        let gid = user.primary_group_id();
        std::os::unix::fs::chown(path, Some(uid), Some(gid))?
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::path::PathBuf;

    #[test]
    fn returns_xdg_state_home_when_set() {
        let app_name = "myapp";

        let get_envvar = |key: &str| {
            if key == "XDG_STATE_HOME" {
                Some(OsString::from("/custom/state"))
            } else {
                None
            }
        };

        let get_home_dir_fn = || Ok(PathBuf::from("/home/user"));

        let result = get_state_home_impl(app_name, get_envvar, get_home_dir_fn).unwrap();

        assert_eq!(result, PathBuf::from("/custom/state/myapp"));
    }

    #[test]
    fn falls_back_to_default_when_xdg_state_home_not_set() {
        let app_name = "myapp";

        let get_envvar = |_key: &str| None;

        let get_home_dir_fn = || Ok(PathBuf::from("/home/user"));

        let result = get_state_home_impl(app_name, get_envvar, get_home_dir_fn).unwrap();

        assert_eq!(result, PathBuf::from("/home/user/.local/state/myapp"));
    }

    #[test]
    fn returns_error_when_home_dir_not_found() {
        let app_name = "myapp";

        let get_envvar = |_key: &str| None;

        let get_home_dir_fn = || Err(HomeDirNotFoundError {});

        let result = get_state_home_impl(app_name, get_envvar, get_home_dir_fn);

        assert!(result.is_err());
    }
}
