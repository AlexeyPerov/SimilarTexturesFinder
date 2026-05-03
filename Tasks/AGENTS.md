# Repository guidance for AI agents

When working on **Similar Texture Finder**, attach **[Task.md](Task.md)** for scope, CLI + JSON contracts, MVP priorities, clustering rules, and config/result field meanings.

Attach **[Task_similarity_appendix.md](Task_similarity_appendix.md)** only when implementing or adjusting similarity metrics (pipelines, normalization, ORB/game-dev specifics).

Do not duplicate long metric narratives into chat — rely on those files instead.

Normative operational defaults (extensions, symlinks, exit codes, `groups[].score`) live in `Task.md` sections 0–6; avoid inventing alternate clustering or JSON semantics without updating the task first.
