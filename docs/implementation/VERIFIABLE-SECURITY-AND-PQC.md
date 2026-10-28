# Verifieerbare beveiliging en postkwantumintegratie

**Besluitdatum:** 27 oktober 2026. **Basis:** `ece94a5cfebecaec763eb2855aa9f84e55dcdcbc` (N01, PR #7). Dit document verwerkt de aangeleverde microkernel-/CHERI-/PQC-blauwdruk met controle van de echte repository en primaire bronnen. Uitvoering en teststatus staan in PROJECT-STATE.md; een ontwerpeis is geen geslaagd bewijs.

## 1. Productdoel en bewijsgrens

Nieuw OS houdt een eigen kleine Rust-kern, expliciete bevoegdheden en geïsoleerde diensten. Formele verificatie wordt een eigen ontwikkelspoor. seL4 is een voorbeeld van afgebakende bewijzen, geen overgenomen kernel of bewijs voor onze code. CHERI is een afzonderlijke toekomstige platformproef; het huidige x86_64/QEMU-profiel heeft geen CHERI-bescherming.

De ambitie blijft een moderne verbonden consumentenervaring. Gebruikers koppelen een apparaat en kiezen een bevoegdheid eenmaal; normale herhaling binnen die grenzen verloopt automatisch. Een nieuwe identiteit of bevoegdheid vraagt opnieuw toestemming. Formele rechtencontrole vereist geen vraag bij iedere synchronisatie of netwerkovergang. Privacycontract revisie 3 blijft leidend: eigen NAS én ontvangen ruimte bij vrienden/familie, ook via expliciet toegestaan internet; inhoudssleutels blijven bij de gebruiker en de beheerder kan zijn NAS-kopie zonder gasttoestemming verwijderen.

Een formeel bewijs heeft de vorm: onder aannames A voldoet implementatie I aan specificatie S voor eigenschap P. Leg A, I, S en P vast. Een fout in de bedoelde specificatie, niet-geverifieerde systeemdienst, updateketen, protocolintegratie of rechtenconfiguratie is óók een softwareprobleem. Het resterende risico beperkt zich dus niet tot fysieke chips en menselijke misleiding. Tests, modelchecking, broncodeverificatie en binaire refinement zijn verschillende bewijsniveaus.

## 2. Correcties op de aangeleverde blauwdruk

| Onderwerp | Leidende formulering |
|---|---|
| seL4 in 2026 | AArch64-securitybewijzen zijn in augustus gepubliceerd. De septemberreductie van 250.000 regels/57% betreft de abstract-to-design refinementlaag, niet alle bewijzen. De configuratiematrix onderscheidt C-level functional correctness, toegang/informatieflow en binary correctness per architectuur. Geen universele machinecodegarantie overnemen. |
| Kernelomvang | Een kleine kernel maakt review en bewijs beter beheersbaar. Minder regels betekenen niet een bewezen procentuele vermindering van aanvallen. De TCB wordt per garantie bepaald; voor NAS-privacy tellen onder meer client, sleutels, updater en toestemming mee. |
| Bestaande OS'en | XNU noemt zichzelf hybride; Windows kent ook user-mode-drivers. Een exploiteerbare kernel-driverfout kan breed doorwerken, maar iedere overflow is niet automatisch een volledige overname. |
| Driverisolatie | CPU-pagetables beperken geen willekeurige DMA door een apparaat. Apparaten, mappings, IOMMU en herstel moeten apart onder controle zijn. M5/M6 houden virtio-transport/MMIO/DMA nog deels in de kernel. |
| Capabilities | Onvervalsbare verwijzingen en beperkte delegatie zijn noodzakelijk. Een verkeerd geconfigureerde broker kan nog steeds eigen rechten verkeerd inzetten. Callerbinding, rights attenuation, generatie/intrekking en initialisatie worden afzonderlijke invarianten. |
| Rust/CHERI | Safe Rust steunt op correcte unsafe-/compilergrenzen. CHERI gebruikt beschermde hardwaretags/bounds; volledige bescherming van levensduur en subobjecten vraagt passende compiler, ABI, allocator en revocation. Niet iedere use-after-free verdwijnt door de ISA alleen. |
| Crypto | NIST-algoritmen berusten op onderzochte hardheidsaannames; functionele verificatie bewijst niet dat geen toekomstige cryptanalyse bestaat. Een library met bewezen onderdelen bewijst niet onze integratie of alle backends/machinecode. |
| XTS en opslag | AES-XTS geeft vertrouwelijkheid, geen authenticatie. Een Merkleboom vereist een vertrouwd verankerde root, lengte-/positie-/versiebinding en rollbackbeleid. Een losse checksum is geen integriteitsgarantie tegen een aanvaller. |
| Quantumclaims | Grover/preimage en collision-aanvallen zijn verschillende problemen. Een 256-bit hash heeft niet simpelweg 128-bit quantumcollisionresistentie. Een kleine Grover-simulatie bewijst geen veiligheid van PQC. |
| Entropy en opruimen | Hashen van voorspelbare klokwaarden schept geen entropy. Zeroize is nuttige geheugenhygiëne, geen garantie dat alle kopieën/registers of crashgeheugen gewist zijn; abort voert Drop niet betrouwbaar uit. |

De meegeleverde afbeelding kan een aanvalsscenario illustreren. Labels als "volledige overname" zijn scenario-uitkomsten onder genoemde aannames; een groene kleur is geen werkelijk uitgevoerd isolatiebewijs. Voor Nieuw OS mag een scherm pas "geverifieerd" zeggen wanneer versie, eigenschap en bewijs beschikbaar zijn.

## 3. Protocolkeuze: geen eigen hybride tunnel

De aangeleverde KDF verwerkt `X || M || optional_psk? || context` zonder framing tussen de optionele PSK en context. `Some(P), context=C` en `None, context=P||C` leveren identieke input op. Dit is een ambigu formaat, geen gevonden BLAKE3-botsing. Het bewijst op zichzelf geen exploiteerbare aanval op een nog niet bestaand systeem, maar het fragment is geen aanvaardbaar sessieprotocol. Transcript-/rolbinding, authenticatie, key confirmation, replay en downgradebeleid moeten samen kloppen.

Kies voor de eerste NAS-transportproef de onderhouden TLS-implementatie van de ontwikkelhost met **TLS 1.3 en uitsluitend X25519MLKEM768**, zoals vastgelegd in **RFC 10024 (augustus 2026)**. De standaard combineert de geheimen binnen het TLS-transcript en de bestaande key schedule. Geen eigen BLAKE3-KDF, extra PSK-combiner of zelfgeschreven recordencryptie toevoegen. Dit is een proefprofiel; brede app-/webcompatibiliteit en productmigraties krijgen later afzonderlijk beleid.

### N02 — uitvoerbaar hostprofiel

- Server en client gebruiken alleen `127.0.0.1`; geen echte NAS, internetpoort, routerconfiguratie of DNS-discovery.
- TLS 1.3, exact `X25519MLKEM768`, exact `TLS_AES_256_GCM_SHA384`, vast ALPN; geen klassieke fallback, sessiehergebruik of 0-RTT in deze proef.
- Tijdelijke test-CA en wederzijdse certificaatcontrole; verwachte server- en clientcertificaatpins. Serverhostnamecontrole blijft vereist.
- Toestemming controleren vóór de socketpoging. Na echte authenticatie een opaque transportfact maken; bij iedere applicatiewrite opnieuw rechten controleren. Verbindingseinde maakt het feit ongeldig.
- Alleen een begrensde echo van N01-ciphertext met namespacecontrole. De NAS-endpoint krijgt geen documentdecryptiesleutel. Dit is geen opslag-/sync-API en geen vervanger voor N01-revoke/evict.
- Handshake- en applicatielimieten; aantallen, bytes en tijd begrenzen. Fouten sluiten de verbinding zonder naar een zwakker profiel te gaan.
- Bewijs vermeldt de werkelijk onderhandelde groep/protocol/cipher en de gebruikte runtime. Een ondersteunde configuratiestring alleen telt niet als geslaagde PQC-handshake.

**Expliciete grens:** de testcertificaten gebruiken klassieke Ed25519-authenticatie. Hybride sleuteluitwisseling is dus geen postkwantumbeveiliging van de hele authenticatie-/boot-/updateketen. Een toekomstige quantumaanvaller tegen online authenticatie vraagt aanvullende oplossingen. De labels LAN/internet in de policytests zijn harnasaannames bij werkelijk loopbacktransport; geen waargenomen WAN-route of native socketafdwinging. Node/OpenSSL zijn ontwikkelmiddelen, geen besloten eigen-OS-runtime.

## 4. Werkelijke native integratiegrenzen

| Grens op basiscommit | Consequentie voor de volgende native taak |
|---|---|
| Rust 1.90.0; `x86_64-unknown-uefi` en `x86_64-unknown-none` | Geen custom-target-JSON of libc. Verifieer iedere dependency tegen precies dit target en de gebruikte compiler. |
| `tools/build_users.py` bouwt probes met directe rustc en `#[path]`-modules | Een nieuwe Cargo-crate wordt niet automatisch aan ring 3 gekoppeld. Eerst een expliciete, reproduceerbare servicebuildroute maken. |
| Geen x87/SSE/AVX in het huidige profiel | Portable backend expliciet selecteren en de gegenereerde code controleren; AVX2 niet stil aanzetten. |
| 64 KiB userstack met guards; vaste userheap zonder algemene allocator | Stackpieken en heap-/allocvereisten meten, ook foutpaden. 128 KiB als willekeurig nieuw minimum is geen bewijs. |
| M6-text maximaal 64 KiB; rodata maximaal 4 KiB | Crypto-code en sleutels passen niet bij voorbaat. ELF-layout en regressies gezamenlijk aanpassen als dat nodig blijkt. |
| Message320: maximaal 256 payloadbytes; blokprotocol 192 bytes | Grote keys/ciphertexts/signatures vragen begrensde overdracht, expliciet eigendom en levenstijd; geen willekeurige userpointer doorgeven. |
| Geen Entropy-capability/GetRandom | Geen native sleutels genereren tot bronbeleid, gereedheid, falen en syscall-/servicegrens geïmplementeerd en getest zijn. |
| `services/block/` gebruikt FNV in synthetische M5/M6-proeven | Dit is diagnostiek, geen cryptografische integriteit. Schrijfherstel en de open M6a-gate niet als geverifieerde encrypted storage presenteren. |

`libcrux-ml-kem` en `libcrux-ml-dsa` **0.0.11** zijn onderzochte kandidaten, geen toegevoegde dependencies. De geraadpleegde tag noemt Rust 1.89 als minimum en Apache-2.0; de no_std-featurekeuze en Rust-buildscripts moeten inclusief transitieve boom worden gecontroleerd. `default-features=false` is geen zelfstandig bewijs dat AVX2 ontbreekt. Leg backendkeuze, exacte source-/lockhashes, buildscriptuitvoer en beschikbare bewijzen vast. De ML-KEM-verificatiestatus is gedeeltelijk en door de maintainers als evoluerend gekwalificeerd.

De parameterkeuze ML-KEM-768/ML-DSA-65 blijft kandidaat voor integratie. FIPS 203 gebruikt implicit rejection: een gemanipuleerde ciphertext hoeft geen publiek decapsulation-Err te geven; de daaropvolgende authenticatie/key confirmation moet falen. Test dus het echte protocolgedrag. Handtekeningen bewijzen herkomst onder een sleutel, geen goede bedoelingen of recht op document-/netwerkcapabilities.

## 5. Afgebakende vervolgopdrachten

| ID | Exclusief resultaat | Acceptatie vóór claim |
|---|---|---|
| V01 — capabilitybewijs | Formeel model van rechtenvermindering, overdracht, objectgeneratie en intrekking | Machinegecontroleerde invarianten, modelaannames en relatie met de echte Rust-functies. Modelcheck zonder refinement niet als volledig codebewijs noemen. |
| P01 — native cryptobuild | Geïsoleerde `crates/pqc-core`-adapter met gecontroleerde standaardlibrary; geen dependencies in `kernel/` | Gepinde dependencyboom/licenties; hostvectors; echte no_std/ring-3-build; portable instructies; gemeten stack/ELF-budgets. Nog geen eigen netwerkprotocol. |
| P02 — entropy | Gespecificeerde entropycapability en bronbeleid met gereedheid/foutpad | Geen sleutelgeneratie vóór ready; lege/mislukkende bron blijft geblokkeerd; testseed nooit productpad. DMA/virtio-rng en VM-snapshots apart beoordelen. |
| P03 — systeemimage | Ondertekende manifesten met algoritmeversie, doel, lengte, digest, versie en rollbackbeleid | Beide vereiste signatures over exact dezelfde canonieke boodschap, pinned trust anchors, foutieve/missende/verwisselde/oude metadata geweigerd in echte guest-run; firmwaregrens benoemd. |
| P04 — opslag | Geauthenticeerd opslagformaat met crash-/nonce-/rollbackontwerp boven de blokdienst | Eerst volledige M6a-poort/regressie; daarna bitflip, block-swap, replay, afgebroken write, herstel en keyrotatie op wegwerpimages. AES-XTS alleen voldoet niet. |
| N03 — native NAS | Eigen netwerkdienst, transportlibrary, veilige pairing/key lifecycle en opslagadapter | Peergebonden rechten vóór verbinding en per operatie; E2EE tot de gebruikersclient; adminrevoke/evict blokkeert oude uploads; echte OS-/netwerkproeven. |
| Q02 — educatief Grover | Afzonderlijke zesqubit-orakel-/diffusieproef in qsim | Alle 64 doelen, normbehoud, succeskans na zes iteraties, afname bij teveel iteraties en annulering. Geen claim van fysieke quantumversnelling of PQC-bewijs. |

Qsim ondersteunt nu H/X/Z/S/CX en maximaal 12 qubits. Een algemene zesbit-Groverproef vraagt dus een daadwerkelijke simulatoruitbreiding, niet alleen een test van de bestaande gates. De 28–34-qubitinschatting uit de aangeleverde tekst is geen eigenschap of geteste grens van deze repository.

Bij P03 blijven pakketauthenticiteit en appautorisatie gescheiden: een geldig pakket krijgt uitsluitend de door het OS/de gebruiker verleende rechten. Appwinkelkeuze en privacy mogen niet afhankelijk worden van één leverancier. Bij N03 blijven automatisch hervatten, bruikbaar sleutelherstel en multi-devicewerk productvereisten. Een voorlopige testlimiet is geen permanente beperking voor consumenten.

## 6. Primaire bronnen

Geraadpleegd 27 oktober 2026; projectkeuzes en integratiegevolgen zijn onze analyse, geen certificering door de genoemde organisaties.

- Proofcraft. (2026, 21 augustus en 21 september). *News 2026*. https://proofcraft.systems/news-2026/
- seL4 Project. (z.d.). *Verified configurations*; *Frequently asked questions*. https://docs.sel4.systems/projects/sel4/verified-configurations.html ; https://sel4.systems/About/FAQ.html
- University of Cambridge. (z.d.). *CHERI frequently asked questions*. https://www.cl.cam.ac.uk/research/security/ctsrd/cheri/cheri-faq.html
- Wang, Y., et al. (2026). *CHERI-D: Secure and efficient inline object ID for CHERI temporal memory safety*. https://arxiv.org/abs/2606.19055
- Apple. (z.d.). *XNU README*. https://github.com/apple-oss-distributions/xnu
- Microsoft. (z.d.). *User mode and kernel mode*. https://learn.microsoft.com/en-us/windows-hardware/drivers/gettingstarted/user-mode-and-kernel-mode
- Kwiatkowski, K., Kampanakis, P., Westerbaan, B. E., & Stebila, D. (2026). *Post-Quantum Traditional (PQ/T) Hybrid Key Agreement Mechanisms for TLS 1.3* (RFC 10024). https://www.rfc-editor.org/rfc/rfc10024.html
- NIST. (2024). *FIPS 203; FIPS 204; FIPS 205*. https://csrc.nist.gov/pubs/fips/203/final ; https://csrc.nist.gov/pubs/fips/204/final ; https://csrc.nist.gov/pubs/fips/205/final . Controleer de gepubliceerde errata bij elke implementatiepin.
- NIST. (2010). *SP 800-38E: XTS-AES*. https://csrc.nist.gov/pubs/sp/800/38/e/final
- NIST. (2018). *SP 800-90B: Entropy sources*. https://csrc.nist.gov/pubs/sp/800/90/b/final
- Brassard, G., Høyer, P., & Tapp, A. (1997). *Quantum algorithm for the collision problem*. https://arxiv.org/abs/quant-ph/9705002
- Libcrux contributors. (2026). *libcrux-ml-kem 0.0.11; verification status*. https://docs.rs/libcrux-ml-kem/0.0.11/libcrux_ml_kem/ ; https://github.com/celabshq/libcrux/blob/libcrux-ml-kem-v0.0.11/libcrux-ml-kem/proofs/verification_status.md
- RustCrypto. (2026). *zeroize*. https://docs.rs/zeroize/latest/zeroize/
- OpenSSL Project. (2026). *SSL_CTX_set1_curves, OpenSSL 3.5*. https://docs.openssl.org/3.5/man3/SSL_CTX_set1_curves/
- OpenJS Foundation. (2026). *TLS API* (actuele documentatie 26.12.1; werkelijk geteste runtime wordt apart vastgelegd). https://nodejs.org/api/tls.html
