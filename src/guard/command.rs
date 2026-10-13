//! Shell commands: confined to the workspace they run at once; reaching outside asks.
use super::{Disposition, Policy, metadata, sensitive};
use std::path::Path;

/// Programs whose purpose reaches outside the task workspace: network, privilege, system
/// state, other processes, dynamic evaluation, or remote services.
const OUTSIDE: &[&str] = &[
    "curl",
    "wget",
    "ssh",
    "scp",
    "sftp",
    "rsync",
    "ftp",
    "telnet",
    "nc",
    "ncat",
    "netcat",
    "socat",
    "invoke-webrequest",
    "iwr",
    "invoke-restmethod",
    "irm",
    "start-bitstransfer",
    "bitsadmin",
    "certutil",
    "sudo",
    "su",
    "doas",
    "runas",
    "pkexec",
    "shutdown",
    "reboot",
    "halt",
    "poweroff",
    "systemctl",
    "service",
    "launchctl",
    "crontab",
    "schtasks",
    "sc",
    "reg",
    "regedit",
    "setx",
    "netsh",
    "net",
    "diskpart",
    "format",
    "mkfs",
    "mount",
    "umount",
    "dd",
    "kill",
    "pkill",
    "killall",
    "taskkill",
    "stop-process",
    "eval",
    "invoke-expression",
    "iex",
    "start-process",
    "saps",
    "npx",
    "pnpx",
    "bunx",
    "brew",
    "apt",
    "apt-get",
    "yum",
    "dnf",
    "pacman",
    "winget",
    "choco",
    "scoop",
    "docker",
    "podman",
    "kubectl",
    "gh",
    "az",
    "aws",
    "gcloud",
    "set-executionpolicy",
    "base64",
];

/// Subcommands that fetch, install or publish packages over the network.
fn installs(program: &str, sub: &str) -> bool {
    match program {
        "npm" | "pnpm" | "yarn" | "bun" => matches!(
            sub,
            "install"
                | "i"
                | "ci"
                | "add"
                | "update"
                | "upgrade"
                | "publish"
                | "exec"
                | "dlx"
                | "login"
                | "link"
                | "x"
        ),
        "pip" | "pip3" | "pipx" | "gem" => matches!(sub, "install" | "download" | "uninstall"),
        "uv" => matches!(sub, "pip" | "add" | "sync" | "tool" | "remove" | "publish"),
        "poetry" => matches!(sub, "add" | "install" | "update" | "publish" | "remove"),
        "cargo" => matches!(
            sub,
            "install" | "publish" | "add" | "login" | "update" | "remove"
        ),
        "go" => matches!(sub, "get" | "install"),
        "dotnet" => matches!(sub, "add" | "tool" | "nuget" | "restore"),
        _ => false,
    }
}

/// Git subcommands that only read the working tree, index and history.
const GIT_READS: &[&str] = &[
    "status",
    "diff",
    "log",
    "show",
    "blame",
    "grep",
    "ls-files",
    "ls-tree",
    "rev-parse",
    "describe",
    "shortlog",
    "cat-file",
    "diff-tree",
    "rev-list",
    "help",
    "version",
];

impl Policy {
    /// User decision (2026-10-13): a command confined to the task workspace runs at once,
    /// like Codex's workspace-write sandbox; one reaching outside asks a human at once.
    /// The check is textual and conservative: anything it cannot place asks, never allows.
    pub(super) fn command(&self, command: &str) -> Disposition {
        let words: Vec<&str> = command
            .split(|c: char| c.is_whitespace() || ";|&()<>{}`'\"=,".contains(c))
            .filter(|w| !w.is_empty())
            .collect();
        let names: Vec<String> = words
            .iter()
            .map(|word| {
                let name = word
                    .rsplit(['/', '\\'])
                    .next()
                    .unwrap_or(word)
                    .to_ascii_lowercase();
                [".exe", ".cmd", ".bat", ".ps1", ".com"]
                    .iter()
                    .find_map(|ext| name.strip_suffix(ext))
                    .map(str::to_owned)
                    .unwrap_or(name)
            })
            .collect();
        for (i, name) in names.iter().enumerate() {
            if OUTSIDE.contains(&name.as_str())
                || matches!(name.as_str(), "-encodedcommand" | "-enc" | "-ec")
            {
                return Disposition::Human;
            }
            let next = names[i + 1..]
                .iter()
                .find(|w| !w.starts_with('-'))
                .map_or("", String::as_str);
            if installs(name, next) {
                return Disposition::Human;
            }
            if name == "git" {
                let mut j = i + 1;
                while names.get(j).is_some_and(|w| w.starts_with('-')) {
                    j += if matches!(names[j].as_str(), "-c" | "-C") {
                        2
                    } else {
                        1
                    };
                }
                if !names
                    .get(j)
                    .is_some_and(|s| GIT_READS.contains(&s.as_str()))
                {
                    return Disposition::Human;
                }
            }
        }
        for word in &words {
            let lower = word.to_ascii_lowercase();
            if lower.contains("://")
                || word.starts_with('~')
                || word.starts_with("\\\\")
                || [
                    "$home",
                    "${home",
                    "%userprofile%",
                    "%appdata%",
                    "%localappdata%",
                    "$env:userprofile",
                    "$env:appdata",
                    "$env:localappdata",
                    "$env:home",
                ]
                .iter()
                .any(|v| lower.contains(v))
                || word.split(['/', '\\']).any(|part| part == "..")
            {
                return Disposition::Human;
            }
            // Protected and secret names are judged inside the workspace only: the
            // workspace itself may live under a folder such as ~/.agentconductor.
            match self.local(word) {
                Some(local)
                    if !sensitive(&local.to_ascii_lowercase().replace('\\', "/"))
                        && !metadata(&local) => {}
                _ => return Disposition::Human,
            }
        }
        Disposition::Automatic
    }

    /// The word as a workspace-relative path; None for an absolute path outside it.
    fn local(&self, word: &str) -> Option<String> {
        if !self.inside(word) {
            return None;
        }
        let bytes = word.as_bytes();
        let absolute = word.starts_with('/')
            || (bytes.len() >= 3 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':');
        if !absolute || word == "/dev/null" {
            return Some(word.to_owned());
        }
        let trimmed = word.trim_end_matches(['/', '\\']);
        #[cfg(windows)]
        let trimmed = &match trimmed.as_bytes() {
            [b'/', letter, b'/', ..] if letter.is_ascii_alphabetic() => {
                format!("{}:{}", *letter as char, &trimmed[2..])
            }
            _ => trimmed.to_owned(),
        };
        if self.is_root(trimmed) {
            return Some(String::new());
        }
        self.relative(trimmed).ok()
    }

    /// False for an absolute path outside the workspace; relative words are inside.
    fn inside(&self, word: &str) -> bool {
        let bytes = word.as_bytes();
        let drive = bytes.len() >= 3
            && bytes[0].is_ascii_alphabetic()
            && bytes[1] == b':'
            && matches!(bytes[2], b'/' | b'\\');
        if !(word.starts_with('/') || drive) || word == "/dev/null" {
            return true;
        }
        let word = word.trim_end_matches(['/', '\\']);
        // Git Bash on Windows spells C:\Work as /c/Work.
        #[cfg(windows)]
        let word = &match word.as_bytes() {
            [b'/', letter, b'/', ..] if letter.is_ascii_alphabetic() => {
                format!("{}:{}", *letter as char, &word[2..])
            }
            _ => word.to_owned(),
        };
        self.is_root(word) || self.relative(word).is_ok()
    }

    /// The workspace folder itself, in any spelling an agent uses for it.
    fn is_root(&self, word: &str) -> bool {
        let plain = |path: &Path| {
            let text = path.to_string_lossy();
            let text = text
                .strip_prefix(r"\\?\")
                .unwrap_or(&text)
                .replace('\\', "/");
            let text = text.trim_end_matches('/').to_owned();
            if cfg!(windows) {
                text.to_lowercase()
            } else {
                text
            }
        };
        let word = plain(Path::new(word));
        word == plain(&self.workspace) || word == plain(&self.configured)
    }
}
