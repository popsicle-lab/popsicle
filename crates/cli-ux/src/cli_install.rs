//! Silent macOS CLI install when launched from `Popsicle.app` (ADR-016 / DMG flow).

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const BIN_NAME: &str = "popsicle";
const PATH_LINE: &str = r#"export PATH="$HOME/.local/bin:$PATH""#;
const ZSHRC_MARKER: &str = ".local/bin";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallOutcome {
    pub dest: PathBuf,
    pub copied: bool,
    pub path_line_added: bool,
}

pub fn launched_from_app_bundle() -> bool {
    std::env::current_exe()
        .ok()
        .is_some_and(|path| path.to_string_lossy().contains(".app/Contents/MacOS/"))
}

/// Best-effort silent install when running inside a macOS app bundle. Never panics.
pub fn ensure_silent_if_app_bundle() {
    #[cfg(target_os = "macos")]
    {
        if launched_from_app_bundle() {
            let _ = ensure_silent();
        }
    }
}

#[cfg(target_os = "macos")]
pub fn ensure_silent() -> io::Result<InstallOutcome> {
    let src = std::env::current_exe()?;
    let home = std::env::var("HOME")
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "HOME not set for CLI install"))?;
    let dest_dir = PathBuf::from(&home).join(".local").join("bin");
    let dest = dest_dir.join(BIN_NAME);

    fs::create_dir_all(&dest_dir)?;
    if let Ok(global_dir) = crate::global_config::global_home() {
        let _ = fs::create_dir_all(&global_dir);
    }

    let copied = if should_copy(&src, &dest)? {
        fs::copy(&src, &dest)?;
        set_executable(&dest)?;
        true
    } else {
        false
    };

    let path_line_added = ensure_zshrc_path_line(&PathBuf::from(&home).join(".zshrc"))?;

    Ok(InstallOutcome {
        dest,
        copied,
        path_line_added,
    })
}

#[cfg(not(target_os = "macos"))]
pub fn ensure_silent() -> io::Result<InstallOutcome> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "CLI install is macOS-only",
    ))
}

fn should_copy(src: &Path, dest: &Path) -> io::Result<bool> {
    if !dest.is_file() {
        return Ok(true);
    }
    let src_meta = fs::metadata(src)?;
    let dest_meta = fs::metadata(dest)?;
    if src_meta.len() != dest_meta.len() {
        return Ok(true);
    }
    match (src_meta.modified(), dest_meta.modified()) {
        (Ok(sm), Ok(dm)) => Ok(sm > dm),
        _ => Ok(true),
    }
}

#[cfg(unix)]
fn set_executable(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms)
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) -> io::Result<()> {
    Ok(())
}

fn ensure_zshrc_path_line(zshrc: &Path) -> io::Result<bool> {
    if zshrc.is_file() {
        let content = fs::read_to_string(zshrc)?;
        if content.contains(ZSHRC_MARKER) {
            return Ok(false);
        }
        let mut line = String::new();
        if !content.ends_with('\n') {
            line.push('\n');
        }
        line.push_str("\n# Popsicle CLI (app install)\n");
        line.push_str(PATH_LINE);
        line.push('\n');
        let mut file = fs::OpenOptions::new().append(true).open(zshrc)?;
        file.write_all(line.as_bytes())?;
        return Ok(true);
    }
    fs::write(zshrc, format!("# ~/.zshrc\n{PATH_LINE}\n"))?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_dir(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("popsicle-cli-install-{name}-{stamp}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn should_copy_when_dest_missing() {
        let dir = test_dir("missing");
        let src = dir.join("src");
        let dest = dir.join("dest");
        fs::write(&src, b"bin").unwrap();
        assert!(should_copy(&src, &dest).unwrap());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn should_skip_when_dest_matches_src_size_and_is_newer() {
        let dir = test_dir("skip");
        let src = dir.join("src");
        let dest = dir.join("dest");
        fs::write(&src, b"same-bytes").unwrap();
        fs::copy(&src, &dest).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));
        assert!(!should_copy(&src, &dest).unwrap());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn zshrc_appends_path_once() {
        let dir = test_dir("zshrc");
        let zshrc = dir.join(".zshrc");
        fs::write(&zshrc, "# existing\n").unwrap();
        assert!(ensure_zshrc_path_line(&zshrc).unwrap());
        let content = fs::read_to_string(&zshrc).unwrap();
        assert!(content.contains(PATH_LINE));
        assert!(!ensure_zshrc_path_line(&zshrc).unwrap());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn zshrc_created_when_missing() {
        let dir = test_dir("new-zshrc");
        let zshrc = dir.join(".zshrc");
        assert!(ensure_zshrc_path_line(&zshrc).unwrap());
        assert!(zshrc.is_file());
        let _ = fs::remove_dir_all(dir);
    }
}
