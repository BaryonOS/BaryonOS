<a id="os-ontwerp-20112026-en-onderzoekshorizon-20312036"></a>
# OS design 2011–2026 and research horizon 2031–2036

**Main conclusion — analysis:** forward-looking OS design becomes useful primarily through coherence between tasks, input methods, devices and user control; a new visual style is not in itself evidence of better usability.

**Status:** phase 4, desk research, accessed on **2026-11-02**. No user test of nieuw-os, product decision or implementation. Product vision, selection and technical development belong to phases 5–8. The twenty-one DT source records below separate product documentation, standards, empirical research and institutional reporting.

<a id="vragen-en-zoekstrategie"></a>
## Questions and search strategy

Research questions: which interaction patterns have changed since 2011; what supports task execution; which claims are primarily aesthetic; which developments warrant research over the next five to ten years?

Targeted searches covered original release announcements and guidelines from Microsoft, Apple, Android and GNOME, W3C documents and original studies on notifications, readability and AI interaction. Search terms included the product name plus `release`, `design`, `adaptive`, `privacy dashboard`, `reduced motion`, and the exact research titles. Historical announcements were distinguished from availability. Dates were checked on the source page; for GNOME, also in the official release calendar. No systematic literature review or representative inventory of all OS variants was conducted.

**Evidence limit:** manufacturer documentation establishes that a feature is described or announced, not that it makes people faster or more satisfied. The studies have different populations and outcome measures. They provide no market-wide ranking or specific evidence for European nieuw-os users.

<a id="tijdlijn-gedateerde-feiten-geen-verhaal-van-onvermijdelijke-vooruitgang"></a>
## Timeline: dated facts, no narrative of inevitable progress

| Date and status | Documented development | Research implication — analysis |
|---|---|---|
| 2011-06-01 publication / 06-02 Computex demonstration | Windows 8 preview targets touch devices alongside existing PC functionality. [DT01] | One system acquires different input contexts; identical controls are not automatically suitable controls. |
| 2013-06-10, announcement | iOS 7 simplifies typography and visual design, but also uses translucency, layers and motion; Control Center groups quick settings. [DT02] | The shift towards flat design did not mean completely abandoning depth. |
| 2014-06-25, announcement | Material Design introduces coherent surfaces, elevation, shadows and motion. [DT03] | A design system can explain relationships; decorative shadows alone do not. |
| 2014-10-16, Yosemite available | Handoff, cross-device AirDrop, Today widgets and expanded Spotlight; some Continuity features require iOS 8.1 and suitable hardware. [DT04] | Task continuity becomes part of the desktop experience, with ecosystem requirements. |
| 2017-05-11, announcement | Microsoft presents Fluent for different devices and input through touch, pen, voice and gaze, among other methods. [DT05] | The design question shifts towards consistent meaning across different interactions. |
| 2021-03-24, release | GNOME 40 arranges workspaces horizontally, with corresponding keyboard, mouse and gesture controls. [DT07] | Spatial coherence and alternative input are more concrete research units than animation style. |
| 2021-06-24, announcement | Windows 11 describes Snap Layouts, Snap Groups and separate Desktops. [DT08] | Window organisation is offered more visibly; this does not mean these patterns were invented then. |
| 2021-10-19, Pixel rollout | Android 12 links system colours to wallpaper, updates widgets and offers a privacy dashboard and camera/microphone indicators. [DT06] | Personalisation and visible data access become system features. |
| 2023-06-05, announcement | Vision Pro/visionOS introduces Apple's spatial windows with eye, hand and voice controls. [DT09] | New hardware enables different interactions; desktop replacement is not established by this. |
| 2023-10-05, recommendation | WCAG 2.2 is published, including additional criteria for focus and target size; updated version consulted: 2024-12-12. [DT10] | Accessibility gains testable requirements alongside visual preferences. |
| 2024-06-10, announcement | Apple Intelligence connects generative features with personal context and both local and server processing. [DT11] | AI is integrated into existing tasks; this announcement does not establish complete subsequent delivery of all announced features. |
| 2025-06-09, announcement | macOS Tahoe combines Liquid Glass, Continuity/Live Activities and Spotlight actions. [DT13] | Search develops further towards action; glass is a separate visual layer. |
| 2026-03-18, release | GNOME 50 adds Reduced Motion and improves Orca, including global settings and language switching. [DT14] | New releases continue to improve fundamental accessibility. This is a milestone, not a claim that 50 is the latest GNOME version. |

<a id="welke-veranderingen-zijn-functioneel"></a>
## Which changes are functional?

The assessments below are **analysis**, based on the cited sources; they are not already selected nieuw-os features.

| Theme | Possible practical effect | Limitation or risk |
|---|---|---|
| Flat and Material | Quiet typography, recognisable groups and consistent layers can support a clear hierarchy. [DT02] [DT03] | Less ornament is not a measured task improvement. Controls that have become invisible require separate investigation. |
| Glass and translucency | Foreground controls and underlying content can be visually distinguished. Apple's own guideline reserves glass for navigation, warns against stacked glass and offers reduced transparency, increased contrast and reduced motion. [DT12] | Background-dependent contrast, visual noise and rendering costs require measurement; a glass effect does not establish modernity. |
| Dynamic colour and dark mode | Colour personalisation and light/dark modes can support preferences and usage conditions. [DT06] [DT18] | An accent colour must not obscure the meaning of errors or focus. Dark mode is not universally more readable or energy-efficient. [DT10] [DT18] [DT19] |
| Touch, gestures and adaptive layouts | GNOME links gestures to alternative keyboard/mouse actions. Android recommends adapting to available width, height, input and posture, through rearranging, revealing and other presentation forms. [DT07] [DT15] | A scaled-down desktop is not yet a mobile interface. Hidden gestures and drag-only controls do not provide access for everyone. [DT10] [DT20] |
| Command palettes and universal search | A shared entry point for discoverable objects and explicit actions can reduce navigation steps; Spotlight demonstrates such actions. [DT13] | This is a research hypothesis for nieuw-os: result coverage, permissions, predictability and discoverability must be tested. A command palette does not need a language model. |
| Virtual desktops, snapping and tiling | Workspaces and window layouts offer concrete ways to group tasks and compare multiple sources. [DT07] [DT08] | The sources do not establish the general superiority of automatic tiling. Small windows, focus order, restoration and beginner understanding remain test questions. |
| Widgets, context and ambient computing | Widgets and Live Activities make status available without opening the full app. [DT04] [DT13] | Here, ambient means making information available in the background. Permanently moving or profiling information can instead demand attention; relevance must be measurable. |
| AI and natural language | Tasks can be supported from context; guidelines emphasise understandable boundaries, correction, dismissal and global control. [DT11] [DT16] | Free-form language is ambiguous. Local execution alone does not prevent incorrect actions, excessive permissions or unclear data processing. |
| Continuity and privacy dashboards | Handoff enables task transfer; Android shows recent sensor use and offers permission management from the overview. [DT04] [DT06] | Integration within one ecosystem does not establish open interoperability. A dashboard is useful only if its display matches actual permissions. |
| Spatial computing | Spatial placement and multimodal input offer a different workspace. [DT09] | Hardware, ergonomics and accessible alternatives are prerequisites. No comparative evidence for productivity gains on an ordinary desktop was collected here. |

**Patterns to examine critically:** designing exclusively for fixed screen sizes, hiding essential controls behind one gesture, aesthetics without contrast checks and AI without a recovery path conflict with current adaptive, accessibility and AI guidelines. [DT10] [DT15] [DT16] [DT20] This does not make them established failures across the market. Glass, dark mode and virtual desktops are likewise not inherently obsolete or future-proof: task, implementation and target group determine their value.

<a id="empirisch-bewijs-en-beperkingen"></a>
## Empirical evidence and limitations

1. **Notifications — Fitz et al.** A randomised field study with 237 smartphone users in India lasted two weeks. Three daily batches were associated with less reported stress and greater perceived control than usual notifications. Hourly batching produced little change; complete suppression increased anxiety and FoMO. [DT17] **Interpretation:** timing and predictability warrant research, not only an on/off button. **Limit:** Android, self-reports and one sample; no evidence for a universal three-times-daily setting or European desktops. The abstract is internally contradictory about productivity; no effect claim on that subject is adopted here.

2. **Contrasting finding on notifications — Dekker et al.** A preregistered RCT analysed 205 Android users aged 18–30 after a baseline week and one intervention week. Disabling notifications had no statistically significant effect on screen time, checking frequency, perceived control or distraction; less perceived habitual behaviour and more FoMO were reported. [DT21] The productivity result contains an interaction due to a decline in the control group, so no simple claim of a null result follows. **Limit:** short duration, age selection and exclusion for insufficient adherence (at least 75% notification reduction). **Interpretation:** disabling differs from batching; this study does not refute Fitz's batching result, but neither does it support the general assumption that fewer notifications automatically improve well-being. No significant effect is not evidence of the absence of any effect.

3. **Readability — Piepenbrock et al.** The original abstract reports better visual acuity and proofreading performance with dark characters on a light background, in both younger and older participants. [DT18] **Interpretation:** dark mode must remain a choice subject to investigation. **Limit:** abstract checked, full methods not consulted; specific tasks, no evidence that all users or environments favour light presentation.

4. **AI control — Amershi et al.** Eighteen interaction guidelines were developed iteratively and evaluated, among other methods, by 49 design/HCI practitioners on twenty AI products. They include correction, uncertainty, dismissal and control over system behaviour. [DT16] **Interpretation:** useful test questions for possible OS automation. **Limit:** guideline validation, not a randomised effect measurement among end users or an evaluation of current generative OS agents.

5. **Energy — Dash and Hu.** Purdue describes measurements of six apps on four OLED phones; savings from dark presentation depend strongly on brightness and device. [DT19] **Limit:** the original ACM article was not directly accessible; the institutional explanation and author attribution were checked. Therefore no battery percentages are adopted and no extrapolation is made to LCD, QEMU or nieuw-os energy use.

Motion is supported here by accessibility guidelines, not by a newly collected experimental study. WCAG 2.2 distinguishes text contrast, keyboard operation, focus, alternatives to dragging and target size, among other criteria. Disabling non-essential interaction animation is criterion 2.3.3 at **AAA**, not AA. [DT10] WCAG2ICT provides informative application to non-web software; this document claims no native OS conformance. [DT20]

<a id="horizon-20312036-expliciete-hypotheses"></a>
## Horizon 2031–2036: explicit hypotheses

These are **research hypotheses**, not forecasts, product promises or implementation decisions.

| Hypothesis | Why plausible | How to test or refute it? |
|---|---|---|
| Combined object/action controls continue to grow alongside windows. | Spotlight actions and contextual AI exist as product directions. [DT11] [DT13] | Compare task duration, errors and discoverability with menus; reject when learning and correcting cost more than navigating. |
| Local AI becomes more useful for bounded tasks, with ordinary controls as a recovery route. | Local processing is already part of product architectures; control and correction are researched guidelines. [DT11] [DT16] | Measure result quality, latency, memory, energy, offline operation and correction costs on target hardware. Do not assume cloud fallback. |
| Adaptivity shifts from device category to task space and input. | Android describes adaptation at component/pane level; GNOME combines input methods. [DT07] [DT15] | Test the same task at multiple scales, window sizes and input methods, including assistive technology. |
| Attention and energy become explicit design budgets. | Notification effects differ by intervention; display energy is context-dependent. [DT17] [DT21] [DT19] | Measure interruptions and recovery duration, plus actual energy use; CPU idle is not a battery measurement. |
| Continuity becomes more important than one identical interface on every device. | Task transfer and Live Activities connect different interfaces. [DT04] [DT13] | Investigate resumption with offline devices, revoked permissions and alternative providers. Open interoperability does not follow automatically from current ecosystems. |
| Spatial remains task-dependent alongside conventional screens. | A spatial platform exists, but these sources do not establish general replacement. [DT09] | Compare specific spatial tasks on comfort, accessibility, cost and performance; keep rejection open as a possible outcome. |

<a id="afzonderlijke-repository-inferentie"></a>
## Separate repository inference

This connection uses the phase-1 source inspection; external trends do not establish that nieuw-os works. The research worktree is on main `e08fb0e48c1bb0781e98357f588c2f73b6ae8500`. The development source examined separately is pinned at `1f0dd9c93959e3b1335656da1e548525a288133e`; this content is not integrated into the research worktree.

The [native UI on main](https://github.com/simonvanieperen/nieuw-os/blob/e08fb0e48c1bb0781e98357f588c2f73b6ae8500/ui/native/README.md) is an independent Rust path; host rendering is not evidence of native system features. The newer [window session](https://github.com/simonvanieperen/nieuw-os/blob/1f0dd9c93959e3b1335656da1e548525a288133e/sdk/examples/window-session/src/scene.rs) and [U01 acceptance report](https://github.com/simonvanieperen/nieuw-os/blob/1f0dd9c93959e3b1335656da1e548525a288133e/docs/verification/mature-session-20261102/U01-ACCEPTANCE.md) do not make that older baseline current. Their bounded native evidence must likewise not be broadened into full desktop maturity.

Research questions for the next phases:

- How do adaptive layouts relate to the current logical minimum size and discrete scales in the window session?
- Which semantics, permissions and result coverage are missing between the bounded [app-local search](https://github.com/simonvanieperen/nieuw-os/blob/1f0dd9c93959e3b1335656da1e548525a288133e/ui/native/src/desktop.rs) and task-oriented system search?
- What export and assistive integration are needed on top of the [app-local semantics](https://github.com/simonvanieperen/nieuw-os/blob/1f0dd9c93959e3b1335656da1e548525a288133e/ui/native/README.md), and how do the current [font/Unicode boundaries](https://github.com/simonvanieperen/nieuw-os/blob/1f0dd9c93959e3b1335656da1e548525a288133e/ui/graphics/README.md) limit European multilingualism?
- What measurable task improvement justifies extra context, motion or automation within the existing [privacy contract](https://github.com/simonvanieperen/nieuw-os/blob/e08fb0e48c1bb0781e98357f588c2f73b6ae8500/docs/LOCAL-PRIVACY-CONTRACT.md)? Apple's hybrid AI architecture is not a precedent for cloud inference in nieuw-os.

<a id="bronnenregister"></a>
## Source register

All records: **access date 2026-11-02**. A = primary announcement/release; G = manufacturer guide; N = standard/guideline; E = original research; I = report by the institution conducting the research. Links are provided directly with the titles through the DT references.

| ID / title | Author, publication date; type | Supported claim and limitation |
|---|---|---|
| [DT01 — Microsoft Previews Windows 8][DT01] | Microsoft; 2011-06-01, event 06-02; A | Touch/PC preview; no measurement of effects on use. |
| [DT02 — Apple Unveils iOS 7][DT02] | Apple; 2013-06-10; A | Visual design, layers, Control Center; marketing claims not adopted. |
| [DT03 — Google I/O: Design, Develop, Distribute][DT03] | Monica Tran / Google; 2014-06-25; A | Introduction of Material/elevation; no comparative UX evidence. |
| [DT04 — OS X Yosemite Available Today as a Free Upgrade][DT04] | Apple; 2014-10-16; A | Continuity, widgets, Spotlight; ecosystem/hardware requirements. |
| [DT05 — Build 2017: Sparking the next wave of creativity][DT05] | Terry Myerson / Microsoft; 2017-05-11; A | Fluent and input diversity; historical announcement. |
| [DT06 — A dozen things to love in Android 12][DT06] | Dave Burke / Google; 2021-10-19; A | Dynamic color, widgets, privacy dashboard; Pixel rollout, no universal OEM parity. |
| [DT07 — Introducing GNOME 40][DT07] | GNOME; 2021-03-24; A | Workspaces and input alternatives; date from [release calendar](https://release.gnome.org/calendar/), no comparative task study. |
| [DT08 — Introducing Windows 11][DT08] | Panos Panay / Microsoft; 2021-06-24; A | Snapping/grouping/desktops; not their historical origin. |
| [DT09 — Introducing Apple Vision Pro][DT09] | Apple; 2023-06-05; A | Spatial interface; no evidence for desktop replacement. |
| [DT10 — WCAG 2.2][DT10] | W3C AG Working Group; first 2023-10-05, version consulted 2024-12-12; N | Testable web criteria; not automatically native conformance. |
| [DT11 — Introducing Apple Intelligence][DT11] | Apple; 2024-06-10; A | Context and local/server models; announcement, no complete audit of current availability. |
| [DT12 — Meet Liquid Glass][DT12] | Apple Developer; WWDC 2025, day not stated on page; G | Hierarchy, no stacked glass, accessibility variants; manufacturer guide, no effect study. |
| [DT13 — macOS Tahoe 26 makes the Mac more capable…][DT13] | Apple; 2025-06-09; A | Spotlight actions, Liquid Glass, Continuity; product claims not treated as effect measurements. |
| [DT14 — Introducing GNOME 50][DT14] | GNOME; 2026-03-18; A | Reduced Motion and Orca; date from official release calendar; no user study. |
| [DT15 — Adapt layouts][DT15] | Android Developers / Google; updated 2026-10-22; G | Window space, input, pane adaptation; platform-specific guideline. |
| [DT16 — Guidelines for Human-AI Interaction][DT16] | Saleema Amershi et al.; CHI 2019, 2019-05-04–09; E | Eighteen guidelines, 49 practitioners; no end-user effect trial. |
| [DT17 — Batching smartphone notifications can improve well-being][DT17] | Nicholas Fitz, Kostadin Kushlev, Ranjan Jagannathan, Terrel Lewis, Devang Paliwal, Dan Ariely; online 2019-07-16, journal 2019-12; E | RCT/timing; India, short duration, self-report; [original text](https://static1.squarespace.com/static/57a40c19414fb54f51f8095f/t/614a55faa7b89e25f4e48ad1/1632261627146/2019%2BFitz%2BBatching.pdf) read. |
| [DT18 — Positive display polarity is advantageous for both younger and older adults][DT18] | Cosima Piepenbrock, Susanne Mayr, Iris Mund, Axel Buchner; online 2013-05-08; E | Display polarity; only original abstract checked. |
| [DT19 — Dark mode may not save your phone’s battery life as much as you think…][DT19] | Kayla Wiles / Purdue; 2021-07-28; I | Institution's own research by Dash/Hu, [MobiSys paper](https://doi.org/10.1145/3458864.3467682); full text blocked, limited to context dependence. |
| [DT20 — Guidance on Applying WCAG 2 to Non-Web ICT][DT20] | W3C; Mary Jo Mueller, Phil Day, Daniel Montalvo (eds.); 2025-12-11; N | Application to software; informative Group Note, no independent normative obligation. |
| [DT21 — Beyond the Buzz: Investigating the Effects of a Notification-Disabling Intervention on Smartphone Behavior and Digital Well-Being][DT21] | Cynthia A. Dekker, Susanne E. Baumgartner, Sindy R. Sumter, Jakob Ohme; online 2024-03-28, journal 2025; E | Disabling intervention; abstract, method and results read; short duration, age/adherence selection, no batching trial. |

[DT01]: https://news.microsoft.com/source/2011/06/01/microsoft-previews-windows-8/
[DT02]: https://www.apple.com/newsroom/2013/06/10Apple-Unveils-iOS-7/
[DT03]: https://android-developers.googleblog.com/2014/06/google-io-design-develop-distribute.html
[DT04]: https://www.apple.com/newsroom/2014/10/16OS-X-Yosemite-Available-Today-as-a-Free-Upgrade/
[DT05]: https://blogs.windows.com/windows-insider/2017/05/11/build-2017-sparking-the-next-wave-of-creativity-with-the-windows-10-fall-creators-update/
[DT06]: https://blog.google/products-and-platforms/platforms/android/android-12/
[DT07]: https://release.gnome.org/40/
[DT08]: https://blogs.windows.com/windowsexperience/2021/06/24/introducing-windows-11/
[DT09]: https://www.apple.com/newsroom/2023/06/introducing-apple-vision-pro/
[DT10]: https://www.w3.org/TR/2024/REC-WCAG22-20241212/
[DT11]: https://www.apple.com/newsroom/2024/06/introducing-apple-intelligence-for-iphone-ipad-and-mac/
[DT12]: https://developer.apple.com/videos/play/wwdc2025/219/
[DT13]: https://www.apple.com/newsroom/2025/06/macos-tahoe-26-makes-the-mac-more-capable-productive-and-intelligent-than-ever/
[DT14]: https://release.gnome.org/50/
[DT15]: https://developer.android.com/design/ui/mobile/guides/layout-and-content/adapt-layout
[DT16]: https://www.microsoft.com/en-us/research/wp-content/uploads/2019/01/Guidelines-for-Human-AI-Interaction-camera-ready.pdf
[DT17]: https://doi.org/10.1016/j.chb.2019.07.016
[DT18]: https://pubmed.ncbi.nlm.nih.gov/23654206/
[DT19]: https://www.purdue.edu/newsroom/archive/releases/2021/Q3/dark-mode-may-not-save-your-phones-battery-life-as-much-as-you-think,-but-there-are-a-few-silver-linings.html
[DT20]: https://www.w3.org/TR/2025/NOTE-wcag2ict-22-20251211/
[DT21]: https://pure.uva.nl/ws/files/245971989/Beyond_the_Buzz.pdf
