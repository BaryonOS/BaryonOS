# Releaseprofielen

De profielen zijn auditindeling; zij wijzigen de strengere conjunctieve
acceptatie uit `docs/FULL-TEST-RELEASE.md`. Niets hieronder verklaart een
release gereed.

| Profiel | Doel en minimum | Niet beloofd |
|---|---|---|
| `TECHNICAL_PREVIEW` | Exact reproduceerbare x86_64/UEFI/QEMU-start; ontwikkelaarstools; kernel-/serviceproeven; synthetische wegwerpdata; expliciete beperkingen en recoveryprocedure. | Daily use, accounts, brede drivers, algemene internettoegang, fysieke hardware, security of compatibiliteitsclaims. |
| `EXPERIMENTAL_DESKTOP` | Native compositor/sessie, input, Materiaal/editor en enkele nuttige apps op VM of één exacte referentieconfiguratie; persistence/cold recovery; duidelijke fouten; basis keyboard/focus. | Consumer-installatie, alle media-/webformaten, hardwarebreedte, langdurige betrouwbaarheid, volledige accessibility of securityaudit. |
| `DAILY_DRIVER_BETA` | Installer/recovery, echte accounts/lock, persistente versleutelde opslag en onafhankelijke backup, browser/netwerk/updates, nuttige apps, accessibility/i18n, apprechten, energie-/hardwarematrix en intensieve fout-/duurtests. | Universele hardware-/appcompatibiliteit of afwezigheid van resterende risico's. Iedere ondersteunde configuratie is benoemd. |
| `CONSUMER_RELEASE` | Begrijpelijke setup/migratie, ondersteunde hardware, toegankelijke basiservaring, getest dagelijks gebruik, signed/recoverable updates, support/securityresponse, licentie/SBOM, documentatie, onderhoudseigenaarschap en onafhankelijk gevalideerde releasebundle. | Absolute veiligheid, elk apparaat, volledige pariteit met andere OS'en of “quantumproof”. |

## Eerste vereiste profiel per domein

- Boot, kernisolatie, IPC: `TECHNICAL_PREVIEW`.
- Native desktop, Materiaal, input, beperkte opslag: `EXPERIMENTAL_DESKTOP`.
- Authenticated user data, app store/package lifecycle, netwerk/browser,
  betrouwbare backup, accessibility, power, NAS en lokale AI: minimaal
  `DAILY_DRIVER_BETA`.
- Installer/migratie, fysieke supportmatrix, incident-/supportproces en
  onafhankelijke releasevalidatie: `CONSUMER_RELEASE`.

`matrix.json` geeft per feature de eerste profielpoort. Voor fysieke
energie-/driverfuncties blijft werkelijk hardwarebewijs vereist; een VM-profile
wordt niet als hardware-support geherinterpreteerd.
