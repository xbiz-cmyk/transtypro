# Recommended Antigravity Settings for transtypro

Use a safe, review-driven setup.

## Recommended mode

Use Review-driven development for most work.

Use Planning mode for:
- architecture
- phase planning
- OS-specific implementation
- privacy design
- database schema
- text insertion and shortcut design
- packaging

Use Fast mode only for:
- small UI fixes
- typo fixes
- isolated tests
- simple docs updates

## Terminal execution policy

Recommended:
- Request review for terminal commands during early phases.
- Never allow destructive commands to run automatically.

Commands that need approval:
- rm -rf
- git reset --hard
- git clean -fdx
- sudo
- chmod -R
- chown -R
- npm install -g
- cargo install
- brew install
- winget install
- any command outside the repo

## Review policy

Recommended:
- Request review for implementation plans.
- Request review before merge.
- Require walkthrough artifact after each phase.

## Browser JavaScript policy

Recommended:
- Request review.
- Disable if not needed.

## Agent artifact requirements

Ask every agent to produce:
- task list
- implementation plan
- changed files
- test proof
- walkthrough
- known limitations

## Workspace pattern

Use separate workspaces or git worktrees for parallel agents.

Do not run multiple agents on the same branch editing the same files.
