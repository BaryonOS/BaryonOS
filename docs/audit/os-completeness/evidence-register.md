# Bewijsregister

## Betekenis

| Niveau | Wat het hier betekent |
|---|---|
| `SOURCE_ONLY` | Code/spec bestaat; uitvoering is hiermee niet aangetoond. |
| `HOST_TESTED` | Test op host/runtime buiten de eigen OS-kernel. |
| `VM_TESTED` | Eigen kernel/dienst draaide in een emulator, binnen het vermelde profiel. |
| `HARDWARE_TESTED` | Uitgevoerd op een exact benoemd fysiek apparaat. Niet aangetroffen voor deze productrelease. |
| `HUMAN_TESTED` | Afgebakende echte gebruikerstest. Niet aangetroffen. |
| `INDEPENDENTLY_REVIEWED` | Onafhankelijke menselijke/externe review. AI-review telt niet. Niet aangetroffen als volledige securityaudit. |
| `RELEASE_VALIDATED` | Finale geïntegreerde artifact en complete releaseprocedure gevalideerd. Niet aangetroffen. |

Een rapport behoudt de broncommit/configuratie van de run. Een oudere
`VM_TESTED`-run wordt niet aan `380111b` toegeschreven zonder herhaling.
GitHub-CI op exact HEAD is bewijs van die workflow, niet automatisch bewijs
voor dagelijks gebruik, fysiek gedrag of alle releasecriteria.

## Geselecteerde bronrecords

| ID / claim | Bewijsbron | Wat dit ondersteunt | Wat dit niet ondersteunt |
|---|---|---|---|
| `BOOT-UEFI` | `boot/loader/src/main.rs`, `kernel/src/main.rs`; `docs/verification/m2-m4/ACCEPTANCE.md` | Loader/kernelimplementatie en eerder QEMU-profiel | Fysieke firmware, complete release-image of brede hardware |
| `K-MEMORY` | `docs/verification/m2-m4/ACCEPTANCE.md`; `docs/progress.json` K02 | Paging/frame/W^X-eigenschappen binnen getest profiel | SMP, formeel bewijs of universele isolatiegarantie |
| `K-CAPABILITY` | `docs/verification/m2-m4/ACCEPTANCE.md`; `docs/verification/regression-m6b/FINAL-ACCEPTANCE.md` | Native IPC/capabilitytests volgens testcontract | Alle huidige productdiensten zijn daarop aangesloten |
| `K-PROCESS` | `docs/verification/K05-ACCEPTANCE.md`; `docs/progress.json` K03/K05/K07 | Process-/resource-eigenschappen in genoemde gates | Algemene desktopappcompatibiliteit en SMP |
| `STORAGE-VOLUME` | `docs/verification/M6E-ACCEPTANCE.md`, `M6C-ACCEPTANCE.md`, `S08-ACCEPTANCE.md` | Virtio-opslag, catalogus, crash- en koude herstelproeven in hun profielen | Geëncrypteerd algemeen FS, backup of fysieke schijf |
| `STORAGE-CRYPTO` | `docs/implementation/ENCRYPTED-OBJECT-STORAGE.md`; B02 in `docs/progress.json` | Ontwerp-/acceptatiecontract | Native encrypted persistentie; B02 staat open |
| `DESKTOP-COMPOSITOR` | `docs/verification/mature-session-20261102/U01-ACCEPTANCE.md`; run 41535553113 | Eerder native VM-bewijs; exacte-head workflow is later succesvol afgerond | Geen volledige daily-use releasepoort of fysieke displayclaim |
| `MATERIAL-EDITOR` | `docs/verification/full-release-gui-9596abc/`; `docs/FULL-TEST-RELEASE.md` | Native frames op specifieke oudere commits; de vastgelegde GUI-route haalde Saved/Reopen niet | Volledige editoracceptatie op audit-HEAD |
| `BROWSER` | `docs/verification/browser-report.json`; `ui/prototype/README.md` | Host/browserprototype-interactie | Native browserengine, netwerk of download |
| `ACCESSIBILITY` | `docs/verification/browser-report.json`; `ui/native/README.md` | Prototype-/semantiekvoorbereiding | Native screenreader, menselijke test of 200%-acceptatie |
| `NETWORK-STACK` | `docs/verification/mature-session-20261102/n01-integrated.json`; `docs/progress.json` N01 | Afgebakende native packetstackgate op de vastgelegde bron | Algemeen internet, DNS, actuele TCP-review op nieuwe bron, netwerkrechten |
| `TLS-PEER` | `services/nas/TRANSPORT.md`; `docs/verification/pqc-transport/` | Host/loopback transportonderzoek | Native TLS/peer-authenticatie en productie-/release trust |
| `PRIVACY-SECURITY` | `docs/LOCAL-PRIVACY-CONTRACT.md`; `docs/progress.json` B01–B06 | Bindende privacy/security-eisen en scoped B01-gate | Volledige native LP-suite, B02 en externe review |
| `UPDATES-RECOVERY` | `docs/verification/B03-ACCEPTANCE.md`, `b03/update-gate.json` | Native update-signature-/rollbackgate in vastgesteld profiel | Complete installer, production chain, recovery image en finale release |
| `SDK-PACKAGES` | `docs/verification/A01-ACCEPTANCE.md`; `docs/progress.json` A01/A02/A03 | A01's bounded native SDK app/catalog path is accepted for 2 points | Package installation/update/removal, complete permission enforcement and A02/A03 |
| `QUANTUM-LOCAL` | `docs/verification/L02-ACCEPTANCE.md`; source `433bab9feab74473517eb8d44f2fb4475848e0cf`, integrated at `7fb8c722fc853692251e26fcdcea0ed86954ef15` | Historical native QEMU acceptance for the bounded L02 QSIM job service; this is source-bound VM evidence | Fresh integrated FR-L04 regression, physical QPU, quantum speedup or cloud execution |
| `RELEASE-VALIDATION` | `docs/FULL-TEST-RELEASE.md`; exact-head CI-run 41535553113 | CI workflows succesvol op audit-HEAD | Gevalideerde releasebundle, alle FR-rijen, hardware- of human gates |

## Exacte-head workflowwaarneming

Op `380111b` zijn alle 15 opgevraagde workflows voltooid met `success`,
waaronder native window session/Windows QEMU. `FR-I01` heeft een eigen
geslaagde workflow op deze HEAD, maar FR-I01 is een deelgate en sluit geen
complete release-rij. De CI-status is geen releasevalidatie.

Niet uitgevoerd door de audit: producttests/builds, QEMU-runs, hardwaretests,
gebruikersproeven, menselijke securityreview en releasevalidatie. De audit
heeft bestaande rapporten gelezen en bronstand/CI via read-only queries
gecontroleerd.
