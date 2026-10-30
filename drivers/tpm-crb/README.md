# TPM2 CRB-transportkern

Safe/no_std en dependencyvrij, uitsluitend voor het afzonderlijk beschreven
QEMU10.0.13 locality0-CRB-profiel. Codecs en registertransport zijn gescheiden.
Er is geen MMIO-adapter, fysieke TPM-write, kernelintegratie of native ankerclaim.

Contract, grenzen, bronnen en open acceptatie:
[TPM2-CRB.md](../../docs/contracts/TPM2-CRB.md).

`cargo +1.90.0 test --offline --locked --manifest-path drivers/tpm-crb/Cargo.toml`
toetst de hostmodellen. Een echte `x86_64-unknown-none` librarybuild bewijst
targetcompatibiliteit; de werkelijke adapter/VM/coldstartproeven volgen apart.
De normatieve TCG-PTP-PDF was niet bereikbaar en blijft een expliciete open
broncontrole. Geen hostregistermodel wordt als TPM of rollbackbescherming
gepresenteerd.
