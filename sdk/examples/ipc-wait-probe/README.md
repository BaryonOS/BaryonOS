# WAIT-PROBE-1 — twee echte CPL3-processen, één ELF

Zelfstandig `no_std`-acceptatieprogramma voor het opt-in kernelprofiel
`native-ipc-wait` (ABI **1.10**, niet RTC-revisie 1.9). Geen allocator, nieuwe
dependencies, privilege-I/O, display, entropy, netwerk of persoonlijke data.
De integrator levert manifest, schedulerfixture, cleanupcontrole en QEMU-gate.
Een geslaagde build/hosttest is **geen native runtimebewijs**.

Bronbasis bij aanvang: `2a596f4df94573a9a8d74e9cbef668dedc30cf38`,
branch `simon/native-test-version`. De probe hergebruikt
`../materiaal/linker.ld`: RX-entry op `0x40000000`, private RW-startup op
`0x40010000`, gescheiden R/RW/RX-segmenten. `_start` doet `cld`, controleert
SysV-entry `RSP % 16 == 8` en springt zonder extra returnadres naar Rust.
`support.rs` volgt de bestaande scalair/volatile memory-intrinsics.

## Startupcontract

Alle onderstaande getallen zijn **byte-offsets van u64-woorden** vanaf de
private databasis `0x40010000`, geen woordindices.

| Offset | Invoer door kernel |
|---|---|
| 32 | Rol: 0 = initiator/sender, 1 = peer/receiver |
| 56 | Eigen endpoint-root, `ALL_RIGHTS`; RECV/SEND/DERIVE/REVOKE blijven kernelrechten |
| 64 | SEND-only afstammeling van het endpoint-root van de peer |
| 72 | SEND-only afstammeling van het eigen endpoint-root, **geen RECV** |
| 80 | Niet gelezen; mag nul zijn |
| 176 | Werkelijke PID van de peer, verplicht niet-nul |

Er worden geen PID's 1/2 aangenomen. Elk peerbericht moet exact de aangeleverde
sender-PID, tag, payloadlengte, bytes en nul gedelegeerde capabilities hebben.
Alle gewone handles gaan door `Handle::from_raw`; endpointwrappers claimen
geen autoriteit. De negatieve raw ABI-tests gebruiken bewust ook handle nul.
De SEND-only handle op 72 mag niet simpelweg dezelfde ALL_RIGHTS-handle als
56 zijn: de negatieve RECV-rechtentest moet daarop `-5` krijgen.

## Evidencecontract

| Offset | Uitvoer |
|---|---|
| 8 | Heartbeat, verhoogd bij begin van iedere fase |
| 400 | `done=1`, **als laatste geschreven**, zowel bij succes als fout |
| 408 | Aantal geslaagde expliciete assertions |
| 416 | Foutcode; nul uitsluitend na alle rolfases |
| 424 | Werkelijk gemeten totale PIT-ticks sinds start, ook bij fout |
| 432 | Afgerond-fasebitmasker, zie hieronder |
| 440 | Starttick |
| 448 | Actieve fase (1–6), 7 na volledige rolacceptatie |
| 464 | Rol0: gemeten ticks in lege-RECV-wacht |
| 472 | Rol0: gemeten ticks in volle-SEND-wacht |
| 480 | Beide rollen: gemeten ticks van gezamenlijke count0-wacht |
| 488 | Rol0: gemeten ticks in de SEND-wacht die `-10` retourneert |
| 496 | Rol1: exact 17 geconsumeerde FULL-faseberichten |
| 504 | Rol1: 16 finale berichten niet gedraind na rootrevoke |

`432` heeft vaste bits, pas gezet na alle lokale faseassertions:

| Bit | Betekenis | Rol0 | Rol1 |
|---|---|---|---|
| 0 (`0x01`) | Negatieve raw-wait-ABI, alle items gevalideerd | Ja | Nee |
| 1 (`0x02`) | Onmiddellijke readiness en niet-consumerende maskers | Ja | Nee |
| 2 (`0x04`) | Lege RECV → vertraagde peer-SEND → bytes | Wacht/ontvangt | Vertraagt/verzendt |
| 3 (`0x08`) | Gezamenlijke absolute count0-timerwacht | Ja | Ja |
| 4 (`0x10`) | Volle SEND → peerdrain → wake, exacte FIFO zonder replay | Vult/wacht/verzendt | Draint/controleert |
| 5 (`0x20`) | Volle queue → rootrevoke → sender `-10`, geen drain | Wacht/controleert | Revoke/controleert |

**Exact eindmasker rol0: `0x3f`; rol1: `0x3c`.** Het programma controleert
zelf deze exacte maskers. Succes schrijft `done=1` en roept `exit(0)` aan.
Elke fout schrijft evidence en roept `exit(nonzero)` aan; paniekcode is 255.
De kernel moet beide exits, beide exacte maskers, nul foutcodes, assertions,
ticks, `blocked/wake_ready/wake_timeout/idle_irqs > 0` en lege authority-engine
na cleanup controleren. Evidence is procesdata, geen vervanging van deze
kernelobservaties.

## Protocol en syscallbewijs per fase

Alle berichten zijn SDK `Outgoing`/`Received`, tag `0x5741495450520001`.
Payload is precies 16 bytes: kind:u8 op 0, drie nulbytes, sequence:u32-LE
op 4 en at:u64-LE op 8. Er worden geen capabilities in berichten verzonden.
Fasedeadline is eenmaal `now+200`; preparatie-/drain-/revoke-timers zijn
`now+40`. Alle berekeningen controleren overflow. Een `EAGAIN` vernieuwt
**nooit** de deadline. SEND-succes wordt nooit herhaald.

1. **Negatieven (rol0, fase1).** Raw `0x22` controleert count17/MAX,
   count0 met niet-nul pointer, elk gereserveerd argument, deadline buiten
   horizon: `-1`. Nullpointer, kernelpointer en een 16-byte-record dat de
   startup-pagegrens op `0x40010ff8` overschrijdt: `-3`. Interests0/3/MAX:
   `-1`. RECV op eigen SEND-only: `-5`. Eerst ready SEND en daarna handle0
   moet `-4` blijven; ready SEND gevolgd door onrechtmatige RECV moet `-5`
   blijven. Beide arrayvolgordes worden getest. Count0 met verstreken
   deadline retourneert nul; SEND-readiness retourneert SDK-`u16` 1.
2. **Onmiddellijk (rol0, fase2).** `0x21` geeft eerst `EAGAIN`. `0x20`
   verstuurt één echt selfbericht. Twee opeenvolgende `0x22`-calls met
   SEND+RECV leveren allebei masker3: geen reservering/consumptie door
   wait. Daarna één receive met exacte bytes, vervolgens `EAGAIN`.
   Zestien ready SEND-items geven `u16::MAX`.
3. **Lege RECV (beide, fase3).** Rol0 verstuurt EMPTY, controleert lege
   eigen queue en wacht RECV. Rol1 ontvangt EMPTY, wacht 40 echte ticks
   met count0 en verstuurt DATA met niet-triviale sequence/payload.
   Rol0 eist masker1, positief gemeten wachtinterval, exact één DATA en
   daarna lege queue. Foutcodes160–163/200.
4. **Allebei Waiting (beide, fase4).** IDLE draagt één gedeelde absolute
   deadline. Rol1 stuurt IDLE_ACK en doet count0-wait tot die deadline.
   Rol0 ontvangt ACK en doet dezelfde count0-wait. Beide controleren dat
   de deadline toekomstig was, masker0 terugkeert en de tickstand minstens
   de deadline bereikt. Met precies twee processen moet de kernel echt
   PIT-idle via `sti; hlt` observeren; de app kan die instructie niet zelf
   certificeren. Foutcodes170/201–203.
5. **Volle SEND (beide, fase5).** FULL draagt toekomstige drain-tick,
   rol1 ACKt en slaapt zonder te ontvangen. Rol0 verstuurt sequences0–15,
   een poging met16 geeft `EAGAIN`; hij wacht SEND vóór de drain-tick.
   Rol1 draint op die tick. Rol0 eist ready1 en positieve wachtticks,
   verstuurt sequence16 **precies eenmaal** na wake. Rol1 ontvangt exact
   0–16 in volgorde, controleert daarna `EAGAIN` en ACKt DRAINED17.
   Zowel replay, verkeerde bytes als een ontbrekend bericht faalt.
   Foutcodes150–154/180–185/210–212.
6. **Revoke zonder drain (beide, fase6; als laatste).** REVOKE/ACK spreken
   een revoke-tick af. Rol0 vult opnieuw exact16, krijgt `EAGAIN` en
   wacht SEND. Rol1 slaapt zonder drain, controleert RECV-readiness en
   doet `0x12(root)`; de root blijft zelf geldig. RECV is erna nog ready;
   de eigen SEND-afstammeling is nu `-10`. Er volgt **geen receive** op
   de volle queue. Rol0 eist `-10` van de wacht, positief gemeten ticks,
   en nogmaals `-10` van zowel SEND als WAIT; nooit ready/vals succes.
   Rol1 stuurt REVOKED over de nog geldige tegenovergestelde richting.
   Rol0 ontvangt die bevestiging vóór exit. Rol1 blijft nog 40 ticks
   leven om root-exit niet als vervanging voor de revoke te gebruiken.
   De kernel moet de echte wake-error/queue/cleanupobservaties leveren;
   de 40-tick-houdtijd is geen algemeen scheduler-fairnessbewijs.
   Foutcodes190–195/220–226.

Algemene fouten: ABI/startup1–8, klok90–91, encoding92,
bounded SEND100–103, bounded RECV/envelope110–115. De syscallchecks in
fase1 gebruiken120–138 en onmiddellijke readiness140–148.

De huidige SDK mist een publieke `native::syscall`; daarom zit uitsluitend
voor negatieve `0x22`-tests een lokale raw int80-wrapper in `main.rs`, met
hetzelfde registercontract en dezelfde `decode_return` als SDK-native.
Geen Rust-code dereferentieert de bewust ongeldige testpointers.

## Bouwen en controles

Geen toolchain-installatie. Gebruik de bestaande geïsoleerde omgeving:

```sh
export PATH="$PWD/.tools/nieuw-os:$PATH"
export LD_LIBRARY_PATH="$PWD/.tools/nieuw-os/host-cc/usr/lib/x86_64-linux-gnu"
export CARGO_TARGET_DIR="$PWD/target/ipc-wait-probe-host"
python3 -B tools/dev.py cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/ipc-wait-probe/Cargo.toml
python3 -B tools/dev.py cargo +1.90.0 check --offline --locked --manifest-path sdk/examples/ipc-wait-probe/Cargo.toml
python3 -B tools/dev.py cargo +1.90.0 fmt --manifest-path sdk/examples/ipc-wait-probe/Cargo.toml --all -- --check
python3 -B sdk/examples/ipc-wait-probe/build.py
```

`build.py` respecteert een expliciete `CARGO_TARGET_DIR`; relatieve paden
worden vanaf de repositoryroot opgelost. Zonder die variabele gebruikt het
`target/ipc-wait-probe-host`. De integrator levert een geïsoleerde per-appmap
(bijvoorbeeld `$NOS_TARGET_DIR/ipc-wait-app`), nooit een gedeelde Cargo-map.
De builder gebruikt gepinde compiler1.90.0, bijpassende rust-lld en bestaande
scalar-flags. Het roept de bestaande
`tools/build_users.py::inspect_user` aan (ook gebruikt door
`tools/build_materiaal.py`) en print ELF-segmenten, hash en
`NOS_IPC_WAIT_PROBE_ELF`. Artifact:
`target/ipc-wait-probe-host/x86_64-unknown-none/release/ipc-wait-probe`.
Met een expliciete targetmap staat het artifact steeds op
`$CARGO_TARGET_DIR/x86_64-unknown-none/release/ipc-wait-probe`, en zowel de
inspectie als `NOS_IPC_WAIT_PROBE_ELF` verwijzen naar datzelfde pad.
Integratiecontrole met afwijkende targetmap:

```sh
CARGO_TARGET_DIR="$PWD/target/ipc-wait-probe-explicit" python3 -B sdk/examples/ipc-wait-probe/build.py
```

Dit is een echte native releasebuild plus ELF-inspectie; geen QEMU-run.
De scalar-buildroute vermijdt SSE/XMM-context; de uitgevoerde ELF-disassemblage
bevatte geen XMM/YMM/ZMM-registerreferenties.

Uitgevoerd tijdens implementatie: lockfile via Cargo gegenereerd; hosttests
3/3, hostcheck, formatter/check, native releasebuild en ELF-inspectie:
**exit0**. De laatste vier commando's hierboven zijn alle succesvol uitgevoerd.
Laatste ELF-SHA256:
`3889333416ba3259853caa345aa0809f51b1f9fe5df0cb6ee51e194005aabf2a`.
`cargo fmt` kon aanvankelijk niet draaien: component `rustfmt` ontbreekt
in de aanvankelijk aangeleverde toolchain. Na beschikbaarstelling door de
omgevingseigenaar is de formatter uitgevoerd en de check herhaald: exit0.
Geen component toegevoegd door deze taak.
QEMU/native uitvoering, kernelcleanup en hele-releaseacceptatie:
**niet uitgevoerd door deze deelagent**. Integrator is eigenaar van
status/scoreblad; dit bron-/buildresultaat kent geen voortgangspunten toe.
