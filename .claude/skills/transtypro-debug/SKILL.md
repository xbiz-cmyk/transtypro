---
name: transtypro-debug
description: Use to debug a failing build, test, Tauri command, audio/transcription issue, provider issue, or UI bug.
---

# transtypro debug skill

Debug process:

1. Reproduce the issue.
2. Capture exact error.
3. Identify smallest failing layer:
   - frontend
   - Tauri command
   - Rust service
   - database
   - OS-specific API
   - provider
4. Fix the root cause.
5. Add regression test if possible.
6. Document the fix.

Do not make broad rewrites while debugging.
