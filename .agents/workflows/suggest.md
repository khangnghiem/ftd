---
description: Structured suggestions with analysis, tradeoffs, and priorities
---

# Suggest Workflow

When the user asks for suggestions, follow this structured approach:

1. **Understand scope**: Determine what area the user wants suggestions for:
   - A specific file or feature (narrow)
   - A crate or subsystem (medium)
   - The whole project (broad)

2. **Research**: Before suggesting anything, gather context:
   - Read relevant source files, docs, and specs
   - Check `docs/REQUIREMENTS.md` for planned vs completed features
   - Check `docs/CHANGELOG.md` for recent changes
   - Check open issues/PRs for in-flight work
   - Review `examples/` for current usage patterns

3. **Categorize suggestions** using these buckets:

   | Category        | Icon | Description                                   |
   | --------------- | ---- | --------------------------------------------- |
   | **Quick Win**   | 🎯   | Low effort, high impact — do it now           |
   | **Enhancement** | ✨   | Medium effort improvement to existing feature |
   | **New Feature** | 🚀   | Significant new capability                    |
   | **Refactor**    | 🔧   | Code quality / architecture improvement       |
   | **Bug Risk**    | ⚠️   | Potential issue worth investigating           |
   | **DX**          | 🛠️   | Developer experience improvement              |

4. **Format each suggestion** as:

   ```markdown
   ### <Icon> <Title>

   **Effort:** Low/Medium/High · **Impact:** Low/Medium/High

   <1-2 sentence description of what and why>

   **Tradeoff:** <What you'd give up or risk>
   ```

5. **Prioritize**: Order suggestions within each category by impact-to-effort ratio (best ROI first).

6. **Limit scope**: Max 10 suggestions per invocation. If there are more, mention "there are more — ask me to go deeper on any area."

7. **Present to user** and ask which ones to act on. Do NOT start implementing unless asked.
