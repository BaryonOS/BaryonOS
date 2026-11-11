#!/usr/bin/env python3
"""Conservatively classify existing GitHub Projects V2 items.

Only unambiguous conventional title prefixes and explicit labels are used.
Existing field values are never overwritten. No status or issue state changes.
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
PRIORITIES = {
    "p0": "P0 Critical",
    "p1": "P1 High",
    "p2": "P2 Normal",
    "p3": "P3 Low",
}
TITLE_PATTERN = re.compile(
    r"^\s*(feat|fix|security|refactor|test|docs|chore|build|ci|research)"
    r"(?:\(([a-z0-9_.-]+)\))?!?:", re.IGNORECASE
)

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
            ... on Issue {
              number title repository { nameWithOwner }
              labels(first: 50) { nodes { name } }
            }
            ... on PullRequest {
              number title repository { nameWithOwner }
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

UPDATE_QUERY = """
mutation($project: ID!, $item: ID!, $field: ID!, $option: String!) {
  updateProjectV2ItemFieldValue(input: {
    projectId: $project, itemId: $item, fieldId: $field,
    value: { singleSelectOptionId: $option }
  }) { projectV2Item { id } }
}
"""


def unique(values):
    """Return a value only if all recognized signals agree."""
    values = set(values)
    return next(iter(values)) if len(values) == 1 else None


def classify(title, labels):
    """Compute suggestions, without estimating effort or inferring urgency."""
    suggestions = {}
    type_values, subsystem_values, priority_values = [], [], []
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
        elif label.startswith("priority:"):
            priority = label.split(":", 1)[1].strip()
            if priority in PRIORITIES:
                priority_values.append(PRIORITIES[priority])

    for field, values in (
        ("Work Type", type_values),
        ("Subsystem", subsystem_values),
        ("Priority", priority_values),
    ):
        chosen = unique(values)
        if chosen is not None:
            suggestions[field] = chosen
    return suggestions


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


def proposed_updates(item, fields):
    content = item.get("content")
    if not content:
        return []
    labels = [node["name"] for node in (content.get("labels") or {}).get("nodes", [])
              if node]
    suggested = classify(content.get("title", ""), labels)
    current = {
        node["field"]["name"] for node in item["fieldValues"]["nodes"]
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
    return updates


def run(token, org, repo, number, limit, apply):
    cursor, seen, changed, suggested_count = None, 0, 0, 0
    while seen < limit:
        data = graphql(token, PROJECT_QUERY,
                       {"org": org, "number": number, "cursor": cursor})
        project = (data.get("organization") or {}).get("projectV2")
        if project is None:
            raise RuntimeError("Project not found or token lacks Projects access")
        fields = {node["name"]: node for node in project["fields"]["nodes"]
                  if node and node.get("name") and "options" in node}
        connection = project["items"]
        for item in connection["nodes"]:
            if seen >= limit:
                break
            seen += 1
            content = item.get("content") or {}
            if (content.get("repository") or {}).get("nameWithOwner", "").lower() != repo.lower():
                continue
            for field, value, field_id, option_id in proposed_updates(item, fields):
                suggested_count += 1
                print(f"{'APPLY' if apply else 'DRY-RUN'} #{content['number']}: {field} = {value}")
                if apply:
                    graphql(token, UPDATE_QUERY, {
                        "project": project["id"], "item": item["id"],
                        "field": field_id, "option": option_id,
                    })
                    changed += 1
        if not connection["pageInfo"]["hasNextPage"] or seen >= limit:
            break
        cursor = connection["pageInfo"]["endCursor"]
        if not cursor:
            raise RuntimeError("Pagination cursor missing")
    print(f"Scanned {seen} items; suggested {suggested_count}; updated {changed}.")
    if not apply:
        print("Dry-run: no project fields were modified.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--organization", default="BaryonOS")
    parser.add_argument("--repository", default="BaryonOS/BaryonOS")
    parser.add_argument("--project", type=int, default=5)
    parser.add_argument("--limit", type=int, default=200)
    parser.add_argument("--apply", action="store_true")
    args = parser.parse_args()
    if not 1 <= args.limit <= 500:
        parser.error("--limit must be between 1 and 500")
    token = os.environ.get("BARYON_PROJECTS_TOKEN")
    if not token:
        parser.error("BARYON_PROJECTS_TOKEN is required (Projects read/write permission)")
    try:
        run(token, args.organization, args.repository, args.project, args.limit, args.apply)
    except (RuntimeError, error.URLError, KeyError, TypeError) as exc:
        print(f"Project classification failed: {exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
