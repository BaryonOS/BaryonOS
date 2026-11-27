#!/usr/bin/env python3
"""Conservatively classify and synchronise existing GitHub Projects V2 items.

Work Type and Subsystem come only from unambiguous conventional title prefixes
and explicit labels; existing values are never overwritten. Priority, Effort and
planning dates remain specialist-owned. This writer only supports project
single-select fields.

Terminal GitHub facts win: a merged PR or an issue explicitly closed as completed
becomes Done; other closed items become Closed. Existing nonterminal statuses
remain specialist-owned. Draft flags and an explicit ``blocked`` label only
initialise empty statuses or reset a terminal status after reopening. Project
Status is organisational only; it never replaces a GitHub Actions result.

Open issues and pull requests of the repository that are missing from the
project are added (bounded per run). Nothing is added when the project scan was
truncated, so the dry-run and apply outputs stay exact. No issue, PR, label or
milestone is modified.

Afterwards a read-only PR hygiene report lists open pull requests that miss a
per-PR integrity duty of docs/handoffs/README.md (labels, milestone, issue link,
D3 assessment file). It only reports: it never changes the exit code, and it
prints pull request numbers and fixed rule names, never PR text.
"""

import argparse
import json
import os
import re
import sys
from urllib import error, request


TYPE_PREFIXES = {
    "feat": "Feature",
    "fix": "Bug",
    "security": "Security",
    "refactor": "Refactor",
    "test": "Test",
    "docs": "Documentation",
    "chore": "Maintenance",
    "build": "Maintenance",
    "ci": "Maintenance",
    "research": "Research",
}
SUBSYSTEM_SCOPES = {
    "kernel": "Kernel",
    "memory": "Memory",
    "scheduler": "Scheduler",
    "ipc": "IPC",
    "security": "Security",
    "storage": "Storage",
    "volume": "Storage",
    "services": "Services",
    "service": "Services",
    "hw": "Hardware",
    "hardware": "Hardware",
    "drivers": "Hardware",
    "net": "Networking",
    "network": "Networking",
    "networking": "Networking",
    "ui": "UI",
    "veld": "UI",
    "materiaal": "UI",
    "toolchain": "Toolchain",
    "cargo": "Toolchain",
    "ci": "CI/CD",
    "ci-cd": "CI/CD",
    "docs": "Documentation",
    "documentation": "Documentation",
}
TITLE_PATTERN = re.compile(
    r"^\s*(feat|fix|security|refactor|test|docs|chore|build|ci|research)"
    r"(?:\(([a-z0-9_.-]+)\))?!?:", re.IGNORECASE
)

STATUS_FIELD = "Status"
BLOCKED_LABELS = {"blocked", "status:blocked"}
# Open items keep every selected nonterminal status, including Blocked.
# Removing a label or changing draft state is not a specialist's status decision.
CORRECTABLE_OPEN_STATUSES = {"", "done", "closed"}
# Candidate option names in order of preference; the first existing one wins.
STATUS_CANDIDATES = {
    "todo": ("Backlog", "Todo", "To do"),
    "in progress": ("In Progress",),
    "in review": ("In Review", "In Progress"),
    "blocked": ("Blocked",),
    "done": ("Done",),
    "closed": ("Closed",),
}
MAX_ADDS_PER_RUN = 50
MAX_OPEN_CONTENT = 500

PROJECT_QUERY = """
query($org: String!, $number: Int!, $cursor: String) {
  organization(login: $org) {
    projectV2(number: $number) {
      id
      fields(first: 50) {
        nodes {
          ... on ProjectV2SingleSelectField {
            id
            name
            options { id name }
          }
        }
      }
      items(first: 50, after: $cursor) {
        pageInfo { hasNextPage endCursor }
        nodes {
          id
          content {
            __typename
            ... on Issue {
              id number title state stateReason repository { nameWithOwner }
              labels(first: 50) { nodes { name } }
            }
            ... on PullRequest {
              id number title state isDraft repository { nameWithOwner }
              labels(first: 50) { nodes { name } }
            }
          }
          fieldValues(first: 40) {
            nodes {
              ... on ProjectV2ItemFieldSingleSelectValue {
                name
                field { ... on ProjectV2SingleSelectField { name } }
              }
            }
          }
        }
      }
    }
  }
}
"""

OPEN_CONTENT_QUERY = """
query($owner: String!, $name: String!, $cursor: String) {
  repository(owner: $owner, name: $name) {
    %s(states: OPEN, first: 100, after: $cursor) {
      pageInfo { hasNextPage endCursor }
      nodes { id number }
    }
  }
}
"""

HYGIENE_QUERY = """
query($owner: String!, $name: String!, $cursor: String) {
  repository(owner: $owner, name: $name) {
    %s(states: OPEN, first: 50, after: $cursor) {
      pageInfo { hasNextPage endCursor }
      nodes {
        number title isDraft body
        author { __typename }
        milestone { number }
        labels(first: 50) { totalCount nodes { name } }
        files(first: 100) { totalCount nodes { path } }
      }
    }
  }
}
"""
# Per-PR integrity duties 2, 4 and 7 (docs/handoffs/README.md); Closes/Fixes/Resolves also link.
LINK_PATTERN = re.compile(
    r"\b(?:refs|part of|closes|fixes|resolves)\s+(?:[\w.-]+/[\w.-]+)?#\d+", re.IGNORECASE)
ASSESSMENT_PATTERN = re.compile(r"docs/progress/assessments/[^/]+\.json")

UPDATE_QUERY = """
mutation($project: ID!, $item: ID!, $field: ID!, $option: String!) {
  updateProjectV2ItemFieldValue(input: {
    projectId: $project, itemId: $item, fieldId: $field,
    value: { singleSelectOptionId: $option }
  }) { projectV2Item { id } }
}
"""

ADD_QUERY = """
mutation($project: ID!, $content: ID!) {
  addProjectV2ItemById(input: { projectId: $project, contentId: $content }) {
    item { id }
  }
}
"""


def unique(values):
    """Return a value only if all recognized signals agree."""
    values = set(values)
    return next(iter(values)) if len(values) == 1 else None


def classify(title, labels):
    """Compute suggestions, without estimating effort or inferring urgency."""
    suggestions = {}
    type_values, subsystem_values = [], []
    match = TITLE_PATTERN.match(title or "")
    if match:
        prefix, scope = (part.lower() if part else None for part in match.groups())
        type_values.append(TYPE_PREFIXES[prefix])
        if scope in SUBSYSTEM_SCOPES:
            subsystem_values.append(SUBSYSTEM_SCOPES[scope])
        elif not scope and prefix in ("ci", "docs"):
            subsystem_values.append("CI/CD" if prefix == "ci" else "Documentation")

    for label in labels:
        label = label.lower().strip()
        if label.startswith("type:"):
            value = label.split(":", 1)[1].strip()
            if value in (name.lower() for name in TYPE_PREFIXES.values()):
                type_values.append(next(name for name in TYPE_PREFIXES.values()
                                        if name.lower() == value))
        elif label.startswith(("subsystem:", "area:")):
            scope = label.split(":", 1)[1].strip()
            if scope in SUBSYSTEM_SCOPES:
                subsystem_values.append(SUBSYSTEM_SCOPES[scope])

    for field, values in (
        ("Work Type", type_values),
        ("Subsystem", subsystem_values),
    ):
        chosen = unique(values)
        if chosen is not None:
            suggestions[field] = chosen
    return suggestions


def derive_status(content, labels, current):
    """Return a terminal fact or an initial/reopened status key, or None.

    ``current`` is the item's present Status option name ("" when empty).
    """
    kind = content.get("__typename")
    state = content.get("state")
    current = (current or "").casefold()
    blocked = any(label.lower().strip() in BLOCKED_LABELS for label in labels)
    if kind == "PullRequest":
        if state == "MERGED":
            return "done"
        if state == "CLOSED":
            return "closed"
        if state != "OPEN" or current not in CORRECTABLE_OPEN_STATUSES:
            return None
        if blocked:
            return "blocked"
        return "in progress" if content.get("isDraft") else "in review"
    if kind == "Issue":
        if state == "CLOSED":
            return "done" if content.get("stateReason") == "COMPLETED" else "closed"
        if state != "OPEN" or current not in CORRECTABLE_OPEN_STATUSES:
            return None
        if blocked:
            return "blocked"
        return "todo"
    return None


def resolve_option(field, key):
    """Return the first existing option for a managed status key."""
    for candidate in STATUS_CANDIDATES.get(key, ()):
        for option in field["options"]:
            if option["name"].casefold() == candidate.casefold():
                return option
    return None


def proposed_updates(item, fields):
    content = item.get("content")
    if not content:
        return []
    labels = [node["name"] for node in (content.get("labels") or {}).get("nodes", [])
              if node]
    suggested = classify(content.get("title", ""), labels)
    current = {
        node["field"]["name"]: node["name"] for node in item["fieldValues"]["nodes"]
        if node and node.get("field") and node.get("name")
    }
    updates = []
    for name, desired in suggested.items():
        if name in current or name not in fields:
            continue
        field = fields[name]
        option = next((o for o in field["options"]
                       if o["name"].casefold() == desired.casefold()), None)
        if option:
            updates.append((name, desired, field["id"], option["id"]))

    if STATUS_FIELD in fields:
        present = current.get(STATUS_FIELD, "")
        key = derive_status(content, labels, present)
        option = resolve_option(fields[STATUS_FIELD], key) if key else None
        if option and option["name"].casefold() != present.casefold():
            updates.append((STATUS_FIELD, option["name"],
                            fields[STATUS_FIELD]["id"], option["id"]))
    return updates


def missing_content(project_content_ids, open_items):
    """Open repository items absent from the project, in a stable order."""
    seen, missing = set(project_content_ids), []
    for node in open_items:
        if node["id"] not in seen:
            seen.add(node["id"])
            missing.append(node)
    return missing


def pr_hygiene(node):
    """Per-PR duties an open pull request misses: (missing rule names, unknown rule names).

    Bot pull requests are exempt. CI pull requests (a ``ci`` title type or the CI/CD
    subsystem) need no milestone. A cut-off file list leaves the assessment unknown.
    """
    if (node.get("author") or {}).get("__typename") == "Bot":
        return [], []
    title = node.get("title") or ""
    labels = node.get("labels") or {}
    names = [n["name"] for n in labels.get("nodes") or [] if n]
    match = TITLE_PATTERN.match(title)
    ci = bool(match) and match.group(1).lower() == "ci" or \
        classify(title, names).get("Subsystem") == "CI/CD"
    missing, unknown = [], []
    if not labels.get("totalCount"):
        missing.append("labels")
    if not node.get("milestone") and not ci:
        missing.append("milestone")
    if not LINK_PATTERN.search(node.get("body") or ""):
        missing.append("link")
    files = node.get("files") or {}
    paths = [f["path"] for f in files.get("nodes") or [] if f]
    found = any(ASSESSMENT_PATTERN.fullmatch(p) for p in paths)
    if not found and files and files.get("totalCount") == len(paths):
        missing.append("assessment")
    elif not found:  # the file list was cut off or not returned
        unknown.append("assessment")
    return missing, unknown


def report_hygiene(token, repo):
    """Print the PR hygiene report; any failure becomes one warning, never an exit code."""
    try:
        nodes = list(open_content(token, repo, "pullRequests", HYGIENE_QUERY))
        flagged = 0
        for node in sorted(nodes, key=lambda n: n["number"]):
            missing, unknown = pr_hygiene(node)
            if missing or unknown:
                flagged += bool(missing)
                draft = " (draft)" if node.get("isDraft") else ""
                parts = [f"missing {', '.join(missing)}"] if missing else []
                parts += [f"unknown {', '.join(unknown)}"] if unknown else []
                print(f"PR hygiene #{int(node['number'])}{draft}: {'; '.join(parts)}")
        print(f"::warning::PR hygiene: {flagged} of {len(nodes)} open pull requests miss a "
              "per-PR duty (report only; docs/handoffs/README.md).")
    except Exception as exc:  # report only: never fail the classification run
        print(f"::warning::PR hygiene report skipped ({type(exc).__name__}).")


def graphql(token, query, variables):
    req = request.Request(
        "https://api.github.com/graphql",
        data=json.dumps({"query": query, "variables": variables}).encode("utf-8"),
        headers={
            "Authorization": "Bearer " + token,
            "Accept": "application/vnd.github+json",
            "Content-Type": "application/json",
            "User-Agent": "baryon-project-classifier",
        },
        method="POST",
    )
    try:
        with request.urlopen(req, timeout=25) as response:
            result = json.load(response)
    except error.HTTPError as exc:
        raise RuntimeError(f"GitHub GraphQL HTTP {exc.code}") from exc
    if result.get("errors"):
        raise RuntimeError("GitHub GraphQL rejected request: " +
                           "; ".join(e.get("message", "unknown") for e in result["errors"]))
    return result["data"]


def open_content(token, repo, connection, query=OPEN_CONTENT_QUERY):
    """Yield open issues or pull requests of one repository, bounded."""
    owner, name = repo.split("/", 1)
    cursor, count = None, 0
    while True:
        data = graphql(token, query % connection,
                       {"owner": owner, "name": name, "cursor": cursor})
        repository = data.get("repository")
        if repository is None:
            raise RuntimeError("Repository not found or token lacks read access")
        page = repository[connection]
        for node in page["nodes"]:
            if node:
                count += 1
                if count > MAX_OPEN_CONTENT:
                    raise RuntimeError(f"More than {MAX_OPEN_CONTENT} open {connection}")
                yield node
        if not page["pageInfo"]["hasNextPage"]:
            return
        cursor = page["pageInfo"]["endCursor"]
        if not cursor:
            raise RuntimeError("Pagination cursor missing")


def run(token, org, repo, number, limit, apply, add_missing=True):
    cursor, seen, changed, suggested_count = None, 0, 0, 0
    content_ids, truncated, project_id = set(), False, None
    while True:
        data = graphql(token, PROJECT_QUERY,
                       {"org": org, "number": number, "cursor": cursor})
        project = (data.get("organization") or {}).get("projectV2")
        if project is None:
            raise RuntimeError("Project not found or token lacks Projects access")
        project_id = project["id"]
        fields = {node["name"]: node for node in project["fields"]["nodes"]
                  if node and node.get("name") and "options" in node}
        connection = project["items"]
        for item in connection["nodes"]:
            if seen >= limit:
                truncated = True
                break
            seen += 1
            content = item.get("content") or {}
            if content.get("id"):
                content_ids.add(content["id"])
            if (content.get("repository") or {}).get("nameWithOwner", "").lower() != repo.lower():
                continue
            for field, value, field_id, option_id in proposed_updates(item, fields):
                suggested_count += 1
                print(f"{'APPLY' if apply else 'DRY-RUN'} #{content['number']}: {field} = {value}")
                if apply:
                    graphql(token, UPDATE_QUERY, {
                        "project": project_id, "item": item["id"],
                        "field": field_id, "option": option_id,
                    })
                    changed += 1
        if truncated or not connection["pageInfo"]["hasNextPage"]:
            break
        if seen >= limit:
            truncated = True  # more pages exist beyond the limit
            break
        cursor = connection["pageInfo"]["endCursor"]
        if not cursor:
            raise RuntimeError("Pagination cursor missing")

    added = planned = 0
    if add_missing and truncated:
        print("Project scan truncated by --limit; skipping additions.")
    elif add_missing:
        open_items = list(open_content(token, repo, "issues"))
        open_items += list(open_content(token, repo, "pullRequests"))
        missing = missing_content(content_ids, open_items)
        if len(missing) > MAX_ADDS_PER_RUN:
            print(f"{len(missing)} open items missing; adding the first {MAX_ADDS_PER_RUN}.")
            missing = missing[:MAX_ADDS_PER_RUN]
        for node in missing:
            planned += 1
            print(f"{'APPLY' if apply else 'DRY-RUN'} #{node['number']}: add to project")
            if apply:
                graphql(token, ADD_QUERY, {"project": project_id, "content": node["id"]})
                added += 1
    print(f"Scanned {seen} items; suggested {suggested_count}; updated {changed}; "
          f"missing {planned}; added {added}.")
    if not apply:
        print("Dry-run: no project fields or items were modified.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--organization", default="BaryonOS")
    parser.add_argument("--repository", default="BaryonOS/BaryonOS")
    parser.add_argument("--project", type=int, default=5)
    parser.add_argument("--limit", type=int, default=500)
    parser.add_argument("--apply", action="store_true")
    parser.add_argument("--no-add-missing", action="store_true",
                        help="Do not add open issues/PRs missing from the project")
    parser.add_argument("--skip-without-token", action="store_true",
                        help="Exit 0 with a warning when the token is not configured")
    args = parser.parse_args()
    if not 1 <= args.limit <= 1000:
        parser.error("--limit must be between 1 and 1000")
    if args.repository.count("/") != 1:
        parser.error("--repository must be owner/name")
    token = os.environ.get("BARYON_PROJECTS_TOKEN")
    if not token:
        if args.skip_without_token:
            print("::warning::BARYON_PROJECTS_TOKEN is not configured; "
                  "Project synchronisation skipped.")
            return 0
        parser.error("BARYON_PROJECTS_TOKEN is required (Projects read/write permission)")
    code = 0
    try:
        run(token, args.organization, args.repository, args.project, args.limit,
            args.apply, add_missing=not args.no_add_missing)
    except (RuntimeError, error.URLError, KeyError, TypeError) as exc:
        print(f"Project classification failed: {exc}", file=sys.stderr)
        code = 1
    report_hygiene(token, args.repository)
    return code


if __name__ == "__main__":
    sys.exit(main())
