# Metadata — uitsluitend historisch bewijs

Zie de [ongewijzigde oorspronkelijke toelichting](README.md.historical.txt)
en `metadata-evidence.json`. De actuele baseline-replay herhaalt geen
metadataproef. De oorspronkelijke conclusie blijft: geen volledige
metadata-roundtrip aangetoond. [Actuele controle-/replayroute](../../COMMANDS.md).

<!-- coderabbit-ai-generated:v1 -->

<!-- coderabbit-ai-generated-section:v1:07660c25eebe222989c7d866635acd5a2691771bc2dc9e5129e24f09e6e83429 -->

De oorspronkelijke metadata-probe vergeleek de SHA-256 van reguliere bestanden,
grootte en symlinkdoelen na restore; er is geen afzonderlijke volledige
bytesgewijze vergelijking uitgevoerd. De historische toelichting beschrijft
dat op één plaats onjuist.

[Primaire bronnen en de koppeling naar dit meetresultaat](../../PRIMARY-SOURCES.md).
De handleidingen onderbouwen de opties, niet een universele conclusie dat
Btrfs metadata verliest.

<!-- /coderabbit-ai-generated:v1 -->
