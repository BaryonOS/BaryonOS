# Compatibiliteit en migratie

Geen compatibiliteitslaag of volledige migratieroute is als product aangetoond.
Er is geen geaccepteerde import-/exportmatrix voor documenten, e-mail,
kalender, contacten, browserdata, passwords, settings, Wi-Fi-profielen of
cloudproviders. NTFS/exFAT/ext4, SMB/NFS/WebDAV en archief-/mediaformaten
blijven concrete keuzes en testwerk, geen algemene claims.

## Realistische volgorde

1. Behoud exporteerbaarheid en maak lokaal verifieerbare backups.
2. Kies een kleine, gebruikersrelevante formaatset (bijv. plain text,
   Markdown, PNG/JPEG, PDF en expliciet gekozen office-interchange features).
3. Definieer per formaat import, export, metadata, fidelity/loss reporting,
   malformed/oversize behavior en license/dependency.
4. Test fixtures uit echte documenten, niet alleen synthetische headers;
   behandel round-trip als feature-by-feature.
5. Evalueer filesystem mounting, SMB/NFS/WebDAV en Windows/Linux app
   compatibility afzonderlijk op isolatie, update ownership, licensing,
   performance en data-loss/recovery.
6. Games, anti-cheat, professional apps en browser extensions zijn geen
   default beta claim zonder concrete user task and support owner.

Een eigen browserengine is in deze audit **niet aanbevolen** als eerste
compatibiliteitsroute; onderhoud en beveiligingspatching vormen een blijvende
last. Dit is een auditadvies, geen aangenomen projectbesluit. De
`FULL-TEST-RELEASE.md` vraagt eerst een Chromium/Gecko feasibility comparison.
