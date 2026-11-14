# Naming and information architecture

Status: **proposal**, 13 November 2026, branch `claude/baryon-os-ui-ux-design-152adb`.
Evidence level: desk analysis by the design agent plus the audit findings in
[AUDIT-2026-11-13.md](AUDIT-2026-11-13.md). No participants were tested, so no
claim about user preference or comprehension is made. Task trials (U06) are
required before acceptance.

## 1. Binding decisions and what this document changes

- The product name **BaryonOS** stays.
- The names decision of 5 November 2026
  ([DESKTOP-SHELL.md](../implementation/DESKTOP-SHELL.md) §7: Horizon, Orbit,
  Zenit, Nebula, Signaal, Kern, Haven) **remains binding** until the project
  owner decides on proposal P-NAMES-1 (§8). This document does not override it.
- On this branch the new names are used in the concepts, the design system and
  the interactive prototype (one string table), so a rejection costs one edit.
  Native UI strings are **not** renamed on this branch.
- Code identifiers, service APIs, file formats and technical contracts
  (`Materiaal`, `nieuw_os_materiaal`, `Horizon` page, `Veld`, crate names) are
  never renamed by this proposal.

## 2. Method and criteria

Every user-facing name was assessed on: what it does and where users meet it;
clarity without explanation; discoverability (would a first-time user look for
this word?); memorability; pronunciation in Dutch, English, German and French;
consistency with the rest of the system; international usability and
translation; fit with the space and particle identity; and collision risk with
well-known products or with other meanings.

Principle adopted from ADR-058 (familiar first): **places and tools that users
must find get functional names**, translated per language, as on Windows,
macOS and common Linux desktops. **Brand names are reserved for the few things
that are BaryonOS-specific** and that a functional word does not cover well.
A brand name never carries meaning alone: its first appearance and its
accessible name always include a descriptor.

## 3. Assessment of current names

| Current | Function and where seen | Assessment | Recommendation |
|---|---|---|---|
| Nieuw OS | Former product name; Veld header, accounts footer, compositor strip | Outdated; contradicts the brand | **Replace** with BaryonOS (visible text only) |
| Horizon | Desktop page and wallpaper | Pleasant, but nobody searches for "Horizon" to find the desktop; tied to the retired planet wallpaper | **Replace** with Bureaublad / Desktop; Horizon may survive as a wallpaper name |
| Orbit | Dock | Short; identical in NL, EN, DE and FR; fits the identity; "dock" and "taakbalk" are not universally known in Dutch either | **Retain + supplement**: accessible name "Orbit, dock"; settings page "Orbit en dock" |
| Zenit | Launcher | Meaning (highest point) unrelated to launching; spelled Zenith in EN, DE and FR; well-known sports club and watch brand | **Replace** with Menu; button accessible name "BaryonOS-menu" |
| Signaal | Search | Misleading (suggests messages or notifications); collides with the Signal messenger, which is widely used in NL | **Replace** with Zoeken / Search |
| Nebula | Files | Suggests a vague cloud, which contradicts the local-first promise; several products use the name | **Replace** with Bestanden / Files |
| Materiaal, Mijn materiaal | Object model; Veld window title | Accurate model name but not a word users look for; on screen it competes with Nebula for the same app (F6) | **Retain as internal model name**; user-facing words are Bestanden, Versies, Toegang and Opslag |
| Kern | Settings | Reads as kernel or core; sounds risky to change; also "kern" as in nuclear | **Replace** with Instellingen / Settings |
| Haven | App store | NL harbour, EN safe place: fits a curated store where installing grants nothing; easy to say; moderate collision risk with other apps named Haven | **Retain + supplement**: "Haven — apps ontdekken en installeren"; fallback name "Apps" |
| Werkruimte | Veld header label | Has no action and no meaning for the user in a single-workspace system | **Remove** from the header; reuse "Werkruimte 1, 2…" only when multiple workspaces exist |
| Berekening / Rekenmachine | Calculator title versus dock label | Same app, two names | **Replace** both with Rekenmachine / Calculator |
| Notities | Notes app | Clear and functional | **Retain** |
| Assistent op dit apparaat | Local assistant | Functional, honest about where it runs; no persona name needed | **Retain** |
| Webvenster | A third-party web app in the fixtures | Not a BaryonOS app; keep as sample data | Unchanged |

## 4. Recommended name system

| Element | Dutch | English | Notes |
|---|---|---|---|
| Product | BaryonOS | BaryonOS | Never translated |
| Desktop | Bureaublad | Desktop | |
| Dock | Orbit | Orbit | Accessible name "Orbit, dock" |
| Launcher | Menu | Menu | Opened by the mark or the OS key; accessible name "BaryonOS-menu" |
| Search | Zoeken | Search | Same word in Orbit, menu and apps |
| Files | Bestanden | Files | Inspector sections: Versies, Toegang, Opslag |
| Settings | Instellingen | Settings | |
| App store | Haven | Haven | Descriptor "Apps ontdekken en installeren" |
| Notifications | Meldingen | Notifications | Opened from the clock |
| Quick settings | Snelle instellingen | Quick settings | Opened from the status area |
| Lock / sign out | Vergrendelen / Afmelden | Lock / Sign out | |
| Power | Aan/uit: Slaapstand, Opnieuw opstarten, Afsluiten | Power: Sleep, Restart, Shut down | |
| Recovery environment | Herstel | Recovery | Noun, system level only |
| First-party apps | Notities, Foto's, Muziek, Rekenmachine, Browser | Notes, Photos, Music, Calculator, Browser | Functional single nouns |
| Assistant | Assistent op dit apparaat | On-device assistant | |
| NAS roles | Mijn NAS, Ruimte ontvangen, Ruimte aanbieden | My NAS, Receive space, Offer space | The NAS keeps the name its owner gives it ("Thuis-NAS") |

## 5. Information architecture

**Orbit**, left to right: Menu (mark) · Zoeken · pinned and running apps ·
status (privacy indicator, network, sound, battery) · clock and date. Clicking
the status opens Snelle instellingen; clicking the clock opens Meldingen.

**Menu**: search field at the top (typing searches directly) · Vastgezet
(pinned apps) · Verder waar je was (recent files) · Alle apps · footer with
the user, Vergrendelen and Aan/uit.

**Search results** are grouped as Apps, Bestanden, Instellingen, Acties and
Assistent op dit apparaat. Internet results appear only after the user turns
them on and chooses a service.

**Instellingen**: Systeem · Apparaten · Netwerk en internet · Weergave · Apps ·
Gebruikers · Opslag en NAS · Tijd en taal · Toegankelijkheid · Privacy en
beveiliging · Updates. Against DESKTOP-SHELL §2 this renames Personalization
to Weergave and Accounts to Gebruikers (there are no vendor accounts) and adds
Opslag en NAS (pairing, own NAS, receiving and offering space).

**Bestanden**: sidebar Recent · Bureaublad · Documenten · Afbeeldingen ·
Downloads; Collecties; Plaatsen (Dit apparaat, each paired NAS by its own
name); Prullenbak. The inspector shows Versies, Toegang and Opslag.

## 6. Vocabulary rules

- **Storage labels**: "Op dit apparaat" · "Op Thuis-NAS · versleuteld" ·
  "Op dit apparaat en Thuis-NAS". Sync states: "Bijgewerkt",
  "Wordt bijgewerkt…", "Wacht op Thuis-NAS", "Thuis-NAS niet bereikbaar".
- **Privacy indicator**: "Op dit apparaat" when nothing leaves the device;
  otherwise "<App> gebruikt internet", "<App> vraagt internet",
  "<App> gebruikt de microfoon". Always icon plus text, never colour alone.
- **Permission question**: title "<App> wil <wat> <actie>"; rows App, Wat,
  Waarheen, Hoe lang; origin line "Vraag van het systeem · getoond boven alle
  apps". Buttons name the consequence: "Eenmalig toestaan",
  "Toestaan tot je het sluit", "Niet toestaan" or "Annuleren". Never "OK" or
  "Ja". The safe choice is the default.
- **Actions** keep one name through a flow: Openen, Opslaan, Nieuw, Hernoemen,
  Verplaatsen, Delen, Verwijderen (to Prullenbak), Definitief verwijderen,
  Intrekken (access), Toestaan / Niet toestaan.
- **Restore versus recovery**: versions and deleted items are "teruggezet"
  ("Deze versie terugzetten", "Vergelijken en terugzetten"); "Herstel" is
  reserved for the recovery environment and "Herstelcode" for the recovery
  code. This resolves the ambiguity of "herstellen" in Dutch. (The concept
  boards still use "Vergelijken en herstellen"; the design system and
  prototype use the new wording.)
- **Errors** say what happened and what happens next, without apology:
  "Thuis-NAS is niet bereikbaar. Je werk staat veilig op dit apparaat en wordt
  bijgewerkt zodra de NAS terug is."
- **Status labels** use sentence case, no exclamation marks and no internal
  terms (no "capability", "revision", "kernel").

## 7. Localisation and cross-device consistency

- Functional names are translated through one string table per language;
  brand names (BaryonOS, Orbit, Haven) are not.
- Layouts allow at least 35 % text expansion (German) and never truncate
  primary actions; labels are not concatenated from fragments, so grammar can
  change per language; plural rules follow CLDR; dates and times are local
  ("do 13 nov", "Thu 13 Nov").
- Avoid idioms and wordplay in UI text; use "je" in Dutch and second person in
  other languages; prepare for right-to-left mirroring of Orbit and windows.
- Client, mobile and NAS roles use the same names and the same IA; the NAS
  administrator surface adds "Ruimte en quota", "Gasten", "Gast blokkeren" and
  "Kopieën van gast verwijderen", none of which grant access to content.

## 8. Proposal P-NAMES-1 and old → new mapping

Proposal for the project owner: replace the user-facing names Horizon, Zenit,
Signaal, Nebula and Kern by functional names, retain Orbit and Haven with
descriptors, and remove the remaining "Nieuw OS" strings. Until accepted,
DESKTOP-SHELL §7 stays authoritative and native strings stay unchanged.

| Old user-facing name | New (NL / EN) | Code identifiers (unchanged) |
|---|---|---|
| Nieuw OS | BaryonOS | `nieuw_os_*` crates and paths |
| Horizon | Bureaublad / Desktop | `Page::Home`, `theme::wallpaper` |
| Orbit | Orbit (dock) | `dock()` in `ui/native/src/desktop.rs`, Veld `shell.rs` |
| Zenit | Menu ("BaryonOS-menu") | `Target::Start` in `ui/native/src/desktop.rs` |
| Signaal | Zoeken / Search | `Page::Search` |
| Nebula, Mijn materiaal, Materiaal (as app) | Bestanden / Files | `Page::Material`, `sdk/examples/materiaal` |
| Kern | Instellingen / Settings | none yet |
| Haven | Haven (apps ontdekken en installeren) | none yet |
| Berekening | Rekenmachine / Calculator | `sdk/examples/calculator` |
| Werkruimte (header) | removed; "Werkruimte n" with multiple workspaces | Veld `shell.rs` |
| Vergelijken en herstellen | Vergelijken en terugzetten | Materiaal revisions |
