# Native herstelhardening — taakcontract en bewijs

29 oktober 2026. Basis `00e6af5db4d315d2536471ccbce1cb1f4cf14958`,
branch `codex/native-recovery-hardening`. Eigen Rust no_std-kernel,
x86_64, één CPU, UEFI/OVMF, QEMU/TCG; geen hardware of echte schijf.

Doel: herstel de concrete B01-P1-fouten uit #52 en voltooi de bruikbare
S08-timingwijzigingen uit #48. Invarianten: ingetrokken sleutels herleven niet
in een levende instantie; entropy-gezondheidstoestand blijft over buffers
bestaan; timeout/eigenaarsexit laat geen herbruikbare onbevestigde DMA achter;
opslagfoutcompletion telt mee in diagnostiek zonder write-replay/rechtenwijziging.
Geen nieuwe crypto, afhankelijkheid, hardwareclaim of nieuw roadmappunt.

Interfaces: B01 IPC blijft v1. Herstel van een andere master is alleen in een
ongebruikte dienst toegestaan; herstel van dezelfde master behoudt tombstones.
Kernel is generieke broker; crypto blijft ring 3. Root bezit gedeelde feature-
lijsten en bewijs. Builders hebben afzonderlijke worktrees (WORK-QUEUE.md).

Acceptatie: gerichte keyring-/driver-/harnastests en clippy/fmt; echte B01-gate
met niet-antwoordend apparaat en herstart; S08-gate en native-volumegate op
gecombineerde bevroren bron. Negatief: oude blob na revoke, masterwissel,
RCT/APT over buffergrenzen, ontbrekende completion, cap-/DMA-cleanup en
malformed/foutcompletions. Nog NIET UITGEVOERD bij dit eerste checkpoint.

Bronnen: projectcontract B01-ENTROPY-AND-KEYS / ADR-043, NIST SP 800-90B
§4.4 (doorlopende RCT/APT), OASIS virtio 1.2 (reset en queue-eigendom).
OSDev: Virtio W003311, revisie 30307, 17 maart 2026, is aanvullende leerbron;
Random Number Generator W002935 onderscheidt bronentropy en CSPRNG.
Geen corpuscode overgenomen. Rollback via gewone revert van herstelcommits;
B01 blijft open als het noodzakelijke bewijs niet slaagt.
