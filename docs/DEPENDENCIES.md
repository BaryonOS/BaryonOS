# Afhankelijkheden en herkomst
Revisie 5, 26 oktober 2026. Versies zijn nu geselecteerd; de gecombineerde toolchain is nog niet gebouwd en packages zijn hier niet geïnstalleerd. Details en primaire bronnen staan in de implementatiespecificaties. Geen fictieve lockfiles of firmwarehashes toevoegen.

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
