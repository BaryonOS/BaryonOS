# Quality contract

Load this reference for UI/UX work or an explicit evaluation of Token Guard.
It is a workflow requirement, not proof of equal model performance.

## Preserve the assignment

- Keep the approved brief, required content, design decisions, unresolved choices
  and acceptance criteria. Do not shorten product copy, omit edge states or
  replace the chosen design to reduce agent context.
- Record authoritative reference paths and source versions/hashes at checkpoints.
  Preserve original images and boards; a text description or reduced thumbnail
  does not replace the full reference. Reopen the original when detail is needed.
- In BaryonOS, follow `docs/DESIGN-AMBITION.md`, the current design checkpoint,
  `docs/design/CONCEPTS.md` and `docs/design/TYPOGRAPHY.md` where applicable.
  Authoritative concept boards under `ui/design/concepts/` remain read-only
  unless their modification is part of the user's assignment.

## Verify the affected experience

- Inspect rendered before/after states under the same declared viewport, scale,
  theme, fonts, representative data and relevant application state. Include
  responsive/scaled states required by the project. Inspect details at usable
  resolution; retain an overview when examining crops.
- Exercise the complete affected journey: primary action, keyboard/focus,
  feedback, applicable loading/empty/error states, cancellation and recovery.
  Evaluate motion dynamically when it changes; a static frame does not test it.
- Check applicable accessibility requirements, including labels, focus visibility,
  keyboard operation, scaling and contrast on actual rendered backgrounds.
  Automated checks cover only what they test; missing manual or assistive-tool
  checks remain open.
- Record visual, interaction and accessibility evidence separately from code
  tests. A successful build or screenshot capture is not an approval of the
  rendered design. Label AI judgement, human review, user testing, host prototype
  and native OS evidence separately.
- If a critical detail is missing, expand reads or repeat the relevant inspection.
  If required evidence cannot be obtained, keep that acceptance item unverified.
  Never silently turn an unverified UI criterion into a pass to save tokens.

## Measure the workflow without inventing a guarantee

1. Fix the task, starting source, required outputs, model/reasoning settings,
   tools and acceptance rubric before comparing runs. Do not reduce the task
   scope in the Token Guard condition.
2. Compare separate equivalent baseline and Token Guard sessions. Repeat matched
   trials because model outputs vary; record order, failures and extra work.
3. For a UI pilot, use an isolated permission-dialog fixture with a reference
   render and known defects in hierarchy, keyboard focus, contrast and recovery.
   Keep authoritative project designs untouched. Test the same journeys and
   render conditions, then review randomly labelled results against the fixed
   rubric. This describes an evaluation to execute, not an evaluation completed.
4. Require every mandatory criterion and no material regression in the reviewed
   sample. Record reviewer disagreements. AI-only scoring cannot establish human
   usability, and a finite pilot cannot guarantee every future task.
5. Report total task input/output and cached usage when the client exposes them,
   plus displayed bytes, reads, retries, agent overhead and elapsed time. Output
   byte reduction alone is neither billed-token saving nor quality equivalence.

Token Guard must not automatically lower model/reasoning settings, mask relevant
data, compress visual references, impose a two-read limit or clear the session.
Native compaction is host-controlled; preserve its returned context as specified
by that host and keep the checkpoint sufficient to recover missing evidence.
