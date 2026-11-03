# TCP loss/backpressure native probe receipt

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
