# G02 — native SDK-route in bestaande catalogusproef

Bron `0617670585c818b9650f4ddef6e714f1be9fb034`, basis
`1acf6e8cde2a63849dc49c948ad97ba7f2c39cad`. De bestaande service-/clientrollen
zijn inhoudelijk behouden. Vier cfg-regels kiezen optioneel een nieuwe
`userspace/sdk_probe_runtime.rs`; zonder `--cfg nos_sdk_runtime` blijft de
bestaande runtime geselecteerd.

## Werkelijke routes

| Handeling | Opt-in route |
|---|---|
| Canonieke IPC-send met geldige handlevorm | `nieuw_os_sdk_runtime::native::send`, met geleende eigen berichtbuffer |
| IPC-receive | `native::receive`, met eigen gecontroleerde stackbuffer |
| Normale yield, ABI-info en cap-close | SDK-wrappers op bestaande syscallnummers |
| Malformed ABI-argumenten of invalid handlevorm | Oorspronkelijke raw syscall bereikt de kernel; lokale SDK-afwijzing geldt nooit als kernel-negatiefbewijs |
| READ/WRITE/FLUSH/poll, vaste startupvelden en faultinjectie | Bestaande fixture-/blokroute |

De bridge verhoogt de bestaande syscallteller op offset 8 precies vóór de
gewone SDK-aanroep. De bestaande, door de kernel geselecteerde service-crashgate
op offset 184 wordt direct erna gecontroleerd. Wachten herhaalt alleen een send
na pre-publication `EAGAIN`; na enqueue volgt geen RPC-replay. Fouten uit SDK-
representatiecontrole worden een gefaalde probe, geen verwachte kernel-errno.

De bridge verplaatst geen document- of appbeleid naar de kernel. De oorspronkelijke
M6c-testrollen, native catalogusdienst, journal/wire, linker en `m6_runtime.rs`
zijn gecontroleerd tegen de basis. Alleen de vier runtimekeuzeregels zijn aan
de probe toegevoegd; de overige vijf bestanden zijn bytegelijk.

## Uitgevoerde controle

De [opname](report.json) bevat echte opdrachten, logs, toolchain,
bron-/artifacthashes en exitstatussen. Runtime-build, probe-build, gerichte
rustfmt en diffcontrole slagen met exit 0. Alle geregistreerde broninputs zijn
ongewijzigd gedurende de run. Het ELF voldoet aan de bestaande
`inspect_user`-grenzen: entry `0x40000000`, 45.426 bytes RX, 16.384 bytes RW en
2.887 bytes RO, zonder overlappende pagina's of W+X-segment.

Probe SHA-256:
`088f0daba5f5a92850596fb50836c99d5df70095adc7a0e993fb36965fb066d4`.
De bestaande ongebruikte `Catalog::EMPTY` geeft één buildwaarschuwing; er is
geen compileerfout. Binaries staan buiten Git.

```sh
python3 -B docs/verification/parallel-g02/verify.py
```

De native integrator gebruikt de normale M6c-compileerflags en voegt alleen
`--cfg nos_sdk_runtime --extern nieuw_os_sdk_runtime=<gebouwde-rlib>` toe.
De exacte twee rustc-opdrachten staan in het report. Gebruik een eigen
`G02_BUILD_DIR` bij parallelle herhaling.

## Bewijsgrens

Dit deelrapport bewijst compilatie en behoud van de bestaande probe-inhoud.
Het voert geen QEMU uit. De rootintegratie moet de echte catalogusberichten,
WRITE/FLUSH, koude recovery en relevante foutproeven op de gecombineerde bron
uitvoeren en apart vastleggen. Tot dat native bewijs beschikbaar is, is er
geen nieuwe native SDK-acceptatieclaim en geen aanvullend roadmapcriterium.
