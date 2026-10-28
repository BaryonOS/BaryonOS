# I08 — allocatorvrije native quantum-requestcodec

## Resultaat en scope

Nieuw zelfstandig pakket `experiments/quantum/job-protocol`, zonder externe
dependencies. Het implementeert het door de hoofdagent vastgelegde interne
256-byte `QJ01`-requestcontract voor de voorgestelde gebruikersdienst-v1.
Getypeerde constructie, canonieke encoding en strikte decoding zijn gereed.

De codec bezit een vaste gate-array en valideert alle limieten, ID's, gate-
indices, reserveringen en padding. Cancel/Status/ResultPage worden alleen
als requests gecodeerd; er is geen dienstactie of antwoordprotocol toegevoegd.
Caller-identiteit en authority staan bewust buiten de payload en moeten
later door het IPC-envelope en dienstbeleid worden gebonden.

Er veranderen geen bestaande quantum-, kernel-, syscall-, IPC-, capability-,
opslag- of gedeelde bouwbestanden. De opdracht bezit uitsluitend het nieuwe
pakket en deze bewijsmap. Het volledige payloadschema en API-voorbeeld staan
in de README van het pakket.

## Werkelijk gecontroleerd

- 13 hosttests voor alle operaties/gates, exacte golden bytes, 64 gates,
  de grenzen van alle velden, alle afgekorte lengtes en extra bytes.
- Gerichte negatieve controles op ieder gereserveerd veld en ongebruikte
  bytepositie, onbekende codes, niet-nul control bij een enkele-qubitgate,
  ongeldige targets/CX-control en verboden velden buiten Submit.
- Een canonicaliteitstest met 6.400 enkele-bytemutaties: wat wordt geaccepteerd
  encodeert exact terug, zonder stilzwijgend genegeerde bytes.
- Rustfmt en host/native Clippy. Alleen de vaste request-enum heeft een
  gedocumenteerde variantgrootte-uitzondering; het type wordt bewust niet
  gealloceerd of geboxed.
- Native release-library-build voor `x86_64-unknown-none` met Rust 1.90.0.
- Cargo dependencyboom bevat uitsluitend `qsim-job-protocol`. `nm` van de
  native library toont geen allocator-/allocdependency; core en de normale
  scalar memory-intrinsics blijven linkafhankelijkheden van de uiteindelijke
  gebruikersapp.

`checks.json` bindt exacte commando's, exitstatussen, logs, bronbestanden en
native libraryhash. Gegenereerde binaries staan niet in Git. Dit bewijs
hoort bij de nieuwe zelfstandige codec; het wordt niet toegevoegd aan de
eerder afgesloten native acceptatie van broncommit `3e41bbf9`.

## Grenzen en integratie

Er is **geen nieuwe VM-run** uitgevoerd. Een native library-build bewijst
geen OS-jobdispatch, autorisatie, quota-enforcement, werkende annulering of
resultaatuitwisseling. De codec verandert de bestaande bronbinding of
acceptatiebundels niet; de rootintegrator beheert die gedeelde bestanden.

De kleinste resterende afhankelijkheid is een door de hoofdagent afgestemde
userspace-dispatcher en replycontract, met actorbinding vanuit het bestaande
IPC-envelope. De nieuwe dependency moet vóór dat gebruik worden meegenomen
in de OS-bronfingerprint en de gerichte native acceptatie. Scorevoorstel:
**L02 blijft open, +0 punten**.
