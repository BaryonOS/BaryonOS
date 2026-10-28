# R06 — gerichte M6-transportreview

Status na gerichte delta-herlezing: beide concrete bevindingen op bron gesloten;
geen resterende blocker binnen deze twee fixes. Geen sourcewrites, builds of
gastuitvoering door reviewer. Alleen dit ignored verslag geschreven. Basis
M6a448974b; HEAD tijdens oorspronkelijke bronlezing
`0d33990001c2c974a19a6b265a410f210d0a641d`.

## Scope en identificatie

Gelezen: kernel/AGENTS.md, drivers/AGENTS.md, M6A-WRITE-RECOVERY v1,
drivers/virtio-blk/src/lib.rs, kernel/src/m2/m3/m6.rs, relevante M3-dispatch en
cleanup, sector_input_plan512 in kernel-core/user.rs en aansluitende bytecopy.
Eigen pure virtio-core/capability-core-API niet opnieuw als onafhankelijke
reviewer geauditeerd. Focus uitsluitend publicatie/buffers, WRITE→FLUSH,
Unknown/ticket/rechten en cleanup/quarantaine.

SHA256 gelezen bron:

| Bestand | SHA256 |
|---|---|
| drivers/virtio-blk/src/lib.rs | 43E72D81986902B2ABC7393CA7338B427FED616E1F7F2D6A9666220DDD32CE55 |
| kernel/src/m2/m3/m6.rs | 4D27C0DF7FCD969FBD9B054EE9769DBC87FD1C3848F0E52A16AE9DD16E9B5CE5 |
| crates/kernel-core/src/user.rs | EC481BC119BBC4548851DEAA60D69AF36DBB29AAD9396A9BE1EF7D9890E9945B |

## Oorspronkelijke bevindingen, hieronder op bron gesloten

1. **Operatierecht ná ticketcheck, afwijkend van expliciete foutvolgorde.**
   m6.rs:453–458: poll vergelijkt binding/ticket vóór check_block met het recht
   voor de opgeslagen operatie. Tegenvoorbeeld: actuele READ-only cap naar
   hetzelfde scopeobject/PID, pending WRITE, poll(ticket+1,badptr,320).
   Huidige resultaat EINVAL; v1 vereist EACCESS voor ontbrekend WRITE vóór
   ticketvalidatie. Advies: gebruik uitsluitend de intern opgeslagen operatie
   om het vereiste recht vóór binding/ticketvergelijking te controleren, nog
   altijd vóór outputplan. Geen aangetoonde rechtenbypass, wel contractfout.

2. **Overlappende exclusieve Runtime-referenties bij eindcleanup.**
   m6.rs:1061 leent r als &'static mut Runtime. De loop1171–1174 roept
   s.cleanup aan; M3-cleanup roept m5::detach, dat op m6.rs:627 opnieuw runtime()
   leent. Het oorspronkelijke r wordt daarna nog gebruikt voor r.pool op
   1190–1194. Trigger: normaal eindpad met een levend proces. IF0/single-CPU
   maakt gelijktijdige exclusieve Rust-referenties niet geldig. Advies: kopieer
   pool/benodigde observerwaarden en beëindig de eerste borrow vóór de loop;
   gebruik na cleanup geen oud r. Concrete unsafe-invariantfout, geen door de
   reviewer geobserveerde QEMU-corruptie.

## Positieve bronwaarnemingen

- WRITE-input krijgt een apart exact512-plan; IPC/outputmaximum320 blijft.
  Pointer blijft u64, checked end en numerieke userrange controleren overflow/
  canonicaliteit; ieder deel controleert effectieve USER en Owner::User. RO-input
  is toegestaan. Bytewise RO/NX-scratchreads nemen één volledige snapshot vóór
  Policy.prepare en driverpublicatie; geen userpointer blijft over I/O bestaan.
- Vijf Owner::Kernel-pages blijven gepind. Driver prepare valideert exact de
  operationele chain vóór poolwrites. Policy.publish staat direct vóór
  publish_prepared; vrijgave gaat via fence→avail.idx→fence→notify. Ontbrekende
  notify wordt niet als bewijs van ontbrekende exposure gebruikt.
- READ en WRITE verschillen in richting van de bounce-descriptor; FLUSH heeft
  een echte type4-header/sector0 en header→status-chain zonder datapayload.
  Used-id wordt niet als adres gebruikt; idx/head/len/status komen door de pure
  decoder vóór enige bouncecopy. Alleen succesvolle READ kopieert bouncebytes.
- Serialized Policy blokkeert een volgende operatie tot de vorige completion
  is waargenomen en opgehaald. De observer accepteert PUT-ACK uitsluitend na
  echte WRITE- en FLUSH-completions; de cliëntreceive wordt afzonderlijk gezien.
- Mutator-timeout, completionfout en protocolfout worden Unknown. Poll valideert
  alle320 outputbytes vóór consumptie. Resetack bewaart terminale mutatorstatus;
  consume tijdens reset/quarantaine geeft de pool niet vrij. Owner-exit wist
  fout/policyresultaat en vraagt waar nodig reset; quarantinepad start niets.
- Herinitialisatie/zero gebeurt pas na waargenomen resetstatus0 én BME-off.
  Ontbrekende ack houdt pool offline/quarantaine. DRIVER_OK-fout na BME-enable
  vraagt reset en wist BME; ook deze initialisatiefout quarantinet de pool.

## Bewijsgrenzen

Tweede AI-bronreview, geen onafhankelijke menselijke audit. Reviewer heeft geen
builds, guestlogs, echte WRITE/FLUSH, UD2-cuts, QEMU-processkills of koude recovery
uitgevoerd. Root/builder moeten de samengevoegde versie daadwerkelijk testen.
Completion-snapshotinjectie is foutafhandeling na echte completion, geen bewijs
van een kwaadwillig device of DMA-containment. Software-epochs zijn geen virtio
hardware-nonce. Resetack is quiescence, geen rollback/durability. Alleen latere
FLUSH geeft de gekozen afgebakende backendgarantie; QEMU-abort bewijst geen
fysieke stroomuitval of atomische512B-write.

## Gerichte delta-herlezing

Uitsluitend de twee gemelde fixes en direct omliggende context herlezen.
HEAD was nog `0d33990001c2c974a19a6b265a410f210d0a641d`; de globale werkboom/
sourcefingerprint was volgens root nog niet bevroren. De actuele gelezen
kernel/src/m2/m3/m6.rs heeft SHA256
`DCA909EF3051F2A9D7E79ACE4011C5054AFC3F2883F4D4C6463D2F0C2A37EBD1`.
Dit is een identificatie van de twee bronfixes, geen finale releasefingerprint.

1. **Pollrecht vóór ticket — gesloten op bron.** m6.rs:453–455 autoriseert nu
   het recht uit de intern opgeslagen Request.operation vóór de vergelijking
   van binding/ticket op456–458 en vóór outputplan. De genoemde READ-only-cap/
   pending-WRITE/verkeerd-ticket-combinatie bereikt daarmee EACCESS vóór EINVAL.

2. **Runtime-borrow over cleanup — gesloten op bron.** m6.rs:1173 kopieert de
   DmaPool (Copy) vóór de cleanup-loop. Het oorspronkelijke r wordt daarna niet
   meer gebruikt; controles op1192–1200 gebruiken uitsluitend de lokale pool.
   Daardoor kan de borrow vóór de nested cleanup→detach-callback eindigen.
   Geen nieuwe Runtime-borrow of oud r wordt over die callback vastgehouden.

Root meldde normale M6-guest en device-rights raw33/oracletrue als diagnostiek.
Reviewer heeft deze gastuitvoering/logs niet uitgevoerd of zelfstandig
gecontroleerd. Er wordt geen specifieke nieuwe gecombineerde-foutproef of
dynamische Rust-aliasingchecker geclaimd. De sluiting berust op herlezen bron;
de finale globale fingerprint en volledige samengevoegde gate blijven apart.
De overige hierboven genoemde TCB-, hardware- en crashbewijsgrenzen blijven
ongewijzigd. Dit is geen algemene veiligheidsvrijgave of menselijke audit.

## Laatste gerichte journal-borrowcontrole

Alleen de door builder gemelde laatste wijziging herlezen: m6.rs:550 gebruikt
nu `let current = runtime().latest.unwrap()` in plaats van een as_ref-borrow.
journal::State is Clone+Copy (services/block/src/journal.rs:8–9), waardoor
current een zelfstandige lokale kopie is. De verwijzing `&current` op552 wijst
dus niet meer in Runtime wanneer dezelfde aanroep runtime().put_value leent.
De lokale generation blijft ook bij de volgende decode onafhankelijk van de
runtime-borrow. **Dit concrete aanvullende aliaspad is op bron gesloten.**

Actuele gelezen m6.rs SHA256:
`31323BF4BBCD7729AB044B030732E02058FB1C51EE2CC57961AA61FB0A5AEAD3`.
HEAD bleef `0d33990001c2c974a19a6b265a410f210d0a641d`; geen finale globale
fingerprint geclaimd. Geen algemene herreview, builds of gastuitvoering gedaan.
Alleen dit ignored rapport bijgewerkt. Rootmeldingen over extra host-/QEMU- en
recoverydiagnostiek zijn geen door reviewer zelfstandig uitgevoerd bewijs;
de finale samengevoegde gate blijft afzonderlijk vereist.
