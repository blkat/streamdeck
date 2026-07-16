use crate::external_tools::configure_subprocess;
use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Interpréteur utilisé pour lancer un fichier script.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptShell {
    PowerShell,
    Cmd,
    Bash,
    Python,
}

impl ScriptShell {
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "cmd" | "batch" => ScriptShell::Cmd,
            "bash" | "sh" => ScriptShell::Bash,
            "python" | "py" => ScriptShell::Python,
            _ => ScriptShell::PowerShell,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ScriptShell::PowerShell => "powershell",
            ScriptShell::Cmd => "cmd",
            ScriptShell::Bash => "bash",
            ScriptShell::Python => "python",
        }
    }

    pub fn default_for_platform() -> Self {
        #[cfg(windows)]
        {
            ScriptShell::PowerShell
        }
        #[cfg(not(windows))]
        {
            ScriptShell::Bash
        }
    }
}

/// Chemins personnalisés (réglages). Vide = détection automatique.
#[derive(Debug, Clone, Default)]
pub struct ShellPaths {
    pub powershell: Option<String>,
    pub cmd: Option<String>,
    pub bash: Option<String>,
    pub python: Option<String>,
}

pub fn infer_shell_from_path(path: &Path) -> ScriptShell {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
    {
        Some(ext) if ext == "ps1" => ScriptShell::PowerShell,
        Some(ext) if ext == "bat" || ext == "cmd" => ScriptShell::Cmd,
        Some(ext) if ext == "sh" => ScriptShell::Bash,
        Some(ext) if ext == "py" || ext == "pyw" => ScriptShell::Python,
        Some(ext) if ext == "exe" => ScriptShell::Cmd,
        _ => ScriptShell::default_for_platform(),
    }
}

/// Lance un script de touche : chemin fichier + interpréteur, ou commande inline (anciennes données).
pub fn run_slot_script(
    script_path_or_cmd: &str,
    shell: Option<&str>,
    paths: &ShellPaths,
) -> Result<()> {
    let trimmed = script_path_or_cmd.trim();
    if trimmed.is_empty() {
        bail!("aucun script configuré");
    }

    let path = Path::new(trimmed);
    if path.exists() {
        let shell = shell
            .filter(|s| !s.trim().is_empty())
            .map(ScriptShell::from_str)
            .unwrap_or_else(|| infer_shell_from_path(path));
        return run_script_file(shell, path, paths);
    }

    run_inline_command(trimmed, shell, paths)
}

fn run_script_file(shell: ScriptShell, path: &Path, paths: &ShellPaths) -> Result<()> {
    let path_str = path.display().to_string();
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase());

    // Exécutable Windows : lancement direct.
    if ext.as_deref() == Some("exe") {
        return spawn_process(path, &[path_str]);
    }

    match shell {
        ScriptShell::PowerShell => {
            let exe = resolve_powershell(paths)?;
            spawn_process(
                &exe,
                &[
                    "-NoProfile".into(),
                    "-NonInteractive".into(),
                    "-ExecutionPolicy".into(),
                    "Bypass".into(),
                    "-File".into(),
                    path_str,
                ],
            )
        }
        ScriptShell::Cmd => {
            if matches!(ext.as_deref(), Some("bat") | Some("cmd")) {
                let exe = resolve_cmd(paths)?;
                spawn_process(&exe, &["/C".into(), path_str])
            } else {
                bail!(
                    "CMD : utilisez un fichier .bat/.cmd ou choisissez PowerShell / Python / Bash"
                );
            }
        }
        ScriptShell::Bash => {
            let exe = resolve_bash(paths)?;
            spawn_process(&exe, &[path_str])
        }
        ScriptShell::Python => {
            let exe = resolve_python(paths)?;
            spawn_process(&exe, &[path_str])
        }
    }
}

fn run_inline_command(cmd: &str, shell: Option<&str>, paths: &ShellPaths) -> Result<()> {
    let shell = shell
        .filter(|s| !s.trim().is_empty())
        .map(ScriptShell::from_str)
        .unwrap_or_else(ScriptShell::default_for_platform);

    match shell {
        ScriptShell::PowerShell => {
            let exe = resolve_powershell(paths)?;
            spawn_process(
                &exe,
                &[
                    "-NoProfile".into(),
                    "-NonInteractive".into(),
                    "-ExecutionPolicy".into(),
                    "Bypass".into(),
                    "-Command".into(),
                    cmd.to_string(),
                ],
            )
        }
        ScriptShell::Cmd => {
            let exe = resolve_cmd(paths)?;
            spawn_process(&exe, &["/C".into(), cmd.to_string()])
        }
        ScriptShell::Bash => {
            let exe = resolve_bash(paths)?;
            spawn_process(&exe, &["-c".into(), cmd.to_string()])
        }
        ScriptShell::Python => {
            let exe = resolve_python(paths)?;
            spawn_process(&exe, &["-c".into(), cmd.to_string()])
        }
    }
}

fn spawn_process(program: &Path, args: &[String]) -> Result<()> {
    let mut cmd = Command::new(program);
    cmd.args(args);
    configure_subprocess(&mut cmd);
    cmd.spawn()
        .with_context(|| format!("lancer {}", program.display()))?;
    Ok(())
}

fn resolve_powershell(paths: &ShellPaths) -> Result<PathBuf> {
    resolve_executable(
        paths.powershell.as_deref(),
        &[
            #[cfg(windows)]
            r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe",
            "pwsh",
            "powershell",
        ],
        "PowerShell",
    )
}

fn resolve_cmd(paths: &ShellPaths) -> Result<PathBuf> {
    resolve_executable(
        paths.cmd.as_deref(),
        &[
            #[cfg(windows)]
            r"C:\Windows\System32\cmd.exe",
            "cmd",
        ],
        "CMD",
    )
}

fn resolve_bash(paths: &ShellPaths) -> Result<PathBuf> {
    resolve_executable(
        paths.bash.as_deref(),
        &[
            "/bin/bash",
            "/usr/bin/bash",
            "bash",
        ],
        "Bash",
    )
}

fn resolve_python(paths: &ShellPaths) -> Result<PathBuf> {
    #[cfg(windows)]
    let defaults = &["python", "python3", "py"];
    #[cfg(not(windows))]
    let defaults = &["python3", "python"];

    resolve_executable(paths.python.as_deref(), defaults, "Python")
}

fn resolve_executable(custom: Option<&str>, defaults: &[&str], label: &str) -> Result<PathBuf> {
    if let Some(custom) = custom.filter(|s| !s.trim().is_empty()) {
        let p = PathBuf::from(custom.trim());
        if p.is_file() {
            return Ok(p);
        }
        bail!("{label} : chemin introuvable ({custom})");
    }

    for candidate in defaults {
        if candidate.contains(std::path::MAIN_SEPARATOR) || candidate.contains('/') {
            let p = PathBuf::from(*candidate);
            if p.is_file() {
                return Ok(p);
            }
        } else if let Some(found) = find_on_path(candidate) {
            return Ok(found);
        }
    }

    bail!(
        "{label} introuvable — renseignez le chemin dans Réglages ou installez-le sur le PATH"
    )
}

fn find_on_path(name: &str) -> Option<PathBuf> {
    #[cfg(windows)]
    {
        let output = Command::new("where").arg(name).output().ok()?;
        if !output.status.success() {
            return None;
        }
        first_line_path(&String::from_utf8_lossy(&output.stdout))
    }
    #[cfg(not(windows))]
    {
        let output = Command::new("which").arg(name).output().ok()?;
        if !output.status.success() {
            return None;
        }
        first_line_path(&String::from_utf8_lossy(&output.stdout))
    }
}

fn first_line_path(stdout: &str) -> Option<PathBuf> {
    stdout
        .lines()
        .next()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
}

/// Compatibilité ancienne API.
pub fn run_command(command: &str) -> Result<()> {
    run_slot_script(command, None, &ShellPaths::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infer_ps1() {
        assert_eq!(
            infer_shell_from_path(Path::new(r"C:\scripts\foo.ps1")),
            ScriptShell::PowerShell
        );
    }

    #[test]
    fn infer_bat() {
        assert_eq!(
            infer_shell_from_path(Path::new("run.bat")),
            ScriptShell::Cmd
        );
    }

    #[test]
    fn infer_py() {
        assert_eq!(
            infer_shell_from_path(Path::new("/tmp/test.py")),
            ScriptShell::Python
        );
    }

    #[test]
    fn shell_from_str() {
        assert_eq!(ScriptShell::from_str("cmd"), ScriptShell::Cmd);
        assert_eq!(ScriptShell::from_str("python"), ScriptShell::Python);
        assert_eq!(ScriptShell::from_str("bash"), ScriptShell::Bash);
    }
}
