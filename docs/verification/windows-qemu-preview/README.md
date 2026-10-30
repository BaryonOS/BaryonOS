# Windows-QEMU-preview: herstelde readinesscontrole

30 oktober 2026; branch `simon/windows-qemu-test-version`, basis
`6ec0bac45f6deb1d59791ee724f3997bcb1aa710` met de bij dit bewijs
gecommitte readinessfix. **Historisch testbewijs:** beide rapporten bewaren
`source_commit` en `source_sha256`, maar geen `source_fingerprint_version`.
De versie van het hashformaat is daarmee niet zelfstandig in die rapporten
vastgelegd; een v3-binding wordt hier niet geclaimd. De basiscommit alleen
bevat de fix nog niet. De oorspronkelijke rapportbytes blijven ongewijzigd.

De eerste native run faalde na 45 seconden (runner exit 1), hoewel de gast
alle readinessmarkers schreef. Het orakel verwachtte ten onrechte Source
vóór Policy. Deze diensten starten onafhankelijk; de bestaande kernelobserver
meldt hun werkelijke gereedkomen. Geen wijziging aan scheduling, rechten of
gastcode: de runner eist nu de geordende bootketen, daarna het manifest en
iedere afzonderlijke readinessmarker precies eenmaal. Duplicaten, ontbrekende
markers, readiness vóór manifest en gastfouten blijven geweigerd.

De tweede run slaagt (runner exit 0, QEMU exit 0 na QMP `quit`) in 3,874 s.
QMP maakte een echt 800×600-frame met 310 kleuren. Dit is uitsluitend
boot/readiness en een niet-leeg framebufferbeeld; geen bewijs van correcte
lay-out, handmatige bediening, opslagherstel of Windows-compatibiliteit.
De gastbinair bleef gelijk: kernel
`e9fb2565fd74c40e9754f2b179806834c59c76177d911812951958d3d084e192`.

## Exacte uitvoering

```sh
. .tools/nieuw-os/rtc-env.sh
export PATH="$PWD/.tools/nieuw-os:$PATH"
export NOS_BUILD_DIR="$PWD/build/windows-qemu"
export NOS_TARGET_DIR="$PWD/target/windows-qemu"
export CARGO_TARGET_DIR="$PWD/target/windows-qemu"
python3 -B -u tools/build.py --kernel-feature native-system-materiaal
python3 -B -u tools/run_materiaal_preview.py \
  --bios-rom "$PWD/.tools/nieuw-os/emulator/usr/share/seabios/bios-256k.bin" \
  --vga-rom "$PWD/.tools/nieuw-os/emulator/usr/share/seabios/vgabios-stdvga.bin"
python3 -B -m unittest tools.tests.test_materiaal_preview tools.tests.test_package_windows_qemu tools.tests.test_materiaal_probe
git diff --check
```

Herbouw, herhaalde preview, 31 hosttests en diffcontrole: alle exit 0.
Volledige lokale commandologs: `/tmp/nieuw-os-windows-qemu-build.log`,
`/tmp/nieuw-os-windows-qemu-preview.log` (mislukt),
`/tmp/nieuw-os-windows-qemu-preview-retry.log` (geslaagd).

## Bewaargrens en open punten

`failed-result.json` en `passed-result.json` bewaren de oorspronkelijke
rapporten met exacte commandoregel, bron-, ELF-, firmware- en screenshot-
hashes. De bijbehorende seriële uitvoer is eveneens bewaard. Binaire images
en framebufferdump blijven lokaal onder de gerapporteerde runmappen.

De lokale QEMU-build meldt een ontbrekende optionele `kvmvapic.bin`-ROM in
stderr; de gast startte en bovenstaande readiness-/framecontrole slaagde.
Dit is geen gevalideerde Windows-firmwareconfiguratie. Linux gebruikt hier
`rng-random`, terwijl de launcher `rng-builtin` vereist; SDL en PowerShell
zijn op deze host niet uitgevoerd. Geen ZIP gemaakt. Afstemming met
`simon/native-test-version` over de uiteindelijke geteste bron loopt.
Score blijft **34% → 34%**; het 50%-doel en de volledige release blijven open.
