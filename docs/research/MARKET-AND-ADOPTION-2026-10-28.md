# Baryon OS — market, consumer adoption and sustained switching

**Date:** 28 October 2026

**Status:** research foundation / strategic hypotheses

**Scope:** desktops and laptops in Europe, with emphasis on the EU and the Netherlands

## Main conclusion

The most promising position for Baryon OS lies in the **combination of everyday usability, understandable control over data, recoverability, high-quality execution and a reliable switching route**, rather than one isolated property such as privacy, European origin or local AI.

The studies provide no evidence that broad European adoption has already been demonstrated. They primarily identify which conditions must first work convincingly and which hypotheses require testing with real users.

A useful positioning hypothesis is:

> **Your work, clearly organised and under your control.**

Materiaal can make that promise concrete by enabling the same content to be retrieved, edited with suitable tools, its changes understood, access bounded and errors recovered from.

## 1. Market context

### European desktop market

For September 2026, Statcounter reported the following shares of measured desktop web traffic in Europe:

| Platform | Share |
|---|---:|
| Windows | 77.28% |
| macOS / OS X combined | 16.50% |
| Linux | 4.61% |
| Chrome OS | 1.60% |
| FreeBSD | 0.01% |

These are web-traffic measurements, not counts of installed computers. They nevertheless show how strongly existing platforms and their ecosystems determine the starting position.

### European digital autonomy has appeal

In the Digital Decade Eurobarometer 2026, the European Commission reported, among other findings:

- 82% want less dependency on digital technology from third countries;
- 85% want priority for digital services developed and controlled in Europe;
- 58% say they would choose an EU provider, even at a slightly higher price.

This supports European control as a positioning component. It does not establish that the same percentages would switch to a new desktop OS.

### Dissatisfaction is not migration

A German consumer survey of 800 current or recent Windows 10 users reported around the end of support:

- 46%: switching to Windows 11 or already switched;
- 34%: continuing to use Windows 10, even without updates;
- 4%: switching to another OS or already switched;
- 12%: still uncertain.

The commercial implication is significant: even a clear reason to change does not automatically produce a switch to an alternative OS.

## 2. Where the market gap probably lies

The studies support six connected opportunities:

| Opportunity | User value | Product implication |
|---|---|---|
| Control without system administration | Understand where data are stored and who can access them | Safe defaults, understandable permissions and clear location |
| Coherence around content | Less searching, duplication and use of incorrect versions | Materiaal as a central layer above interchangeable tools |
| Reliable switching | Know beforehand whether personal tasks work | Suitability check, trial migration and follow-up support |
| Calm and predictability | Fewer unexpected interruptions and less confusion | Consistent controls, adjustable notifications, recoverable changes |
| Hardware that remains usable | Know how long the system will continue operating securely | Bounded reference hardware and a maintenance commitment |
| Local AI under personal control | Assistance without unnecessary data transfer | Selected context, correctable actions and no cloud fallback |

Individual elements already exist on other platforms. Differentiation must therefore demonstrably arise from **their coherence**.

## 3. Why European alternatives generally remain limited

Existing European alternatives show interest, but also that interest does not equal broad primary adoption.

- In April 2026, Zorin reported more than 3.3 million downloads since Zorin OS 18. Downloads establish neither active nor sustained use.
- TUXEDO combines Linux software, preconfigured hardware and support. This shows how a manageable hardware matrix can improve the user experience.
- Linux Mint and KDE show the importance of continuity, community and maintenance, but do not remove the app/hardware ecosystem problem.
- EU OS is a community initiative for government/business use, not evidence of a broad consumer market.
- European mobile initiatives such as /e/OS/Murena and Jolla show that sovereignty and privacy can attract users, while scale, hardware, apps and distribution remain constraints.

The recurring barriers are:

1. essential apps and websites;
2. drivers and peripherals;
3. migration of files, accounts and habits;
4. support when something goes wrong;
5. maintenance and funding over several years;
6. the interdependence between users and developers.

This is particularly relevant to Baryon OS because an independent kernel does not automatically inherit existing Linux drivers and Linux compatibility.

## 4. What consumer research adds

### Usefulness, effort, enjoyment and habit interact

UTAUT2 was studied among 1,512 mobile internet users and models adoption through factors including expected usefulness, required effort, enjoyment, price value, support and habit. For Baryon OS, this means that one strong privacy claim is insufficient: the complete experience must provide benefits at an acceptable level of effort.

### Habit and switching friction are real

Farhang et al. studied 239 Windows users. Of 78 participants who had not upgraded to Windows 10, 38 cited satisfaction with their existing version; privacy and compatibility also played a role.

Grzybowski and Nicolle analysed 9,799 actual smartphone switches and found effects of both preference and switching costs. The mobile context cannot be transferred directly to desktop, but supports the influence of existing investments and habits on switching.

### A good visual first impression is necessary but insufficient

Research by Tuch and colleagues shows that visual complexity and familiar organisation influence rapid aesthetic judgements. Other research shows that poor interaction can reduce later aesthetic evaluations. The interface must therefore both look polished and be immediately usable.

### Personal information management fits Materiaal closely

- *Stuff I've Seen* studied local retrieval among 234 employees over six weeks and found time and people useful as cues.
- Bergman et al. found a clear preference for folders over tags within their research design among 75 Gmail and 23 Windows users.
- Boardman and Sasse found substantial individual differences in organisation across files, email and bookmarks.

Materiaal must therefore add new context and history without unnecessarily removing familiar names, folders, search and direct opening.

### Privacy behaviour depends on context

Tsai et al. found that prominent, understandable privacy information could influence real shopping choices. In another purchasing experiment, Beresford et al. found that 39 of 42 buyers chose the shop costing one euro less despite its more sensitive data requests.

The conclusion is that **privacy must be combined with price, convenience and concrete value**; this does not mean privacy is unimportant.

### Notification calm requires adjustable testing

In a randomised field experiment with 237 smartphone users, Fitz et al. found more favourable perceived attention and control with batched notifications. A later preregistered study with 205 participants found no clear productivity gain from disabling notifications for a week.

An adjustable, predictable notification architecture is therefore defensible for Baryon OS; general health or productivity claims are not.

### Local AI must be understandable and correctable

- Qualitative CHI research into local ML showed that on-device processing alone does not automatically produce trust.
- Dietvorst et al. found that people were more willing to use an imperfect algorithm if they could adjust its output.
- A meta-analysis of 106 experiments on human–AI collaboration found strongly task-dependent effects.

Local AI must therefore begin with concrete, selected context and correctable suggestions, while the core workflow remains fully usable without AI.

## 5. Design trends: direction, not a European rule for colour

Current design systems show different developments:

- Apple Liquid Glass: material effects, layering and platform consistency;
- Google Material 3 Expressive: more expressive colour, shape, typography and motion;
- Microsoft Mica/Fluent: subtle materials and hierarchy with performance-aware degradation;
- GNOME Adwaita: consistent light/dark/high-contrast variants.

There is **no convincing evidence for one colour or visual style collectively preferred by Europeans**.

The strongest research hypothesis for Materiaal therefore remains:

- quiet, predominantly opaque content surfaces;
- clear hierarchy;
- familiar actions;
- carefully restrained depth and motion;
- customisable accents;
- complete high contrast and reduced motion;
- text labels for important actions.

The existing visual specification remains authoritative until changed by a usability test or explicit project decision.

## 6. Speed as user experience

The market study proposes preliminary product targets that have not yet been measured:

| Experience | Research hypothesis |
|---|---|
| First visible response to a click | p95 < 100 ms |
| Ordinary local search | p95 < 200 ms |
| Usable after sleep | p95 < 2 s |
| Opening a small local document | p95 < 1 s |
| 60 Hz animation/scrolling | around 16.7 ms per frame |

These values are **not current Baryon OS performance** or accepted repository criteria. They may later be adopted as measurement budgets when real native workflows and physical reference hardware exist.

## 7. Initial audience as a hypothesis to test

A logical first research group is:

> independently deciding home users who value privacy and control and use their computers mainly for web services, administration, documents, photos and video calls, without an indispensable application outside the supported package.

This is not yet an established market segment.

Selection must be based on actual tasks, including occasional but critical tasks such as tax returns, specific peripherals or an educational assessment.

Heavy gaming, specialist professional software and broad enterprise deployment belong in the initial commercial scope only once their concrete dependencies demonstrably work.

## 8. Switching must become a product function

The recommended introduction route consists of:

1. **Suitability check** — inventory apps, websites, files and devices.
2. **Personal compatibility overview** — works / works with limitations / alternative / unsupported / untested.
3. **Trial** — perform personal tasks without immediately removing the existing system.
4. **Migration** — controlled transfer of data and settings within the supported scope.
5. **Follow-up support** — assistance with the first real blockers and recovery questions.

Users must know beforehand whether their indispensable tasks work. A personal task list is more useful for this than a general claim such as “95% compatible”.

## 9. Measuring sustained switching

Attention, downloads or a successful demonstration are insufficient.

A useful research definition of sustained choice is:

> After 90 days, a person voluntarily uses Baryon OS as the primary environment for the agreed personal task set, can perform all indispensable tasks and has not needed to return to the old OS because of a blocker during the last 30 days.

Repeat the assessment at 180 days.

Measure:

- proportion of candidates suitable for the supported profile;
- successful migrations;
- primary use after 90 and 180 days;
- reasons for returning;
- unaided task success;
- support time and costs;
- successful recovery trials.

A proposed threshold such as 75% retention in a pilot is a **project hypothesis**, not an industry standard.

## 10. Recommended research programme

### Phase A — understand real tasks

Start with task interviews and observations in the Netherlands, Germany and France. Investigate recent examples of searching, recovering, sharing, switching and using essential apps/devices.

### Phase B — compare Materiaal

Test a coherent flow:

**retrieve → edit → recover → grant limited access**

Measure task duration, errors, assistance, understanding, appeal and repetition after practice.

### Phase C — messaging and switching intention

Compare honest positioning statements, for example:

- “Find your material and continue where you left off.”
- “See what changes and keep control of access.”
- “Keep your work organised and choose when to receive help.”

Measure independent trials and voluntary returns as well as click intention.

### Phase D — native pilot

Once the browser, hardware, updates, recovery and agreed daily tasks actually work, conduct a supported pilot on bounded reference hardware and measure 90/180 days.

## 11. Implications for the development roadmap

The research changes no existing native score, but gives priority to the following product gates:

1. maintainable browser and daily web tasks;
2. one convincing native Materiaal flow;
3. large/realistic documents and reliable recovery;
4. installation, migration, backup and key recovery;
5. compositor/input/accessibility with measured responsiveness;
6. reference hardware and peripherals;
7. app/SDK route for useful daily software;
8. a multiyear update and support model.

The project's own kernel remains the architectural choice. Market validation comes only when people voluntarily continue using the system for their real tasks.

## 12. References

### Market, policy and European alternatives

- Statcounter. *Desktop Operating System Market Share Europe*. https://gs.statcounter.com/os-market-share/desktop/europe
- Statcounter. *FAQ / methodology*. https://gs.statcounter.com/faq
- European Commission. *Digital Decade 2026: Special Eurobarometer*. https://digital-strategy.ec.europa.eu/en/library/digital-decade-2026-special-eurobarometer
- Verbraucherzentrale Bundesverband. (2025). *Befragung Ende Windows 10*. https://www.vzbv.de/sites/default/files/2025-12/Tabellenband_vzbv_Befragung_EndeWindows10_2025.pdf
- Zorin Group. (2026). *Zorin OS 18.1 is released*. https://blog.zorin.com/2026/04/15/zorin-os-18.1-is-released/
- TUXEDO Computers. *TUXEDO OS*. https://www.tuxedocomputers.com/en/TUXEDO-OS_1.tuxedo
- Linux Mint. *About Linux Mint*. https://www.linuxmint.com/about.php
- KDE e.V. *What is KDE e.V.?* https://ev.kde.org/whatiskdeev/
- EU OS. *FAQ*. https://eu-os.eu/faq/
- Interoperable Europe / OSOR. *Schleswig-Holstein's open source strategy* and LiMux retrospective. https://interoperable-europe.ec.europa.eu/
- Valve. *Steamworks — Proton*. https://partner.steamgames.com/doc/steamhardware/proton
- Chromium Project. *Supported Platforms*. https://chromium.googlesource.com/chromium/src/+/HEAD/docs/supported_platforms.md

### Design and accessibility

- Apple. *Human Interface Guidelines — Materials*. https://developer.apple.com/design/human-interface-guidelines/materials
- Microsoft. *Mica material*. https://learn.microsoft.com/en-us/windows/apps/design/style/mica
- Google Design. *Material 3 Expressive research*. https://design.google/library/expressive-material-design-google-research
- GNOME. *Human Interface Guidelines — UI styling*. https://developer.gnome.org/hig/guidelines/ui-styling.html
- W3C. (2024). *Web Content Accessibility Guidelines (WCAG) 2.2*. https://www.w3.org/TR/WCAG22/
- Google web.dev. *RAIL performance model*. https://web.dev/articles/rail

### Consumer adoption and user experience

- Venkatesh, V., Thong, J. Y. L., & Xu, X. (2012). *Consumer acceptance and use of information technology: Extending the unified theory of acceptance and use of technology*. MIS Quarterly, 36(1). https://aisel.aisnet.org/misq/vol36/iss1/13/
- Bhattacherjee, A. (2001). *Understanding information systems continuance*. MIS Quarterly, 25(3). https://aisel.aisnet.org/misq/vol25/iss3/2/
- Farhang, S., et al. (2018). Windows upgrade/adoption research. https://www.cs.cit.tum.de/fileadmin/w00cfj/ct/papers/2018-ACSAC-Farhang.pdf
- Grzybowski, L., & Nicolle, A. (2021). Research into smartphone OS switching. https://onlinelibrary.wiley.com/doi/10.1111/joie.12239
- Dumais, S., et al. (2003). *Stuff I've Seen*. https://www.microsoft.com/en-us/research/wp-content/uploads/2003/01/siscore-sigir2003-final.pdf
- Bergman, O., et al. (2013). Folder- versus tag-based retrieval. https://asistdl.onlinelibrary.wiley.com/doi/10.1002/asi.22906
- Boardman, R., & Sasse, M. A. (2004). Personal information management. https://discovery.ucl.ac.uk/id/eprint/13438/1/chi2004.pdf
- Tsai, J. Y., et al. Privacy information and purchasing behaviour. https://pubsonline.informs.org/doi/10.1287/isre.1090.0260
- Beresford, A. R., et al. Privacy and price in actual purchasing behaviour. https://www.sciencedirect.com/science/article/pii/S0165176512002182
- Fitz, N., et al. (2019). Notification batching. https://www.sciencedirect.com/science/article/pii/S0747563219302596
- Dietvorst, B. J., Simmons, J. P., & Massey, C. Correctability and algorithm aversion. https://faculty.wharton.upenn.edu/wp-content/uploads/2016/08/Dietvorst-Simmons-Massey-2018.pdf
- Vaccaro, M., et al. (2024). Meta-analysis of human–AI collaboration. https://pmc.ncbi.nlm.nih.gov/articles/PMC11659167/

## Evidence boundary

The sources substantiate market observations and research hypotheses. They establish no current Baryon OS compatibility, performance, accessibility, security, market share or product–market fit. Such claims require source-bound native execution and project user research.
