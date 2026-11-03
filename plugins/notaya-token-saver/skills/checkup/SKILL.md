---
name: checkup
description: Read-only token checkup of Claude Code settings, CLAUDE.md size, skills and MCP servers. Run by hand with /notaya-token-saver:checkup.
disable-model-invocation: true
---

# Token checkup

Run the checkup script. It only reads files (settings, CLAUDE.md, skills, `.mcp.json`) and prints a report. It writes nothing and never prints the values of `env` entries.

```
node "${CLAUDE_SKILL_DIR}/checkup.mjs"
```

Then, in plain English and in a few lines:

1. Say which "THINGS TO CONSIDER" items matter most for this person and why. Do not add items the report did not raise.
2. Do not change any settings file yourself. Show the exact line to add and where it goes, and let the person decide.
3. If a number in the report is a rough estimate, say so.
4. Remind them that `/context` and `/usage` show what the script cannot see.
