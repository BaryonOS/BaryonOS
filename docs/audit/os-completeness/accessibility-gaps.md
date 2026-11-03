# Toegankelijkheid en internationalisatie

## Huidige stand

De gekozen product- en UI-specificaties noemen toetsenbord, focus, schaal,
contrast en reduced motion. De browserprototype heeft beperkte
keyboard/focus- en responsive checks. Native documentatie noemt semantiek,
maar er is geen aangetoond native accessibility tree/event channel,
screenreader, braille, switch access, echte 200%-schaalproef of menselijke
gebruikerstest.

UTF-8-dataopslag en een Nederlandstalig prototype bewijzen geen volledige
internationalisatie. Onbewezen zijn onder meer font fallback, complex text
shaping, RTL, IME/dead keys, locale-formattering, taalwissel, vertalingen en
screenreader-taal.

## Vereiste verificatie

1. Publiceer een native accessibility API en event semantics (focus, role,
   name, value, state, live status).
2. Test setup, login, Materiaal, rechten, appinstallatie, fout- en recovery
   taken met alleen keyboard en een native screenreader.
3. Controleer 200% zoom, contrast, large text, reduced motion, niet-kleurige
   signalen, captions/audio feedback en focusherstel na surface-/appcrash.
4. Leg participantgroep, taken, succes/hulp/fouten en privacy vast. AI-review
   of screenshotinspectie is geen menselijke gebruikerstest.
5. Test Unicode-combining marks, RTL, CJK/IME, locales/datum/tijd/valuta,
   font fallback en screenreader-language switching op native apps.

WCAG 2.2 is nuttig voor webprototype en testbare interactieprincipes; haar
scope is webcontent. Geen native OS-conformiteit volgt daar automatisch uit.
