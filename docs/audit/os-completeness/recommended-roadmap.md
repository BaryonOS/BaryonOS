# Aanbevolen uitvoeringsvolgorde

Deze audit schrijft geen tweede roadmap voor. Koppel het werk aan de bestaande
`docs/WORK-QUEUE.md`, `docs/ROADMAP.md`, `docs/FULL-TEST-RELEASE.md` en
`docs/PROJECT-STATE.md`; behoud hun ID's en vaste voortgangsrubric.

## Fase 0 — bewijs en bronhygiëne

- Archiveer de volledige succesvolle exact-head CI op `380111b` zonder oudere
  PR-resultaten te heretiketteren; herhaal relevante native regressies zodra
  productcode wijzigt.
- Houd open PR #114/#119/#121/#128–#133 gescheiden; integreer alleen via de
  bestaande integrator/branchregels.
- Herhaal de relevante native window/network/storage regressies op één
  afgesproken bron.

**Poort:** actuele branch, artifact hashes, alle verplichte workflows en
resultaten op dezelfde commit staan controleerbaar vast.

## Fase 1 — samenhangende kernel-, rechten- en gegevensbasis

- Integreer proces/service/storage/network capabilities met expliciete
  fault/Unknown/revoke semantiek.
- Werk volume admission, encrypted storage, keyring lifecycle en onafhankelijke
  cold backup/restore uit met negatieve tests.
- Koppel auth/session/lock tot werkelijke rechten; geen loginmock.
- Behoud signed updategate, voeg persistentie/recovery toe.

**Poort:** VM-FR native vertical slice: boot → account/sessie → Materiaal
save/reopen → koude recovery, plus denial/tamper/failure tests.

## Fase 2 — dagelijkse desktop op één vast profiel

- Voltooi compositor/window/input en Materiaal/editor op één geïntegreerde image.
- Voeg settings, nuttige apps en expliciete formats toe.
- Implementeer native accessibility tree/events en test toetsenbord/screenreader
  vóór de UI-set als klaar wordt gepresenteerd.
- Meet interactie/duur op vooraf vastgelegde criteria; geen QEMU-metingen als
  fysieke energie-/hardwareclaims.

**Poort:** reproduceerbare end-to-end taken en herstel, met echte gebruikers
en fout-/lege/loading-toestanden.

## Fase 3 — netwerk, browser, NAS en migratie

- Native NIC/config/DNS/TLS plus peer-/appbestemmingsrechten en revoke.
- Maintained browserengine evalueren; daarna native download/file portal.
- NAS pairing/sync/encrypted objects/device revocation/conflicts en admin
  quota/revoke/delete op twee native clients.
- Prioritaire import/export/filesystem/compatibility formats testen.

**Poort:** gecontroleerd verkeer bewijst identity/scope; offline herstel en
geen stille providerfallback; formaten leveren meetbare loss reporting.

## Fase 4 — onderscheidende productfuncties

- Valideer Materiaal-continuïteit via waargenomen user tasks.
- Local AI alleen na runtime/model/licence/resource experiment en native
  netwerkdeny; geselecteerde context en cancel.
- Bouw quantum Q1 apart; simulator, host prototype, OS port en physical QPU
  blijven verschillende bewijsniveaus.

**Poort:** user value en security meetbaar, zonder basisfuncties te vervangen.

## Fase 5 — beta/consumer release en onderhoud

- Benoemd fysiek profiel, install/recovery, updates, logs/diagnostics,
  licensing/SBOM/support/security-response en known issues.
- Onafhankelijke menselijke security- en accessibility-/usabilityreview.
- 100 cold starts/24-hour workload en complete FR-R01–R04 op definitieve
  hashes plus clean-environment reproduction, volgens bestaande opdracht.

**Poort:** alle verplichte acceptatie in `FULL-TEST-RELEASE.md`; anders blijft
de status tussentijdse VM-validatie en geen voltooide testrelease.
