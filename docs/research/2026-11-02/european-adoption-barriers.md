# European adoption and switching barriers — phase 3

Research date: 2 November 2026 (Europe/Amsterdam). Status: literature and source research; no original participants, market forecast, or implementation decision. Method and evidence labels: [README](README.md). Sources EA-01–EA-14: [sources.md](sources.md).

## Main finding

[ANALYSIS] European origin may support interest, but it does not replace functional tasks, suitable applications and devices, comprehensible migration, and sustained support. The reviewed sources measure different outcomes: general preference, stated willingness, concrete compatibility, and actual or intended switching. These outcomes must not be combined into a single adoption percentage.

<a id="oorspronkelijke-eurobarometer-gecontroleerd"></a>

## Original Eurobarometer checked

[SOURCE] The Commission published *Special Eurobarometer 572 — Digital Decade* on 17 June 2026. The report lists 26,494 respondents; the technical annex describes fieldwork from 5 February to 1 March 2026 among EU nationals aged 15 and older who lived in the 27 member states. Samples were stratified and multistage; national and EU results were weighted. Interviews were conducted in person or, in some countries, by video. This was not a sample restricted to desktop users. [EA-01] [EA-02]

The original bilingual master questionnaire contains QB6/ST1045, a question about agreement with statements. **QB6.4/ST1045 item 4** reads exactly:

> You, personally, would be willing to switch to an EU-based digital service provider even if it means slightly higher costs

[SOURCE] The response options range from totally agree and tend to agree to tend to disagree and totally disagree, plus do not know. The text above is the exact statement; the introductory question asks for the degree of agreement. Location: master-questionnaire PDF page 20 (printed page 18); original data annex, printed page 21/PDF page 23. [EA-03] [EA-04]

| EU27, QB6.4 | Percentage |
|---|---:|
| Totally agree | 21 |
| Tend to agree | 37 |
| Tend to disagree | 20 |
| Totally disagree | 14 |
| Do not know | 8 |
| Agree, total | **58** |
| Disagree, total | **34** |

[SOURCE] This table was read in the original annex and checked visually. The report confirms 58% on page 44. The question and the EU27 result are mutually consistent. The annex does contain a country-label discrepancy: QB6.4 shows `CY_TCC`, while report page 7 describes `CY` for the Cypriot population included in EU27. This discrepancy was not corrected independently, and no Cypriot country comparison is derived from it. Download routes, page numbers, hashes, and the calculation check are recorded in [eurobarometer-check.json](eurobarometer-check.json). [EA-01] [EA-02] [EA-03] [EA-04]

[ANALYSIS] The supported formulation is: **58% agree that they would be willing to switch to an EU-based digital service provider, even at slightly higher cost.** The question does not mention an operating system, a specific provider, an amount, actual payment, or a migration action. It therefore does not establish:

- that 58% of European computer users would install a new operating system;
- that 58% actually paid more or would accept a particular price;
- that a European provider automatically offers better privacy or security;
- that interest leads to trial, switching, or sustained use.

The claim **“56% would switch to a European operating system”** is not supported by the reviewed original sources and is not used. This does not establish that such a figure exists nowhere; no verifiable primary operating-system question was found here.

The same survey asks in QB7 about reasons for switching services. Aggregated responses list security/reliability (50%), personal data (49%), consumer protection (39%), reduced dependence (33%), and the EU economy (30%); multiple responses were possible. These are offered, hypothetical reasons for switching services, not independently discovered causes of operating-system migration. [EA-02] (p. 48)

## Different types of willingness and behaviour

| Outcome | Evidence found | Must not be inferred |
|---|---|---|
| Preference for European digital services | Eurobarometer statement and policy preferences | A question about service preference is not an operating-system choice |
| Willingness to pay slightly more | Agreement with unspecified higher costs | No price threshold, purchase, or observed payment |
| Changing operating system | Other sources tied to specific populations, such as vzbv and CMA | No EU-wide switching percentage for Baryon OS |
| Organisational migration | Concrete programmes with existing software, support, and exceptions | No evidence that consumers do the same |
| Sustained use | No Baryon OS field research found in the project baseline | Downloads, intention, and a task trial do not demonstrate retention |

## Why dissatisfaction is insufficient

[SOURCE] An original vzbv/eye square online-panel survey of adult current or recent Windows 10 users in Germany (6–17 November 2025; weighted by age, gender, and education) reports 46% who had switched or intended to switch to Windows 11, 34% who said they would continue using Windows 10 even when updates were no longer available, and 4% who had switched or intended to switch to another operating system. F2 lists **unweighted n=800 and weighted base 776**. This was not an EU-wide population sample; the response categories combine behaviour with intention. The 34% does not measure observed unprotected use, and the 4% does not measure only completed switching. [EA-05] (method sheet and F2)

[SOURCE] The CMA's 2022 research distinguishes satisfaction and brand preference from switching costs: learning, transferring data and applications, subscriptions, and existing devices and services. It also warns about selection: people who actually switch may be more proficient than those who avoid doing so. In October 2025, the CMA again confirmed limited switching pressure and strong mobile ecosystems. This concerns the United Kingdom and mobile platforms, not a measured causal model for European desktops. [EA-06] [EA-07]

<a id="barrièrematrix"></a>

## Barrier matrix

Evidence strength concerns the existence or mechanism of a barrier, not its frequency among all users. Impact and the consequence for Baryon OS are [ANALYSIS].

| Barrier | Verifiable mechanism / evidence | Impact and consequence for Baryon OS |
|---|---|---|
| Habit and learning curve | CMA: expected effort, different controls, and selection among switchers [EA-06] | Retain familiar basic actions and a real trial task; do not reinvent every convention |
| Necessary software | Adobe documents Windows and macOS as supported desktop platforms [EA-08] | An independent kernel does not inherit application compatibility; each task requires a demonstrably functional route |
| Professional and creative workflows | Adobe's desktop requirements do not automatically cover plug-ins, fonts, codecs, and interchange [EA-08] | A similar application name or import function does not replace a production workflow |
| Office and file formats | LibreOffice can open and save Office formats, but VBA object models and support differ [EA-09] | Test format, macro, layout, and round trips separately; make no general Office-parity claim |
| Games and anti-cheat | Valve: title-specific EAC/BattlEye configuration; kernel anti-cheat is not supported by Proton [EA-10] | Establish compatibility per game and release. Proton is not a ready-made layer for the custom kernel |
| Drivers and peripherals | Baryon OS has a bounded virtual profile; physical support remains open [RB-01] | Test the exact GPU, Wi-Fi, printer, scanner, audio, dock, and sleep/wake behaviour in advance |
| Phone and cloud integration | CMA: first-party devices and shared functions increase switching costs [EA-06]; the EU describes portability improvements [EA-11] | Loss of messages, calendars, photographs, or keys can block migration; test the entire chain |
| Data migration | The EU factsheet acknowledges incomplete transfers; new routes and APIs lower specific barriers [EA-11] | Provide trial import, a completeness report, source preservation, and recovery; do not claim “everything transferred” without verification |
| Organisational policy and IT management | EU OS targets its proof of concept at central management; Schleswig-Holstein retains technical exceptions [EA-12] [EA-13] | Identity, management, updates, specialised software, and help-desk support are separate product prerequisites |
| Trust and brand | Satisfaction and brand preference coexist with barriers [EA-06] | Demonstrate recovery, comprehensible limitations, and a visible support route; do not assume trust |
| Support and maintenance | The migration case includes investment and continued development [EA-13] | Explain long-term updates, responsibility, and funding in advance |
| Unclear benefit | Eurobarometer measures reasons for switching services; the Windows survey shows no mass departure [EA-02] [EA-05] | Combine a demonstrably better everyday task with retention of necessary functions |
| Network and ecosystem effects | CMA: applications, users, and complementary devices reinforce existing platforms [EA-06] [EA-07] | Developers and collaboration partners must also be able to participate; one attractive shell is insufficient |
| Fear of technical complexity | Expected learning effort has been studied; specific “fear of Linux” was not measured representatively here [EA-06] | Do not make a CLI the normal recovery route; test concrete concerns and support needs in original project interviews |
| Fragmentation and inconsistency | Linux/mobile source exploration describes different package and portal routes and manufacturer-dependent functions [LM-06/08/12 in the register](linux-mobile-needs.md#bronregister) | Provide one visible system contract, with choice but without divergent basic controls; the effect on switching remains a hypothesis |
| Unclear revenue model | No payment or trust test was found for Baryon OS | Test maintenance and funding hypotheses explicitly; do not infer willingness to subscribe from 58% |

## Why Europe does not yet show uniform operating-system switching

[SOURCE] EU OS describes itself as a community proof of concept for administrators, not an official EU project or a new consumer operating system. Zorin reports downloads and helps Windows users find alternatives; downloads do not count sustained active users. These are different initiatives with different target groups. [EA-12] [EA-14]

[SOURCE] Schleswig-Holstein reported on 4 December 2025 that almost 80% of workstations **outside the tax administration** used LibreOffice and that approximately 20% still depended on Office for parts of their work because of specialised processes. This is an Office migration, not a demonstrated percentage of Linux desktops. The source is the implementing government, not an independent impact evaluation. [EA-13]

[ANALYSIS] The sources support the coexistence of European preference and existing dependencies involving applications, hardware, management, and collaboration. They do not establish one universal reason why all European operating-system alternatives remain limited. A representative, current comparison of choice, price, and actual desktop operating-system use across different EU countries is absent.

[SOURCE] The May 2026 EU factsheet describes both existing and announced portability and interoperability routes. It does not demonstrate universal availability on 2 November or functional integration with Baryon OS. Older migration barriers must therefore be retested for each current version and region. [EA-11]

## Combination of benefits that could justify switching

[HYPOTHESIS] The most defensible combination to test is: necessary tasks continue to work, Material is easier to find and resume, changes and errors are recoverable, data and permissions are comprehensible, and migration offers a safe return path with accessible support. European control and optional local AI may strengthen that combination; no source shows that either is sufficient alone.

Three test groups can be used in phases 5–6 as **research segments**, not invented personas:

1. People with browser and document tasks and limited system knowledge: observe finding, opening, sharing, saving, and recovering without assistance.
2. Users with professional applications, games, or specific tools: begin with strict compatibility blockers, not stylistic preferences.
3. Organisations and administrators: test identities, management, maintenance, interchange, training, and recovery alongside individual operation.

Acceptance for a later migration trial: agree on concrete applications, devices, and formats in advance; perform trial import without source loss; have participants perform tasks themselves; record errors, assistance, and reasons for returning; measure sustained voluntary use separately. This is a proposed test, not research already performed or a change to the existing product roadmap.

<a id="directe-bronverwijzingen"></a>

## Direct source references

[EA-01]: https://digital-strategy.ec.europa.eu/en/library/digital-decade-2026-special-eurobarometer
[EA-02]: https://data.europa.eu/doi/10.2759/3200731
[EA-03]: https://access.gesis.org/dbk/82384
[EA-04]: https://data.europa.eu/doi/10.2759/4731876
[EA-05]: https://www.vzbv.de/sites/default/files/2025-12/Tabellenband_vzbv_Befragung_EndeWindows10_2025.pdf
[EA-06]: https://assets.publishing.service.gov.uk/media/62a22d5bd3bf7f036750b0d8/Appendix_D__002_.pdf
[EA-07]: https://www.gov.uk/government/news/cma-confirms-apple-and-google-have-strategic-market-status-in-mobile-platforms
[EA-08]: https://helpx.adobe.com/download-install/apps/system-requirements/creative-cloud-desktop-app-system-requirements.html
[EA-09]: https://help.libreoffice.org/latest/en-US/text/shared/guide/ms_user.html
[EA-10]: https://partner.steamgames.com/doc/steamhardware/proton
[EA-11]: https://digital-markets-act.ec.europa.eu/factsheet-how-dma-making-smartphones-better-interoperability-and-data-portability-case-studies-2026-05-11_en
[EA-12]: https://eu-os.eu/faq/
[EA-13]: https://www.schleswig-holstein.de/DE/landesregierung/ministerien-behoerden/I/Presse/PI/2025/cds/251204_cds_open-source
[EA-14]: https://blog.zorin.com/2026/04/15/zorin-os-18.1-is-released/
