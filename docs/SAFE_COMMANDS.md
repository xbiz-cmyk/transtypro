# Safe Command Policy

## Allowed without special approval

```bash
pwd
ls
git status
git diff
git log --oneline -5
npm install
npm run dev
npm run build
npm run lint
npm run test
cargo fmt
cargo clippy --all-targets --all-features
cargo test
```

## Ask before running

```bash
rm -rf
git reset --hard
git clean -fdx
git push --force
sudo
chmod -R
chown -R
npm install -g
cargo install
brew install
winget install
powershell commands outside repo
commands modifying files outside repo
```

## General rule

If the command can delete, overwrite, change system settings, or install globally, ask first.
