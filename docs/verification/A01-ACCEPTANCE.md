# A01 — gewone native SDK-app met gedocumenteerde capabilitygrenzen

## Geaccepteerde eigenschap (28 oktober 2026)

De bestaande publiek beschreven Nieuw OS SDK bouwt een afzonderlijk applicatie-ELF voor de eigen `no_std`-kernel/ABI. De app `catalog-read` start als een eigen ring-3-proces naast een afzonderlijke catalogusdienst en gebruikt alleen expliciet verleende SEND/RECV-endpoints. De gedocumenteerde SDK-API, getypeerde handles, ABI-versiecontrole en foutcontracten staan in `sdk/README.md`, `sdk/runtime/README.md`, `sdk/catalog-client/README.md` en `sdk/examples/catalog-read/README.md`.

- PR: [#31](https://github.com/simonvanieperen/nieuw-os/pull/31)
- Geteste PR-head: `0d68762e1f339c8b6d32440376282f1615dd2ae9`.
- Op `main` geïntegreerd via squash-commit: `a4b71ccf598353543b1c75d63b056e5d5933fa06`.

## Uitgevoerde brongebonden native proeven

- [CI-run 40759843033](https://github.com/simonvanieperen/nieuw-os/actions/runs/40759843033): **geslaagd**; onder meer SDK-build, `host-quality` en `qemu-smoke`. De QEMU-gate bouwt en start afzonderlijk een gewone SDK-app en gebruikersdienst.
- Normaal geval: LIST + GET voor één echt object uit de userspace-dienst, app exitcode 0, blijvende dienst, correcte replygegevens en volledige cleanup.
- Weigeringsgeval: zonder app-endpoints start dezelfde app, ontvangt geen toegang, eindigt met exitcode 1; de dienst ontvangt geen verzoek en resources worden vrijgegeven.
- Manifestcontrole: positieve SEND/RECV-rechten per proces, negatieve rechten en `BLOCK_READ`-afwijzing, exact aantal live handles, plus afwijzing van verboden RECV-overdracht bij de grant boundary.
- [Resource-pressure-run 40759843104](https://github.com/simonvanieperen/nieuw-os/actions/runs/40759843104): **geslaagd** op dezelfde PR-head.
- CodeRabbit-reviewbevindingen zijn daadwerkelijk in de branch hersteld; oude formele reviews zijn met vastgelegde motivering afgesloten nadat reviewthreads opgelost waren.

Reproductie in de vastgelegde Linux/QEMU-toolchain:

```sh
python3 -B sdk/examples/catalog-read/build.py
python3 -B tools/build.py --kernel-feature native-sdk-app
python3 -B tools/run_sdk_app.py --case normal
python3 -B tools/build.py --kernel-feature test-sdk-app-denied
python3 -B tools/run_sdk_app.py --case denied
```

## Grenzen

A01 bewijst de afgebakende SDK-app-/catalogusketen en de geteste ABI/capabilities, geen algemene procesinstaller, grafische desktop, willekeurige externe apps, multiplatform hardware of formeel geverifieerde kernel. Installatie/update en bredere integratie blijven A02/A03-open. De reviews zijn AI-reviews, geen onafhankelijke menselijke beveiligingsaudit. Beoordeling volgens bestaande rubric v1: **A01 = 2 acceptatiepunten**; geen dubbelpunt voor K04 of een hostprototype.
