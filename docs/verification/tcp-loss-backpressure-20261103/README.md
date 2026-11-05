# TCP loss/backpressure native probe receipt

> **Samenvatting (NL, integrator 5 november 2026).** Deze opt-in-proef laat de testpeer de
> data-ACK voor een begrensd verzoek van 8192 bytes tijdelijk achterhouden. Daarbij moet de
> guest aantonen dat `send()` nul bytes accepteert. De peer controleert of de overlappende
> retransmissie dezelfde verzoekbytes bevat, geeft daarna de cumulatieve ACK vrij en
> verifieert herstel en nette afsluiting. TCP mag een herhaalde verzending anders
> segmenteren dan het origineel. De peer vergelijkt daarom bytes per sequentiepositie, niet
> pakketgrenzen. Bron: RFC 9293, "Transmission Control Protocol (TCP)", IETF, augustus 2022,
> https://www.rfc-editor.org/rfc/rfc9293 (§3.8.1 Retransmission Timeout, §3.10.7.4
> segmentverwerking). Na de integratorfix `8626e0a7` (maximaal 2 identieke duplicaten na de
> vrijgave, strikte causaliteit, baselinegrens van 1024 bytes) is het native bewijs
> hieronder, vastgelegd op de oude bronboom, voor de nieuwe boom **onbekend**. De Engelse
> tekst hieronder is het oorspronkelijke receipt en is ongewijzigd.
>
> **Actueel hostbewijs (branch-head na `8626e0a7`):** `python3 -B -m unittest discover -s sdk/examples/tcp-probe -p test_peer.py`
> 21/21 OK en `python3 -B -m unittest discover -s tests -p 'test_tcp_*.py'` 20/20 OK.
> **Native QEMU** (`tools/test_tcp_system.py --scenario baseline|loss-backpressure`) is na
> deze fix **niet uitgevoerd**; de status op de nieuwe boom is onbekend. De 17/17-receipt
> hieronder is historisch.

## Scope and proof boundary

Task: `GENERAL-N01-RETRANS-BACKPRESSURE`. Source base: `d10b5cdaf56774a39e3ef10463f674c6188a463a`; branch: `general-n01-retrans-backpressure`. The gate's source fingerprint is `64fd2283cedc6960c2ac70cd397cdfa70064e79a0141b1478e7eb12e7cbb2cc0`.

The opt-in probe sends an 8192-byte request through the existing native TCP path. In both normal and cold-recovery QEMU runs the guest observed a zero-length send while the peer withheld the data ACK, then completed the request after a retransmission. The peer captured three initial contiguous segments (512, 512, and 436 bytes), followed by an overlapping 1460-byte retransmission at the first sequence number. It checked the retransmitted bytes against the already received request before releasing a cumulative ACK. Nine subsequent request segments and the complete response/close exchange were observed. This accommodates valid TCP resegmentation; it does not require retransmitted packet boundaries to match.

The default/refusal TCP scenario and both existing normal/recovery native gates also passed on the same final source tree. This is native QEMU evidence for the existing kernel TCP probe and synthetic local peer only. It is not full N01/#39 acceptance: no userspace network service, real NIC, internet, physical hardware, or human security audit is proved.

## Verification

All commands exited `0`:

```text
python3 -B -m unittest discover -s sdk/examples/tcp-probe -p test_peer.py
Ran 17 tests

python3 -B -m unittest discover -s tests -p 'test_tcp_*.py'
Ran 17 tests

PATH=/home/simon/.cargo/bin:$PATH rustfmt --edition 2021 --check sdk/examples/tcp-probe/src/main.rs
git diff --check
```

The native gates used Rust `1.91.1`, QEMU `10.2.1`, machine `pc-q35-8.2`, and the existing OVMF/C toolchain artifacts under `/tmp`. They used isolated build targets under `/tmp/nieuw-os-general-n01-retrans-backpressure/`.

```text
python3 -B tools/test_tcp_system.py --scenario loss-backpressure
exit 0; normal and recovery passed; QEMU raw exit 33 for each

python3 -B tools/test_tcp_system.py --scenario baseline
exit 0; normal and recovery passed; QEMU raw exit 33 for each
```

The native invocations used:

```text
PATH=/home/simon/.cargo/bin:/tmp/sm-oom-qemu-root/usr/bin:/tmp/nieuw-os-c-toolchain/bin:$PATH
CC=/tmp/nieuw-os-c-toolchain/bin/cc
LD_LIBRARY_PATH=/tmp/sm-oom-qemu-root/usr/lib/x86_64-linux-gnu
OS_OVMF_CODE=/tmp/sm-oom-qemu-root/usr/share/OVMF/OVMF_CODE_4M.fd
OS_OVMF_VARS=/tmp/sm-oom-qemu-root/usr/share/OVMF/OVMF_VARS_4M.fd
NOS_QEMU_MACHINE=pc-q35-8.2
NOS_BUILD_DIR="$PWD/build/general-n01-retrans-backpressure"
NOS_TARGET_DIR=/tmp/nieuw-os-general-n01-retrans-backpressure/target
CARGO_TARGET_DIR=/tmp/nieuw-os-general-n01-retrans-backpressure/cargo
```

## Hashes

Probe ELF SHA-256: `51ccf9d91df495fccecac6612dc329f926b6ce1c8c4631b187b2c6ba0ce8c15c`.

| Evidence | SHA-256 |
|---|---|
| Opt-in gate JSON | `702cea183bb2dc6d9c1d0a8e58fb782c8300b77cddd56dc2167ce992d53702ef` |
| Opt-in normal result JSON | `2db53fb6d012e1c2bd1f1af45ea463c3fedd57f21f29732558423ffc97d5baa7` |
| Opt-in recovery result JSON | `e24c4b6316a976f5a6c7072f27e6238b5978cb5493f59bb5782d5bd27f52c41f` |
| Opt-in normal packet transcript | `1d005a927c803f46300dd5ee0972ce27a5c0df26bb33587c7fd12e2940ac5917` |
| Opt-in recovery packet transcript | `1d005a927c803f46300dd5ee0972ce27a5c0df26bb33587c7fd12e2940ac5917` |
| Opt-in gate log | `d2f8d2e0ba0456e969c81d64d6b096e7841ea568ba1241d84b6ae5ad5ea854c1` |
| Baseline gate JSON | `a34768e7be4f6d10a2704d44e10478b890d67b055746a1786bfd2739aba5f824` |
| Baseline gate log | `b818ac81ab1c762ea161e70b73991becf4d89883389e71e0397d112eeae7e20b` |

Generated QEMU images, serial logs, reports, and packet transcripts remain outside Git under `build/general-n01-retrans-backpressure/`; the two command logs are under `/tmp/nieuw-os-general-n01-retrans-backpressure/logs/`. The report JSON records each run directory and raw exit status.

## Integratiecontrole PR177, 3 november 2026

Bron `3dd73410473087b3a7c7fb020ea2f962d4d9ab0f` integreert main
`ab85d6339ce5bdddb10a2de171d0c1b04e4b62ff`. Alleen historische auditdocumenten
kwamen uit main; de transportcode bleef gelijk aan `6b1dee4b`.
Dezelfde source-fingerprint `64fd2283cedc6960c2ac70cd397cdfa70064e79a0141b1478e7eb12e7cbb2cc0`
is voor beide nieuwe gates gecontroleerd.

De hierboven genoemde peer- en transcripttestcommando's slaagden opnieuw
(17 en 17 tests, exit 0), evenals rustfmt en `git diff --check`.
Met dezelfde native omgevingsinstellingen, maar eigen paden, zijn uitgevoerd:

```text
NOS_BUILD_DIR="$PWD/build/pr-agent-d-177-loss"
NOS_TARGET_DIR="$PWD/target/pr-agent-d-177-native"
CARGO_TARGET_DIR="$PWD/target/pr-agent-d-177-host"
python3 -B tools/test_tcp_system.py --scenario loss-backpressure

NOS_BUILD_DIR="$PWD/build/pr-agent-d-177-baseline"
python3 -B tools/test_tcp_system.py --scenario baseline
```

Beide commando's: exit 0, vijf hostchecks exit 0, normal en cold-recovery
`passed=true`, `tcp_gate=true`, raw QEMU-exit 33. QEMU 10.2.1,
`pc-q35-8.2`, een CPU, TCG, 256 MiB; kernel Rust 1.90.0, probe Rust 1.91.1.
Beide loss-runs tonen 512+512+436 nieuwe bytes zonder ACK, vervolgens 1460
inhoudelijk gelijke overlappende retransmissiebytes, een cumulatieve ACK en
negen vervolgsegmenten. Baseline toont de expliciete RST-weigering op 8085,
geen loss-counters. Geen logforcing, quota- of retrywijziging; bestaande
timeouts blijven gelijk. Alleen de nieuwe backpressure-case heeft een eigen
begrensde 1500-ticktermijn. Het bewijst uitsluitend de kerneltransportprobe,
niet een complete app-/netwerkdienstketen of volledige N01-acceptatie.

| Nieuw artifact | SHA-256 |
|---|---|
| Loss-gate JSON | `d83bc66d7bb0b6e0ec796d8f8514d79a3d710665ae309fcf3e35cf61f7c06dae` |
| Baseline-gate JSON | `ebbec234c866d13bb123015b95c693729b67f3eee4762bd2e6e3429cc5d4caef` |
| Loss `network-peer.json`, beide runs | `1d005a927c803f46300dd5ee0972ce27a5c0df26bb33587c7fd12e2940ac5917` |
| Baseline `network-peer.json`, beide runs | `0857059674333e5d605ad47c0188469f89ab7eb207a09cf7cf12c3eab6b6dc62` |
| Probe ELF in beide gates | `fb0ec557c9ed840a3898605454004335ea0eb45994bffde19c6d8fba39b37421` |

De nieuwe ELF-hash is apart geregistreerd, niet gelijkgesteld aan de
oorspronkelijke build. De gate-JSON bevat de exacte runpaden en koppelt de
geteste ELF aan het manifest. Volledige logs: sessie-artifacts
`files/pr177-loss-native.log` en `files/pr177-baseline-native.log`; gegenereerde
images en serial blijven onder de bovengenoemde eigen buildpaden buiten Git.
Geen fysieke hardwareproef, menselijke audit of releasevalidatie uitgevoerd.
