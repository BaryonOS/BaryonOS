# Afhankelijkheden en herkomst

M6b voegt uitsluitend de eigen lokale crate `object-core 0.1.0` toe: `no_std`,
geen allocator of externe dependencies, `forbid(unsafe_code)` in de core.
De native adapter gebruikt de bestaande vaste syscallbuffers en gemotiveerde
unsafe-grens. Root-Cargo.lock behoudt `r-efi 7.1.0` met dezelfde checksum.

De SDK-runtime gebruikt nu rechtstreeks `zeroize 1.9.1` (exact gepind,
`default-features = false`) voor het wissen van IPC-envelopes. Dit is dezelfde
reeds gebruikte RustCrypto-dependency uit `services/keyring`; de vastgepinde
crate is `no_std`, heeft voor deze configuratie geen transitieve dependency of
buildscript en is Apache-2.0 OR MIT. Herkomst: crates.io-locksum en
<https://github.com/RustCrypto/utils/tree/master/zeroize>. Dit wist alleen
expliciet beheerde buffers. De native sessie-UI gebruikt sinds2november2026
dezelfde exacte pin rechtstreeks voor owned wachtwoord-/ticketbuffers; geen
nieuwe externe crate, feature of buildscript. Het bewijst geen opruiming van registers, compiler-
kopieën, crashgeheugen of kopieën buiten de SDK-eigen envelopes.

### Netwerkdiensten: smoltcp, rustls en spin (30 oktober 2026)

Deze externe crates zitten uitsluitend in de zelfstandige workspaces van
ring-3-netwerkdiensten, elk met een eigen `Cargo.lock`; niet in kernel, loader
of root-workspace. Herkomst- en buildscriptcontrole staan bij de component.

| Crate (exacte pin) | Gebruikt door | Herkomst / licentie | no_std / transitief | Detailbron |
|---|---|---|---|---|
| `smoltcp =0.14.0` | `services/net-stack`, `services/net-broker` | smoltcp-rs/smoltcp; 0BSD | `default-features = false`, alleen Ethernet/IPv4/TCP; transitief `managed`, `heapless`, `hash32`, `byteorder`, `bitflags`, `cfg-if`, `stable_deref_trait` volgens lockfile | ADR-046 in `DECISIONS.md`, `services/net-stack/THIRD-PARTY.md` |
| `rustls =0.23.46` | `services/net-tls` | rustls/rustls; Apache-2.0 OR ISC OR MIT | Alleen `custom-provider`, geen `std`/ring/AWS-LC; transitief `rustls-webpki`, `rustls-pki-types`, `once_cell`, `subtle`, `untrusted`, `zeroize` | `services/net-tls/README.md` (dependencycontrole) |
| `spin =0.12.3` | `services/net-tls` | zesterer/spin; MIT | Alleen `mutex,spin_mutex`; geen dependencies of buildscript | `services/net-tls/README.md` |

Ongekozen optionele of platformpackages (zoals `ring`, `windows-*`) kunnen in
`services/net-tls/Cargo.lock` staan; alleen de featureboom en compileruitvoer
bewijzen wat wordt gebouwd. **Updatebeleid:** versies blijven exact gepind en
`--locked`; een update vraagt een eigen PR met lockfilediff, herhaalde
licentie-/buildscriptcontrole en de componentgates. **Verwijderplan:** elke
crate is beperkt tot één dienstworkspace; verwijderen betekent die dienst of
adapter vervangen, zonder kernel-, ABI- of root-lockwijziging.

De nieuwe Linux-ontwikkelvariant gebruikt Rust 1.90.0, QEMU 10.0.2
(`1:10.0.2+ds-2+deb13u1~bpo12+1`) en matching OVMF 2025.02-8+deb13u1.
Rust kwam van static.rust-lang.org; Debian-pakketten zijn via ondertekende
InRelease, Packages-SHA256 en pakket-SHA256 gecontroleerd en uitsluitend
lokaal uitgepakt. De metadata, URLs, binary-/firmwarehashes en hostlibraries
staan in `verification/native-linux-toolchain/`. Dit zijn hostgereedschappen,
geen nieuwe runtimebibliotheken van de eigen kernel of objectdienst.


N02 (27 oktober 2026) gebruikt aanvullend uitsluitend ingebouwde `node:tls`, `node:net` en de bestaande `node:crypto`-API op Node 24.22.1/OpenSSL 3.5.9. Geen npm- of Cargo-dependency toegevoegd. De aparte system-OpenSSL-CLI **3.0.13** genereert tijdelijke Ed25519-testcertificaten; die CLI voert niet de hybride TLS-handshake uit. Deze gebeurt in Node's OpenSSL **3.5.9**. Runtimewaarden, bronhashes en testuitvoer staan onder `verification/pqc-transport/`. Geen certificaat/private key of productie-CA in Git. TLS-groep X25519MLKEM768 is gestandaardiseerd in RFC 10024 (augustus 2026); native protocol/libraryport blijft een afzonderlijke dependencyselectie.

Onderzocht, **niet geïnstalleerd/toegevoegd**: `libcrux-ml-kem` en `libcrux-ml-dsa` 0.0.11, officiële repository `celabshq/libcrux`, Apache-2.0 en manifest-Rustminimum 1.89.0. De exacte no_std-features, Rust-buildscripts/SIMD-selectie, transitieve licenties en werkelijke targetbuild moeten nog worden vastgelegd vóór opname. De projectversie Rust 1.90.0 alleen is geen compatibiliteitsbewijs. De gedeeltelijke formele verificatiedekking staat in de eigen librarydocumentatie; geen blanket constant-time-/machinecodeclaim. Bronnen en native budgetten: `implementation/VERIFIABLE-SECURITY-AND-PQC.md`.

N01 (27 oktober 2026) gebruikt de aanwezige Node.js 24.22.1-hostruntime met OpenSSL 3.5.9 en uitsluitend ingebouwde modules (`node:crypto`, filesystem en test-/process-API's). Geen npm-package, Cargo-dependency, externe crypto-implementatie of buildscript toegevoegd. Node/OpenSSL zijn ontwikkel-/proefmiddelen, geen gekozen native Nieuw OS-runtime. AES-GCM, randomness en authenticated-decryption-API's zijn gecontroleerd tegen de primaire Node v24-documentatie; concrete versies/testbewijs staan in verification/nas/report.json. Native crypto-/TLS-bibliotheekselectie, no_std-geschiktheid, licentie-/supplychaincheck en key lifecycle blijven afzonderlijke gates vóór die integratie.

Actuele uitvoeringsstatus 27 oktober 2026: Rust 1.90.0, r-efi 7.1.0, serde 1.0.229 en serde_json 1.0.154 zijn daadwerkelijk opgehaald en gebouwd. De root- en quantumlockfiles zijn gegenereerd. De gebruikte emulator is de toegestane Windows-ontwikkelvariant QEMU 11.1.0 met bijpassend packaged CODE/VARS-paar; exacte herkomst/hashes en bewijsgrenzen staan in DEVELOPMENT.md, third-party/README.md en ../experiments/quantum/THIRD-PARTY.md. De onderstaande oorspronkelijke selectietabel is historische context; PROJECT-STATE.md beschrijft de actuele verificatie.

M2–M4 voegen uitsluitend eigen lokale crates toe: kernel-core 0.1.0 (frames/ELF/copybeleid) en capability-core 0.1.0 (vaste rechten-/IPC-transacties). Beide zijn dependencyvrij/no_std en verbieden unsafe in de library. De kernelarch-/kopieergrens bevat de gemotiveerde unsafe/assembly; de eigen crates hebben geen extern buildscript. Root-Cargo.lock behoudt r-efi exact 7.1.0 met dezelfde checksum. Er is voor M2–M4 geen nieuwe externe runtimebibliotheek toegevoegd. Corebron/host-/targetchecks en actuele testcasegrenzen staan in verification/m2-m4/.
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
