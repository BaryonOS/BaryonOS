# G08 — onafhankelijke review van het quantumjob-requestformaat

## Uitkomst en bron

Geen blokkerende bevinding binnen het afgesproken bereik: de requestcodec op
commit `148db2dba1e389afb33fb375e5b42466e2a76fd2` voert het centrale QJ01-contract
van precies 256 bytes uit. De review gebruikte bestanden uit dat onveranderlijke
Git-object, niet de ondertussen verder ontwikkelde I09-werkboom. De implementatie
is niet gewijzigd.

`checks.json` bindt de gelezen bronnen, gebruikte commando's en ongewijzigde
stdout/stderr-bestanden met SHA-256. Deze controle betreft uitsluitend de
requestcodec; er is geen nieuwe VM of simulator gestart en geen werkende native
jobservice aangetoond.

## Gecontroleerde grenzen

| Grens | Bevinding |
| --- | --- |
| Lengte en header | Alleen exact 256 bytes; QJ01, versie 1 en de vier bekende operaties. De lengte wordt vóór indexeren gecontroleerd. |
| Correlatie | Job-ID en request-ID moeten beide niet nul zijn. Ze geven geen procesidentiteit, capability of bevoegdheid. |
| Submit | Qubits 1–6, gates 0–64, shots 1–4096 en gevraagd werkbudget 1–1.048.576; narrowing en arraytoegang volgen pas na de grenscontrole. |
| Gates | Alleen H/X/Z/S/CX; target ligt binnen het qubitbereik. Alleen CX heeft een control, binnen bereik en verschillend van target; bij de andere gates is control nul. |
| Ongebruikte bytes | Alle gereserveerde bytes, ongebruikte gates en operatievreemde velden moeten nul zijn. Cancel/Status bevatten alleen de header en IDs; ResultPage voegt uitsluitend een pagina 0–3 toe. |
| Canonieke representatie | De encoder begint met een volledig nulbuffer. Geen genegeerde metadata of alternatieve geaccepteerde representatie gevonden; alle 256 posities worden gecontroleerd of hebben expliciete betekenis. |
| Rust-interface | Private gevalideerde velden, vaste arrays, geen allocator of unsafe; een los aangemaakte gate krijgt bij opname in Submit alsnog bereikcontrole. |

Het werkbudget is een begrensd **verzoek**. De codec belooft niet dat een circuit
binnen dat budget voltooit en voert zelf geen quota, annulering of scheduling uit.
Dat onderscheid staat ook in het gereviewde contract. Een klein maar geldig budget
bij een groot circuit is daarom geen codecdefect.

## Aanvullend gericht bewijs

`independent_codec.rs` bevat twee review-eigenschappen boven de echte immutable
library:

1. Zes geldige fixtures — minimale/maximale/één-gate Submit, Cancel, Status en
   ResultPage — krijgen op iedere bytepositie alle 256 bytewaarden. Van
   **393.216** probes werden **34.277** canoniek geaccepteerd en **358.939**
   afgewezen. Elke geaccepteerde invoer moest na decode→encode bytegelijk zijn;
   er was geen panic of mislukte controle.
2. Alle **390** combinaties van 1–6 qubits en 0–64 gates controleren de telling,
   ongebruikte nulbytes en exacte roundtrip.

Beide eigenschappen slagen. De bestaande I08-bundel is afzonderlijk gelezen:
haar bronhashes passen bij dezelfde commit en haar hosttests, lint- en native
librarycompilecommando's rapporteren succes. Die bestaande compilecontrole
blijft librarybewijs; zij bewijst geen OS-start of native jobuitvoering.

## Open integratiewerk

De toekomstige jobservice moet de door de kernel geauthenticeerde IPC-afzender
aan eigendom en lifecycle van jobs koppelen, en het gevraagde budget en annulering
daadwerkelijk afdwingen. Het wireformaat bevat bewust geen actor- of rechtenveld
waarmee een aanvrager zichzelf autoriteit kan geven. Zodra deze nieuwe crate in
een OS-image wordt gelinkt, moet zij ook in de toepasselijke bronfingerprint
worden opgenomen. Deze punten zijn volgende implementatiestappen, geen claims
dat de requestcodec ze al afhandelt.

De bestaande roadmapcriteria en gewichten zijn niet gewijzigd.
