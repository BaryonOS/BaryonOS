## ADR-PERMISSION-PERSISTENCE-20261119 — Remembered permission decisions, lifetimes and revocation

Status: **Proposed**, 19 November 2026 (Europe/Amsterdam), drafted by the UX-C run 2
writer for review by the project owner. Not accepted; not in `docs/DECISIONS.md`.
Until acceptance, nothing here is native behaviour: the design system and browser
prototype (`ui/design/prototype/contract.js`, `docs/design/DESIGN-SYSTEM.md` §11.1)
simulate it and label every remembered decision "Prototype: onthouden keuzes worden
nog niet door BaryonOS afgedwongen."

**Scope (reading R5).** Persistent grants and remembered denials; the five-field key;
the lifetime of temporary grants and temporary denials; storage protection; change and
revocation; the relation to capability enforcement and the Security Floor. It does
not change the capability model, the policy service's per-request check, kernel or
process isolation, or `docs/LOCAL-PRIVACY-CONTRACT.md`.

**Context.** The owner decision of 15 November 2026 (O-G12, O-G4, O-G2, items 1-8)
replaced the C1 three-choice prompt with two answers, Toestaan and Weigeren, plus a
native checkbox "Keuze onthouden" unchecked at every opening (R3). Approved reading R2
keeps an unremembered refusal in volatile state until the app closes. The owner
decisions of 18 and 19 November 2026 (OQ-1..OQ-6, MF-1..MF-6) settle the remaining
policy questions. The shell-only protected prompt layer does not exist natively
(DESKTOP-SHELL.md §3).

### Decided content (owner decisions; built as prototype)

1. **Key (OQ-4, MF-3).** A remembered decision is keyed on exactly app identity,
   right, data (one exact item, never a class or pattern), destination (present only
   for rights that send or connect) and scope. Any difference in one field is a new
   question. A remembered decision never covers a broader or different request. There
   is no per-app or per-right anti-nag throttling: a distinct legitimate request is
   never denied silently.
2. **Remembered Toestaan and Weigeren.** With "Keuze onthouden" checked, Toestaan
   stores an allow and Weigeren stores a deny for the exact key. Both are listed in
   Appmachtigingen with all five fields and can be changed or forgotten there.
3. **OQ-1.** No remembered Toestaan for an irreversible single external send; a
   remembered Weigeren for it is permitted and exact. A combined document-and-internet
   right may be remembered as Toestaan only with a named destination, one exact data
   item, a bounded scope and an extra confirmation step; otherwise the answer yields
   only a temporary grant. Changing a remembered deny to allow outside the prompt is
   refused for both classes.
4. **OQ-2 (Escape).** Escape or closing the prompt without an answer is a fail-closed
   cancellation with the same effect as a temporary denial; nothing is granted and
   nothing is stored, even when the checkbox is checked. A prompt that the shell cannot
   draw denies that request only and records no temporary denial or suppression, and a
   distinct request that arrives while a prompt is open waits for it; it never cancels
   the open prompt (OQ-4).
5. **OQ-6 (app reason).** An app may supply a plain-text reason of 1-120 characters
   without control, format (bidirectional) or line-separator characters; otherwise the
   reason is not shown at all and never truncated. It is shown after the BaryonOS
   facts, attributed to the app and framed apart from system text; markup is inert.
6. **Per-request check.** A remembered or temporary decision only suppresses the
   prompt. The policy service still performs the capability, object-scope and
   authorisation check on every request; a remembered allow never substitutes for it.

### Proposed changes awaiting acceptance (not built natively)

7. **OQ-3 (proposed change to R2).** Identical unsolicited re-prompts after a
   temporary denial would stay suppressed across app relaunches until logout, in
   volatile state only. Precondition: native attribution of a new deliberate user
   gesture, so that a user-initiated request can always ask again. Until acceptance,
   R2 (suppression until the app closes) remains the built behaviour; the gallery
   labels the until-logout behaviour "Voorgesteld, nog niet gebouwd".
8. **OQ-5 lifecycle.** A temporary grant ends when its logical app session ends: app
   termination, logout or a user-session switch. Hiding or minimising a window ends
   nothing. There is no hidden timeout. Any change of the security context (app
   identity or signature, user, session, device trust) invalidates temporary grants and
   requires revalidation of remembered decisions before use.
9. **MF-5 eligibility record.** The compact repeat prompt needs a volatile record of
   requests temporarily allowed in the current user session. It grants nothing,
   applies only to low-risk exact repeats (read or save-copy, no destination, not
   irreversible, not combined, same circumstance) and is cleared at logout or session
   switch.

### Storage, change and revocation

- Remembered decisions are stored per user, outside every app's reach, in the
  policy service's protected store; apps cannot read, enumerate or write them.
  Integrity protection and the key-isolation requirements of the Security Floor apply.
  Exact storage format, encryption at rest and synchronisation across paired devices
  are open questions for the native design.
- Temporary grants, temporary denials and the eligibility record are volatile and are
  never written to persistent storage.
- "Vergeten" removes exactly one key. Revocation does not recall copies the app
  already made; the prompt and Appmachtigingen state this.
- No usage log is kept for this feature unless a later decision adds one; the UI
  shows no usage metadata.

### Security Floor

Remembering a decision may reduce prompts but must not disable kernel or process
isolation, capability enforcement, key isolation, memory protection or the protection
of unshared data. A failure to read the store fails closed (prompt or deny), never open.

### Rationale and alternatives

Two answers plus one checkbox give equal choice (MF-4) and a single lifetime rule
per answer (MF-1). The rejected C1 model offered "Altijd toestaan" without an
equivalent remembered refusal and a four-field key. Per-app quieting was rejected
because it can silently deny distinct legitimate requests (OQ-4). Timeouts were
rejected because they create hidden lifetimes (OQ-5).

### Evidence and open items

Prototype only: `ui/design/prototype/tests/contract.test.mjs` (node:test) and the
measurement matrix in `build/design-evidence/uxc-run2/run2-writer/`. No native
implementation, no user study and no human audit exist. Comprehension, the ~120-word
target and native gesture detection are untested hypotheses.
