# Consumer value and differentiation — research addition, 28 October 2026

**Status:** targeted, critical literature exploration and design implications.

**Purpose:** substantiate which user value Baryon OS must test to become not only technically independent but also necessary and recognisable as a product.

**Evidence boundary:** this document contains literature findings and project hypotheses. It is not project user research, a market-share forecast, a formal systematic review or evidence that Materiaal causes consumers to switch.

## 1. Main conclusion

The most substantiated preliminary direction is a **noticeably better daily task with as little uncertainty as possible when trying and switching**, rather than an independent kernel as a consumer promise.

The following product promise is therefore used for investigation with Materiaal:

> **Find your material immediately, continue working with it and retain control over changes and access.**

The independent kernel, capability architecture, local AI and personal storage are important project choices. Their consumer value must become visible through behaviour that can be experienced: retrieving work, continuing safely, understanding changes, restricting access and recovering from errors.

The combination can provide a distinctive product experience without claiming that every individual principle is unprecedented.

## 2. Four different adoption phases

A positive first impression, actually trying, switching and continuing use are different outcomes and must not be treated as one measure.

| Phase | User question | What Baryon OS must demonstrate | First useful measure |
|---|---|---|---|
| Attention | Why should this attract attention? | Recognisable benefit and polished presentation | Understanding of the promise; voluntarily starting a demonstration |
| Trying | Is this worth the time? | One useful task with little preparation | Task success; assistance; abandonment |
| Switching | Can this take over current use? | Essential apps, files, devices and migration | Successful migration tasks; actual transferred usage |
| Staying | Will this be chosen again later? | Continuing usefulness, reliability and recovery | Voluntary reuse; return to the old way of working |

Information-systems continuance research supports the relevance of expectations, satisfaction and perceived usefulness after initial use to the intention to continue (Bhattacherjee, 2001). An impressive demonstration is therefore not equivalent to sustained product value.

## 3. What the evidence supports and does not support

### 3.1 Practical value, enjoyment and feasibility

UTAUT2 was studied among 1,512 mobile internet consumers and treats acceptance as a combination of factors including expected usefulness, required effort, enjoyment, price value, support and habit (Venkatesh et al., 2012). The model provides no proven ranking of features for a new desktop OS.

An older meta-analysis of innovation adoption found relative advantage, compatibility and complexity to be relatively consistent factors across varied innovations (Tornatzky & Klein, 1982). “Compatibility” must be interpreted more broadly here than binary Windows app compatibility: existing experience, working methods and expectations also count.

**Project implication:** demonstrate a concretely better task and measure what someone must learn to perform it.

### 3.2 Satisfied users need a reason to change

Farhang et al. (2018) studied Windows users around the transition to Windows 10. Satisfaction with the existing system was an important reported reason not to upgrade; privacy and compatibility also played a role.

In historical data on 9,799 device switches, Grzybowski and Nicolle (2021) found indications of persistent preferences and switching friction in smartphone OS choices. The study provides no current monetary or time value for desktop migration.

**Project implication:** first try Baryon OS alongside the existing system; show beforehand which files, programs and devices are supported, and keep export/return possible.

### 3.3 Innovation must connect to learned controls

Research on skill-based habits shows that acquired skills can reinforce preference for existing products (Murray & Häubl, 2007).

A 2025 desktop study compared Windows, macOS and Linux using tasks, eye measurements and the System Usability Scale among more than sixty analysed computer-science students, predominantly experienced with Windows. Subjective ratings and task performance did not always agree; familiarity is therefore an important possible explanation. The study does not determine which OS is universally best for consumers (Łazaruk et al., 2025).

**Project implication:** Materiaal may innovate in organisation, but selection, navigation, familiar shortcuts, names, folders and a clear route back must remain recognisable. Measure both first use and use after practice.

### 3.4 Design helps the first impression but does not replace usability

Tuch et al. (2012a) found that visual complexity and prototypicality influence rapid aesthetic judgements. Other research in the same line shows that attractive design does not automatically resolve a poorly usable experience (Tuch et al., 2012b).

Hassenzahl et al. (2010) analysed more than five hundred recollections of positive technology experiences. Competence, stimulation and relatedness, among other factors, were associated with positive feelings. These are correlations in selected positive memories and do not establish causal OS preference.

**Project implication:** the visual identity must make the product benefit quickly understandable and remain effective during searching, errors and recovery.

## 4. Promising differentiation for Materiaal

### 4.1 Retrieval through remembered context

Stuff I've Seen studied personal information retrieval among 234 employees over six weeks. Time and people proved useful cues alongside conventional location and file information (Dumais et al., 2003).

**Hypothesis:** Materiaal can offer local search entry points by title/type, collection, recent activity, time and other explicitly available context, without first requiring the user to know which app “owns” the object.

Retain existing names and folders at the same time. Bergman et al. (2013) found a clear preference for folders over tags in the populations studied; this does not establish that alternative organisation never works, but is a reason to preserve a familiar route.

### 4.2 Continuing without assembling everything again

Boardman and Sasse (2004) found substantial individual differences in how people organise information across tools. This cautions against one automatic all-in-one stream.

**Hypothesis:** provide coherence around user-selected material and task context without automatically merging all information sources.

Benefits to measure:

- less searching;
- less version confusion;
- fewer unintended copies;
- faster resumption after interruption.

### 4.3 History, recovery and access rights as one understandable experience

Version history, sandboxing and permissions exist elsewhere in different forms. The project opportunity lies in their **consistent connection around the same material**:

> This changed; this tool made the change; this version is confirmed as saved; this can be recovered; this access currently applies.

A consent screen alone is insufficient. Research into the historical Android permission model found limited attention to and understanding of installation permissions (Felt et al., 2012).

**Research question:** after an action, can the user correctly explain which tool may use which material, for which action, for how long and whether it has network access?

### 4.4 Privacy as observable behaviour

Privacy is not a guaranteed driver of switching. Tsai et al. (2011) found that prominent privacy information could influence purchasing behaviour; Beresford et al. (2012) found in another purchasing experiment that a price difference could outweigh it.

For Baryon OS, this means that privacy cannot be only a slogan. For material, the following must be visible and technically true:

- where it is stored;
- which tool or person has access;
- which actions are permitted;
- how long that applies;
- whether content can be transmitted through network permissions.

In *local-first software*, Kleppmann et al. (2019) describe ownership, offline availability and collaboration as connected design objectives. This is a design and architecture paper, not evidence that consumers therefore switch to a new OS.

**Project implication:** let the core experience begin without a NAS. Investigate linking, key recovery and recovery after device loss separately; privacy gains must be weighed against additional administration or learning time in the same usability trial.

### 4.5 Temporary storage without immediate organisation

Rutishauser and Fritz (2026) studied a temporary storage area for web material, connected to groups formed later, among 29 participants, mainly students and IT professionals. The approach helped some participants manage their material, but did not reduce tab counts for everyone. It used successive phases rather than an independent randomised control group.

**Hypothesis:** test an optional temporary location for a link, photo or document whose destination is not yet clear. Measure later retrieval as well as additional clutter and tidying effort. This is a research option, not a new mandatory product pillar.

## 5. Calm, interruptions and local AI

### 5.1 Adjustable interruptions

A randomised field experiment with 237 smartphone users found more favourable perceived attention and control with three daily notification batches (Fitz et al., 2019). This does not establish a universally optimal frequency for a desktop OS. A preregistered study with 205 participants found no clear improvement in checking behaviour, screen time or perceived productivity after one week of disabling notifications; fear of missing out increased (Dekker et al., 2025).

**Project implication:** investigate predictable batching, user-selected exceptions and an easily retrievable notification overview. In addition to perceived calm, measure missed important information and time to resume a task. Do not present this as a proven health or productivity intervention.

### 5.2 Local AI: retain it only if task value is demonstrated

Local processing is a privacy architecture choice, but does not automatically produce understanding or trust. Qualitative research into on-device machine learning shows that users may retain unclear expectations about what is analysed and why (West et al., 2025).

Dietvorst et al. (2018) found greater willingness to use an imperfect algorithm when people could adjust its output to some extent. A meta-analysis of 106 human–AI experiments also shows that added value differs substantially by task (Vaccaro et al., 2024).

**Project implication:** let the user explicitly select material, show the context used, make the result correctable and compare the same task without AI.

## 6. Product priorities as research hypotheses

This sequence is project advice, not a statistically proven market ranking:

1. **Prerequisite:** the initial audience's essential tasks, file formats, programs and hardware demonstrably work.
2. Easily retrieve material and resume work.
3. Understandable history, access and recovery.
4. Limited learning burden and feasible migration.
5. Concrete privacy with a simple daily workflow.
6. Polished, distinctive identity and adjustable calm.
7. Optional local AI with demonstrated added value per task.

## 7. First integrated demonstration

### Scenario DV-01 — the same material remains central

1. Retrieve **Tuinplan** (Garden plan) by title, collection or recent context.
2. Open the object and select tool A.
3. Grant tool A only the required object permissions.
4. Make a change and show “opgeslagen” (“saved”) only after the relevant storage state is confirmed.
5. Close or terminate tool A.
6. Open the same supported material with tool B without an unintended new copy as the default workflow.
7. Show history and recover an earlier version as a new revision.
8. Revoke A's access; a new protected action by A must be denied.
9. Ask the user to explain what was saved, what was recovered and which tool still has access.

### Product evidence

Measure at least task success, time/assistance, unintended copies, understanding of storage state and permissions, recovery success and voluntary preference to perform the same task again through Materiaal.

### System evidence

The native variant must separately demonstrate that access boundaries are genuinely enforced, confirmed work survives tool termination within the storage contract, revocation blocks future protected actions and uncertain storage is never presented as success.

A browser mock establishes interaction logic only.

## 8. Research programme

- **Phase A:** 12–18 interviews/task observations as a practical starting proposal; not a statistically required size.
- **Phase B:** pilot with approximately 24–36 participants for comparable tasks; determine effect size/power beforehand for confirmatory conclusions.
- **Phase C:** test different honest product messages on independently performed tasks, not only clicks.
- **Phase D:** once one workflow is sufficiently stable, first conduct a four-week early field trial recording voluntary use, errors, recovery, assistance and reasons for returning to the old way of working. Longer 90-/180-day validation from the market/adoption synthesis remains a separate later step.

Measure task success, errors and assistance needs separately from attractiveness. NASA-TLX can additionally measure perceived mental workload; the instrument measures workload and does not establish OS preference.

## 9. Boundaries for positioning

Supported statements:

- Baryon OS is building its own object-oriented product experience around material, tools, permissions and recovery.
- The combination is being specifically investigated for comprehensibility and daily value.
- The independent kernel is intended to enable consistent enforcement of system boundaries.

Unsupported without new evidence:

- “nobody else has this”;
- “consumers switch en masse because of this”;
- “privacy is the main purchasing reason”;
- “local AI is inherently more attractive”;
- “an independent kernel automatically makes the system safer or more usable”.

## 10. References

Beresford, A. R., Kübler, D., & Preibusch, S. (2012). Unwillingness to pay for privacy: A field experiment. *Economics Letters, 117*(1), 25–27. https://doi.org/10.1016/j.econlet.2012.04.077

Bergman, O., Gradovitch, N., Bar-Ilan, J., & Beyth-Marom, R. (2013). Folder versus tag preference in personal information management. *JASIST, 64*(10), 1995–2012. https://doi.org/10.1002/asi.22906

Bhattacherjee, A. (2001). Understanding information systems continuance: An expectation-confirmation model. *MIS Quarterly, 25*(3), 351–370. https://doi.org/10.2307/3250921

Boardman, R., & Sasse, M. A. (2004). “Stuff Goes into the Computer and Doesn't Come Out”: A cross-tool study of personal information management. *CHI '04*, 583–590. https://doi.org/10.1145/985692.985766

Dekker, C. A., Baumgartner, S. E., Sumter, S. R., & Ohme, J. (2025). Beyond the Buzz: Investigating the Effects of a Notification-Disabling Intervention on Smartphone Behavior and Digital Well-Being. *Media Psychology, 28*(1), 162–188. https://doi.org/10.1080/15213269.2024.2334025

Dietvorst, B. J., Simmons, J. P., & Massey, C. (2018). Overcoming algorithm aversion. *Management Science, 64*(3), 1155–1170. https://doi.org/10.1287/mnsc.2016.2643

Dumais, S., Cutrell, E., Cadiz, J. J., Jancke, G., Sarin, R., & Robbins, D. C. (2003). Stuff I've Seen. *SIGIR '03*, 72–79. https://doi.org/10.1145/860435.860451

Farhang, S., Weidman, J., Kamani, M. M., Grossklags, J., & Liu, P. (2018). Take It or Leave It: A Survey Study on Operating System Upgrade Practices. *ACSAC 2018*, 490–504. https://doi.org/10.1145/3274694.3274733

Felt, A. P., Ha, E., Egelman, S., Haney, A., Chin, E., & Wagner, D. (2012). Android permissions: User attention, comprehension, and behavior. *SOUPS 2012*. https://research.google/pubs/android-permissions-user-attention-comprehension-and-behavior/

Fitz, N., Kushlev, K., Jagannathan, R., Lewis, T., Paliwal, D., & Ariely, D. (2019). Batching smartphone notifications can improve well-being. *Computers in Human Behavior, 101*, 84–94. https://doi.org/10.1016/j.chb.2019.07.016

Grzybowski, L., & Nicolle, A. (2021). Estimating Consumer Inertia in Repeated Choices of Smartphones. *The Journal of Industrial Economics, 69*(1), 33–82. https://doi.org/10.1111/joie.12239

Hassenzahl, M., Diefenbach, S., & Göritz, A. (2010). Needs, affect, and interactive products—Facets of user experience. *Interacting with Computers, 22*(5), 353–362. https://doi.org/10.1016/j.intcom.2010.04.002

Kleppmann, M., Wiggins, A., van Hardenberg, P., & McGranaghan, M. (2019). Local-first software: You own your data, in spite of the cloud. *Onward! 2019*, 154–178. https://doi.org/10.1145/3359591.3359737

Łazaruk, K., Tokarski, P., Rybak, K., Plechawska-Wójcik, M., & Dzieńkowski, M. (2025). The Force of Habit: Comparing Graphical User Interfaces of Popular Operating Systems Using Eye-Tracking Analysis. *IEEE Access, 13*, 169236–169261. https://doi.org/10.1109/ACCESS.2025.3614018

Murray, K. B., & Häubl, G. (2007). Explaining cognitive lock-in. *Journal of Consumer Research, 34*(1), 77–88. https://doi.org/10.1086/513048

NASA. (n.d.). *NASA Task Load Index (TLX).* https://www.nasa.gov/human-systems-integration-division/nasa-task-load-index-tlx/

Rutishauser, R. A., & Fritz, T. (2026). From Tabs to Structures: Understanding and Supporting Web Page Management. *CHI ’26*, Article 1447, 1–18. https://doi.org/10.1145/3772318.3791979

Tornatzky, L. G., & Klein, K. J. (1982). Innovation characteristics and innovation adoption-implementation. *IEEE Transactions on Engineering Management, EM-29*(1), 28–45. https://doi.org/10.1109/TEM.1982.6447463

Tsai, J. Y., Egelman, S., Cranor, L., & Acquisti, A. (2011). The effect of online privacy information on purchasing behavior. *Information Systems Research, 22*(2), 254–268. https://doi.org/10.1287/isre.1090.0260

Tuch, A. N., Presslaber, E. E., Stöcklin, M., Opwis, K., & Bargas-Avila, J. A. (2012a). The role of visual complexity and prototypicality regarding first impression of websites. *IJHCS, 70*(11), 794–811. https://doi.org/10.1016/j.ijhcs.2012.06.003

Tuch, A. N., Roth, S. P., Hornbæk, K., Opwis, K., & Bargas-Avila, J. A. (2012b). Is beautiful really usable? *Computers in Human Behavior, 28*(5), 1596–1607. https://doi.org/10.1016/j.chb.2012.03.024

Vaccaro, M., Almaatouq, A., & Malone, T. (2024). When combinations of humans and AI are useful. *Nature Human Behaviour, 8*, 2293–2303. https://doi.org/10.1038/s41562-024-02024-1

Venkatesh, V., Thong, J. Y. L., & Xu, X. (2012). Consumer acceptance and use of information technology. *MIS Quarterly, 36*(1), 157–178. https://doi.org/10.2307/41410412

West, J., Cagiltay, B., Zhang, S., Li, J., Fawaz, K., & Banerjee, S. (2025). “Impressively Scary.” *CHI '25*. https://doi.org/10.1145/3706598.3713256

## 11. Relationship to existing project documents

- [DESIGN-AMBITION.md](../DESIGN-AMBITION.md) preserves the binding product ambition.
- [PRODUCT-SPEC.md](../implementation/PRODUCT-SPEC.md) remains authoritative for the frozen P2 build scope.
- [PRODUCT-AND-SOVEREIGNTY.md](../PRODUCT-AND-SOVEREIGNTY.md) contains the broader product and privacy vision.
- This document adds **rationale, hypotheses and a test programme** and earns no native progress point in itself.
