# Sources, explanation and architectural decisions

Use for a source question, tutorial explanation, architecture comparison or design
proposal. Do not modify code unless that is requested.

## Procedure

1. Restate the technical question in terms of target architecture, boot mode and
   subsystem. A source-only question does not require repository access.
2. Open the corresponding topic from [the route](../onderwerpenroute.md), then the
   actual linked source. Use [the book guide](../boek-leeswijzer.md) for a learning path.
3. Read surrounding assumptions and warnings, not just a search hit. Follow redirects
   to their targets. Mark talk/user pages as such rather than presenting them as specs.
4. For an exact register format, instruction semantic, ABI requirement or changing tool
   flag, verify the applicable primary specification/tool manual when the decision
   depends on it. If it is unavailable, label the claim unverified; do not invent a URL,
   section number, recent version or test.
5. Distinguish source statements, source examples, project requirements and your own
   recommendation. Provide one recommended route when the evidence supports a choice.
6. Supply a minimal example only after identifying its architecture and assumptions.
   Keep an educational example visibly separate from production-ready implementation.
7. Close with the actual answer, source IDs, relevant limits and an actionable next
   step only when it helps the request. Do not append an unsolicited development plan.

## Citation forms

Use the host's real source citation mechanism where available. Otherwise use ordinary
attribution, for example `OSDev Wiki contributors, “Page Frame Allocation”, revision
30500, W001461` or `Helin & Renberg (2015), p. 53, B053`. The source index contains the
revision URL. Do not invent `filecite` or web handles.

APA-form references are in [the source notices](../../BRONNEN-EN-LICENTIES.md).
A citation establishes provenance, not the correctness of every source statement.

## Scope and conflict rules

Never equate a full text import with full technical verification. The supplied book
uses historical x86 examples. Preserve those examples as sources; explain explicitly
when proposing another architecture or modernizing a build. Do not silently correct
the stored source. Record an erratum separately with its evidence and test.

For architecture comparisons, evaluate trust boundary, recovery, bootstrap dependencies,
IPC cost, driver model, available skills and verification scope. Do not decide that all
major OS designs are the same, or that a small kernel alone guarantees security.
