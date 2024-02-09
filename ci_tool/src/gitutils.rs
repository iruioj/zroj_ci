use std::{path::Path, process::Command};

use anyhow::anyhow;

/// Check whether a repository has staged changes (not yet committed)
pub fn has_staged(git_root: impl AsRef<Path>) -> anyhow::Result<bool> {
    let code = Command::new("git")
        .current_dir(git_root)
        .args(["diff-index", "--quiet", "--cached", "HEAD", "--"])
        .spawn()?
        .wait()?
        .code()
        .ok_or(anyhow!("no exit code (may be stopped by signal)"))?;
    Ok(code != 0)
}

/// Check whether a working tree has changes that could be staged
pub fn has_unstaged(git_root: impl AsRef<Path>) -> anyhow::Result<bool> {
    let code = Command::new("git")
        .current_dir(git_root)
        .args(["diff-files", "--quiet"])
        .spawn()?
        .wait()?
        .code()
        .ok_or(anyhow!("no exit code (may be stopped by signal)"))?;
    Ok(code != 0)
}

/// Synchronously do a `git checkout` of `commit`.
pub fn git_checkout(
    git_root: impl AsRef<Path>,
    commit: &str,
    quiet: bool,
    force: bool,
) -> anyhow::Result<()> {
    let mut command = Command::new("git");
    command.current_dir(git_root);
    command.args(["checkout", commit]);
    if quiet {
        command.arg("--quiet");
    }
    if force {
        command.arg("--force");
    }
    if command.spawn()?.wait()?.success() {
        Ok(())
    } else {
        Err(anyhow!(
            "Failed to `git checkout {}`, see error message on stdout/stderr.",
            commit,
        ))
    }
}

/// Returns the name of the current git branch. Or `None` if there is no current
/// branch.
pub fn current_branch(path: impl AsRef<Path>) -> anyhow::Result<Option<String>> {
    let branch = trimmed_git_stdout(path, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    if &branch == "HEAD" {
        Ok(None)
    } else {
        Ok(Some(branch))
    }
}
/// Returns the current commit hash.
pub fn current_commit(path: impl AsRef<Path>) -> anyhow::Result<String> {
    trimmed_git_stdout(path, &["rev-parse", "--short", "HEAD"])
}

/// Returns a commit list with one-line messages
pub fn commit_list(path: impl AsRef<Path>, dev: &str, base: &str) -> anyhow::Result<String> {
    // get the lca
    let old = trimmed_git_stdout(&path, &["merge-base", dev, base])?;
    trimmed_git_stdout(&path, &["rev-list", "--oneline", &format!("{old}..{dev}")])
}

fn trimmed_git_stdout(path: impl AsRef<Path>, args: &[&str]) -> anyhow::Result<String> {
    let mut git = Command::new("git");
    git.current_dir(path);
    git.args(args);
    trimmed_stdout(git)
}

fn trimmed_stdout(mut cmd: Command) -> anyhow::Result<String> {
    let output = cmd.output()?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(anyhow!(String::from_utf8_lossy(&output.stderr).to_string()))
    }
}

/// Resolves a git reference provided at the CLI to an actual commit, allowing
/// us to validate refs and use "relative" values like HEAD and more.
pub fn resolve_ref(path: impl AsRef<Path>, committish: &str) -> anyhow::Result<String> {
    trimmed_git_stdout(path, &["rev-parse", committish])
}
