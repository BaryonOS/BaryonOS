#!/usr/bin/env python3
"""Required `merge-hold` check: block merging while out-of-band acceptance is pending.

A pull request labelled `acceptance-pending` passes only when a trusted
participant (author association OWNER, MEMBER or COLLABORATOR) has commented
the exact line `acceptance-released: <head commit SHA>`. Labels and comments
are read live through the REST API, so event ordering and re-runs cannot
replay a stale decision. A new push changes the head SHA and re-arms the hold
automatically. Any API or input error fails closed. See ADR-MERGE-HOLD-20261113.

On a `merge_group` event the check decides as late as it can. It first waits
until the other required checks (`host-quality`, `qemu-smoke`) have completed on
the group commit, then walks the queue entries ahead from the nearest outward:
each must have left the queue or completed `host-quality`, `qemu-smoke` and
`merge-hold` on its own commit. The walk ends at the first entry whose
`merge-hold` concluded `success`; that run waited the same way, so every entry
ahead of it had finished. A `merge-hold` that ended otherwise proves nothing
about the entries ahead of it, so those are checked too. The queue merges in
order, so this group can merge only after those entries; labels and comments
are therefore read close to this group's merge, not when its own checks end.
A hold added after that final read, until GitHub merges the group, is not seen
(ADR-MERGE-HOLD-20261113). The 45-minute deadline covers the group's
own checks and the whole chain ahead, each position adding up to one poll.
A group whose commit has left the queue (ejected or rebuilt) can never merge;
once its own checks have completed, two consecutive well-formed reads, one
poll apart, that both lack it end the wait with a failure. It then evaluates
every pull request whose changes the merge-queue commit contains: the queue
entry whose commit is the group head plus
every entry ahead of it that has not merged yet (a merged one is already on
`main`). Each must pass the same decision for its own head
SHA. A released hold additionally requires the tree of that pull request's own
queue entry commit (the commit `main` moves to when it merges) to equal the
released head tree, so acceptance never carries over to a tree it did not run on.
"""

import json
import os
import re
import sys
import time
from urllib import request

LABEL = "acceptance-pending"
RELEASE_PREFIX = "acceptance-released: "
TRUSTED = {"OWNER", "MEMBER", "COLLABORATOR"}
SHA = re.compile(r"[0-9a-f]{40}")
MAX_PAGES = 50
API = "https://api.github.com"
# Required checks merge-hold waits for on merge_group (ci.yml job names, run by
# GitHub Actions, app id 15368). The wait budget stays below the job timeout.
WAIT_CHECKS = ("host-quality", "qemu-smoke")
# An entry ahead has finished once these ran on its own commit; "merge-hold" is
# this check (merge-hold.yml job name); a successful run waited for the entries ahead.
AHEAD_CHECKS = WAIT_CHECKS + ("merge-hold",)
ACTIONS_APP_ID = 15368
POLL_SECONDS = 60
WAIT_SECONDS = 45 * 60
# shortcut: the pr-<N>-<sha> suffix is observed GitHub practice, not documented;
# an unexpected ref fails closed and stalls the queue instead of merging.
QUEUE_REF = re.compile(r"refs/heads/gh-readonly-queue/main/pr-([1-9][0-9]*)-[0-9a-f]{40}")
QUEUE_QUERY = """query($owner: String!, $name: String!) {
  repository(owner: $owner, name: $name) {
    mergeQueue(branch: "main") {
      entries(first: 100) {
        pageInfo { hasNextPage }
        nodes {
          position
          headCommit { oid }
          pullRequest { number state baseRefName headRefOid }
        }
      }
    }
  }
}"""


def decide(labels, comments, head_sha):
    """Return (passed, message) for one pull request head."""
    if not SHA.fullmatch(head_sha or ""):
        return False, "Head SHA is missing or malformed; failing closed."
    if LABEL not in labels:
        return True, "No acceptance hold is set."
    release = RELEASE_PREFIX + head_sha
    for comment in comments:
        if comment.get("author_association") not in TRUSTED:
            continue
        lines = (comment.get("body") or "").replace("\r", "").split("\n")
        if any(line.strip() == release for line in lines):
            return True, f"Acceptance released for {head_sha}."
    return False, (
        f"The '{LABEL}' label is set and no trusted '{release}' comment exists. "
        "Post it only after the required acceptance has passed on this head commit."
    )


def call(token, url, payload=None):
    """One authenticated API request (GET, or POST with a JSON body)."""
    req = request.Request(
        url,
        data=None if payload is None else json.dumps(payload).encode(),
        headers={
            "Authorization": "Bearer " + token,
            "Accept": "application/vnd.github+json",
            "X-GitHub-Api-Version": "2022-11-28",
            "User-Agent": "baryon-merge-hold",
        },
    )
    with request.urlopen(req, timeout=20) as response:
        return json.load(response)


def fetch_all(token, url):
    """Read every page of a REST list endpoint; raise on any failure."""
    items, page = [], 1
    while True:
        batch = call(token, f"{url}?per_page=100&page={page}")
        if not isinstance(batch, list):
            raise TypeError("Unexpected API response")
        items.extend(batch)
        if len(batch) < 100:
            return items
        page += 1
        if page > MAX_PAGES:
            raise ValueError("Too many pages")


def pull_request_hold(token, repo, number, head_sha):
    """Read one pull request live; return (passed, message, labels)."""
    base = f"{API}/repos/{repo}/issues/{number}"
    labels = [item["name"] for item in fetch_all(token, base + "/labels")]
    comments = fetch_all(token, base + "/comments") if LABEL in labels else []
    passed, message = decide(labels, comments, head_sha)
    return passed, message, labels


def commit_oid(node):
    """The entry's commit SHA, or None while it has no commit (nullable in GraphQL)."""
    return (node.get("headCommit") or {}).get("oid")


def queue_nodes(token, repo):
    """Read every entry of the main merge queue live; raise on any doubt."""
    owner, name = repo.split("/")
    answer = call(token, API + "/graphql",
                  {"query": QUEUE_QUERY, "variables": {"owner": owner, "name": name}})
    if answer.get("errors"):
        raise ValueError(f"GraphQL errors: {answer['errors']}")
    entries = answer["data"]["repository"]["mergeQueue"]["entries"]
    if entries["pageInfo"]["hasNextPage"]:
        raise ValueError("merge queue has more entries than one page")
    return entries["nodes"]


class LeftQueue(ValueError):
    """This group's commit is no longer in the merge queue, so the group can never merge."""


def left_queue(nodes, head_sha, number):
    """True only if this well-formed queue read positively shows the group has left it.

    The list is not empty, no entry has head_sha, every entry names its pull
    request by an integer number, and the group's pull request is absent
    (ejected or merged) or has another built commit (rebuilt). An entry of that
    pull request without a commit may still be this group, so it is not proof of
    absence; unbuilt entries of other pull requests do not matter. An empty list
    may be a lagging listing, so it is not proof either.
    """
    if not nodes or any(commit_oid(node) == head_sha for node in nodes):
        return False
    pulls = [node.get("pullRequest") for node in nodes]
    if not all(isinstance(pull, dict) and isinstance(pull.get("number"), int) for pull in pulls):
        return False
    return all(SHA.fullmatch(commit_oid(node) or "")
               for node, pull in zip(nodes, pulls) if pull["number"] == number)


def own_position(nodes, head_sha, number):
    """Position of the single entry whose commit is head_sha (other entries may be unbuilt)."""
    if left_queue(nodes, head_sha, number):
        raise LeftQueue(f"merge group commit {head_sha} is no longer in the merge queue "
                        "(ejected or rebuilt), so this group can never merge")
    own = [node for node in nodes if commit_oid(node) == head_sha]
    if len(own) != 1:
        raise ValueError(f"no single queue entry has commit {head_sha}")
    position = own[0]["position"]
    if not isinstance(position, int):
        raise TypeError("queue position is not an integer")
    return own[0], position


def group_members(token, repo, head_ref, head_sha):
    """Return the open queue entries whose changes the group commit contains.

    An entry ahead whose pull request has already MERGED into main is skipped:
    it is on main, so no decision here can hold it back, and failing on it
    would only eject this group when GitHub merges the entry ahead just as its
    merge-hold succeeds. Every unmerged entry is still evaluated.
    """
    match = QUEUE_REF.fullmatch(head_ref)
    if not match or not SHA.fullmatch(head_sha):
        raise ValueError(f"unexpected merge group ref {head_ref!r} or SHA {head_sha!r}")
    nodes = queue_nodes(token, repo)
    own, position = own_position(nodes, head_sha, int(match.group(1)))
    if (own.get("pullRequest") or {}).get("number") != int(match.group(1)):
        raise ValueError(f"no single queue entry matches {head_ref} at {head_sha}")
    members = []
    for node in nodes:
        if node["position"] > position:
            continue
        pull = node.get("pullRequest")
        if not pull:
            raise ValueError(f"queue entry at position {node['position']} has no pull request")
        if pull["baseRefName"] != "main":
            raise ValueError(f"queued pull request #{pull['number']} is not an open PR to main")
        if pull["state"] == "MERGED" and node is not own:
            continue
        if pull["state"] != "OPEN":
            raise ValueError(f"queued pull request #{pull['number']} is not an open PR to main")
        if not SHA.fullmatch(commit_oid(node) or ""):
            raise ValueError(f"queue entry for #{pull['number']} has no commit")
        members.append(node)
    return members


def tree(token, repo, sha):
    return call(token, f"{API}/repos/{repo}/git/commits/{sha}")["tree"]["sha"]


def completed_runs(token, repo, sha):
    """The completed check runs on sha by name (absent or running counts as not done)."""
    # shortcut: one page of 100 check runs; a commit with more runs waits until
    # the deadline and fails closed. Page through if the workflows ever grow that far.
    url = (f"{API}/repos/{repo}/commits/{sha}/check-runs"
           f"?app_id={ACTIONS_APP_ID}&filter=latest&per_page=100")
    return {run["name"]: run for run in call(token, url)["check_runs"]
            if run["status"] == "completed"}


def unfinished(token, repo, sha, names):
    """Names of the check runs on sha that have not completed."""
    done = completed_runs(token, repo, sha)
    return [f"{name} on {sha}" for name in names if name not in done]


def still_running(token, repo, sha, number):
    """What this group still waits for: its own checks, then the entries ahead.

    Entries ahead are walked from the nearest outward. The walk stops at the
    first entry whose `merge-hold` concluded `success`: only a successful run
    went through this same wait, so every entry ahead of it had finished. A
    `merge-hold` that ended otherwise (failed early, cancelled, runner lost)
    proves nothing about the entries ahead of it, so the walk continues. Each
    run makes one REST request per entry walked plus one GraphQL query per
    poll; usually that is the nearest entry only. "The rate limit for
    GITHUB_TOKEN is 1,000 requests per hour per repository"
    (https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api);
    beyond it reads fail, are retried and, at the deadline, fail closed.
    """
    pending = unfinished(token, repo, sha, WAIT_CHECKS)
    if pending:
        return pending
    nodes = queue_nodes(token, repo)
    _, position = own_position(nodes, sha, number)
    ahead = [node for node in nodes if node["position"] < position]
    for node in sorted(ahead, key=lambda node: node["position"], reverse=True):
        oid = commit_oid(node)
        if not SHA.fullmatch(oid or ""):
            return [f"queue entry at position {node['position']} has no commit yet"]
        done = completed_runs(token, repo, oid)
        pending = [f"{name} on {oid}" for name in AHEAD_CHECKS if name not in done]
        if pending:
            return pending
        if done["merge-hold"].get("conclusion") == "success":
            return []
    return []


def wait_for_checks(token, repo, sha, number):
    """Block until this group's checks and the entries ahead have finished; raise at the deadline.

    Only the timing depends on this: the decision afterwards reads the queue,
    labels and comments live and fails closed. API errors and unbuilt entries
    are retried, because waiting longer never weakens the hold. An entry ahead
    that leaves the queue (merged or removed) no longer counts. Two consecutive
    polls whose well-formed reads both show that this group itself has left the
    queue fail at once; any other poll in between resets that count, because a
    single read may lag behind the queue.
    """
    deadline = time.monotonic() + WAIT_SECONDS
    absent = 0
    while True:
        try:
            pending = still_running(token, repo, sha, number)
            absent = 0
        except LeftQueue:
            absent += 1
            if absent >= 2:
                raise
            pending = ["merge group commit not listed in the queue once; reading again"]
        except (KeyError, IndexError, ValueError, TypeError, AttributeError, OSError) as exc:
            absent = 0
            pending = [f"queue or check-run read failed: {exc}"]
        if not pending:
            return
        if time.monotonic() >= deadline:
            raise ValueError(f"required checks did not complete in time: {pending}")
        time.sleep(POLL_SECONDS)


def merge_group_hold(token, repo):
    """Return (passed, lines) for every pull request in the merge group."""
    head_sha = os.environ["MERGE_GROUP_HEAD_SHA"]
    if os.environ["MERGE_GROUP_BASE_REF"] != "refs/heads/main":
        raise ValueError("merge group does not target main")
    head_ref = os.environ["MERGE_GROUP_HEAD_REF"]
    match = QUEUE_REF.fullmatch(head_ref)
    if not match or not SHA.fullmatch(head_sha):
        raise ValueError(f"unexpected merge group ref {head_ref!r} or SHA {head_sha!r}")
    # Decide last: labels and comments are read only after this group's checks
    # and every entry ahead (which merge first) have finished.
    wait_for_checks(token, repo, head_sha, int(match.group(1)))
    members = group_members(token, repo, head_ref, head_sha)
    ok, lines = True, []
    for node in members:
        pull, entry_commit = node["pullRequest"], node["headCommit"]["oid"]
        number, pr_head = pull["number"], pull["headRefOid"]
        passed, message, labels = pull_request_hold(token, repo, number, pr_head)
        # Compare with this PR's own queue entry: that commit is what main
        # becomes when it merges, and its own group checks that tree.
        if passed and LABEL in labels and tree(token, repo, entry_commit) != tree(token, repo, pr_head):
            passed, message = False, (
                f"Acceptance for {pr_head} covers its own tree, but its queue entry "
                f"{entry_commit} lands a different tree. Update the branch and re-release, "
                "or let the pull requests ahead merge first.")
        ok = ok and passed
        lines.append(f"#{number}: " + (message if passed else "::error::" + message))
    return ok, lines


def main():
    try:
        token = os.environ["GH_TOKEN"]
        repo = os.environ["REPO"]
        event = os.environ["EVENT_NAME"]
        if event == "pull_request":
            number = int(os.environ["PR_NUMBER"])
            passed, message, _ = pull_request_hold(token, repo, number, os.environ["HEAD_SHA"])
            lines = [message if passed else "::error::" + message]
        elif event == "merge_group":
            passed, lines = merge_group_hold(token, repo)
        else:
            raise ValueError(f"unsupported event {event!r}")
    except (KeyError, IndexError, ValueError, TypeError, AttributeError, OSError) as exc:
        print(f"::error::merge-hold could not read the pull request ({exc}); failing closed.")
        return 1
    print("\n".join(lines))
    return 0 if passed else 1


if __name__ == "__main__":
    sys.exit(main())
