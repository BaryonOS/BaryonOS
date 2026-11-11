# English translation: verification and publication record

The active project-authored documentation and community text within the recorded PR #269 translation scope have been translated into English, and the recorded GitHub community updates were published. Source integration with the then-current `main` baseline completed and its targeted checks passed. This historical report does not claim that every active document in the current repository is English. The product score remained **41/100** at that checkpoint. This work provides no new native OS, hardware or product acceptance.

**Report date:** 9 November 2026. **Working branch:** `docs/english-project-20261109`.
**Community publication:** complete for the recorded scope below.
**Source integration:** compared and validated against `2ed4c087746021da3c8b96511af3209a35d0f441`; source changes are delivered through [PR #269](https://github.com/simonvanieperen/BaryonOS/pull/269). Consult the live PR for its current check and merge state; this report does not assert that the translation branch has merged or that an OS release has been deployed.

## Source and scope

| Reference | Meaning |
|---|---|
| `2ed4c087746021da3c8b96511af3209a35d0f441` | **Final integration comparison baseline.** Separate PR #273 reached `main` while the translation was in progress, introducing a compact English README, canonical current-status/roadmap structure and explicit MPL-2.0 licensing boundaries. Reconciliation and bounded checks against this baseline passed. |
| `9922810a18f2272f534e1539e2d980d3d8922bb3` | Original translation baseline used by the preserved language/link audit and initial preservation checks. This remains the baseline recorded in those historical artifacts. |
| `f704f078213170e72292a5a491f0ee83bd588955` | Earlier source-comparison and independent tooling-review baseline, including the separately integrated security work. At that comparison, the translation added no Rust, C, assembly, linker-script, Cargo manifest or Cargo lockfile changes. This earlier finding is not substituted for a final comparison against `2ed4c087`. |
| `a462e0dda14658db7e4c4fa0dbad8f971201022d` | HEAD during the recorded language/link audit and targeted validation. Both examined the working tree, including relevant uncommitted integration changes. Input hashes in the validation log identify the tested bytes. |
| Final changed-path and Markdown-file counts | **310 changed paths** using `git diff --no-renames`, including **267 active Markdown documents** and this verification report (**268 Markdown paths**); old and new form filenames count separately; comparison is against `2ed4c087`. The active-document total excludes this verification report and preserved historical evidence. |

The newer `main` structure and licensing decisions are incoming authoritative project changes. Integration preserves that canonical structure, the new licence files and their scope rather than restoring the older expanded entry-point documents. The translations add no independent licensing decision. Earlier command logs, audits, baseline identifiers and input hashes remain historical records of the inputs they actually examined.

The translated scope includes project entry points, architecture and implementation specifications, contracts, current research and user documentation, component documentation, contributor instructions, issue/discussion templates, agent roles, review configuration and generated progress presentation. The implementation/contract handoff covered all **62 implementation documents and 14 contracts**, preserving complete requirements and evidence distinctions rather than replacing long documents with summaries.

Historical archives, raw execution evidence and source-corpus bodies remain preserved. Licence texts are not translated or silently relicensed; the newer licensing boundary from `main` is retained during integration. Original quotations retain their wording, with clearly labelled English translations where needed. Literal UI strings, test fixtures, protocol values, paths, identifiers, hashes and product names such as **Materiaal** and **Veld** have not been silently renamed. English narrative does not imply that the existing OS interface has been localised or that historical tests were rerun.

## Final integration checks

The combined source was checked after local merge `eb8c1e927a3322ed0ae1a9c312296767ab9cdf8d`, with final working-tree input hashes recorded in [postmerge-validation.json](postmerge-validation.json). All 18 pipeline tests passed again after the PR-message wording correction. The progress check passed at **41/100** with main's compact renderer and the preserved Dutch progress anchor. All six changed Python sources compiled in memory and **27 changed JSON/TOML/YAML inputs** parsed successfully, including the renamed forms; CodeRabbit remained `en-GB`. Captured test inputs and the configuration inventory stayed stable during the run.

The corpus validator and six search tests were not needlessly rerun. Five relevant implementation/test inputs match their earlier logged hashes, and 19 protected corpus inputs match the original tested Git snapshot. The earlier passing **3,132-record integrity check** and **six search tests** therefore remain applicable to the unchanged inputs. This continuity check is distinguished from a new execution.

The [post-merge preservation audit](postmerge-scope-audit.json) compares against `2ed4c087`: all **506 Rust files**, **1,608 component non-Markdown files**, **187 build/configuration/CI files**, the **four applied licensing files**, **28 frozen source/corpus assets**, and **4,407 existing history/evidence files** remain unchanged. All **51 criteria in ten tracks** match main in every criterion field: 21 accepted, 30 open, 41 points and 48 evidence references to 40 unique paths. These scope sets overlap; their counts must not be added together.

A concurrent workflow advanced the remote translation branch to `6dcae7c03e3d2882975d1acaa9e4d5aa78e43a11`. All 37 incoming changed paths equal their corresponding `main` version at `2ed4c087`; the two update-tool Rust files and two security-verification reports are already preserved exactly in the local tree. They are prior independent work, not new runtime edits by this translation. No branch force update is required.

The [post-merge link audit](postmerge-link-audit.json) covers **267 changed active Markdown documents**, their relevant inbound links and the Discussion template's internal repository reference. It finds **no new missing local targets or broken heading links**. All **11 required compatibility anchors** are retained, together with **23/23 explicit anchors from pre-merge commit `547ee50`** and **27/27 from initial audit HEAD `eb8c1e9`**. The audit checks **5,213 scoped references**, with **5,212 present file/directory targets** and **4,541 working fragment references**; the only absent repository target is the unchanged generated `build/toolchain-provenance.json` reference. The new `COMMUNITY.md` alias also preserves the existing Discussion form's project-contract link. Existing generated-artifact references and historical archive limitations are reported separately, not relabelled as translation failures.

The four renamed Discussion forms retain their 23 field schemas, IDs, types and validation requirements. Their payloads remain byte-identical to the already translated pre-rename forms. Compared with Dutch `main`, 66 presentation fields, including 22 dropdown choices, intentionally use English; no repository tool or workflow consumes those choice labels as machine tokens. Claiming that every dropdown value was unchanged would therefore be inaccurate.

### Concrete review findings

Older CodeRabbit reviews on [`47af0d0`](https://github.com/simonvanieperen/BaryonOS/pull/269#pullrequestreview-5820308098) and [`6cdec9c`](https://github.com/simonvanieperen/BaryonOS/pull/269#pullrequestreview-5820370426) were checked against the reconciled source. The confirmed issues were resolved: compatibility anchors; explicit English/Dutch pipeline-field guidance; PR merge readiness distinguished from criterion acceptance; current capability-mask and privacy-revision references; English command placeholders and number formatting; current Baryon OS prose; and quantum-table/image-limit Markdown rendering. Actual command paths and script filenames remain unchanged. The reported profile-observation list defect was already absent from the current source.

The old licence-publication wording was superseded by main's applied licensing documentation. The regex hint does not demonstrate a new vulnerability: the track ID is restricted to one uppercase letter before use in the pattern, and the existing main behaviour is retained. Earlier remote reviews and their advisory coverage warnings are not presented as approval of the final head. The Graphify static review explicitly did not run tests or prove the changed functions; the separate local tests above provide their own bounded evidence. Normal current-head review and CI remain authoritative for merging.

## Intended code and configuration behaviour

The change includes a small amount of development tooling so that English documentation remains usable and reproducible. The following describes the intended translation changes. The post-merge checks below verify the reconciled tooling, configuration and generated-document structure; earlier logs retain their original input scope.

| File or surface | Intended behaviour and compatibility |
|---|---|
| [`tools/agent_pipeline.py`](../../../tools/agent_pipeline.py) | Task parsing accepts `Criterion:` and the existing `Criterium:` alias. Exactly one criterion field across both spellings must identify an open criterion; duplicate aliases are rejected even if their values agree. `Gate:` validation is unchanged. Dependency references are collected from both `Depends on:` and `Afhankelijk van:`. Existing Dutch tasks remain accepted. |
| Pipeline prompts and draft PR text | Agent instructions, feedback and generated descriptions use English while retaining scope restrictions, Graphify guidance, gate requirements and draft-PR behaviour. The translation does not relax capability permissions or publication controls. |
| [`tools/progress.py`](../../../tools/progress.py) | Diagnostic text, README/table output and SVG labels use English and the Baryon OS name. Criterion identifiers, weights, statuses and evidence paths remain unchanged. The renderer emits the legacy README progress anchor so regeneration retains existing fragment links. The calculation remains **41/100**. |
| Bundled skill search and validator | Error/diagnostic messages use English. Search, source identifiers, source-body checksums, count checks and integrity validation remain unchanged. |
| [`build_corpus.py`](../../../.agents/skills/os-dev/scripts/build_corpus.py) | Future generated book headings and index titles use `page` instead of `pagina`. This fixes the remaining future-build title issue; the existing corpus was not regenerated. This report claims source inspection and compilation of the fix, not a newly executed corpus build. |
| GitHub forms and agent configuration | User-facing descriptions and prompts use English. The independent review reported unchanged form IDs, input types, required flags and all 16 agent-tool permission entries. |
| [`.coderabbit.yaml`](../../../.coderabbit.yaml) | Review language is `en-GB`; other review-policy behaviour is unchanged. Local validation checked YAML syntax and the language value, not remote provider acceptance. |

The compatibility tests explicitly compare English and Dutch task fields and reject duplicate criterion aliases. They exercise development tooling, not kernel or guest execution. See the complete command record in [validation.log](validation.log).

## Executed targeted validation on recorded inputs

These results apply to the earlier integrated inputs identified above and in the log; they are not a rerun or acceptance of the final tree after incorporating `2ed4c087`.

The validation used the existing Python **3.12.14**, PyYAML and `tomllib`. No dependency installation was necessary. Python bytecode writing was disabled; the six changed Python sources were compiled in memory.

| Check | Executed command or method | Result |
|---|---|---|
| Pipeline/gate tests | `python3 -B -m unittest discover -s tools/tests -p test_agent_pipeline.py -v` | **18 tests passed**, exit 0, no skips. |
| Bundled OS skill integrity | `python3 -B .agents/skills/os-dev/scripts/validate_skill.py` | **PASS**, exit 0: **3,132 source records**, comprising **3,054 Wiki pages and 78 book pages**. |
| Bundled search tests | `python3 -B -m unittest discover -s .agents/skills/os-dev/tests -p test_zoeken.py -v` | **6 tests passed**, exit 0, no skips. |
| Progress calculation and generated views | `python3 -B tools/progress.py --check` | Exit 0: **41/100 points (41%), rubric v1, consistent**. This checks calculation, references and presentation, not the validity of historical execution claims. |
| Changed Python sources | In-memory `compile(source, path, "exec", dont_inherit=True)` for all six changed `.py` files; exact argument vector recorded in the log. | All six compiled, exit 0; no bytecode files written. |
| Changed data/configuration files | `json.loads`, `tomllib.loads` and `yaml.safe_load` over the 19 changed JSON/TOML/YAML files present at validation time. | All 19 parsed, exit 0. The explicit CodeRabbit assertion confirmed `language == "en-GB"`. This is syntax checking, not complete service-specific schema validation. |
| Existing CodeRabbit tests | Targeted search of the existing relevant test directories. | No dedicated test found. The recorded `rg` exit 1 means no matches, not a failed test. The direct YAML/language check above covers the changed setting locally. |

The six Python files were `tools/agent_pipeline.py`, `tools/progress.py`, `tools/tests/test_agent_pipeline.py`, and the bundled skill scripts `build_corpus.py`, `validate_skill.py` and `zoeken.py`.

The [log](validation.log) preserves exact commands, argument vectors, stdout, stderr, exit codes, durations, UTC timestamps and input SHA-256 values. Its initial run observed no input changes during execution. After a concurrent root edit to the future-build title, a narrow compilation recheck of `build_corpus.py` also passed with a stable source hash:

`731ac7898f34a6f3de6bb20d6230b23de358731b9b0543ad83613da33812382b`.

No failed checks were silently repaired or relabelled. No Rust, QEMU, physical-hardware, product browser-UX or human usability test was executed for this translation. Browser interaction used to publish GitHub text is not a product UX test. No new criterion is accepted and no score increase is proposed.

## Subsequent bounded documentation checks

Two hardware documents were subsequently reconciled against incoming `origin/main` at `3947aeb21886b6dcfd8a256cd8f03469954f91b0`. The `translate_specifications` handoff reports that [`DRIVER-ARCHITECTURE.md`](../../drivers/DRIVER-ARCHITECTURE.md) retained its complete English body, with the comparison accounting for three new reference lines. [`HARDWARE-TARGETS.md`](../../drivers/HARDWARE-TARGETS.md) was translated in full: **74/74 lines**, all **3 profile rows and 10 driver-family rows**, **23 code spans**, **4 link targets**, **39 numeric tokens**, **9 criterion-ID occurrences**, **5 headings** and **3 ordered items** were preserved. No conflict markers remained; all relative link targets existed; working-tree and staged `git diff --check` both exited 0. Guest-DMA/IOMMU evidence limits, serial-adapter placement, the unproven screen-output route and USB-boot/no-internal-disk restrictions were retained. These were documentation and structural checks, with no hardware execution or new acceptance.

Discussion-category synchronisation was committed separately as `0891f42c3ba2c38f1411bd9e237ecb50d993b53e`. The [independent Discussion review](discussion-review.json) records **10 active category links** using the six observed English slugs, **6 YAML files parsed**, and **4 renamed Discussion forms** whose bodies were byte-identical to baseline `a462e0d`. The forms retained field counts **6/6/5/6**, including their original IDs and requirements. The four filenames now follow `architecture-and-design`, `product-and-user-experience`, `questions-and-help` and `test-results-and-demos`; the remaining two live slugs are `announcements` and `polls`. The scoped whitespace check passed. The subsequent post-merge link and configuration checks below cover the newer-main reconciliation.

A further `translate_specifications` handoff reconciled `docs/DECISIONS.md`, `docs/DEPENDENCIES.md` and `docs/implementation/EUROPE-AND-RELEASE.md` with `2ed4c087`. The existing English decision history remained a byte-identical prefix with all nine existing heading aliases; the new licensing ADR retained incoming-main wording apart from trimmed trailing Markdown whitespace. Only the M5 licence status changed in the dependency document. The Europe/release licensing section retained the incoming explicit software scope, exclusions and the absence of release, SBOM, privacy or safety approval; earlier matrices, history and bibliography remained byte-identical. Four new local targets existed, the licensing section's code/link/numeric sequences matched incoming `main`, and two legacy heading aliases were added. Scoped working-tree and staged whitespace checks passed with no conflict markers. This validates those three resolved documents only; the separate post-merge checks below cover the combined source.

## Independent source and translation reviews

### Tooling compatibility review

A separate AI agent (`translate_components`) reported an independent source review against `origin/main` at `f704f078213170e72292a5a491f0ee83bd588955`, including the observed working-tree progress-anchor change. No unintended behaviour or security regression was found in that bounded diff. The review covered task-field aliases and duplicate rejection, unchanged progress identifiers/weights/statuses/evidence, form structure, all 16 agent-tool permission configurations, the CodeRabbit language-only change, and preservation of source hashing and search behaviour. The intended parser-alias logic was the only non-string AST change found; changes to dependency collection order do not affect the membership-based readiness checks.

The reviewer executed `python3 -B -m unittest tools.tests.test_agent_pipeline.Planning -v`: **9 tests passed**, exit 0. This is an additional attributed review result; the separately logged 18-test pipeline run above contains the complete targeted module. The review also compared non-string constants, TOML, YAML, agent frontmatter and the progress register. It performed no runtime gates, remote writes or corpus regeneration.

The review identified one P3 language-completeness issue: the generated book heading already used `page`, but the adjacent source-index title still contained `pagina`, so future English title searches would remain inconsistent. Root corrected the remaining index-title occurrence to `page`; the subsequent source compilation is recorded above. This is an attributed AI source-review handoff, not a human audit or a claim that every external integration was executed.

### Recorded active-prose and local-link audit

The copied [read-only audit](english-final-readonly-audit.json), named final at its earlier `a462e0d` checkpoint, reports the following. It retains its original `9922810` comparison baseline and has not been relabelled as an audit of the newer-main integration:

- **280 active Markdown files** scanned for residual Dutch narrative, with no concrete missed active prose after manual review of candidates and preserved exceptions.
- **648 Markdown files** parsed for local links, with **4,543 working fragment links**.
- **Zero newly broken heading links, zero new unresolved fragment links and zero newly missing local file targets**.
- All **15 initial translation-induced link breakages resolved**, using **11 required compatibility aliases** and a generated legacy progress anchor.
- All **646 baseline heading entries** checked against original Git heading lines.

At that audit, five pre-existing archive links containing fragments were unresolved because their archived target files were absent. They occur in the preserved `docs/history/integration-2026-10-30/` snapshots `main-9a6703d3`, `pr65-current-7960e98a`, `pr65-main748-0e441a93`, `pr65-main94-ec591dd8` and `pr75-c26f57b6`, each referring to the historical host-test-linker-preflight target. They were not introduced by heading translation. The broader archive scan also classified 1,062 pre-existing missing-local-target occurrences; it did not rewrite archived evidence to conceal those historical limitations. That source audit performed no external HTTP-link validation.

### Pull-request translation review

The copied [PR review findings](english-pr-review-findings.json) record complete independent comparison of **22 supplied Dutch/English PR title and body pairs**, with no concrete meaning, numeric, authority, checkbox, evidence-boundary or link errors. Original code spans, URL/link targets, checkbox states and table-row counts were retained. Historical scores and caveats were not converted into current test claims.

That review also records preparation of the four remaining translations, PRs **260, 259, 167 and 155**, with literal/link/checkbox/numeric/table preservation checks. The copied review is a pre-publication artifact: its statement that publication was not performed applies to that review/preparation step. Later live edits are reported separately below.

### Additional issue and PR translations

The [additional metadata record](additional-metadata-translations.json) contains the complete source/target title and body for issue **#283** and PR **#284**, plus sentence-by-sentence semantic review and passing ordered checks for code spans, links, numeric tokens, hashes, issue references, checkbox states, heading levels, table structure and numbered lists. These extend the published scope to **27 PRs and 5 issues**. Their source hashes and historical claims remain distinct from current implementation acceptance.

### Discussion starter-post review

The [Discussion translation pairs](discussion-translations.json) and [independent AI review](discussion-review.json) cover all five complete titles and bodies, **#252–#256**, with no concrete findings. The review preserved **32 canonicalised URL occurrences**, **10 inline code spans**, table/fence structure and each post's original baseline `363b486b06023d3d578f8eca2b23ff49045a1757` and **8 November 2026** date. Numeric sequences match apart from the authorised, clearly labelled translation date. Repository URL prefixes were updated to Baryon OS while retaining paths; the shared-language rule in #252 was explicitly changed to English under the user's request.

The review compares supplied source/target texts. The [saved final Discussions page](discussions-final.txt) separately records the six English category names/slugs and five published English starter titles. The coordinating session completed the full post updates. Neither the pair review nor the listing is a new review of comments, third-party discussion history or runtime behaviour.

### Milestone translation and read-back

All **10 existing milestones** have complete English titles and descriptions, with exact originals retained in [milestones-before.json](milestones-before.json) and full source/target pairs in [milestones-translations.json](milestones-translations.json). Preservation checks passed for **41 bullets**, **41 criterion-ID occurrences**, **23 issue-reference occurrences**, **12 source URLs** and **152 numeric tokens**, with original ordering, line/paragraph structure, authority restrictions, dependencies and evidence boundaries retained. The release requirement remains **Dutch user documentation**; translating the milestone does not authorise changing that criterion. NAS revocation is correctly assigned to the NAS administrator.

The [final milestone read-back](milestones-after.json), recorded at **2026-11-09 00:54:00 UTC**, confirms all ten titles and descriptions exactly match their prepared targets, all have **no due date**, and the final state is **10 open / 0 closed**. It also records the editing incident: milestone **8** was unintentionally closed and immediately reopened. The final read-back verifies restoration of the open state. No milestone completion or deadline was introduced.

## Published GitHub surfaces

Publication is complete for the following scope. This report uses the coordinating session's recorded API/browser observations and supplied before/after artifacts; this report-editing task performed no additional remote writes. Read-back timestamps identify snapshots, not a promise that unrelated concurrent workflows cannot subsequently change GitHub state.

| Surface | Completed scope | Publication evidence and boundary |
|---|---|---|
| Pull-request titles and descriptions | **27 PRs**, including #284. | [Live verification](github-live-verification.json) at **2026-11-09 00:46:30 UTC** records matching titles and bodies for all 27. It also records each PR's state at that snapshot; translation is not a merge or acceptance action. |
| Issue titles and descriptions | **5 issues**, including #283. | The same [live verification](github-live-verification.json) records matching titles and bodies for all five. Original and additional source/target records are retained. |
| GitHub Wiki | **9 original files translated**, plus **6 compatibility pages for the Dutch URLs**. | The coordinating session reports publication and retained access through the existing Dutch URLs. Wiki publication does not establish source-branch merge or release deployment. |
| Discussion categories | **6 categories translated**. | The [final page capture](discussions-final.txt) records Announcements, Architecture and design, Polls, Product and user experience, Questions and help, and Test results and demos. Local links and form filenames were synchronised as described above. |
| Discussion starter posts | **5 complete posts translated**, #252–#256. | The coordinating session completed publication; [source/target pairs](discussion-translations.json), the [independent review](discussion-review.json) and the [final listing](discussions-final.txt) document the textual review and visible publication. |
| Milestone titles and descriptions | **10 milestones translated**. | [Exact final read-back](milestones-after.json): ten matching titles/descriptions, ten open, zero closed and no due dates. The temporary close/reopen incident is preserved in that record. |
| Repository About text | **Updated to English**. | Publication was observed by the coordinating session. This is repository presentation metadata, not a source or product acceptance change. |
| Repository source | Full translation integrated with newer `main` at `2ed4c087` for [PR #269](https://github.com/simonvanieperen/BaryonOS/pull/269). | Post-merge source preservation, targeted tests and local-link checks pass. Live community publication and source-branch publication do not establish a merge into `main`. |

Separate PR **#273** supplied the newer canonical English documentation structure and licensing decisions on `main`. The coordinating session also observed translation PR **#269** being closed and subsequently reopened by a concurrent workflow. No merge of #269 is claimed here. The metadata verification artifact covers the 27 listed PR translations; its statement that no closed PR was reopened describes those translation edits, not all concurrent repository activity.

Existing historical results, evidence boundaries and the distinction between project-owner authority and end-user behaviour remain intact across the translated text. Published discussion history, raw evidence, licences and source archives are not represented as newly authored English runtime evidence.

## Evidence files and publication handoff

| File | Contents |
|---|---|
| [postmerge-validation.json](postmerge-validation.json) | Final integrated pipeline/progress checks, Python compilation, data parsing, input hashes and continuity of earlier corpus/search evidence. |
| [postmerge-scope-audit.json](postmerge-scope-audit.json) | Final source, runtime, licensing, criteria, forms and archival preservation checks; concrete component-document review fixes. |
| [postmerge-link-audit.json](postmerge-link-audit.json) | Final active-document and inbound-reference audit, compatibility anchors, baseline distinctions and specification-format corrections. |
| [validation.log](validation.log) | Actual earlier targeted commands, stdout/stderr, exit status, durations and input hashes, including the narrow Python compilation recheck. These results retain their recorded source scope. |
| [english-final-readonly-audit.json](english-final-readonly-audit.json) | Independent active-prose and local-link audit at the earlier checkpoint, copied byte-for-byte. |
| [english-pr-review-findings.json](english-pr-review-findings.json) | Independent review of 22 PR pairs and four additional translation-preparation results. |
| [pull-requests-before.json](pull-requests-before.json) | Original PR data retained before live edits. The historical validation log records a hash for a separate issue snapshot named `github-before.json`; that file is absent from the PR #269 donor tree and is not claimed as delivered evidence here. |
| [additional-metadata-translations.json](additional-metadata-translations.json) | Exact originals, translations, source hashes and preservation review for issue #283 and PR #284. |
| [github-live-verification.json](github-live-verification.json) | After-publication exact title/body comparisons for all 27 PRs and five issues, with observed states and a timestamp. |
| [discussion-translations.json](discussion-translations.json) | Complete original/English title-body pairs for the five starter posts. |
| [discussion-review.json](discussion-review.json) | Independent semantic/preservation review and local category-link/form checks. |
| [discussions-final.txt](discussions-final.txt) | Browser-observed final English Discussion categories and starter-post titles. |
| [milestones-before.json](milestones-before.json) and [milestones-translations.json](milestones-translations.json) | Ten original milestones and their complete English target texts with per-record preservation checks. |
| [milestones-after.json](milestones-after.json) | Exact post-publication comparisons, restored final open state, absent deadlines and the editing-incident record. |

The copied `english-final-readonly-audit.json` retains SHA-256 `4257eaedab72d2b108b9836fc244346b669f195099fcb1f190fe0b57c41a448f`; `english-pr-review-findings.json` retains SHA-256 `bc366868d7c97d3fc7255a541f29aed87c250b955d2d123019470840210a5b69`. Command input hashes remain in the validation log; later integration must not retrospectively relabel those earlier bytes as the final tree.

**Publication handoff:** the reconciled source and evidence belong to PR #269 and follow its normal review, CI and merge rules. The coordinating session publishes the complete Git tree and checks the remote tree against the local tree; connector-created commits may have different commit IDs because their parent history and creation metadata differ. The original validation log is tracked explicitly despite the repository's general `*.log` ignore rule. No runtime acceptance or progress point is introduced. Historical source, review and community-state records remain tied to their recorded observations.
