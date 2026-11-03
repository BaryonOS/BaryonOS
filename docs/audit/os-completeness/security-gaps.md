# Securitygaten

## Fundament dat al aanwezig is

De repository bevat expliciete kernel-/capability-/usercopy-contracten en
beperkte native negatieve proeven. B01 entropy/keyring en B03 signed update
hebben scoped geaccepteerd bewijs in rubric v1. Dat bewijs zegt niets over
ongecontroleerde productflows buiten die profielen.

## P0-open

- **B02:** native persistent encrypted storage met geauthenticeerde inhoud en
  gevoelige metadata, koude herstelproef, replay/rollbackgrenzen en sleutelverlies.
- **B04:** boot-/distributieketen en release trust-root/key rotation; bouw-/
  dependency herkomst onafhankelijk controleren.
- **B05:** onafhankelijke menselijke security review voor kernel, unsafe,
  DMA, IPC/pointers, parsers, keys, package/update en service boundaries.
- **B06:** LP-01–31 op eigen OS: app-/peer-/bestemmingsrechten, revoke, geen
  stille fallback, model/downloadscheiding, provider ciphertext-only, admin
  NAS-revoke en geen telemetrie.
- Native authentication, key-unlock, lockscreen isolation en credential
  recovery.
- Native TLS identity/downgrade/fallback tests en security patch/incident
  ownership.

## Reviewstopregels

Blokkeer onbetrouwbare pointer/lengte, capability privilege escalation,
uncontrolled DMA, secrets in logs, fail-open authentication/update,
onverifieerbare package execution en success-shaped I/O errors. Deze audit
heeft geen beveiligingsreview van code uitgevoerd; de genoemde punten zijn
productgaten, geen verklaring dat een concrete vulnerability is gevonden.
