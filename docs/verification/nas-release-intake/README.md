# NAS release-intake: begrensd opt-in profiel

**Tussenstand, geen VM-/releaseacceptatie.** Intakebron
`b6ec464b50325e90d2b949aa6f86a93eee7b24b3` behoudt main
`ca45a7dabe6a040921de03bdc2fad3e5045a75ba` en NAS-freeze
`ce52dd6165ba7af3bdcd1533308b556d1927443d`. Alle niet-NAS-bestanden zijn bij
intake exact main; de aparte NAS-workflow is de enige uitzondering.
V3-bronfingerprint bij intake:
`7490e7ea0334996297db54a171b710f474d564b3b8d0a9fd272976bd95ab1d2b`.

## Werkelijk uitgevoerd op intakebron

Bestaande read-only host-tools uit de UI-worktree, pinned Rust 1.90.0,
geen installatie of gedeelde targetwriter.

| Commando | Resultaat |
|---|---|
| `NOS_BUILD_DIR=build/nas-release-intake python3 -B tools/dev.py python3 -B services/nas/native-probe/build.py --target-dir target/nas-release-intake-native --output-dir build/nas-release-intake/users` | exit 0; twee ELF's gebouwd en geïnspecteerd |
| `python3 -B tools/dev.py cargo +1.90.0 test --offline --locked --manifest-path services/nas/native-core/Cargo.toml` | exit 0; 21 tests |
| `python3 -B tools/dev.py cargo +1.90.0 test --offline --locked --manifest-path services/nas/native-volume/Cargo.toml` | exit 0; 9 tests |
| `python3 -B tools/dev.py cargo +1.90.0 test --offline --locked --manifest-path services/nas/native-probe/Cargo.toml` | exit 0; 5 tests |

Voor elk van die drie manifests ook daadwerkelijk uitgevoerd:
`python3 -B tools/dev.py cargo +1.90.0 clippy --offline --locked --manifest-path <manifest> --all-targets -- -D warnings`
en
`python3 -B tools/dev.py cargo +1.90.0 fmt --manifest-path <manifest> -- --check`:
alle zes exit 0. Eigen hosttarget: `target/nas-release-intake-host`.

| Nieuw gebouwd artifact | Bytes | SHA-256 |
|---|---:|---|
| `build/nas-release-intake/users/nas-probe.elf` | 91688 | `608ddcd23492e6a55f70235395d2a49cb51bb643369c16b6bc3885ed153a7344` |
| `build/nas-release-intake/users/nas-volume.elf` | 97816 | `abd6321ad3963badfe7a0f8e5eba6df457f81aa70501d0dcf490dd3bb60fff14` |

Dit zijn nieuwe lokaal gebouwde hashes, niet de eerdere ce52-ELF-hashes.
Logs: `/tmp/nieuw-os-nas-release-intake-build.log` en
`/tmp/nieuw-os-nas-release-intake-{native-core,native-volume,native-probe}-{tests,clippy,fmt}.log`.

## Implementatiegrens van de volgende slice

De manager heeft TL-REL exclusief het opt-in profiel
`native-system-nas-probe` / `test-system-nas-recover` toegewezen.
Het nieuwe kernelmodulepad vervangt alleen voor dit profiel de bestaande
system-fixturepolicy. De bestaande vier ELF-slotbindings worden hergebruikt;
`kernel/build.rs`, `system.rs`, paging/boot/loader, SDK-runtime, keyring,
volume en NAS-leafs blijven ongewijzigd.

Alleen NAS krijgt entropy, alleen volume krijgt gescopeerde blockrechten
(normaal READ/WRITE/FLUSH, koud READ). Admin/gast krijgen uitsluitend
rolgebonden IPC-endpoints. De tweede volumeingress houdt een geldige RECV
voor de bestaande wachtset maar geen actieve client-SEND. Geen nieuw
recht, syscall of wire-ABI.

**Nog niet uitgevoerd voor deze nieuwe profilecode:** kernelbuild,
normale/koude QEMU-run, guestcaller-/grantasserties, stackmeting en
onafhankelijke diskoracle. Een rootrunner en de oracle-intake volgen apart.
Geen TLS, E2EE, apparaatidentiteit, stroomuitval- of volledige P04-claim.
R01-R03, P01-P04 en volledige VM-FR blijven open; scoreadvies **34% → 34%**.
De definitieve mainbasis na PR #74 en herhaalde gates blijven verplicht.
Centrale PROJECT-STATE/assessment blijven bij de manager.
