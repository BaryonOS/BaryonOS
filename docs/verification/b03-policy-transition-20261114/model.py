#!/usr/bin/env python3
"""Finite symbolic check of the UNAPPROVED B03 policy-transition proposal.

No parser, cryptography, storage, QEMU, production policy or native evidence.
The authenticated atoms below stand for independently verified bytes. Atomic,
durable, rollback-resistant anchor storage is an ASSUMPTION, not implemented.
Key bytes, release windows and wire encoding are outside this bounded model.
Image IDs combine image/manifest digest and length; separate byte fields and
actual placement of image bytes in destination slots are not represented.
"""

from dataclasses import dataclass, replace
from itertools import product
import json

PAIR_A = ("ed-a", "pq-a")
PAIR_B = ("ed-b", "pq-b")
SHARED_PAIR = ("ed-a", "pq-b")
ROOT_PAIR = ("root-ed", "root-pq")
ROOT_RELEASE = ("root-ed", "pq-b")


@dataclass(frozen=True)
class Policy:
    version: int
    digest: str
    signers: tuple[tuple[tuple[str, str], bool], ...]  # Hybrid key pair; revoked.
    minimum: int = 10
    target: str = "synthetic-test"
    parent_digest: str = "policy-old"


@dataclass(frozen=True)
class Image:
    version: int
    digest: str
    signer: tuple[str, str]
    target: str = "synthetic-test"


@dataclass(frozen=True)
class Anchor:
    revision: int
    policy: tuple[int, str]
    floor: int
    transition: object = None


@dataclass(frozen=True)
class Transition:
    predecessor: Anchor
    target: str
    policy: tuple[int, str]
    image: tuple[int, str]
    destination_slot: int = 1


OLD = Policy(1, "policy-old", ((PAIR_A, False),), parent_digest="genesis")
NEXT = Policy(2, "policy-next", ((PAIR_A, True), (PAIR_B, False)))
KEEP = replace(NEXT, digest="policy-keep", signers=((PAIR_A, False), (PAIR_B, False)))
AFTER_KEEP = Policy(3, "policy-after-keep", KEEP.signers, parent_digest=KEEP.digest)
RAISE = replace(KEEP, digest="policy-floor", minimum=11)
TOO_HIGH = replace(NEXT, digest="policy-unbootable", minimum=12)
RESURRECT = Policy(3, "policy-resurrect", ((PAIR_A, False), (PAIR_B, False)), parent_digest=NEXT.digest)
FORGET = Policy(3, "policy-forget", ((PAIR_B, False),), parent_digest=NEXT.digest)
FULL = Policy(3, "policy-full", NEXT.signers + tuple(((f"ed-{n}", f"pq-{n}"), True) for n in range(7)),
              parent_digest=NEXT.digest)
DUPLICATE = Policy(3, "policy-duplicate", NEXT.signers + ((PAIR_B, False),), parent_digest=NEXT.digest)
EQUIVOCATE = replace(NEXT, digest="policy-conflict")
SHARED_BASE = replace(OLD, digest="policy-shared-base", signers=((PAIR_A, False), (SHARED_PAIR, False)))
SHARED_REVOKE = Policy(2, "policy-shared-revoke", ((PAIR_A, True), (SHARED_PAIR, False)),
                       parent_digest=SHARED_BASE.digest)
ROOT_REUSE = Policy(2, "policy-root-reuse", ((PAIR_A, True), (ROOT_RELEASE, False)))
WRONG_PARENT = replace(NEXT, digest="policy-wrong-parent", parent_digest="unrelated-policy")
# Membership represents authentication; these sets perform NO signature checks.
ROOT_AUTHENTICATED = (OLD, NEXT, KEEP, AFTER_KEEP, RAISE, TOO_HIGH, RESURRECT, FORGET, FULL,
                      DUPLICATE, EQUIVOCATE, SHARED_BASE, SHARED_REVOKE, ROOT_REUSE, WRONG_PARENT)
I0 = Image(10, "image-old", PAIR_A)
I1 = Image(11, "image-next", PAIR_B)
I2 = Image(12, "image-other", PAIR_B)
SAME_VERSION = Image(10, "image-same-version", PAIR_B)
SHARED_IMAGE = Image(11, "image-shared", SHARED_PAIR)
ROOT_IMAGE = Image(11, "image-root", ROOT_RELEASE)
RELEASE_AUTHENTICATED = (I0, I1, I2, SAME_VERSION, SHARED_IMAGE, ROOT_IMAGE)
# Each fixture binding stands for a valid release-role signature over this exact
# image/manifest identity and policy digest. Byte/signature verification is absent.
BINDING_AUTHENTICATED = frozenset((i.digest, p.digest) for i in RELEASE_AUTHENTICATED
                                for p in ROOT_AUTHENTICATED)
A0 = Anchor(7, (OLD.version, OLD.digest), 10)
TX = Transition(A0, NEXT.target, (NEXT.version, NEXT.digest), (I1.version, I1.digest))
A1 = Anchor(8, TX.policy, A0.floor, TX)


def policy_for(anchor, policies):
    if not isinstance(anchor, Anchor):
        return None
    return next((p for p in policies if p in ROOT_AUTHENTICATED
                 and 1 <= len(p.signers) <= 8
                 and len({pair for pair, _ in p.signers}) == len(p.signers)
                 and all(set(pair).isdisjoint(ROOT_PAIR) for pair, _ in p.signers)
                 and all(set(active).isdisjoint(revoked) for active, is_revoked in p.signers
                         if not is_revoked for revoked, tombstone in p.signers if tombstone)
                 and (p.version, p.digest) == anchor.policy), None)


def eligible(image, policy, floor, bindings):
    return (image in RELEASE_AUTHENTICATED and image.target == policy.target
            and image.version >= floor and (image.signer, False) in policy.signers
            and (image.digest, policy.digest) in bindings & BINDING_AUTHENTICATED)


def boot(anchor, policies, images, bindings=BINDING_AUTHENTICATED):
    """Read a trusted anchor first; a disk header cannot select policy."""
    policy = policy_for(anchor, policies)
    if policy is None or anchor.floor < policy.minimum:
        return None
    preferred = anchor.transition.image if anchor.transition else (I0.version, I0.digest)
    prior = anchor.transition.predecessor.transition if anchor.transition else None
    fallback = prior.image if prior else (I0.version, I0.digest)
    candidates = sorted((i for i in images if (i.version, i.digest) in (preferred, fallback)),
                        key=lambda i: (i.version, i.digest) != preferred)
    return next((i for i in candidates if eligible(i, policy, anchor.floor, bindings)), None)


def slot_for(anchor, image):
    """Locate the selected fixture, including a predecessor used as fallback."""
    identity = (image.version, image.digest)
    while anchor.transition:
        if anchor.transition.image == identity:
            return anchor.transition.destination_slot
        anchor = anchor.transition.predecessor
    return 0 if identity == (I0.version, I0.digest) else None


def commit(current, tx, policies, images, durable_readback, bindings=BINDING_AUTHENTICATED):
    """Abstract CAS after durable readback; None means refuse, not an erased anchor."""
    expected = tx.predecessor
    desired = Anchor(expected.revision + 1, tx.policy, expected.floor, tx)
    new = policy_for(desired, policies)
    if new is None:
        return None
    desired = replace(desired, floor=max(expected.floor, new.minimum))
    if tx.destination_slot not in (0, 1):
        return None
    if current == desired:  # An exact retry is a no-op, never a second write.
        return current
    if current != expected or not durable_readback or expected.revision == 2**64 - 1:
        return None
    old = policy_for(expected, policies)
    if old is None or tx.target != old.target or new.target != tx.target:
        return None
    if new.version <= old.version or new.parent_digest != old.digest:
        return None
    if any(revoked and (pair, True) not in new.signers for pair, revoked in old.signers):
        return None  # Revocation tombstones cannot be dropped or re-enabled.
    candidate = next((i for i in images if (i.version, i.digest) == tx.image), None)
    running = boot(expected, policies, images, bindings)
    if (running is None or candidate is None or candidate.version <= running.version
            or not eligible(candidate, new, desired.floor, bindings)):
        return None
    running_slot = slot_for(expected, running)
    if running_slot is None or tx.destination_slot == running_slot:
        return None
    return desired


def started(anchor, image, policies, bindings=BINDING_AUTHENTICATED):
    """Separate loader-validated-start floor write; no boot-success claim."""
    if boot(anchor, policies, (image,), bindings) != image:
        return None
    if image.version == anchor.floor:
        return anchor
    if anchor.revision == 2**64 - 1:
        return None
    return replace(anchor, revision=anchor.revision + 1, floor=image.version)


def observe_commit(old, new, observed):
    """An uncertain commit is resolved only by an exact trusted readback."""
    return observed if observed in (old, new) else None


def handover(observed, expected, image, policies, bindings=BINDING_AUTHENTICATED):
    if observed != expected or expected.floor < image.version:
        return None
    return boot(observed, policies, (image,), bindings)


def main():
    checks = 0

    def check(condition, label):
        nonlocal checks
        checks += 1
        if not condition:
            raise RuntimeError(label)

    # A partial write is not an authenticated durable atom. A flush can make
    # bytes durable; only readback admits the candidate to the commit operation.
    cuts = (
        ("before staging", (OLD,), (I0,), False),
        ("policy partial", (OLD,), (I0,), False),
        ("policy flushed", (OLD, NEXT), (I0,), False),
        ("image partial", (OLD, NEXT), (I0,), False),
        ("image flushed", (OLD, NEXT), (I0, I1), False),
        ("candidate read back", (OLD, NEXT), (I0, I1), True),
    )
    for label, policies, images, readback in cuts:
        check(boot(A0, policies, images) == I0, label + ": old state survives")
        result = commit(A0, TX, policies, images, readback)
        check(result == (A1 if readback else None), label + ": commit preconditions")

    # A lost acknowledgement permits exactly the old or new durable anchor.
    # Failed/uncertain reads never infer a trusted state from staged disk data.
    observations = (A0, A1, "unavailable", "torn", replace(A1, revision=99))
    for observed in observations:
        expected = I0 if observed == A0 else I1 if observed == A1 else None
        check(boot(observe_commit(A0, A1, observed), (OLD, NEXT), (I0, I1)) == expected,
              "anchor readback")

    loss_cases = 0
    for old_p, new_p, old_i, new_i in product((False, True), repeat=4):
        policies = tuple(p for p, present in ((OLD, old_p), (NEXT, new_p)) if present)
        images = tuple(i for i, present in ((I0, old_i), (I1, new_i)) if present)
        check(boot(A0, policies, images) == (I0 if old_p and old_i else None), "old cut oracle")
        check(boot(A1, policies, images) == (I1 if new_p and new_i else None), "new cut oracle")
        loss_cases += 2

    check(commit(A1, TX, (OLD, NEXT), (I0, I1), True) == A1, "identical retry")
    changed_image = replace(TX, image=(I2.version, I2.digest))
    check(changed_image.policy == TX.policy and changed_image != TX, "policy-only resume ambiguity")
    check(commit(A1, changed_image, (OLD, NEXT), (I0, I1, I2), True) is None, "different image is not resume")
    changed_slot = replace(TX, destination_slot=0)
    check(commit(A1, changed_slot, (OLD, NEXT), (I0, I1), True) is None, "different destination is not resume")
    for slot in (0, -1, 2):
        check(commit(A0, replace(TX, destination_slot=slot), (OLD, NEXT), (I0, I1), True) is None,
              "running or invalid destination slot")
    check(commit(A0, replace(TX, target="other-target"), (OLD, NEXT), (I0, I1), True) is None,
          "wrong target")
    check(commit(A0, TX, (OLD, replace(NEXT, signers=((PAIR_A, False),))), (I0, I1), True) is None,
          "altered policy is not an authenticated atom")
    check(commit(A0, TX, (OLD, NEXT), (I0, replace(I1, signer=PAIR_A)), True) is None,
          "altered image is not an authenticated atom")
    for proposed in (OLD, EQUIVOCATE, RESURRECT, FORGET, FULL, DUPLICATE):
        request = Transition(A1, proposed.target, (proposed.version, proposed.digest),
                             (I2.version, I2.digest), destination_slot=0)
        check(commit(A1, request, (NEXT, proposed), (I1, I2), True) is None,
              "rollback/equivocation/resurrection/tombstone loss/capacity/duplicate")

    keep_tx = replace(TX, policy=(KEEP.version, KEEP.digest))
    keep_anchor = commit(A0, keep_tx, (OLD, KEEP), (I0, I1), True)
    check(keep_anchor is not None and boot(keep_anchor, (KEEP,), (I0,)) == I0,
          "fallback remains possible only while allowed by committed policy and floor")
    after_fallback = Transition(keep_anchor, KEEP.target, (3, AFTER_KEEP.digest),
                                (I2.version, I2.digest), destination_slot=1)
    check(commit(keep_anchor, after_fallback, (KEEP, AFTER_KEEP), (I0, I2), True) is not None,
          "fallback runs in slot zero, so the lost candidate slot one is writable")
    check(commit(keep_anchor, replace(after_fallback, destination_slot=0),
                 (KEEP, AFTER_KEEP), (I0, I2), True) is None, "the actual running fallback slot is protected")
    missing_binding = BINDING_AUTHENTICATED - {(I0.digest, KEEP.digest)}
    check(boot(keep_anchor, (KEEP,), (I0,), missing_binding) is None, "fallback needs its new-policy binding")
    check(boot(A0, (OLD,), (I0,), missing_binding) == I0, "old binding survives until commit")
    no_candidate_binding = BINDING_AUTHENTICATED - {(I1.digest, NEXT.digest)}
    check(commit(A0, TX, (OLD, NEXT), (I0, I1), True, no_candidate_binding) is None,
          "candidate needs its policy binding")
    same_tx = replace(TX, image=(SAME_VERSION.version, SAME_VERSION.digest))
    check(commit(A0, same_tx, (OLD, NEXT), (I0, SAME_VERSION), True) is None, "strict image version increase")
    shared_anchor = replace(A0, policy=(SHARED_BASE.version, SHARED_BASE.digest))
    shared_tx = Transition(shared_anchor, NEXT.target, (2, SHARED_REVOKE.digest), (11, SHARED_IMAGE.digest))
    check(commit(shared_anchor, shared_tx, (SHARED_BASE, SHARED_REVOKE), (I0, SHARED_IMAGE), True) is None,
          "retained active pair cannot share either component with a new tombstone")
    root_tx = replace(TX, policy=(2, ROOT_REUSE.digest), image=(11, ROOT_IMAGE.digest))
    check(commit(A0, root_tx, (OLD, ROOT_REUSE), (I0, ROOT_IMAGE), True) is None, "root component is not a release role")
    wrong_parent_tx = replace(TX, policy=(2, WRONG_PARENT.digest))
    check(commit(A0, wrong_parent_tx, (OLD, WRONG_PARENT), (I0, I1), True) is None,
          "root-authenticated policy must name the committed predecessor digest")
    raise_tx = replace(TX, policy=(RAISE.version, RAISE.digest))
    root_raised = commit(A0, raise_tx, (OLD, RAISE), (I0, I1), True)
    check(root_raised is not None and root_raised.floor == 11, "root minimum is committed atomically")
    check(boot(root_raised, (RAISE,), (I0,)) is None, "root minimum forbids old fallback")
    high_tx = replace(TX, policy=(TOO_HIGH.version, TOO_HIGH.digest))
    check(commit(A0, high_tx, (OLD, TOO_HIGH), (I0, I1), True) is None, "candidate below root minimum")
    raised = started(keep_anchor, I1, (KEEP,))
    check(raised.floor == 11 and raised.transition == keep_anchor.transition,
          "start floor is separate from policy activation")
    floor_observations = (keep_anchor, raised, "unavailable", "torn", replace(raised, revision=99))
    for observed in floor_observations:
        check(handover(observed, raised, I1, (KEEP,)) == (I1 if observed == raised else None),
              "handover needs the exact durable floor acknowledgement")
    check(boot(raised, (KEEP,), (I0,)) is None, "no fallback below raised floor")
    check(started(raised, I0, (KEEP,)) is None, "floor never decreases")
    check(started(raised, I2, (KEEP,)) is None, "unselected image cannot raise the floor")
    check(commit(raised, keep_tx, (OLD, KEEP), (I0, I1), True) is None, "stale retry after floor write")
    check(boot(A1, (OLD,), (I0,)) is None, "disk-only replay cannot restore revoked signer")
    check(boot(A1, (NEXT,), (I0,)) is None, "new policy rejects old signed image")
    check(boot(A1, (NEXT,), (I2,)) is None, "an unrelated image is not the committed candidate or fallback")

    # Expected counterexample, not an antirollback implementation: if an attacker
    # can restore BOTH trusted anchor and disk, the prior valid state boots.
    check(boot(A0, (OLD,), (I0,)) == I0, "whole-state replay counterexample must remain visible")
    print(json.dumps({
        "status": "PASS", "evidence": "finite symbolic proposal model only",
        "checks": checks, "durable_cut_cases": len(cuts),
        "anchor_observations": len(observations), "artifact_loss_cases": loss_cases,
        "floor_write_observations": len(floor_observations),
        "expected_counterexample": "replaying both anchor and disk boots the old image",
        "not_tested": ["cryptography or byte parsing", "native execution or storage",
                       "physical rollback resistance", "production authority or release chain",
                       "release-window semantics or unbounded state exploration",
                       "all eligible fallbacks beyond the candidate and predecessor image",
                       "separate manifest digest/length fields or actual slot-content placement"],
    }, sort_keys=True))


if __name__ == "__main__":
    main()
