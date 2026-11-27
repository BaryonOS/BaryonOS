# LLM Project Docs linter provenance

`docs-lint.py` is vendored from **LLM Project Docs 1.12.0** by DragonBall2 under
the MIT License.

- Upstream repository: <https://github.com/DragonBall2/llm-project-docs>
- Upstream commit: [`16cda78a697ae274aeda6fde01c231f0e5f29575`](https://github.com/DragonBall2/llm-project-docs/commit/16cda78a697ae274aeda6fde01c231f0e5f29575)
- Upstream path: `plugin/scripts/lint.py`
- Upstream SHA-256: `cf2f255958f1bc31ded36f00cdc6aff7a4863578b8e78da7dcf3e44c7b1eb154`
- Vendored SHA-256 after the local adaptation: `e94b41bffe6e4b0b3968c42dfaa7d707a861151df4cb8129619eda526dae1819`
- License: [MIT](LICENSE-llm-project-docs.txt)

The upstream bytes were verified against the named GitHub commit on 26 November 2026.
The only local code change sets the default documentation directory to
`docs/project-docs`, because BaryonOS already has a heterogeneous canonical `docs/`
tree. Explicit `--docs-dir` arguments remain supported.
