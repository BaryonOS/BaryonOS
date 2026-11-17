<a id="os-gebruikersbehoeften-overstap-en-ontwerp--fase-24"></a>

# OS user needs, switching and design — phases 2–4

Research date: **2 November 2026**, Europe/Amsterdam. Status: recorded desk research; no original interviews, no product code, no newly accepted OS functions. Assignment: supplement the earlier repository analysis with current external research, check the original Eurobarometer claim and preserve this research in the repository.

<a id="kernconclusie"></a>

## Main conclusion

[ANALYSE] The supported opportunity for nieuw-os is a coherent daily workflow with control, recovery and a demonstrably feasible switch. A new style, European origin or AI function alone is insufficiently supported as a reason to switch. The sources show both improvements in established platforms and persistent dependencies; “niemand lost dit op” (English: “nobody solves this”) is not a defensible premise.

**Key numerical check:** 58% agreement with switching to an EU-based digital service provider, even at slightly higher costs, is confirmed in Special Eurobarometer 572/QB6.4. This does not measure OS migration. The original question, response distribution, method and limitations are recorded in [the adoption report](european-adoption-barriers.md) and [the source check](eurobarometer-check.json).

<a id="onderzoeksvragen"></a>

## Research questions

1. Which daily frictions and desired functions are actually described for Windows, macOS, Linux desktops and Android?
2. Which signals recur in independent sources, and which are only an isolated report or vendor claim?
3. Which functional, social, organizational and economic barriers prevent switching?
4. What exactly does the Eurobarometer measure about European services and payment, and what remains unknown about OS choice?
5. Which OS design patterns developed between approximately 2011 and 2026, and what evidence exists for their practical value?
6. Which findings could fit the existing independent nieuw-os architecture without presuming a product decision or implementation?

<a id="dossiers-en-leesvolgorde"></a>

## Dossiers and reading order

**Phase 4 extension:** [full installation and day-one journey](journey/README.md), with 31 installation aspects, 28 daily tasks, mental models and separately labelled installer/onboarding/interaction proposals. No implementation. The comparison was recorded first; a bounded supplement to the existing product plan followed.

| Document | Content |
|---|---|
| [Repository baseline](repository-baseline.md) | Phase 1 recorded: main, newer development branch and prototype/native evidence limits |
| [User needs and synthesis](os-user-needs.md) | Comparison, recurrence, impact, European relevance and technical possibilities |
| [Windows and macOS](windows-macos-needs.md) | Aspects per OS, original user signals, current solutions and limitations |
| [Linux and mobile](linux-mobile-needs.md) | GNOME/KDE/Ubuntu, Android and relevant iOS/ChromeOS comparisons |
| [European adoption and barriers](european-adoption-barriers.md) | Eurobarometer, intention versus behavior, compatibility, migration and European alternatives |
| [OS design trends](future-os-design.md) | Timeline, useful patterns, decorative risks and explicit future hypotheses |
| [Source register](sources.md) | Source families, dates, primary sources and verification limits |
| [Eurobarometer check](eurobarometer-check.json) | Original PDF hashes, page numbers, figures and arithmetic check |

This supplements the [explorations of 28 October](../README.md), rather than replacing the product roadmap. Materiaal, the privacy contract, full-release criteria and fixed acceptance weights remain in force. Phases 5–8 must justify any changed product choices, priorities and implementations separately. No new desktop direction has been adopted here.

<a id="zoek--en-selectiestrategie"></a>

## Search and selection strategy

Purposive triangulation used three parallel research dossiers and central source/claim checking. Recency: current platform documentation and recent sources, generally 2024–2026. Older original studies were used only where they support a mechanism or design history; their findings are not extrapolated to 2026. The timeline explicitly includes older releases.

A web search engine was used, followed by opening the original page, publication, issue or report. Search clusters included:

- `Windows 2026 search taskbar updates user feedback`, Microsoft OneDrive file/backup behavior, PowerToys discoverability and NV Access regressions;
- macOS window tiling, Spotlight, iPhone Mirroring EU, AppleVis VoiceOver and research into OS interface habit;
- GNOME/KDE accessibility, window management, package portals, original issues, Ubuntu hardware certification;
- Android privacy choices, notifications, background processes, adaptive interfaces, update/backup and integrity limits;
- `Eurobarometer 2026 58 EU digital service provider`, original SP572 questionnaire, QB6.4, report and data annex;
- CMA mobile ecosystems switching, original vzbv-Windows-10 tables, Office-VBA, Adobe requirements, Valve-Proton and European migration cases;
- original introductions of flat/Material/design materials, dynamic color, adaptive layouts, semantics, AI guidelines and studies of attention, contrast and energy.

Preference was given to original studies and publishers, official specifications/support/release decisions and regulator research, followed by original community reports. Community sources help identify concrete failed actions and expectations; they do not support a general market claim. Search results, votes and duplicate news articles were not counted automatically. The selection includes deliberate counterexamples: existing solutions, improved rollout and satisfaction as a reason to stay.

Inclusion requires an identifiable finding, direct URL, date or explicitly unknown date, source type and limitation. The following were rejected as general support: untraceable percentages, marketing without a denominator, a rant presented as the voice of an entire target group, inaccessible full text treated as if it had been read, and a feature announcement treated as a demonstrated user effect. Beta/Insider/roadmap remains separate from general availability. Each dossier records the sources actually used; no complete search history or PRISMA claim is provided.

<a id="labels-en-bewijssterkte"></a>

## Labels and strength of evidence

- **[BRON]**: what a reviewed source describes, measures or officially supports; its scope remains attached.
- **[SIGNAAL]**: a concrete original report or community request; no representativeness.
- **[ANALYSE]**: synthesis, comparison or technical inference within this research.
- **[HYPOTHESE]**: a need, design interpretation or future expectation to test.
- **[REPOSITORY]**: source code found or version-bound project evidence; no external market finding.

For the synthesis, **strong** means direct empirical evidence or an explicitly verifiable technical boundary for the bounded claim. **Moderate**: several usable primary indications, a limited target group or transfer to nieuw-os. **Weak/open**: isolated signals, product provisions alone or missing measurement. Strong technical evidence can coexist with unknown frequency of use.

“Herhaling” (recurrence) means only recurrence within this purposive source set. For example, several Microsoft pages count as one vendor family; a press release and table from the same survey count as one study. No ranking of complaint frequency among all European users is published. Impact, feasibility and differentiation are editorial assessments, not research percentages or a phase-6 score.

<a id="doelgroepen-en-ontbrekende-eigen-data"></a>

## Target groups and missing original data

Research segments: people with daily browser/document tasks; advanced users with automation or customization; gamers; creative/professional users; people who need assistive technology; administrators/organizations. These are analytical groups, not interviewed personas. Age, country, technical knowledge and disabilities cannot reliably be inferred from platform choice.

No fictitious names, quotations, interviews, feedback scores, mock data or market-share forecasts have been added. Future research must collect actual tasks and necessary software/devices, with participants from more than one skill and accessibility context. Measure appeal, task performance, trying, actual switching and continued use separately.

<a id="grenzen-en-veilige-vervolgroute"></a>

## Limits and a safe next route

The source set is not exhaustive, preregistered or representative of all Europeans. English, Dutch and German sources and technically active communities are overrepresented. Vendors know their products but have commercial interests; user issues overrepresent problems. Many platform features now narrow earlier gaps. Failure to find a source does not prove the absence of a need.

The studies close no native, physical, human or release gate. The documentation branch on main retains 34/100; the separately checked newer development branch reports 41/100. Phases 5–6 can now compare problem-oriented concepts and explicit experiments. Product code follows only after a recorded design/implementation plan within the user's assignment.
