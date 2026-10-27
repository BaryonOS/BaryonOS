# Afhankelijkheden en herkomst

N01 (27 oktober 2026) gebruikt de aanwezige Node.js 24.22.1-hostruntime met OpenSSL 3.5.9 en uitsluitend ingebouwde modules (`node:crypto`, filesystem en test-/process-API's). Geen npm-package, Cargo-dependency, externe crypto-implementatie of buildscript toegevoegd. Node/OpenSSL zijn ontwikkel-/proefmiddelen, geen gekozen native Nieuw OS-runtime. AES-GCM, randomness en authenticated-decryption-API's zijn gecontroleerd tegen de primaire Node v24-documentatie; concrete versies/testbewijs staan in verification/nas/report.json. Native crypto-/TLS-bibliotheekselectie, no_std-geschiktheid, licentie-/supplychaincheck en key lifecycle blijven afzonderlijke gates vóór die integratie.

Actuele uitvoeringsstatus27 oktober2026: Rust1.90.0, r-efi7.1.0, serde1.0.229 en serde_json1.0.154 zijn daadwerkelijk opgehaald en gebouwd. De root- en quantumlockfiles zijn gegenereerd. De gebruikte emulator is de toegestane Windows-ontwikkelvariant QEMU11.1.0 met bijpassend packaged CODE/VARS-paar; exacte herkomst/hashes en bewijsgrenzen staan in DEVELOPMENT.md, third-party/README.md en ../experiments/quantum/THIRD-PARTY.md. De onderstaande oorspronkelijke selectietabel is historische context; PROJECT-STATE.md beschrijft de actuele verificatie.

M2–M4 voegen uitsluitend eigen lokale crates toe: kernel-core0.1.0 (frames/ELF/copybeleid) en capability-core0.1.0 (vaste rechten-/IPC-transacties). Beide zijn dependencyvrij/no_std en verbieden unsafe in de library. De kernelarch-/kopieergrens bevat de gemotiveerde unsafe/assembly; de eigen crates hebben geen extern buildscript. Root-Cargo.lock behoudt r-efi exact7.1.0 met dezelfde checksum. Er is voor M2–M4 geen nieuwe externe runtimebibliotheek toegevoegd. Corebron/host-/targetchecks en actuele testcasegrenzen staan in verification/m2-m4/.
M5 voegt uitsluitend eigen lokale crates `virtio-core 0.1.0`, `virtio-blk-driver 0.1.0` en `block-protocol 0.1.0` toe. De twee cores zijn dependencyvrij/no_std en verbieden unsafe; de driver hangt alleen van virtio-core af. De driver bevat de gemotiveerde PCI/MMIO/DMA-unsafe, geen extern buildscript. Er is geen externe package toegevoegd of gewijzigd in Cargo.lock. Protocolbasis: OASIS VIRTIO 1.2; de code is projectspecifiek en bevat geen gekopieerde bestaande OS-kernel. Licentiekeuze voor het eigen project blijft volgens LICENSE-PLAN.md open vóór publieke publicatie. M5-target-/reviewbewijs staat in verification/m5/; de volledige gate is geslaagd.

Historische selectie, revisie 5 van 26 oktober 2026: de onderstaande tabel beschrijft de oorspronkelijke voorbereiding. Actuele builds en pins staan hierboven en in PROJECT-STATE.md. Geen fictieve lockfiles of firmwarehashes toevoegen.

| Selectie | Rol | Status |
|---|---|---|
| Rust 1.90.0 met gebundelde LLVM-linker | Hostcompiler; UEFI- en no_std-kerneltarget | Vastgelegde baseline, geen claim van nieuwste versie; pin rust-toolchain.toml tijdens B01 |
| r-efi =7.1.0, default-features=false | Kleine no_std UEFI-bindings, geen loader/kernel | Officieel manifest gecontroleerd; fetch, compatibiliteit en transitieve boom nog toetsen |
| QEMU 10.0.14, pc-q35-10.0, qemu64, TCG | Referentie-emulator op host | Versie onderzocht; praktische compatibele pakketroute volgens bootspecificatie toegestaan en gelogd |
| OVMF uit edk2-stable202508, X64 RELEASE/GCC5 | Referentie-UEFI-firmware | CODE/VARS moeten één overeenkomende build vormen; hashes van werkelijke bestanden vastleggen |
| Python 3 | Inventaris/build-/testharnas | tools/preflight.py aanwezig en uitgevoerd; geen externe Python-packages nodig |
| HTML/CSS/JS | Lokaal P2-productprototype | Geen framework, CDN, externe assets of runtime-netwerk |
| qsim-core | Eigen veilige simulatorcore | Ontwerp: no_std+alloc, nul externe dependencies |
| serde =1.0.229; serde_json =1.0.154 | Alleen JSON-adapter van hostquantumproef | Documentatiepins; registry-resolutie nog uit te voeren. Concreet herstelbeleid in QUANTUM-SPEC.md |
| Crypto-library | Latere ML-KEM-768/ML-DSA-65-integratie | Parameters gekozen; library en concrete protocolbinding pas selecteren bij de integratietaak |

Per daadwerkelijk opgenomen dependency vastleggen: naam, versie/commit/hash, primaire bron, licentie, directe/transitieve dependencies, build scripts, onderhoud/securitymeldingen, host/guest/firmware-rol, no_std-ondersteuning, reden, updatebeleid en verwijderplan. Verifieer de werkelijke resolveruitvoer en leg Cargo.lock vast; een documentatiepin is geen succesvolle build.

Geen bestaande kernelcode overnemen zonder expliciete scopewijziging. Kleine overgenomen algoritmen/voorbeelden blijven externe broncode: behoud herkomst/licentie. Een AI-herschrijving wist die verplichtingen niet.

Toolchain, firmware en buildomgeving horen bij de vertrouwensketen. Reproduceerbare builds tonen overeenstemming tussen builds; ze bewijzen niet dat broncode, compiler of hardware vrij van achterdeuren is.

