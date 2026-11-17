# CodeRabbit free-tier repository configuration

Date: 17 November 2026. Parent source: `95d59ad52761c1f8f0201d934556902b90ce63cb`.
Branch: `codex/coderabbit-oss-config`. Task: CR-OSS.

## Scope and invariant

The repository explicitly enables free-tier features, draft PR reviews and coding-guideline discovery. Existing English reviews, quiet feedback, disabled automatic approvals, automatic incremental review and inherited review tools remain unchanged. No source path is excluded and no CI or branch-protection gate is changed. The obsolete docstring pre-merge check is removed because the current vendor schema marks it deprecated and states that it has no effect.

`enable_free_tier` enables free features; it is not a billing lock. Paid subscription and usage controls belong to the account settings. This change does not subscribe to a paid product, enable an Agent, authorize payment, or prove a future zero invoice.

## Validation

| Command | Exit | Local raw evidence |
|---|---:|---|
| `python3 -B tools/tokenguard.py run -- python3 -B build/coderabbit-oss/validate.py` | 0 | `build/tokenguard/run-k4pcrjam.log` |
| `python3 -B tools/tokenguard.py run -- python3 -B tools/progress.py --write` | 0 | `build/tokenguard/run-ixjdgl5f.log` |
| `python3 -B tools/tokenguard.py run -- python3 -B tools/progress.py --check` | 0 | `build/tokenguard/run-obodmbmj.log` |

The temporary validator used installed PyYAML and jsonschema to reject duplicate YAML keys, validate against the downloaded official V2 schema, compare the configuration with its parent allowing only the documented changes, check the vendor's docstring deprecation flag, and reject invalid-boolean and duplicate-key negative controls. [validation.json](validation.json) records source/schema hashes and validator version. Raw logs and the temporary validator remain local and are not shipped as project tooling.

To repeat schema validation after downloading the schema to a temporary file:

```sh
python3 -c 'import json,sys,yaml; from jsonschema import Draft202012Validator; Draft202012Validator(json.load(open(sys.argv[1]))).validate(yaml.safe_load(open(".coderabbit.yaml")))' /path/to/schema.v2.json
python3 -B tools/progress.py --check
```

## Operation and evidence limits

- Repository YAML takes precedence over dashboard repository settings. Existing PR branches need the configuration incorporated; merging this change does not rewrite those branches.
- CodeRabbit discovers root and nested `AGENTS.md`/`CLAUDE.md` automatically and scopes guidelines to their directories. No duplicate instruction tree is introduced.
- For public repositories with fewer than 10 stars, current vendor guidance requires starting reviews manually through the status comment or `@coderabbitai review`. `drafts: true` removes the repository's draft exclusion but does not override vendor eligibility or rate limits.
- Keep paid usage-based reviews, Deep Scan and Agent usage disabled in every account environment. A dashboard summary alone is insufficient when the detailed Usage page contradicts it; vendor confirmation is required.
- Hosted review behavior: NOT RUN during local validation. Required hosted CI remains pending until executed on the PR head. Native build/QEMU, hardware and human audit: NOT RUN, because this change only configures an external reviewer.
- Progress remains 41/100; no product criterion is newly accepted. Rollback: revert this bounded configuration and its checkpoint.

## Primary sources

Checked 17 November 2026:

- CodeRabbit, [Configuration V2 reference](https://docs.coderabbit.ai/reference/configuration), updated 14 November 2026, and [official schema](https://coderabbit.ai/integrations/schema.v2.json).
- CodeRabbit, [Code Guidelines](https://docs.coderabbit.ai/knowledge-base/code-guidelines).
- CodeRabbit, [Open-source activation and review eligibility](https://kb.coderabbit.ai/articles/8856795235-how-do-i-activate-coderabbit-pro-for-my-open-source-project).
- CodeRabbit, [On-demand billing and written confirmation](https://kb.coderabbit.ai/articles/9396519445-on-demand-reviews-your-free-period-and-what-happens-next).
