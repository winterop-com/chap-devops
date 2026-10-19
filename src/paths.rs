//! Filesystem locations that are not part of a generated project.

use std::path::PathBuf;

/// Directory holding the cached registry snapshot.
///
/// Resolution order:
/// 1. `$CHAP_DEVOPS_CACHE_DIR`
/// 2. `$XDG_CACHE_HOME/chap-devops`
/// 3. `%LOCALAPPDATA%\chap-devops\cache` (Windows only)
/// 4. `$HOME/.cache/chap-devops`
/// 5. `./.chap-cache` (last resort, so the CLI still works in a bare container)
///
/// Empty environment variables are treated as unset. The directory is not
/// created here; writers create it on demand.
pub fn cache_dir() -> PathBuf {
    cache_dir_from(&non_empty_env, cfg!(windows))
}

/// Directory for what chap-devops keeps for this user beyond the cache: the
/// deployment `chap-devops run` uses when it is not run inside one.
///
/// Resolution order:
/// 1. `$CHAP_DEVOPS_DATA_DIR`
/// 2. `$XDG_DATA_HOME/chap-devops`
/// 3. `%LOCALAPPDATA%\chap-devops\data` (Windows only)
/// 4. `$HOME/.local/share/chap-devops`
/// 5. `./.chap-devops-data` (last resort, as for the cache)
pub fn data_dir() -> PathBuf {
    data_dir_from(&non_empty_env, cfg!(windows))
}

/// Directory holding the `chap-devops run` groups, one deployment each.
pub fn run_groups_dir() -> PathBuf {
    data_dir().join("run")
}

type Env<'a> = &'a dyn Fn(&str) -> Option<String>;

fn cache_dir_from(env: Env, windows: bool) -> PathBuf {
    if let Some(dir) = env("CHAP_DEVOPS_CACHE_DIR") {
        return PathBuf::from(dir);
    }
    if let Some(dir) = env("XDG_CACHE_HOME") {
        return PathBuf::from(dir).join("chap-devops");
    }
    if let Some(dir) = windows.then(|| env("LOCALAPPDATA")).flatten() {
        return PathBuf::from(dir).join("chap-devops").join("cache");
    }
    if let Some(home) = env("HOME") {
        return PathBuf::from(home).join(".cache").join("chap-devops");
    }
    PathBuf::from(".chap-cache")
}

fn data_dir_from(env: Env, windows: bool) -> PathBuf {
    if let Some(dir) = env("CHAP_DEVOPS_DATA_DIR") {
        return PathBuf::from(dir);
    }
    if let Some(dir) = env("XDG_DATA_HOME") {
        return PathBuf::from(dir).join("chap-devops");
    }
    if let Some(dir) = windows.then(|| env("LOCALAPPDATA")).flatten() {
        return PathBuf::from(dir).join("chap-devops").join("data");
    }
    if let Some(home) = env("HOME") {
        return PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("chap-devops");
    }
    PathBuf::from(".chap-devops-data")
}

fn non_empty_env(key: &str) -> Option<String> {
    match std::env::var(key) {
        Ok(v) if !v.is_empty() => Some(v),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
