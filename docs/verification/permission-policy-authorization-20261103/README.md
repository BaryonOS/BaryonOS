# Policy-autorisatie — rood/groen en begrensde status

3 november 2026; branch `simonvanieperen-vigilant-robot`.
Basis: `8389a7924323b9564667996c6c95d83d91106754`.
Geteste fixbron: `eb94683895cf69ea3a6cb3e3f88c228154b6d921`, gepusht en
via `git ls-remote` bevestigd vóór de brede controles.
Dit checkpoint wijzigt alleen `crates/permission-policy-core/src/lib.rs`
als productcode; geen UI-, kernel-, integratie- of gedeelde voortgangswrites.

## Vastgesteld en hersteld

Voor dezelfde app/resource/purpose/context/activity vervangt de laatst
opgeslagen regel zowel beslissing als foregroundvoorwaarde. `Scope` bevat
alleen die vijf doelvelden; `Rule.foreground_only` is een opgeslagen
voorwaarde. Een aanvraag kan die beleidsvoorwaarde niet meer kiezen.
`evaluate` leest haar uitsluitend uit de gevonden regel. Actieve activiteit
is altijd vereist voor Allow; foreground is bovendien vereist als de
opgeslagen voorwaarde dat vraagt. Deny blijft Denied, ook bij inactiviteit.

`set`, `evaluate` en `decode` gebruiken dezelfde vijfledige identiteit.
Decode weigert dubbele doelen, ongeacht beslissing, conditie of volgorde;
geen stilzwijgende last-wins-migratie van ambigue opslag.
Het versie-1-formaat (`NOSPOL01`, 16-byte header, 32-byte records) blijft
bytegelijk. De Rust-API verplaatst het veld van `Scope` naar `Rule`.
Repositorybrede zoekopdracht in Rust-bronnen en Cargo-manifests vond geen
afnemers buiten deze crate; dit is geen garantie over externe afnemers.

## Eerst rood, daarna groen

`tests-only.patch` voegt uitsluitend zes tests toe aan de basisbron.
Pas na de uitgevoerde rode test zijn productcode en testconstructors
aangepast voor de gescheiden API. `red-source.sha256` bindt de rode bron.

```sh
cargo +1.90.0 test --offline --locked -p permission-policy-core regression_
```

Exit **101**, zes tests gefaald, vijf bestaande tests uitgefilterd.
Concreet: Allow/foreground-only gevolgd door Deny/ongeconditioneerd leverde
nog `PermitsActivation` op in plaats van `Denied`; de omgekeerde volgorde
bleef Denied; decode accepteerde dubbele doelen met afwijkende voorwaarden.
Volledige uitvoer: `red.log`. De opgeslagen patch is opnieuw op de basisbron
toegepast in de lokale bewijsmap; de gereconstrueerde bronhash komt exact
overeen met `red-source.sha256`.

Na herstel slagen **14 tests**: vijf bestaande en negen regressietests,
inclusief beide vervangingsrichtingen, activiteit/foregroundmatrix,
codecbehoud, conflicterende duplicaten in beide volgorden, ieder doelveld,
vervanging bij volle tabel en onafhankelijke versie-1-bytefixtures.
De meeting-app-/microfoontest is uitsluitend beleidsemantiek:
`PermitsActivation` is geen handle of uitgegeven runtime-capability.
Evaluatie wijzigt de opgeslagen keuze niet; uitgifte/intrekking en werkelijk
microfoongebruik zijn niet geïmplementeerd of getest.

## Uitgevoerde gerichte controles

Alle onderstaande commando's: **exit 0**. Exacte meetgegevens:
`directed-results.json`; volledige uitvoer in gelijknamige `.log`-bestanden.

```sh
cargo +1.90.0 test --offline --locked -p permission-policy-core
cargo +1.90.0 fmt -p permission-policy-core -- --check
cargo +1.90.0 clippy --offline --locked -p permission-policy-core --all-targets -- -D warnings
cargo +1.90.0 check --offline --locked -p permission-policy-core -p capability-core -p kernel-core -p platform-profile-core --target x86_64-unknown-none
git diff --check
cargo +1.90.0 fmt --all -- --check
```

Rust 1.90.0, offline/locked, afzonderlijke targetmap:
`target/policy-authorization-review/build`. De eerste hostpoging faalde
omdat `cc` ontbrak; een volgende poging miste de sysroot-loaderalias.
De oude tijdelijke GCC-installatie kon niet worden hergebruikt zonder de
geldende tijdelijke-padgrens te overschrijden. Daarom zijn na de
missing-linkerfout GCC 15/libc-ontwikkelpakketten lokaal uitgepakt onder
`target/policy-authorization-review/toolchain`, niet systeembreed.
`env.sh` legt de werkelijk gebruikte omgeving vast, inclusief `TMPDIR`
onder de projectmap; geen Cargo-dependency gewijzigd.
De extractie gebruikte `apt download gcc-15-x86-64-linux-gnu
cpp-15-x86-64-linux-gnu libgcc-15-dev libc6-dev libc-dev-bin libgcc-s1 libc6`,
daarna `dpkg-deb -x` per pakket en lokale `lib -> usr/lib` /
`lib64 -> usr/lib64`-aliassen. `receipt.json` bevat pakketversies en hashes.

## Brede hostworkspace

De volgende commando's zijn uitgevoerd op de fixbron; alle definitieve
controles **exit 0**:

```sh
cargo +1.90.0 test --offline --locked --workspace --exclude os-loader --exclude os-kernel
cargo +1.90.0 clippy --offline --locked --workspace --exclude os-loader --exclude os-kernel --all-targets -- -D warnings
cargo +1.90.0 check --offline --locked --workspace --exclude os-loader --exclude os-kernel --all-targets
cargo +1.90.0 fmt --all -- --check
git diff --check
```

Definitieve workspace-test: **444 geslaagd, één bestaande test genegeerd**,
inclusief doctests. Geen tests uitgezet om de gate te halen.
De eerste brede testrun eindigde met **exit 101** doordat rustdoc de
linker-sysrootflag niet uit `RUSTFLAGS` overnam; drie HKDF-doctests konden
niet linken. Na `RUSTDOCFLAGS="$RUSTFLAGS"` slaagde dezelfde testopdracht,
zonder bronwijziging. De loader-/kernelbinaries zijn expliciet uitgesloten
van de hostgate; dit is geen native bootbewijs.

Volledige rode/gerichte logs en de definitieve workspace-uitvoer zijn hier
opgeslagen met genormaliseerde eindwitruimte (de inhoud en uitslagen zijn
ongewijzigd). De ruwe lokale hashes en opgeslagen hashes staan afzonderlijk
in `receipt.json`. De eerste workspacefout staat als begrensd foutfragment in
`workspace-tests-first-failure.txt`; het volledige initiële log blijft lokaal
onder `target/policy-authorization-review/workspace-tests.log`.
`receipt.json` bindt alle commando's, exits, doorlooptijden en loghashes aan
de bron; de initiële rode reproductie is niet op wandkloktijd gemeten.
Een afsluitende `git diff 8389a7924323b9564667996c6c95d83d91106754 --check`
vond alleen eindwitruimte in opgeslagen testuitvoer. Na deze expliciete
archiefnormalisatie slaagt ook die volledige wijzigingscontrole.

## Open status / integratiegrens

Geen QEMU/native, hardwaretest of volledige CI uitgevoerd:
deze crate heeft geen runtime-
of UI-afnemer. Opslagauthenticatie, betrouwbare app-/sessie-identiteit,
runtime-uitgifte en intrekking blijven broker-/integratiewerk.
`no_std` en `forbid(unsafe_code)` blijven behouden.

PR #156 was bij inspectie open, geen concept, `CONFLICTING`/`DIRTY`,
`CHANGES_REQUESTED`; alleen CodeRabbit stond in uitvoering.
Lokale head versus `origin/main`: 12 commits alleen op head, 1 alleen op main.
Na de fixpush: 13 commits alleen op head, 1 alleen op main; dezelfde
conflict-/reviewstatus, geen checks op de nieuwe head bij die inspectie.
`pr-after.json` en `divergence-after.json` bewaren die live snapshot.
Geen merge, rebase, conflictoplossing of reviewthread-resolutie uitgevoerd.
Alle vijf reviewthreads blijven open: de policy-Major is hier inhoudelijk
hersteld; vier documentatiepunten vallen buiten deze opdracht
(ADR-link/status, oude policy-bewijsbron, Terminal-kernelbronnen,
controleerbaarheid van oude Terminal-logs).

Gedelegeerde criteriumbeoordeling: **41 → 41/100 voorgesteld**, geen nieuw
native criterium geaccepteerd. K04 is niet gewijzigd; B05/B06 blijven open.
De integrator beheert assessment en gedeelde
score/statusbestanden; deze afgebakende taak schrijft die niet.
Geen onafhankelijke menselijke audit of algemene veiligheidsclaim.
