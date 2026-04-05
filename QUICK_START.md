# Quick Start

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/mikemackintosh/zush/main/scripts/install.sh | bash
```

## Configure

Add to `~/.zshrc`:

```zsh
export PATH="$HOME/.local/bin:$PATH"
export ZUSH_THEME="split"
eval "$(zush-prompt init zsh)"
```

Reload: `exec zsh`

## Switch Themes

```zsh
zush-theme list           # List available themes
zush-theme split          # Switch to split theme
zush-theme list --preview # Preview all themes
```

## Available Themes

Built-in: `aurora`, `boulevard`, `catppuccin`, `cyberpunk`, `dcs`, `ember`, `frost`, `galaxy`, `hacker`, `matrix`, `midnight`, `minimal`, `neon`, `ocean-deep`, `powerline`, `radioactive`, `remote`, `split`, `starship`, `synthwave`, `vaporwave`

## Template Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `user` | Current username | `mike` |
| `host` | Hostname | `macbook` |
| `pwd` | Full working directory | `/Users/mike/projects` |
| `pwd_short` | Shortened path (`~` for home) | `~/projects` |
| `exit_code` | Last command exit code | `0` |
| `execution_time_ms` | Last command duration (ms) | `1234` |
| `git_branch` | Current git branch | `main` |
| `git_staged` | Staged file count | `2` |
| `git_modified` | Modified file count | `3` |
| `git_untracked` | Untracked file count | `1` |
| `git_stash` | Stash count | `1` |
| `git_ahead` | Commits ahead of upstream | `2` |
| `git_behind` | Commits behind upstream | `0` |
| `time` | Current time (HH:MM:SS) | `15:30:45` |
| `is_ssh` | Whether connected via SSH | `true` |

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `ZUSH_THEME` | (from config) | Active theme name |
| `ZUSH_GIT_MINIMAL` | `0` | Show only branch name (skip status) |
| `ZUSH_GIT_DISABLE_UNTRACKED` | `0` | Skip untracked file counting |
| `ZUSH_DISABLE_MODULES` | `0` | Disable all language modules |
| `ZUSH_DISABLE_<MODULE>` | `0` | Disable specific module (e.g., `ZUSH_DISABLE_PYTHON`) |
| `ZUSH_PROMPT_NEWLINE_BEFORE` | `1` | Blank line before prompt |

## Further Reading

- [CONFIGURATION.md](CONFIGURATION.md) - Full configuration guide
- [TEMPLATE_HELPERS.md](TEMPLATE_HELPERS.md) - Template helper reference
- [THEMES.md](THEMES.md) - Theme creation guide
- [BUILD.md](BUILD.md) - Building from source
- [PERFORMANCE.md](PERFORMANCE.md) - Performance tuning
