---
name: transtypro-review
description: Use to review a branch or phase for correctness, privacy, safety, test coverage, and fake implementations.
---

# transtypro review skill

Review the current diff.

Check:
- Does it follow SOUL.md?
- Does it follow AGENTS.md?
- Does it violate local-only privacy?
- Are API keys exposed?
- Are errors user-friendly?
- Are buttons wired to real actions?
- Are placeholders clearly marked?
- Are tests added for pure logic?
- Does the build pass?
- Are docs updated?

Return:

1. Blocking issues
2. Non-blocking issues
3. Missing tests
4. Privacy concerns
5. Recommended next action
