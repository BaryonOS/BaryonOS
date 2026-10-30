# PR #71 review repairs

Reviewed source: `3cf2fa9b5b51fdac84c9bd0eed964b5029606a59`. The correction adds the Python replay-handoff suite to the existing native-probe host/target CI job, makes its repository-boundary negative test independent of any pre-existing target directory, and clearly marks old client CI statements as historical while updating the README evidence hash.

The original test fails with FileNotFoundError in a clean snapshot (exit 1). The corrected suite passes all nine tests (exit 0), before and after which target/ is absent and no repository fixture remains. The assertion specifically checks the repository-boundary error. `manifest.json` records exact source and replacement blobs, commands and exit codes; the logs preserve both outcomes.

The accompanying merge preserves main `c210366dd0f40c61c9f06f251bf525bebdbfbf90`, whose delta from `d791a342` is 918 added historical evidence files with no changes to existing files. Existing identical evidence is retained by blob identity. These host tests are not a new guest run.

Exact previous-boot handle replay still requires the separately documented native export, handoff and startup integration. The compatibility note records the missing binding in candidate PR #96; no native NAS, E2EE or product acceptance is claimed. Roadmap remains 34/100.
