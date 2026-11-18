# Plugin rules
Read the root instructions and docs/TOKEN-GUARD.md. `plugins/token-guard/` holds the one canonical Token Guard skill; the adapters in `.agents/skills`, `.claude/skills` and `.github/skills` only point to it, so keep them consistent. Keep `plugins/token-guard/LICENSE` unchanged.
Token Guard is a presentation helper, not a sandbox or test oracle: it must preserve exit codes and raw logs under `build/tokenguard/`.
After a change: `python3 -B -m unittest discover -s tests -p 'test_tokenguard*.py'` and `python3 -B tools/package_token_guard.py`.
