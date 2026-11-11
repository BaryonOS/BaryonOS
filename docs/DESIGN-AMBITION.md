# Design ambition: an independent European OS with high product quality

## Authoritative addition — 27 October 2026

The design goal is a privacy-respecting, forward-looking European alternative offering the connected, polished experience consumers expect from Windows and macOS. Privacy must not produce a cumbersome or functionally impoverished product. This comparison is an ambition and design brief, not a measured product claim.

**Security, privacy and usability are joint acceptance conditions.** Build connectivity, continuity and collaboration with explicit technical boundaries. Address technical problems directly; do not turn temporary prototype limits into permanent user obligations. Ordinary operation requires no knowledge of certificates, nonces, tunnels or services.

| User task | Desired everyday experience | Security boundary and test |
|---|---|---|
| Getting started | Start locally without a vendor account; use the internet and external apps | Installation grants no general document/NAS permissions; permitted functions work without weakening security settings |
| Pairing a NAS | One guided flow for a personal NAS or received storage space; deliberately confirm identity once | Discovery alone is not trust; reconfirm on a new identity or revoked authority |
| Daily synchronisation | Automatic background operation within the selected scope; no repeated consent requests for each file | Native encryption, tenant isolation, correct revisions and tested recovery paths |
| Moving between home and travel | Automatically prefer LAN, otherwise use previously permitted internet access to the same NAS | No new consent prompt for the same authority; a different peer/relay/identity error remains blocked |
| A second personal device | Safely resume work and selected context after understandable pairing | Key transfer between verified personal devices; no key held by the NAS administrator/vendor |
| Sharing capacity | Offer or receive capacity within the same OS experience, even without a budget for a personal NAS | Administrator can reclaim/delete quota without read access; recipient understands availability limits |
| Error or loss | Local tasks remain usable, with clear status and an executable recovery route | No concealed data loss, no cloud fallback; no backdoor to solve lost keys |
| Consequential action | One clear confirmation by the authorised actor at the relevant moment | Deletion affects only the relevant host copy; do not ask a guest's permission to reclaim the host's own capacity |

For each task, measure completion, number of decisions/prompts, error/recovery behaviour and comprehension with users who are not administrators. Existing consent may prevent everyday interruptions without silently expanding identity or authority. Visual polish, responsiveness, keyboard operation and accessibility must pass in the same usability trial.

One-time pairing/recovery steps are permitted when they confirm a real trust boundary. CLI steps, repeated comparison of the same identity, manual sync and repeatedly granting permission are not the desired default experience. Read-only key recovery and small capacities in N01 are experiment limits; secure persistent keys, rotation and multi-device use are requirements before making a product claim.

## Research addition — consumer value, 28 October 2026

The literature review in [research/CONSUMER-VALUE-2026-10-28.md](research/CONSUMER-VALUE-2026-10-28.md) supports the existing ambition but sharpens the product question: an independent kernel and strong privacy architecture are not a sufficient consumer promise. The first demonstrable value must lie in a recognisable everyday task that works noticeably better or more clearly.

Core promise to investigate:

> **Find your material immediately, continue working with it, and retain control over changes and access.**

Treat **attention, trying, switching and continued use** as different outcomes. Measure task performance, understanding, recovery, need for help and voluntary reuse separately from visual appreciation. Retain familiar basic interactions, names, folders and export where they reduce switching friction; originality should arise from the coherence of Materiaal, tools, history, permissions and recovery, rather than unnecessarily reinventing every convention.

This is a research direction, not evidence that consumers already choose Baryon OS. The P2 product specification remains authoritative for the current build scope; new hypotheses are tested before becoming fixed product requirements.

> Revision 5 update: initial implementation choices are fixed in [implementation/PRODUCT-SPEC.md](implementation/PRODUCT-SPEC.md). Use that document for direct implementation; this file preserves the ambition and background for subsequent phases.
Revision 3, 26 October 2026. Additional requirements are incorporated into this binding product assignment. Narrowing the initial technical experiment must not replace the original ambition.

## Direction
Design an independent OS with its own coherent way of working, aiming for the visual and operational quality of mature platforms and giving users actual control. A familiar desktop with a different theme, launcher or logo is insufficient as the final outcome. *Addition of 30 October 2026 (ADR-058):* the basic interactions themselves—login, desktop, taskbar, start menu/search, windows and shortcuts—deliberately follow familiar conventions so consumers need not relearn them; independent code, visual identity and innovations build on that foundation.

Europe is the chosen product orientation here: privacy, control over data, accessibility, interoperability, recoverability and freedom from vendor dependence. This does not assert that one formal European OS standard exists or that an unbuilt system already complies with legislation. Link concrete product requirements to evidence and assess formal applicability for a specific release/audience.

## Ambition first, investigation next
Take an unusual idea seriously even when it does not fit an existing tutorial. Treat a technical limitation as a research question:
1. Which desired user outcome is blocked?
2. Is the blocker a proven hardware limit, current implementation limit, cost trade-off or unproven assumption?
3. What other representation, interface or architecture might enable the outcome?
4. What is the smallest experiment that can test the hypothesis?
5. What evidence determines continuation, adjustment or postponement?

Existing operating systems not doing something, or an idea not following a Linux pattern, are not grounds for rejection. Nor is the assertion that AI can build everything evidence. Preserve the ambition as a requirement; make uncertainty and experiments visible.

## Three parallel tracks
**Product research begins at M0.** Design and test interactions while developing the independent kernel. Prototypes may simulate capabilities that do not yet exist if clearly labelled. A useful simulation is evidence about an interaction, not about kernel security or functioning hardware.

**System research continues in small, demonstrable steps.** The core delivers progressively more functions that can actually support the chosen product experience. A feedback loop connects both tracks: a product decision may introduce new IPC, storage or authorisation requirements; a technical experiment may suggest alternative interactions.

There is no general rule against thinking about UI before memory isolation. However, an attractive interface must never conceal missing isolation and does not authorise releasing an unsafe prototype as an everyday OS.

**Quantum research also begins during design.** Follow QUANTUM-RESEARCH.md for post-quantum resilience, local quantum execution and the independent OS on a hybrid quantum computer. The product remains usable without a QPU; quantum capabilities have their own evidence and development path.

## Deliverables from the first product researcher
- Three substantially different concepts for interaction and information organisation, not three visual skins.
- Five written user scenarios per concept, including screen states and consequences for storage/permissions.
- Comparison with relevant existing platforms: what is familiar, what is combined differently, and what still requires investigation? No unsupported world-first claim.
- One recommended concept with decisions and rejected assumptions, followed by a small interactive experiment.
- An initial design system: typography, spacing, colour, iconography, focus, component behaviour, motion and language.
- A mapping from every fundamental interaction to a required system contract.

For example, explore different answers to whether tasks, documents/objects or devices should be the primary organising principle. How does someone resume work on a second personal device through a personal NAS? How can an app remain replaceable while the document and permissions remain under user control? These are research questions, not a prescribed final interface.

## High quality is testable
The experience must be coherent from initial installation to error messages, and from app installation to NAS recovery. Investigate:
- clear visual hierarchy and polished details, including empty/loading/error states;
- consistent input, animations, window/workspace behaviour and immediately visible feedback;
- accessibility through keyboard use, scaling, contrast and assistive technology;
- understandable permissions without constantly interrupting notifications;
- an SDK and component library through which third-party apps fit the system;
- reliable installation, removal, updating and undo;
- the same product logic across device classes with appropriate interaction.

A design may resemble familiar patterns where they work. It must provide its own answer to core tasks and data ownership; visual difference alone is insufficient.

## Addition of 28 October 2026 — privacy without visual or functional compromise

The eventual first-party apps and system surfaces must deliberately be designed
for **premium quality and future use**. Respect for privacy is no reason for
outdated interaction, crude administration panels or inconsistent interfaces
resembling disconnected open-source components. The reference quality is the
polish users expect from modern commercial productivity software; the product
must not clone that software.

Materiaal remains the distinctive model: content and object continuity first,
contextual tools second. One design system must support typography, spacing,
focus, motion, sharing, recovery, history, error states and accessibility
across first-party apps. The long-term goal is a shared native rendering/UI
layer once the graphics foundation can technically support it; the current
browser prototype is not an implicitly chosen final runtime.

A user may disable supplementary privacy explanations. The actual Security
Floor and capability boundaries remain active independently. See
[implementation/PRODUCT-SPEC.md](implementation/PRODUCT-SPEC.md) and
[implementation/ENCRYPTED-OBJECT-STORAGE.md](implementation/ENCRYPTED-OBJECT-STORAGE.md).

## Non-negotiable boundaries
An independent OS kernel. No silent transition to a Linux distribution or web shell on an existing kernel. OS data remains local or is client-side/end-to-end encrypted on an explicitly paired personal NAS, a NAS belonging to friends/family or—after explicit selection—as opaque ciphertext at an external storage provider. An external provider receives no Baryon OS content key and is not a silent fallback. The NAS administrator manages capacity and deletion; the user manages the content key. On-device AI inference without cloud fallback. External apps with limited permissions. Privacy choices must not disable the Security Floor. Follow LOCAL-PRIVACY-CONTRACT.md.

Aim for very strong protection, with clear guarantees for each function and testable recovery. Do not substitute an absolute claim such as 'unhackable' for design or evidence. Make security a quality of the experience, not an excuse for an unfinished product.

## First acceptance gate
The initial phase is substantively developed as a product only when three different concepts and one reasoned direction exist. A kernel logging text is a technical step; it does not determine how the eventual OS looks or works. The lead agent tracks both results separately.
