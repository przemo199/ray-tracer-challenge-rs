# CLAUDE.md

All project instructions live in [AGENTS.md](AGENTS.md) so every coding agent shares one source of truth.
Edit that file, not this one, when project conventions change.

@AGENTS.md

## Claude Code specifics

- Shared permissions for routine cargo commands are in `.claude/settings.json`; personal overrides go in
  `.claude/settings.local.json` (git-ignored).
- Prefer the Bash tool for cargo commands; long renders should run with `run_in_background`.
