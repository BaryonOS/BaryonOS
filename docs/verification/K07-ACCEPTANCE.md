# K07 — resource-uitputting en herstel

## Geaccepteerde eigenschap

K07 controleert op dezelfde bron dat geheugen-, CPU-, IPC- en handle-uitputting
per proces begrensd blijven en dat de bruikbare kerneltoestand na de proef
volledig wordt hersteld.

Uitvoeringshead: `5e51f9e08b0d1ebdb8cd2e64c35ae7937888a8ea`.
Geïntegreerde bron: `357124d05c85a1d77cc1ac01fcc53f9c960b9898`.

## Werkelijk uitgevoerde controles

- GitHub Actions **Resource pressure acceptance**, run `40745589874`: geslaagd.
- GitHub Actions **CI**, run `40745589981`: geslaagd.
- Een eerdere identieke gespecialiseerde run `40744934872` en brede CI
  `40744934344` slaagden eveneens.

De gespecialiseerde poort voert twee echte QEMU-profielen uit. De M4-quota-case
controleert handle-slots, delegatiediepte en IPC-queuecapaciteit, inclusief
quota/EAGAIN, hergebruik, survivorprogressie en cleanup. De native qsim-case
controleert twee geïsoleerde processen met private 64-KiB-heaps, echte
`ResourceExhausted`, checkpoint-/CPU-druk, deadline/cancel, timerinterrupts en
exacte framecleanup.

De poort eindigt alleen groen wanneer de bronfingerprint tijdens beide profielen
gelijk blijft. De runtimecode voor deze eigenschappen bestond al; PR #33 voegde
de gezamenlijke actuele acceptatiepoort toe en veranderde geen kernelbeleid.

## Grenzen

Dit is bewijs voor het huidige single-CPU QEMU-profiel. Het is geen SMP-,
fysieke-hardware-, energie- of algemene denial-of-servicegarantie buiten de
geteste quota en resourceklassen.
