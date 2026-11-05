# Sessieroutes: referenties en concrete Nieuw-OS-keuzes

**Datum:** 2 november 2026. **Bronbasis:**
`1f0dd9c93959e3b1335656da1e548525a288133e`. **Status:** brononderzoek en
ontwerp vóór nieuwe visuele integratie; geen nieuwe uitvoeringsacceptatie.
Score **41/100 blijft 41**. Het geaccepteerde criterium U01 is niet hetzelfde
als de volledige release-eis FR-U01 of beveiligde sessie-eis FR-S02.

De actuele code heeft al een eigen native venstersessie, twee geïsoleerde apps,
focus/slepen, proportionele systeemletters, herstel en begrensd klembord.
`ui/native/src/session.rs` heeft `SessionFlow`, `AuthBroker`, `UnavailableBroker`
en een aparte openbare `VmGuest`-identiteit. De standaardbroker weigert echte
authenticatie; een scherm of accountkaart verleent geen kernelrecht. Zie het
[bestaande sessiecontract](../implementation/SESSION-AND-DESKTOP.md) en de
[herziene visuele specificatie](../implementation/QUIET-SESSION-SPEC.md).

## Methode en grens

Primaire handleidingen onderbouwen de routes, technische documentatie de
dienstgrenzen. Beschikbaarheid hangt af van versie, hardware, enrollment en
beleid; een hier niet genoemde methode is niet automatisch afwezig. Er is geen
nieuwe vergelijkende gebruikersstudie, beveiligingsaudit of hardwaretest gedaan.
Apple-guidepagina's gaven rechtstreeks HTTP 429; de guidebevindingen blijven
beperkt tot opgehaalde officiële zoekindexfragmenten. Referentiebeelden zijn
geen door ons geteste OS-installaties.

“Onderzocht patroon” beschrijft **bronfeiten**. De overige stappen zijn
**projectkeuzes en acceptatie-eisen**, geen claim van reeds voltooide code.

## 1. Windows 11: identiteit, beschikbare methode, desktop

**ONDERZOCHT PATROON:** account kiezen, wachtwoord/PIN of ingerichte Hello-
methode, bevestigen, desktop; methodes verschillen per configuratie. Taakbalk
combineert apps/zoeken/open werk/status; Snap ondersteunt vensterindeling.
Bronnen: Microsoft, z.d., Windows 11/10,
[User Account Access](https://support.microsoft.com/en-us/accounts-billing/security/user-account-access-in-windows),
[Sign-In Options](https://support.microsoft.com/en-us/accounts-billing/security/sign-in-options-in-windows),
[Taskbar](https://support.microsoft.com/en-gb/windows/experience/personalization/customize-the-taskbar-in-windows),
[Snap](https://support.microsoft.com/en-us/windows/experience/snap-your-windows).

→ **GEBRUIKERSPROBLEEM:** namens wie meld ik aan, welke methode werkt en waar
vind ik mijn gestarte werk?
→ **NIEUW OS NEEMT OVER:** naam vóór credentials, benoemde methodes, vaste
status-/zoekingang en duidelijke actieve vensters.
→ **BEWUST ANDERS:** lokaal beginnen zonder leverancieraccount; geen
webresultaten tussen persoonlijke Materiaalresultaten; geen Windows-kopie.
→ **SYSTEEMCONTRACT:** geauthenticeerde brokerreply, lokale getypeerde
zoekresultaten en compositor-eigendom van focus/bounds.
→ **LOCATIE:** `ui/native/src/{session,desktop}.rs`,
`sdk/examples/window-session/src/scene.rs`, `ui/prototype/`.
→ **ACCEPTATIE:** verkeerd account/methode opent niets; zoeken vindt apps,
Materiaal, instellingen en sessieacties met toetsenbord.

## 2. macOS: systeemfuncties hebben een vaste plaats

**ONDERZOCHT PATROON:** menubalk met status, Spotlight, Control Center,
privacy-indicatoren en notificaties; systeemmenu voor instellingen/lock/power.
User switch via menubalk/Control Center; toegankelijkheid vóór login;
herkenbare opstart-/herstelbeelden. Bronnen: Apple, z.d., actuele Mac-guide,
[Menu bar](https://support.apple.com/en-gb/guide/mac-help/mchl4af84660/mac),
[User switching](https://support.apple.com/en-ie/guide/mac-help/mchlp2439/27/mac/27),
[Accessible login](https://support.apple.com/guide/mac-help/mchlaa57f797/mac),
[Startup recovery](https://support.apple.com/en-ca/102675).

→ **GEBRUIKERSPROBLEEM:** systeemfuncties mogen niet verdwijnen bij app- of
gebruikerswisseling; hulp moet vóór authenticatie bereikbaar zijn.
→ **NIEUW OS NEEMT OVER:** vaste systeemidentiteit, tijd/status, sessiemenu,
toegankelijkheidsingang en causale sessiefeedback.
→ **BEWUST ANDERS:** geen globale appmenubalk of glas als identiteit;
Materiaal houdt gereedschap, geschiedenis en toegang bijeen.
→ **SYSTEEMCONTRACT:** status uit diensten; privacyveilige prelogin-data;
user switch retireert oude input; herstel opent geen account.
→ **LOCATIE:** systeemchrome in `sdk/examples/window-session/src/scene.rs`,
`ui/native/src/session.rs` en browserroutes.
→ **ACCEPTATIE:** lock/switch met toetsenbord; oude sessie krijgt geen input;
ontbrekende dienst geeft geen schijnsucces.

## 3. GNOME: rustige lockscene, expliciete login

**ONDERZOCHT PATROON:** lock met datum/tijd, bepaalde meldingen,
batterij/netwerk; pointer, Enter of Escape opent login; user switch bij meerdere
accounts. Topbar geeft status; Activities/Super/type zoekt apps en werk;
dash onderscheidt draaiende apps. Bronnen: GNOME, z.d.,
[Lock screen](https://help.gnome.org/gnome-help/shell-lockscreen.html),
[Visual overview](https://help.gnome.org/gnome-help/shell-introduction.html).

→ **GEBRUIKERSPROBLEEM:** gereedheid zien zonder privéwerk te tonen; vanuit
ieder scherm apps en werk kunnen terugvinden.
→ **NIEUW OS NEEMT OVER:** rustige lockscene met “Aanmelden”, zichtbare focus,
vaste zoekingang en open-werkindicatie.
→ **BEWUST ANDERS:** geen verplicht appgrid of verborgen hot corner; standaard
geen documenttitels/notificatie-inhoud vóór authenticatie.
→ **SYSTEEMCONTRACT:** locked/users/auth/unlocking/desktop en privacyfilter.
→ **LOCATIE:** `ui/native/src/`; echte compositor-lock later in
`sdk/examples/window-session/`, niet in de documentapp.
→ **ACCEPTATIE:** Enter zonder pointer; Escape annuleert de poging en maakt
late replies ongeldig; geen verlies van een documentdraft.

## 4. KDE Plasma: open werk en zoeken zijn gewone desktopfuncties

**ONDERZOCHT PATROON:** panel met launcher, vensterlijst, system tray en klok;
launcher biedt apps/recent werk/sessieacties; KRunner via Alt+Space/Alt+F2.
Bronnen: KDE, z.d., stable KF6-docs,
[Panel](https://docs.kde.org/stable_kf6/en/plasma-desktop/plasma-desktop/panel.html),
[KRunner](https://docs.kde.org/stable_kf6/en/plasma-desktop/plasma-desktop/krunner.html).

→ **GEBRUIKERSPROBLEEM:** starten, staat-al-open en activeren moeten te
onderscheiden zijn zonder een abstract pictogramalfabet te leren.
→ **NIEUW OS NEEMT OVER:** benoemde launcher/zoeken, actieve-appindicatie,
window switcher, herkenbare titelbalk en herstelbare minimalisatie.
→ **BEWUST ANDERS:** geen widgetverzameling of beheerpanelen als homescreen;
gevulde Materiaalwerkruimte met locatie/revisie.
→ **SYSTEEMCONTRACT:** owner/visibility/z-order/focus/bounds per venster;
zoekresultaat is App/Material/Setting/Action.
→ **LOCATIE:** `sdk/examples/window-session/src/scene.rs`, appcontrollers,
`ui/surface-protocol/` en browserinteracties.
→ **ACCEPTATIE:** twee apps starten, wisselen, verbergen/herstellen/sluiten;
“vergrendelen” vinden zonder extra documentrechten.

## 5. SDDM: greeter presenteert, backend authenticeert

**ONDERZOCHT PATROON:** aparte greeter-systeemgebruiker; screens/users/sessions
als modellen; authenticatie en power via callbacks. Het thema is geen
verifier. Het screens-model maakt outputs expliciet. Bronnen: SDDM, develop,
[Theming](https://github.com/sddm/sddm/blob/develop/docs/THEMING.md),
[README](https://github.com/sddm/sddm).

→ **GEBRUIKERSPROBLEEM:** visuele aanpassing mag geen identiteit of succes
verzinnen; login moet op het beschikbare scherm passen.
→ **NIEUW OS NEEMT OVER:** publieke user enumeration, geselecteerde gebruiker,
requestcorrelatie, busy/cancel/error en expliciete dienstresultaten.
→ **BEWUST ANDERS:** één contract voor native/browseradapter; testadapter
zichtbaar gelabeld; geen willekeurig demowachtwoord of stille gast.
→ **SYSTEEMCONTRACT:** account/methode/request/boot/servicegeneratie, timeout,
cancel en aparte activatie; output-id/scale is voorlopig een toekomstcontract.
→ **LOCATIE:** `ui/native/src/session.rs`, compositor/input-adapter onder
`sdk/examples/window-session/`; synthetische adapter uitsluitend testprofiel.
→ **ACCEPTATIE:** verkeerde gebruiker, cancel, late reply, key verwijderd,
sensorfout of servicecrash ontsluit geen echte gebruikerssessie.

## 6. Android: authenticatie levert bewijs aan een andere dienst

**ONDERZOCHT PATROON:** Gatekeeper verifieert in TEE, throttlet mislukte
pogingen en levert per-boot geauthenticeerd token aan Keystore; biometrie is
aparte provider. Quick Settings centraal; multi-user/guest waar ondersteund.
Bronnen: AOSP, doorlopende docs,
[Gatekeeper](https://source.android.com/docs/security/features/authentication/gatekeeper);
Google, z.d., [Quick Settings](https://support.google.com/android/answer/9083864?hl=en),
[Users](https://support.google.com/android/answer/2865483?hl=en).

→ **GEBRUIKERSPROBLEEM:** een UI-boolean mag geen account of opslaggeheim
openen; hardware en diensten kunnen tijdens authenticatie uitvallen.
→ **NIEUW OS NEEMT OVER:** service-side retrybeleid, begrensde assertion en
zichtbare hardware-/timeout-/hersteltoestanden.
→ **BEWUST ANDERS:** geen swipe/pattern voor desktop; geen eigen matcher;
hostmodel is geen TEE of werkende biometrie.
→ **SYSTEEMCONTRACT:** account/boot/doel/challenge/verval/intrekking;
authsucces is nog geen succesvolle sleutel-/sessieactivatie.
→ **LOCATIE:** `AuthBroker` in `ui/native/src/session.rs`; toekomstige
auth-/keyringdiensten zijn de afdwingingspunten.
→ **ACCEPTATIE:** replay/stale/wrong-session/rate-limit/lege verplichte PIN/
crash falen gesloten; methodeswitch reset geen limiter.

## 7. ChromeOS: één vaste zoek- en systeemingang

**ONDERZOCHT PATROON:** Search/Launcher voor zoeken/apps; klok opent controls
met lock, ook via Launcher+L. PIN/password en herauthenticatie hangen van
inrichting/beleid af; actuele help beschrijft apart Chromebook-password.
Guest is een aparte route. Bronnen: Google, z.d.,
[Lock/unlock](https://support.google.com/chromebook/answer/2587994?hl=en),
[Keyboard](https://support.google.com/chromebook/answer/1047364?hl=en),
[Sign in](https://support.google.com/chromebook/answer/15641761?hl=en).

→ **GEBRUIKERSPROBLEEM:** dagelijkse acties vinden zonder hun app of
instellingenpagina te onthouden.
→ **NIEUW OS NEEMT OVER:** vaste zoek-/actielaag, statusmenu en lockactie.
→ **BEWUST ANDERS:** lokaal beginnen zonder verplichte cloudaccount; offline
is een status, geen storing van alle lokale taken. Geen claim dat ChromeOS
altijd netwerk vereist.
→ **SYSTEEMCONTRACT:** lokale geautoriseerde index; sessieacties naar broker;
guest alleen binnen een werkelijk beschikbaar geïsoleerd profiel.
→ **LOCATIE:** `ui/native/src/desktop.rs`, `ui/prototype/` en systeemchrome.
→ **ACCEPTATIE:** offline zoeken/openen/lock; niet-geïmplementeerde guest geeft
geen succesvolle accountaanmelding.

## Lokale FIDO2: verschillende standaarden niet verwarren

**ONDERZOCHT PATROON:** Yubico `pam-u2f` ondersteunt lokale PAM-login met
publieke credential-/accountmapping, user presence, PIN en UV. De mapping moet
vóór unlock beschikbaar zijn; opslag in nog versleutelde home kan login
blokkeren. `alwaysok`/`nouserok` zijn gedocumenteerde opties, geen Nieuw-OS-
standaarden. Bron: Yubico, doorlopende [pam_u2f(8)](https://developers.yubico.com/pam-u2f/Manuals/pam_u2f.8.html).

systemd-homed gebruikt een andere lokale unlockroute: FIDO2 `hmac-secret`
voor account/home met afzonderlijk PIN/UP/UV-beleid; geen drop-in signature-
verifier. Bron: systemd, actuele manpage meldt 262,
[homectl](https://www.freedesktop.org/software/systemd/man/latest/homectl.html).
Microsofts OS-FIDO2-documentatie betreft Entra joined/hybrid joined, geen bewijs
voor willekeurige lokale accounts. Bron: Microsoft Learn,
[Windows FIDO2](https://learn.microsoft.com/en-us/entra/identity/authentication/howto-authentication-passwordless-security-key-windows).
Yubico beschrijft keymateriaal op de authenticator, challengegebonden assertion
en publieke-keyverificatie; geen claim over iedere gesynchroniseerde passkey.
Bron: Yubico, [FIDO2 Credentials](https://docs.yubico.com/yesdk/users-manual/application-fido2/fido2-credentials.html).

→ **GEBRUIKERSPROBLEEM:** aanwezigheid/aanraking is geen geldige account-
verificatie; fallback kan beleid omzeilen.
→ **NIEUW OS NEEMT OVER:** enrollment, publieke verifierdata, verse challenge,
vereiste UP/UV/PIN, intrekking, cancel en herstel.
→ **BEWUST ANDERS:** nu testadapter; later gecontroleerde CTAP2/FIDO2-library
en devicecapability; geen eigen crypto, echte passthrough of keys in Git.
→ **SYSTEEMCONTRACT:** private key verlaat hardwareauthenticator niet;
broker valideert gebruiker/RP/challenge/beleid en aparte sessieactivatie.
→ **LOCATIE:** `ui/native/src/session.rs` als UI-grens; toekomstige authdienst
bezit verifier/hardware, nooit renderer.
→ **ACCEPTATIE:** absent/removed/wrong key, stale/replay, timeout/cancel,
ontbrekende verplichte PIN/UV geven geen assertion. Een echte NULL-PIN-bypass
onderstreept de fallbacktest: Yubico, 2021,
[YSA-2021-03](https://www.yubico.com/support/security-advisories/ysa-2021-03/).

## Boot, displays en meting

Windows onderscheidt firmware/Boot Manager/OS Loader/kernel; Androids richtlijn
ondersteunt faseanalyse; Plymouth scheidt bootsplash/diagnose. Bronnen:
Microsoft Learn, [Startup troubleshooting](https://learn.microsoft.com/windows/client-management/advanced-troubleshooting-boot-problems);
AOSP, [Optimize boot times](https://source.android.com/docs/core/perf/boot-times);
freedesktop.org, [Plymouth](https://wiki.freedesktop.org/www/Software/Plymouth/).
Nieuw OS kiest vroege feedback en herstel met echte readiness-mijlpalen,
geen neppercentage. Een splashframe bewijst geen bruikbare input.

Meet firmware/loader/kernel/diensten/opslag/display/first-present/input-ready
apart op dezelfde QEMU-configuratie. QMP/capture/hashes/verificatie en extra
VM-starts blijven apart van renderwerk. TCG en WHPX/KVM/HVF zijn geen gelijke
meetarmen. “45 seconden” is een te toetsen observatie, geen bewezen renderer-
duur; dit onderzoek bevat geen nieuwe cijfers.

Multi-display krijgt output-id, schaal, veilige bounds, inputfocus en herstel
bij outputverlies als contract. De eerste slice mag één bewezen output hebben;
een screenshot bewijst geen meerdere displays, scanout of fysieke hardware.
Alle bronnen zijn geraadpleegd op **2 november 2026**; doorlopende pagina's
zonder vaste revisiedatum zijn als z.d. behandeld. Ontwerpkeuzes zijn geen
OS-versiepariteit, certificering of vergelijkende veiligheidsclaim.
