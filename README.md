# ZROJ CI

## Setup

1. Go to "Github / Setting / Developer Settings / Personal access tokens / Tokens (classic)"
   to create a new token. 
   Make sure to select the `repo` scope.
   For organization repos, you need to enable personal access token (classic) in the setting. 
2. CD into `bot` and create a `config.yaml` looks like
   ```yaml
   gh_token: "<personal access token>"
   working_dir: "../.work"
   ci_tool_dir: "../ci_tool"
   ```
3. Install dependencies:
   - Python: `pip install PyGithub pyyaml typer`
   - Rust: `rustup` is preferred for Cargo installation.
4. Execute `python main.py` under `bot` to start the bot.