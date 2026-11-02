# OS-completeness-audit

## Conclusie

Op `main` staat een zelfstandig x86_64/UEFI-project met een eigen Rust-kernel,
QEMU-tests en afzonderlijke native diensten. Er zijn betekenisvolle native
deelbewijzen voor geheugenisolatie, capability-IPC, opslag, updates,
netwerktransport en een grafische sessie. De actuele productstatus is echter
**onderzoeks-/technisch prototype; geen dagelijks bruikbare desktoprelease**.
De integrale releasepoort blijft geblokkeerd: `docs/FULL-TEST-RELEASE.md`
verklaart alle 62 verplichte productrijen open totdat ze op de finale image
zijn afgetekend. VM-, host- en branchbewijs is niet onderling uitwisselbaar.

Deze audit verandert geen productcode, productcriteria of roadmapgewichten.
De matrix groepeert verwante voorzieningen; bestaande `FR-*`-rijen blijven de
bindende gedetailleerde release-acceptatie. De aantallen in het dashboard zijn
featuregroepen, geen resterend werk of tijdspercentage.

## Bronstand

Auditbasis: `simonvanieperen-cautious-happiness`, HEAD
`380111b68688c981eec6abb8fab29174a29f5601`, exact gelijk aan `origin/main` op
2 november 2026. De werkboom was schoon bij aanvang. Alle 15 opgevraagde
exacte-head workflows zijn inmiddels voltooid met succes, waaronder
`Native window session and Windows QEMU bundle`. Dit is CI-bewijs, geen
aftekening van de 62 volledige releasecriteria of fysieke/human gates. Zie
[bronstand](source-baseline.md).

Open PR's en losse worktrees zijn afzonderlijke ontwikkelresultaten en niet
opgeteld bij main. De belangrijkste voorbeelden zijn onderzoeks-PR #131,
opslag-PR's #132/#133, het gestapelde desktop/Materiaal-werk #128–#130 en
NAS-/sessieproeven #119/#121. [#127](https://github.com/simonvanieperen/nieuw-os/pull/127)
is al in HEAD opgenomen; de exacte-head CI is groen, maar de volledige
releaseacceptatie blijft open.

## Dashboard

De machineleesbare aantallen, per status, bewijsniveau, prioriteit en profiel,
staan in [`verification.json`](verification.json) en worden uit
[`matrix.json`](matrix.json) berekend. De auditgroep is bewust geen extra
score naast de vaste 41/100-projectmeter.

Belangrijkste P0/P1-blokkers:

1. Eén geïntegreerd release-image met volledige native interactie, opslag,
   app-levenscyclus, updates en herstel.
2. Echte authenticatie/key-unlock, versleutelde persistente gebruikersopslag
   en betrouwbare sleutelherstel-/backupgrenzen.
3. Native internet, DNS/TLS, peer-/bestemmingsrechten en geïntegreerde NAS;
   geen hostproef of browsermock als vervanging.
4. Toegankelijkheid, echte accounts, installer, bruikbare apps en
   herstelbaarheid op een benoemd fysiek profiel.
5. Onafhankelijke menselijke security- en gebruikerstests en complete
   release-/onderhoudsvalidatie.

## Leeswijzer

- [Completenessmatrix](completeness-matrix.md) en [evidence-register](evidence-register.md):
  status en herleidbaar bewijs.
- [Koppeling bindende releasecriteria](release-criteria-map.md): alle 62
  `FR-*`-rijen en de relevante auditgroepen; geen rij is afgetekend.
- [Afhankelijkheden](dependency-map.md), [releaseprofielen](release-profiles.md)
  en [prioriteiten](priorities.md): uitvoeringsvolgorde.
- [Productgaten](product-gaps.md), [technische gaten](technical-gaps.md),
  [securitygaten](security-gaps.md), [toegankelijkheid](accessibility-gaps.md),
  [compatibiliteit](compatibility-gaps.md) en [visuele gaten](visual-design-gaps.md).
- [Risicoregister](risk-register.md) en [aanbevolen route](recommended-roadmap.md).
- [Matrixschema](matrix.schema.json) en [validatie-uitvoer](verification.json).

## Bewijsgrens en bronnen

De classificatie geldt voor de auditbasis, tenzij een rapport expliciet een
andere commit noemt. Productstatus is onderscheiden van broncode, documentatie,
hosttests, VM-runs, fysieke hardware, menselijk onderzoek, onafhankelijke
review en releasevalidatie. Geen fysieke hardware-, menselijke
gebruikerstest-, onafhankelijke externe audit- of volledige releasevalidatie
is in deze audit uitgevoerd.

Externe bronnen zijn alleen gebruikt als scopecontrole:

- W3C, *Web Content Accessibility Guidelines (WCAG) 2.2*, W3C Recommendation,
  5 oktober 2023, <https://www.w3.org/TR/WCAG22/>. De richtlijn betreft
  webcontent; zij bewijst geen native toegankelijkheid of wettelijke
  conformiteit van Nieuw OS.
- E. Rescorla / IETF, *The Transport Layer Security (TLS) Protocol Version
  1.3*, RFC 8446, augustus 2018,
  <https://www.rfc-editor.org/rfc/rfc8446>. Dit is protocolcontext, geen
  bewijs van een veilige of complete Nieuw-OS-transportstack.
- Microsoft Support, *Backup and restore with File History*, Windows 11/10,
  bijgewerkt 14 oktober 2025,
  <https://support.microsoft.com/en-au/windows/experience/backup-recovery/backup-and-restore-with-file-history>.
  Dit bevestigt dat bestandsversies en herstel relevante
  desktopgebruikstaken zijn; het is geen gelijkwaardigheidsmaatstaf.
- Microsoft Support, *Use a screen reader to back up and restore data on
  Windows*, bijgewerkt 24 augustus 2026,
  <https://support.microsoft.com/en-us/accessibility/windows/use-a-screen-reader-to-back-up-and-restore-data-on-windows>.
  Dit ondersteunt de scopekoppeling tussen toegankelijkheid en herstel.

De UEFI Forum-URL kon tijdens de audit niet worden opgehaald (HTTP 403) en is
daarom niet als geverifieerde externe bron aangehaald. Normatieve
firmware-/hardwaredetails moeten vóór implementatie tegen de actuele primaire
specificatie worden gecontroleerd.
