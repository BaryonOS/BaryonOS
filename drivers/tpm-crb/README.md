# TPM2 CRB transport core

Safe/no_std and dependency-free, solely for the separately documented
QEMU10.0.13 locality0 CRB profile. Codecs and register transport are separate.
There is no MMIO adapter, physical TPM write, kernel integration or native
anchor claim.

Contract, limits, sources and open acceptance:
[TPM2-CRB.md](../../docs/contracts/TPM2-CRB.md).

`cargo +1.90.0 test --offline --locked --manifest-path drivers/tpm-crb/Cargo.toml`
checks the host models. An actual `x86_64-unknown-none` library build
demonstrates target compatibility; actual adapter/VM/cold-start experiments
follow separately. The normative TCG PTP PDF was inaccessible and remains
an explicit open source check. No host-register model is presented as a TPM
or rollback protection.
