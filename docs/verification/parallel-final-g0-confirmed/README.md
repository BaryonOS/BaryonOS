# Bevestigde native G0-uitvoering

Eén nieuwe echte native run op bron `3e41bbf93822f7044c6020259f4a752925e1966d`
slaagt met raw33 en alle tien G0-checks. Transport is QMP via private pipes,
runmap `/tmp/nieuw-os-g0-confirmed-pipe/graphics-g0-6c34d3a840`,
fingerprintv3 `307c828e9ce7bf49a4e809859ca77ecd332015572dd37e96fb580de98951aa60`.
Alle15 originele artifacts zijn na afloop en na latere toolcalls opnieuw
gehasht; zij zijn onveranderd. De seriële uitvoer bevat425bytes inclusief
G0:PASS/M1:PASS en is bytegelijk aan de rechtstreeks opgeslagen archiefkopie.

Vijf PNG-bestanden in pipe/ coderen uitsluitend de werkelijke QEMU-PPMpixels.
image-roundtrip.json bewaart de oorspronkelijke header en beide hashes;
header+gedecodeerdeRGB-pixels reconstrueren ieder origineel PPM exact.
Dit zijn native screenshots, geen browser- of gegenereerde ontwerpbeelden.
Backspace herstelt exact de opname na a; iedere overgang verandert alleen de
begrensde invoerregio. De officiële VGA-ROM is expliciet gepind op de hash
in result.json. Kernel/loader/firmware en QMP-verkeer zijn eveneens gebonden.

De eerdere run ../parallel-final-g0/ blijft uitgesloten omdat het bewaarde
originele serialbestand later afweek. Oorzaak onbekend; oude artifacts zijn
niet hersteld. Een extra UNIX-socketprobe faalde op EPERM binnen de sandbox;
unix-environment-failure/ bewaart dat mislukte resultaat (raw−9 na cleanup).
De uitvoeringsdriver stopte daarom met exit1 na de al geslaagde pipe-run.
Die driverstatus en beide rauwe uitkomsten blijven intact; alleen de pipe-run
wordt geaccepteerd. Er is geen UNIX-native-succesclaim in deze omgeving.

Reproduceer de geverifieerde route met de centrale omgevingsconfiguratie:

```sh
python3 -B tools/build.py --kernel-feature graphics-g0
python3 -B tools/run_graphics.py --qmp-transport pipe --run-root /tmp/nieuw-os-g0-reproduce --vga-rom /tmp/nieuw-os-par-tools/qemu/usr/share/seabios/vgabios-stdvga.bin --vga-rom-sha256 cc2f735f19b6318922ac3de9506dee498f149a6b75534f7e5c176d4441a7fa4a
```

G0 blijft een eigen grafische RAM-invoerproef, zonder documentbewaring,
compositor of fysiekehardwareclaim. De score krijgt hiervoor geen nieuw
volledig productcriterium.
