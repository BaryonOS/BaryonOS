# Repository-audit — 28 oktober 2026

## Bron en integratie

- Definitieve auditstand op `main`: `51dc38ebe82ac3ace16a52a904ecfe59551210d1`.
- Integratie-PR: #25; gerichte QEMU-stabiliteitsfix: #26.
- PR #16 is inhoudelijk ingehaald en gesloten; PR #27 is als verouderde
  documentatie-closeout gesloten en vervangen.
- Roadmapscores vóór/na: 14% → 14%.

## Gecontroleerde gebieden

Repositorystructuur, root en lokale AGENTS-instructies, architectuur,
kernel/userspace-/capability-/IPC-/drivergrenzen, Rust-workspaces en lockfiles,
Python/Node tooling, tests, documentatie, duplicatie, secrets/credentials,
gegenereerde bestanden, GitHub Actions, ruleset en open pull requests.

Historische `docs/verification/`-bundels en expliciet geïmporteerde
kennis-/skillbronnen zijn als bewijs/kennis behandeld en niet als gewone
duplicatie of buildrommel verwijderd. Bewuste subsystemduplicatie voor
security/fault isolation is niet samengevoegd.

## Belangrijkste herstellingen

- Relevante PR #16-inhoud geïntegreerd op actuele main; stale/overlappende PR
  daarna gesloten.
- Vijf concrete CodeRabbit-bevindingen uit PR #16 gecontroleerd en geldige
  documentatie-/consistentiepunten hersteld.
- DocumentSession heeft een native normal/koude-recoveryroute met
  bron-/compiler-/artifactbinding.
- Dependencyvrije `no_std` Ethernet/IPv4/UDP-parser geïntegreerd zonder
  driver-, socket- of netwerkcapability.
- Clippy-/rustfmtproblemen in relevante Rust-componenten opgelost zonder
  lintonderdrukking.
- CI, CONTRIBUTING, PR-/bugtemplate, beperkte Dependabotvoorstellen,
  binary-attributen en ontbrekende target-gitignorepaden toegevoegd.
- Geen bekende credentialpatronen gevonden in gerichte repositorybrede
  zoekcontroles; dit is geen claim van een externe secretscanner.
- Een post-merge M6c-QEMU-flake is niet weggedrukt: de fout is gelogd,
  op exact dezelfde tree gereproduceerd als pass bij rerun, inhoudelijk
  getraceerd naar de bestaande timeout/Unknown-route en met PR #26 begrensd
  gecorrigeerd door de M6c-brokerdeadline van 20 naar 64 ticks te verhogen.
  Faultfixtures blijven tot de echte deadline onderdrukken; geen retry of
  kunstmatig succes is toegevoegd.

## Definitieve verificatie

GitHub Actions push-run:
<https://github.com/simonvanieperen/nieuw-os/actions/runs/40741321766>

Resultaat: `success`.

`host-quality` slaagt voor:
- Python syntax + DocumentSession-hosttests + skillvalidatie/tests;
- root Rust hosttests, Clippy met warnings-as-errors waar geconfigureerd en rustfmt;
- SDK tests/lint;
- netwerkparser tests/lint en `x86_64-unknown-none` release-crossbuild;
- quantumtests/verificatie;
- productprototype;
- NAS policy/store/transport;
- native releasebuild;
- `tools/progress.py --write/--check` plus clean-treecontrole.

`qemu-smoke` slaagt voor:
- strict preflight;
- baseline M6c-smoke op de definitieve main;
- DocumentSession normal en koude recovery in afzonderlijke VM-invocaties.

De eerdere main-run na #25 liet één normale M6c-flake zien. Dat resultaat is
bewust niet als groen bewijs gebruikt voor deze eindstand; de definitieve
run hierboven is na #26 opnieuw uitgevoerd.

## Duplicatie, rommel en architectuur

De audit heeft geen reden gevonden om bewust gescheiden kernel/userspace-,
driver- of capabilitycomponenten samen te voegen. Historisch bewijs blijft
historisch bewijs. Gegenereerde build-/targetoutput wordt via ignore-/CI-regels
buiten de actieve bronstand gehouden. De eigen Rust-kernel blijft het fundament;
er is geen Linux-, Windows-, macOS-, Android- of BSD-basis geïntroduceerd.

## Open bewijs- en beheergrenzen

- Geen nieuwe fysieke-hardwareacceptatie of volledige historische
  M6c/M6d-mijlpaalreplay op deze commit; CI-QEMU is een integratiesmoke.
- DocumentSession sluit U03/A01 nog niet volledig: geen algemene appstarter,
  interactieve Materiaal-GUI, grote documenten of volledige publieke SDK/ABI.
- De netwerkparser sluit N01 niet: nog geen native netwerkdriver/-service,
  TCP/DNS/TLS of afdwingbare proces-/bestemmingsrechten.
- De actieve main-ruleset vereist PR/lineaire historie en beschermt tegen
  delete/force-push, maar verplicht de nieuwe CI-statuscheck nog niet.
- De beschikbare GitHub-koppeling kon ruleset/security-admininstellingen niet
  wijzigen en exposeert geen veilige branch-deleteactie.
- CodeRabbit had op #26 bij merge nog geen inhoudelijke bevinding gepubliceerd;
  de botstatus was nog in uitvoering. Codex-review op de later gesloten #27
  mislukte wegens bereikt reviewquotum en geldt niet als reviewgoedkeuring.
