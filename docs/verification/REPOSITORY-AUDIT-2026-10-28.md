# Repository-audit — 28 oktober 2026

## Bron en integratie

- Geïntegreerde auditcommit op `main`: `87e4f04b8d2e5527f873d3815aebfb047eb68273`.
- Actuele basis na de afzonderlijke M6c-timingstabilisatie PR #26: `51dc38ebe82ac3ace16a52a904ecfe59551210d1`.
- Integratie-PR: #25.
- Ingehaalde PR's: #16 (inhoud geïntegreerd via #25) en #22 (vervangen door #25).
- Roadmapscores vóór/na: 14% → 14%.

## Gecontroleerde gebieden

Volledige repositorystructuur, root en lokale AGENTS-instructies, architectuur,
kernel/userspace-/capability-/IPC-/drivergrenzen, Rust-workspaces en lockfiles,
Python/Node tooling, tests, documentatie, duplicatie, secrets/credentials,
gegenereerde bestanden, GitHub Actions, ruleset en open PR's.

Historische `docs/verification/`-bundels en de expliciet geïmporteerde
`.agents/skills/os-dev/`-bronnencorpus zijn als bewijs/kennis behandeld en
niet als gewone duplicatie of buildrommel verwijderd.

## Belangrijkste herstellingen

- PR #16 inhoudelijk geïntegreerd op actuele main zonder stale voortgangsteksten.
- CodeRabbit-bevindingen uit PR #16 gecontroleerd en geldige punten hersteld.
- DocumentSession krijgt een echte native normal/koude-recoveryroute met
  bron-/compiler-/artifactbinding.
- Nieuwe dependencyvrije `no_std` Ethernet/IPv4/UDP-parser toegevoegd zonder
  driver-, socket- of netwerkcapability.
- Bestaande Clippy-/rustfmtproblemen in kernel-core, virtio-blk en block-protocol
  opgelost zonder lintonderdrukking.
- `services/catalog` expliciet `publish = false`.
- CI, CONTRIBUTING, PR-/bugtemplate, Dependabotvoorstellen, binary-attributen en
  ontbrekende target-gitignorepaden toegevoegd.
- Actieve projectdocumentatie op aantoonbare samenplak-/formatteerfouten opgeschoond.

## Verificatie

GitHub Actions push-run:
<https://github.com/simonvanieperen/nieuw-os/actions/runs/40740109679>

Resultaat: `success`.

De job `host-quality` slaagt voor:
- Python syntax + DocumentSession-hosttests + OS-Dev-skillvalidator/tests;
- root Rust hosttests, Clippy met `-D warnings` en rustfmt;
- SDK tests/Clippy/rustfmt;
- netwerkparser tests/Clippy/rustfmt en `x86_64-unknown-none` release-crossbuild;
- quantum tests/Clippy/rustfmt en `verify.py`;
- Materiaal-browserprototypetests;
- NAS policy/store/transporttests;
- native M6c-releasebuild;
- `tools/progress.py --write/--check` plus schone Git-werkboom.

De job `qemu-smoke` slaagt voor:
- strict preflight;
- normale M6c-smoke;
- DocumentSession normal en koude recovery in twee afzonderlijke VM-invocaties.

## Duplicatie en rommel

Na integratie bleven in de actieve bronboom slechts twee exacte duplicate blobs
over: de minimale loader-/kernel-runtime `memory.rs` en twee lokale
`.gitignore`-bestanden. De eerste is bewust per afzonderlijke binary gehouden;
de tweede is onschadelijke lokale scopeconfiguratie. Geen build-/targetcache,
IDE-cache, diskimage, ELF/EFI-artifact, private-keybestand of vergelijkbaar
gegenereerd bestand is in de actieve bronboom aangetroffen.

## Aanvullende actuele main-wijziging

PR #26 verhoogt alleen de bestaande M6c-completiondeadline van 20 naar 64 om
een aangetoonde TCG/QEMU-timingflake te vermijden. De foutfixtures blijven
completion tot de echte deadline onderdrukken; timeout-/Unknown-semantiek en
rechten blijven ongewijzigd. De afsluitende CI op deze basis moet opnieuw
host-quality en QEMU-smoke doorstaan.

## Open bewijsgrenzen

- Geen nieuwe fysieke-hardwareacceptatie of volledige M6c/M6d-mijlpaalreplay op
  de auditcommit; de CI-QEMU-run is een integratiesmoke.
- DocumentSession sluit nog niet U03/A01 volledig: geen algemene appstarter,
  interactieve Materiaal-GUI, grote documenten of volledige publieke SDK/ABI.
- De parser sluit N01 niet: nog geen native netwerkdriver/-service, TCP/DNS/TLS
  of afdwingbare proces-/bestemmingsrechten.
- GitHub-securityinstellingen en verplichte statuschecks in de ruleset konden via
  de beschikbare integratie niet volledig worden uitgelezen/gewijzigd.
- De tweede brede CodeRabbit-review van #25 publiceerde niet omdat de PR al was
  gemergd; alleen de eerdere concrete PR16-reviewbevindingen zijn aantoonbaar
  verwerkt.
