# Begrensde renderbatch — 2 november 2026

Basis vóór wijziging: `7648a8ffc121c0939325cb0b3628256e28c11fec`.
Implementatiebestand: `sdk/examples/compositor/src/main.rs`.
Dit rapport beschrijft de afgebakende compositorwijziging; de integrator
verbindt de uiteindelijke bronhash en native uitvoering aan het finale bewijs.

## Probleem en contract

De vorige compositor verwerkte één SUR1-bericht per hoofdloop en stond daarna
zijn CPU-beurt af. Een volle tegel van 32×8 pixels omvat BEGIN, vijf CHUNKs
van maximaal 208 bytes en COMMIT: zeven berichten voordat één tegel zichtbaar
wordt. De kernelqueue kan zestien berichten bewaren; de client wacht na
COMMIT op zijn bevestiging. De native sessieproef van de integrator op de
basisversie liet werkelijk langdurige gedeeltelijke schermopbouw zien.

De nieuwe hoofdloop verwerkt maximaal acht opeenvolgende `Surface::step`-calls
per beurt. Elke call leest eerst opnieuw de monotone klok; de afzonderlijke
verse kloklezing vlak vóór COMMIT blijft behouden. De batch stopt direct
wanneer geen nieuw bericht is ontvangen of een antwoord door backpressure
niet kan worden verstuurd (`reply_again` verandert).

Alle sender-, capability-, epoch-, lengte-, pixel-, replay- en timeoutcontroles
blijven in dezelfde ongewijzigde `Surface::step`/receiver. Een pending reply
blijft verdere ontvangst blokkeren. Een fout sluit dezelfde endpoints en
beëindigt hetzelfde proces. Er zijn geen protocolwijzigingen, extra threads,
apprechten, raw framebuffertoegang of extra overdrachtsbuffers.

Control/focus/retire wordt vóór iedere batch verwerkt. Chrome, statistieken
en de bestaande yield blijven na de batch staan; schedulerpreëmptie blijft
actief. Een nieuwe controlaanvraag kan maximaal de resterende acht begrensde
step-calls wachten, geen onbegrensde queue-drain. Een reeds ontvangen retire
sluit de surface vóór de batch, waardoor volgende calls geen ontvangst doen.

## Verwacht effect en bewijsgrens

Als alle zeven tileberichten beschikbaar zijn, kan de compositor één tegel
nu in één hoofdloop afhandelen in plaats van zeven. Het aantal protocol- en
pixelcontroles blijft gelijk. Dit is geen claim van zeven- of achtvoudige
schermsnelheid: app-input, clientbeurten, rendering, syscalls, framebuffer-MMIO
en eventuele preëmptie kosten nog tijd.

Een afzonderlijke read-only AI-review controleerde zowel de aanpak als de
uiteindelijke diff. Die review is geen menselijke audit. Native bewijs voor
de samengevoegde bron en werkelijke tijden wordt door de integrator uitgevoerd;
de hieronder genoemde hosttests voeren de hoofdloop in `main.rs` niet uit.

## Uitgevoerd

Alle opdrachten gebruiken de bestaande Rust 1.90.0-omgeving, geïsoleerd onder
`build/perf-compositor/`, zonder de build-/targetmap van de integrator te delen.

```sh
source /workspace/.nieuw-os-env/activate.sh
CARGO_TARGET_DIR=/workspace/nieuw-os/build/perf-compositor/host-target \
  cargo +1.90.0 test --offline --locked \
  --manifest-path sdk/examples/compositor/Cargo.toml
CARGO_TARGET_DIR=/workspace/nieuw-os/build/perf-compositor/native-target \
  python3 -B sdk/examples/compositor/build.py
cargo +1.90.0 fmt --manifest-path sdk/examples/compositor/Cargo.toml -- --check
git diff --check -- sdk/examples/compositor/src/main.rs
```

- Hostsuite vóór en na de wijziging: **15 tests geslaagd**; finale opdracht
  exit 0. Daaronder verkeerde afzender/overgedragen caps, vervallen commit
  vóór framebufferwrite, verloren reply, mislukte displaywrite, retirement,
  focusdeadline en wipe/retirevolgorde.
- Native crossbuild + ELF-inspectie: **geslaagd**, exit 0, Rust 1.90.0,
  `x86_64-unknown-none`, release, bestaande statische linker-/scalarflags.
  Compositor-ELF SHA-256:
  `3aad69b5b6320cfd945bbdf2ddcfe197cb8fa698f963f9fe4528956da6828eb7`.
- Formatter en diffcontrole: exit 0.
- Volledige integratie-VM, fysieke hardware, Windows/WHPX en systematische
  snelheidsbenchmark: niet door deze deelagent uitgevoerd.

Lokale logs: `build/perf-compositor/logs/host-before.txt`, `host-after.txt`
en `native-build.txt`. Binaries/caches blijven buiten Git.

Rollback: herstel alleen deze acht-call-loop naar de vroegere enkele
`Surface::step`; er zijn geen image-, protocol- of capabilitymigraties.

## Afzonderlijke launcherfix: QMP in de WHPX-initialisatieproef

De integrator meldde dat de eerdere HMP-tekstprobe op Windows de volle tien
seconden wachtte en ten onrechte TCG koos, terwijl een LF-only QMP-proef
met dezelfde machine direct werkte. Dat Windowsresultaat is externe input
voor deze deelagent; de finale integrator legt bron en reële herhaling vast.

`tools/windows-qemu/Acceleration.ps1` gebruikt daarom nu `-monitor none
-qmp stdio`. Exacte UTF-8-bytes met alleen LF vragen `qmp_capabilities`,
`query-status` en `quit` op, na een echte greeting en met een verwachte
bevestiging vóór de volgende opdracht. Alle stappen delen één deadline van
tien seconden; onverwachte IDs/errors en meer dan 31 interleavende events
per antwoord worden geweigerd. Succes vereist exit 0, exact één correcte
bevestiging voor alle drie request-ID's en status `paused` of `prelaunch`.
Een QMP-greeting alleen is onvoldoende. Een fout, ontbrekend/dubbel antwoord,
`running` of timeout leidt tot de bestaande expliciete fallback/weigering.
De gepauzeerde proef krijgt nog steeds geen disks, netwerk of gastuitvoering.

Uitgevoerd op Linux-PowerShell 7.5.4: 33 beleid-/QMP-/handshaketests, zes
synthetische launcherchecks, twaalf package-hosttests en drie subprocesscases
voor exacte LF-requestbytes, succes en geweigerde ongeldige status/ontbrekende
quitbevestiging; alle exit 0. De testgevallen staan in
`tools/windows-qemu/Test-Acceleration.ps1`; de tijdelijke subprocessfixtures
staan onder `/tmp/nieuw-os-whpx-probe-process-tests`. Windows/WHPX-runtime
is door deze deelagent niet uitgevoerd.
