#!/usr/bin/env python3
"""Required `merge-hold` check: block merging while out-of-band acceptance is pending.

A pull request labelled `acceptance-pending` passes only when a trusted
participant (author association OWNER, MEMBER or COLLABORATOR) has commented
the exact line `acceptance-released: <head commit SHA>`. Labels and comments
are read live through the REST API, so event ordering and re-runs cannot
replay a stale decision. A new push changes the head SHA and re-arms the hold
automatically. Any API or input error fails closed. See ADR-MERGE-HOLD-20261113.

On a `merge_group` event the check first waits until the other required checks
(`host-quality`, `qemu-smoke`) have completed on the group commit, so labels and
comments are read as late as possible: a hold added while the queue is still
testing the group is seen. It then evaluates every pull request whose changes
the merge-queue commit contains: the queue entry whose commit is the group head
plus every entry ahead of it. Each must pass the same decision for its own head
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


def group_members(token, repo, head_ref, head_sha):
    """Return the open queue entries whose changes the group commit contains."""
    match = QUEUE_REF.fullmatch(head_ref)
    if not match or not SHA.fullmatch(head_sha):
        raise ValueError(f"unexpected merge group ref {head_ref!r} or SHA {head_sha!r}")
    owner, name = repo.split("/")
    answer = call(token, API + "/graphql",
                  {"query": QUEUE_QUERY, "variables": {"owner": owner, "name": name}})
    if answer.get("errors"):
        raise ValueError(f"GraphQL errors: {answer['errors']}")
    entries = answer["data"]["repository"]["mergeQueue"]["entries"]
    if entries["pageInfo"]["hasNextPage"]:
        raise ValueError("merge queue has more entries than one page")
    nodes = entries["nodes"]
    own = [node for node in nodes if node["headCommit"]["oid"] == head_sha]
    if len(own) != 1 or own[0]["pullRequest"]["number"] != int(match.group(1)):
        raise ValueError(f"no single queue entry matches {head_ref} at {head_sha}")
    position = own[0]["position"]
    if not isinstance(position, int):
        raise TypeError("queue position is not an integer")
    members = [node for node in nodes if node["position"] <= position]
    for node in members:
        pull = node["pullRequest"]
        if pull["state"] != "OPEN" or pull["baseRefName"] != "main":
            raise ValueError(f"queued pull request #{pull['number']} is not an open PR to main")
        if not SHA.fullmatch(node["headCommit"]["oid"] or ""):
            raise ValueError(f"queue entry for #{pull['number']} has no commit")
    return members


def tree(token, repo, sha):
    return call(token, f"{API}/repos/{repo}/git/commits/{sha}")["tree"]["sha"]


def wait_for_checks(token, repo, sha):
    """Block until every WAIT_CHECKS run on sha has completed; raise at the deadline.

    Only the timing depends on this: the decision afterwards reads labels and
    comments live and fails closed. An absent check run counts as not done, and
    API errors are retried, because waiting longer never weakens the hold.
    """
    # shortcut: one page of 100 check runs; a commit with more runs waits until
    # the deadline and fails closed. Page through if the workflows ever grow that far.
    url = (f"{API}/repos/{repo}/commits/{sha}/check-runs"
           f"?app_id={ACTIONS_APP_ID}&filter=latest&per_page=100")
    deadline = time.monotonic() + WAIT_SECONDS
    while True:
        try:
            runs = call(token, url)["check_runs"]
            done = {run["name"] for run in runs if run["status"] == "completed"}
            pending = [name for name in WAIT_CHECKS if name not in done]
        except (KeyError, ValueError, TypeError, AttributeError, OSError) as exc:
            pending = [f"check-run read failed: {exc}"]
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
    if not QUEUE_REF.fullmatch(head_ref) or not SHA.fullmatch(head_sha):
        raise ValueError(f"unexpected merge group ref {head_ref!r} or SHA {head_sha!r}")
    # Decide last: labels and comments are read only after the slow checks finish.
    wait_for_checks(token, repo, head_sha)
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
