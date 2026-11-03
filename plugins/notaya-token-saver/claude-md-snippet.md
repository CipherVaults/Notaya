## Working habits (save tokens)

- Search first (Grep, Glob), then read only the lines needed. Do not re-read a file that is already in this conversation.
- Keep command output small: only failures, or the last 40 lines.
- Reply briefly: the result and what it depends on. Do not restate the plan or quote large diffs.
- Do not start subagents or parallel agents unless the job really needs them.
- If a command fails the same way twice, change approach or ask.
- Suggest /clear when the topic changes. Do not switch model in the middle of a session.
