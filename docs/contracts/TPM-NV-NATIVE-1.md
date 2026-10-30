# B02-NV-NATIVE-1 — onafhankelijke NV-counterbouwsteen

30 oktober 2026, basis `c532ba88fbf0f53b960e9dbf77421cee29d97096`,
branch `codex/b02-nv-native-20261030`. **Implementatieproef, acceptatie open.**
Geen B02-punten of productieankerclaim. Centrale voortgang is eigendom van
de teamleider; dit contract is geen tweede roadmap.

Doel: met de bestaande eigen CRB-MMIO-adapter en codec echt een TPM2-counter
aanmaken, twee increments waarnemen en die toestand lezen nadat zowel QEMU
als swtpm volledig gestopt en opnieuw gestart zijn. Een verkeerd wachtwoord
en ontbrekende index moeten canoniek worden geweigerd; daarna blijft een
correcte read mogelijk, zonder stil opnieuw uitgevoerde mutatie.

Schrijfclaim, afgestemd met Build en vooraf gemeld aan release:
`kernel/{Cargo.toml,src/m2/m3/tpm_fixture.rs}`, uitsluitend featureselectie
in `tools/build.py`, `tools/run_tpm.py` beheerde backendlifecycle, eigen
`tools/test_tpm_nv.py`, `tools/tests/test_tpm_nv.py` en dit contract/bewijs.
Geen gewone app, TPM-capability, KEY1-wire, publisherroot of keyringmutatie.
De read-only swtpm-toolprefix is van de buildpeer; alle veranderlijke state
blijft onder de eigen worktree `build/tpm/`, mode0700, controlsocket0600.
Geen fysieke TPM, hostdevice, productiekey of gedeelde TPM-state.

Invarianten:

- Alleen expliciete opt-in testfeatures voeren NV-commands uit; bestaande
  `native-tpm-crb`/absent- en gewone profielen veranderen niet.
- Vaste fixture-index `0x01804e56`, SHA256-counterpublicarea zoals de
  bestaande codec; geen willekeurige raw-commands of gekozen deviceadressen.
- Expliciet leeg owner-auth is uitsluitend een nieuwe lege swtpm-fixture;
  index-auth is de zichtbare openbare string `PUBLIC-TPM-NV-FIXTURE-ONLY`.
  Dit is geen productie-authbeleid of geheime sleutel.
- Iedere poll gebruikt echte PIT-deadlines; Unknown/quarantaine is fataal,
  zonder automatische Define/Increment-retry. Alleen exact verwachte
  canonieke TPM-fout met effect Completed geldt als negatieve proef.
- Een nieuwe swtpm-child krijgt een nieuw privé-controllog/socketdirectory.
  Alleen een expliciet aangewezen reeds bestaande eigen state mag worden
  hergebruikt; standaard blijft iedere backend vers. Geen `,lock`-optie.
- De provisionrun noteert de eerste gelezen counter als baseline en vereist
  exact +1 na de volgende increment; neem geen ongespecificeerde beginwaarde
  aan. Cold-read en negatieve runs moeten exact die laatste waarde behouden.
- Host vergelijkt afzonderlijke echte guestwaarnemingen, bron/artifacthashes
  en backendlifecycle; geen hosttool voert de NV-commands namens het OS uit.

Acceptatiecommands: bestaande driver-hosttests, gerichte Python-backend/
NV-harnastests, targetbuild/fmt, `tools/test_tpm_nv.py` voor echte provision,
cold-read, wrong-auth en missing-index, plus bestaande normal/absentproeven.
Bewijs onderscheidt hostmodel, crosscompile en daadwerkelijke native run.

Niet-doelen/open: counter→volume/master-lineagebinding, geauthenticeerd
key-statejournal, write-ahead per GENERATE/REVOKE, powercut-reconciliatie,
productieprovisioning/auth en onafhankelijke menselijke/normatieve audit.
De bestaande TPM2-CRB-broncontrole/grenzen blijven gelden; geen nieuwe
cryptografie of TCG-complianceclaim. Terugrol van zowel disk als swtpm-state
schendt de onafhankelijke-anker-aanname. Ook een geslaagde koude counterproef
bewijst daarom niet de volledige B02-rollbackbescherming.
