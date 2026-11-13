# K06 timing repair: per-boot data, 12 November 2026

Diagnostic data, not acceptance evidence. The analysis is in
[TIMING-REPAIR.md](../../TIMING-REPAIR.md).

## `boots.tsv`

One row per guest boot, tab-separated. Empty fields mean "not printed by that source" or
"the boot stopped before the value was printed"; margin records are printed at the end of
a boot, so a boot that failed earlier has none.

| Column | Meaning |
|---|---|
| `series` | `local-A`, `local-B`, `local-AB` or `hosted-43286858831` (see below) |
| `commit` | Source of the image. `c3c2890+` is the working tree that was committed unchanged as `4616fb3` |
| `qemu` | QEMU version |
| `host_cpus` | Host CPU set of the boot (`taskset` range), or the runner size |
| `case`, `boot` | Case of `tools/run_smp.py` and repetition index |
| `passed`, `failure` | Verdict of the oracle of that commit, and the marker or oracle message of a failed boot |
| `duration_s` | Host wall-clock time of the boot |
| `timer_ratio` | `SMP:TIMER:RATIO` (accepted band 50 000–75 000) |
| `ticks` … `own_median_us` | Fields of `SMP:CLOCK:EDGES`: counted IRQ0s, lost PIT periods, the part charged to the host, counted IRQ0s beyond the periods, longest IRQ0-entry-to-halt span, raw median segment, median segment without host-charged periods. Nominal segment: 999 847 µs |
| `sync_ms`, `sync_stall_ms` | Sync wait and its longest run without progress (limit 2 000 ms) |
| `scratch_ms` | Scratch phase wait (limit 500 ms) |
| `lockpair_ms` | Lock-pair wait (limit 2 000 ms) |
| `lock_max_ms` | Longest contended `IrqLock` wait of the boot (limit 400 ms) |

## Series

| Series | Image | Host | Remarks |
|---|---|---|---|
| `local-A` | `ed05150`, `native-smp` | Development container, QEMU 8.2.2, `pc-q35-8.2`, CPUs 0–3 | Margin records and the progress wait, no clock sampling. 1 ms polls in the sync wait |
| `local-B` | `cc1043f`, `native-smp` | Same; CPUs 0–3, then CPUs 0–1 | With clock sampling. For the CPUs 0–1 part the host watchdog was raised from 60 s to 600 s to observe the tail; builds ran on CPUs 2–3 at the same time |
| `local-AB` | `67e592c` and `c3c2890+`, `native-smp` | Same; CPUs 0–3 | Interleaved one boot at a time in the order A B B A, to measure the cost of the instrumentation under the unchanged 1 % clock rule |
| `hosted-43286858831` | `cc1043f`, `native-smp` and `native-smp-m3-load` | GitHub-hosted `ubuntu-26.04`, 4 vCPUs, QEMU 10.2.1, `pc-q35-10.0` | Workflow `native-smp.yml`, input `margin_diagnostic`; five cases × 50 |

Sync values of `local-A`, `local-B` and the hosted series count 1 ms polls; the committed
source polls every 10 ms, as the fixture did before the repair.

## Hosted artifacts of run 43286858831

| Artifact | Id | Zip SHA-256 |
|---|---|---|
| `DIAGNOSTIC-NOT-ACCEPTANCE-smp-margin-four-43286858831-1-reports` | 13976801095 | `aa6ec81765f6416ec9999215b6575a35422b6fb94d25b2dcd594f052ba8e8504` |
| `DIAGNOSTIC-NOT-ACCEPTANCE-smp-margin-eight-43286858831-1-reports` | 13975273263 | `adaeca4901c705cb4ce1aafa0cba74105e86fe41d1b4e913fdd73ce92966566d` |
| `DIAGNOSTIC-NOT-ACCEPTANCE-smp-margin-nine-capacity-43286858831-1-reports` | 13975902717 | `2106b7349735f76c7648192860de9aaed0eafda33c24dbc6c25ca284b82aaef4` |
| `DIAGNOSTIC-NOT-ACCEPTANCE-smp-margin-m3-load-two-43286858831-1-reports` | 13977811888 | `167ae6f40d6bbf5eeb405a7db08dbdc38df6d5b35275cd989a24befc5cfa3eff` |
| `DIAGNOSTIC-NOT-ACCEPTANCE-smp-margin-m3-load-eight-43286858831-1-reports` | 13978626330 | `ff55637a2b85727759a88f5e1291c0454bb57736c1ee1e9b7d73eb014db80275` |
| `DIAGNOSTIC-NOT-ACCEPTANCE-smp-gate-rehearsal-43286858831-1-reports` | 13976132393 | `44ffb73dbc5562d1378bde3a46f104e94fc1af84f28050e44dd5887b0f55b46f` |

The downloaded zip files matched these digests. GitHub removes the artifacts on
15 November 2026; `boots.tsv` keeps the values used in the analysis, not the serial
transcripts. The gate rehearsal ran `tools/gate.sh` once on the final-gate runner: every
executed stage passed (`host-tests` 259 s, `clippy`, `fmt`, `m2` 22 s, `k05` 32 s,
`k05-smp` 88 s, `k07` 20 s, `rtc` 12 s) in 12 min 55 s. It is a rehearsal of the stage
that no rejected matrix ever reached, not the final gate of an acceptance run.

## `local-first-boots.txt`

Single boots of the first implementation (`cc1043f` before its last accounting fix),
kept as recorded.

## `final-verification.txt`

Step summary of the final local verification of `4616fb3`: lint, host tests, builds and one
boot of twelve cases, with exit status, duration and the marker lines of each boot. The
serial transcripts of these boots were not kept.

## `raw-reports.tar.gz`

The reports from which `boots.tsv` was derived, and the scripts that produced them, as one
reproducible archive (sorted members, zeroed timestamps). Diagnostic material, kept so that
the table can be recomputed after the hosted artifacts have expired.

| Member | Content |
|---|---|
| `local/series/*.json`, `*.log` | Series `local-A` (`a-…`) and `local-B` (`b-…`; `h0-3` and `h0-1` name the host CPU set): one row per boot with verdict, duration, failing lines, every margin record and the clock accounting; the logs of series A hold the serial output |
| `local/ab/*.json`, `*.log` | Series `local-AB`: one report per boot, arm A (`67e592c`) and arm B (the source of `4616fb3`) |
| `hosted-43286858831/<case>/margin-report.json`, `receipt.json` | The report and receipt of each hosted diagnostic job (50 boots per case) and the receipt of the gate rehearsal. Serial transcripts are not included; they remain in the artifacts listed above until GitHub removes them |
| `scripts/` | `series2.py` (one series on a host CPU set), `ab.sh` (the interleaved comparison), `hosted.py` and `analyse.py` (summaries), `mk_boots.py` (builds `boots.tsv`), `final.sh` (the final local verification), as run |

The absolute path of the session work directory was replaced by `<scratch>` in every
member; nothing else was altered. The local reports hold 268 boots and the hosted reports
250, the 518 rows of `boots.tsv`.
