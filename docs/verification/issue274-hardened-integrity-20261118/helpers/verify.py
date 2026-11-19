#!/usr/bin/env python3
"""Validate the Wiki publication from immutable repository and Wiki Git objects."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import stat
import subprocess
from urllib.parse import unquote, urlsplit


REDIRECT_ENV = {
    "GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES", "GIT_COMMON_DIR", "GIT_CEILING_DIRECTORIES",
    "GIT_PREFIX", "GIT_CONFIG", "GIT_CONFIG_GLOBAL", "GIT_CONFIG_SYSTEM",
    "GIT_CONFIG_PARAMETERS", "GIT_NAMESPACE", "GIT_REPLACE_REF_BASE",
    "GIT_SHALLOW_FILE", "GIT_GRAFT_FILE", "GIT_EXEC_PATH",
}


def git_environment() -> dict[str, str]:
    environment = {
        name: value for name, value in os.environ.items()
        if name not in REDIRECT_ENV
        and not re.fullmatch(r"GIT_CONFIG_(?:COUNT|KEY_\d+|VALUE_\d+)", name)
    }
    environment.update({
        "GIT_NO_LAZY_FETCH": "1", "GIT_NO_REPLACE_OBJECTS": "1",
        "GIT_TERMINAL_PROMPT": "0", "GIT_CONFIG_NOSYSTEM": "1",
        "GIT_CONFIG_GLOBAL": "/dev/null",
    })
    return environment


GIT_ENV = git_environment()


def git(repo: Path, *arguments: str) -> bytes:
    return subprocess.check_output(
        ["git", "--no-optional-locks", "-C", str(repo), *arguments], env=GIT_ENV,
    )


def canonical_commit(repo: Path, reference: str) -> str:
    if not re.fullmatch(r"[0-9a-f]{40}", reference):
        raise ValueError("A complete immutable commit SHA is required")
    value = git(repo, "rev-parse", "--verify", reference + "^{commit}").decode().strip()
    if value != reference:
        raise ValueError("Commit identity mismatch")
    return value


def tree(repo: Path, reference: str) -> dict[str, tuple[str, str, str]]:
    result: dict[str, tuple[str, str, str]] = {}
    for row in git(repo, "ls-tree", "-r", "-z", "--full-tree", reference).split(b"\0"):
        if not row:
            continue
        metadata, separator, raw_path = row.partition(b"\t")
        if not separator:
            raise ValueError("Invalid Git tree record")
        mode, kind, object_id = metadata.decode().split(" ")
        result[raw_path.decode("utf-8", "strict")] = (mode, kind, object_id)
    return result


def regular(entry: tuple[str, str, str] | None) -> bool:
    return bool(entry and entry[0] in {"100644", "100755"} and entry[1] == "blob")


def blob(repo: Path, index: dict[str, tuple[str, str, str]], relative: str) -> bytes:
    entry = index.get(relative)
    if not regular(entry):
        raise ValueError("Target is not a regular Git blob: " + relative)
    return git(repo, "cat-file", "blob", entry[2])


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def relative_path(value: str) -> str:
    pure = PurePosixPath(value)
    if pure.is_absolute() or ".." in pure.parts or not pure.parts:
        raise ValueError("Path escapes immutable tree: " + value)
    return pure.as_posix()


def strict_unquote(value: str) -> str:
    if re.search(r"%(?![0-9a-fA-F]{2})", value):
        raise ValueError("Invalid percent encoding")
    try:
        return unquote(value, errors="strict")
    except UnicodeDecodeError as error:
        raise ValueError("Invalid UTF-8 percent encoding") from error


def slug(value: str) -> str:
    value = re.sub(r"<[^>]*>", "", value)
    value = re.sub(r"[\x60*]", "", value).lower()
    value = re.sub(r"[^\w\- ]", "", value, flags=re.UNICODE)
    return value.replace(" ", "-")


def rendered_markdown_label(value: str) -> str:
    return re.sub(r"!?\[([^\]\n]*)\]\([^\)\n]*\)", r"\1", value)


def strip_inline_code(
    line: str,
    preserve_contents: bool = False,
    replacement: str | None = None,
) -> str:
    result: list[str] = []
    index = 0
    marker = "\x60"
    while index < len(line):
        if line[index] != marker:
            result.append(line[index])
            index += 1
            continue
        end = index
        while end < len(line) and line[end] == marker:
            end += 1
        length = end - index
        search = end
        closing = None
        while search < len(line):
            found = line.find(marker, search)
            if found < 0:
                break
            run_end = found
            while run_end < len(line) and line[run_end] == marker:
                run_end += 1
            if run_end - found == length:
                closing = run_end
                break
            search = run_end
        if closing is None:
            result.append(line[index:end])
            index = end
        else:
            if preserve_contents:
                content = line[end:closing - length].replace("\t", " ")
                if content.startswith(" ") and content.endswith(" ") and content.strip():
                    content = content[1:-1]
                result.append(content)
            elif replacement is not None:
                result.append(replacement)
            index = closing
    return "".join(result)


def rendered_html(text: str) -> str:
    while True:
        visible = re.sub(r"<!--[\s\S]*?-->", "", text)
        if visible == text:
            text = text.split("<!--", 1)[0]
            break
        text = visible
    hidden = r"<(?:pre|code)\b[^>]*>[\s\S]*?</(?:pre|code)>"
    text = re.sub(hidden, "", text, flags=re.IGNORECASE)
    opening = re.search(r"<(?:pre|code)\b[^>]*>", text, flags=re.IGNORECASE)
    return text[:opening.start()] if opening else text


def mask_html_comments(line: str, in_comment: bool) -> tuple[str, bool]:
    masked = list(line)
    position = 0
    while True:
        start = position if in_comment else line.find("<!--", position)
        if start < 0:
            break
        end = line.find("-->", start if in_comment else start + 4)
        if end < 0:
            masked[start:] = " " * (len(line) - start)
            return "".join(masked), True
        masked[start:end + 3] = " " * (end + 3 - start)
        position = end + 3
        in_comment = False
    return "".join(masked), in_comment


def inline_destination(value: str) -> str | None:
    match = re.fullmatch(
        r"\s*(?:<([^>\n]+)>|((?:\\.|[^\s])+))(?:\s+(?:\"[^\"\n]*\"|'[^'\n]*'|\([^\)\n]*\)))?\s*",
        value,
    )
    return (match.group(1) or match.group(2)) if match else None


def link_destinations(text: str) -> list[str]:
    lines: list[str] = []
    fence: tuple[str, int] | None = None
    in_comment = False
    for line in text.splitlines():
        scan_line = line
        next_comment = in_comment
        if fence is None:
            scan_line, next_comment = mask_html_comments(line, in_comment)
        marker = re.match(r"^ {0,3}((?:\x60){3,}|~{3,})(.*)$", scan_line)
        if marker:
            current = (marker.group(1)[0], len(marker.group(1)))
            if fence is None:
                if current[0] != "\x60" or "\x60" not in marker.group(2):
                    fence = current
                    continue
            elif current[0] == fence[0] and current[1] >= fence[1] and not marker.group(2).strip():
                fence = None
                continue
        in_comment = next_comment
        if fence:
            continue
        lines.append(strip_inline_code(line, replacement="code"))
    body = rendered_html("\n".join(lines))
    definitions: dict[str, str] = {}
    lines = []
    for rendered in body.splitlines():
        definition = re.match(r"^ {0,3}\[([^\]]+)\]:[ \t]*(?:<([^>]+)>|(\S+))", rendered)
        if definition:
            label = " ".join(definition.group(1).lower().split())
            definitions[label] = definition.group(2) or definition.group(3)
            lines.append("")
        else:
            lines.append(rendered)
    body = "\n".join(lines)
    destinations: list[str] = []
    consumed = list(body)
    pattern = re.compile(r"\[([^\]\n]+)\]\(([^)\n]+)\)|\[([^\]\n]+)\]\[([^\]\n]*)\]")
    for match in pattern.finditer(body):
        if match.group(2) is not None:
            destination = inline_destination(match.group(2))
            if destination is not None:
                destinations.append(destination)
        else:
            label = match.group(4) or match.group(3)
            destination = definitions.get(" ".join(label.lower().split()))
            if destination is not None:
                destinations.append(destination)
        consumed[match.start():match.end()] = " " * (match.end() - match.start())
    for match in re.finditer(r"(?<!!)\[([^\]\n]+)\]", "".join(consumed)):
        destination = definitions.get(" ".join(match.group(1).lower().split()))
        if destination is not None:
            destinations.append(destination)
    destinations.extend(
        match.group(2)
        for match in re.finditer(
            r"<a\b[^>]*\bhref\s*=\s*([\"'])(.*?)\1",
            body,
            flags=re.IGNORECASE,
        )
    )
    return destinations


def anchors(text: str) -> set[str]:
    result: set[str] = set()
    used: set[str] = set()
    rendered_lines: list[str] = []
    heading_lines: list[str] = []
    fence: tuple[str, int] | None = None
    for line in text.splitlines():
        marker = re.match(r"^ {0,3}((?:\x60){3,}|~{3,})(.*)$", line)
        if marker:
            current = (marker.group(1)[0], len(marker.group(1)))
            if fence is None:
                if current[0] != "\x60" or "\x60" not in marker.group(2):
                    fence = current
                    continue
            elif current[0] == fence[0] and current[1] >= fence[1] and not marker.group(2).strip():
                fence = None
                continue
        if fence:
            continue
        rendered_lines.append(strip_inline_code(line))
        heading_lines.append(strip_inline_code(line, preserve_contents=True))
    rendered_text = "\n".join(rendered_lines)
    heading_source = "\n".join(heading_lines)
    heading_parts: list[str] = []
    position = 0
    while True:
        start = heading_source.find("<!--", position)
        if start < 0:
            heading_parts.append(heading_source[position:])
            break
        heading_parts.append(heading_source[position:start])
        end = heading_source.find("-->", start + 4)
        if end < 0:
            heading_parts.append(re.sub(r"[^\n]", "\x00", heading_source[start:]))
            break
        heading_parts.append(re.sub(r"[^\n]", "\x00", heading_source[start:end + 3]))
        position = end + 3
    heading_text = "".join(heading_parts)
    while True:
        visible = re.sub(r"<!--[\s\S]*?-->", "", rendered_text)
        if visible == rendered_text:
            rendered_text = rendered_text.split("<!--", 1)[0]
            break
        rendered_text = visible
    hidden_html = r"<(?:pre|code)\b[^>]*>[\s\S]*?</(?:pre|code)>"
    heading_text = re.sub(
        hidden_html,
        lambda match: re.sub(r"[^\n]", "\x00", match.group()),
        heading_text,
        flags=re.IGNORECASE,
    )
    rendered_text = re.sub(hidden_html, "", rendered_text, flags=re.IGNORECASE)
    opening_html = r"<(?:pre|code)\b[^>]*>"
    unclosed = re.search(opening_html, heading_text, flags=re.IGNORECASE)
    if unclosed:
        heading_text = heading_text[:unclosed.start()]
    unclosed = re.search(opening_html, rendered_text, flags=re.IGNORECASE)
    if unclosed:
        rendered_text = rendered_text[:unclosed.start()]
    result.update(re.findall(r"<a\s+(?:id|name)=[\"']([^\"']+)[\"']", rendered_text, re.IGNORECASE))
    for line in heading_text.splitlines():
        heading = re.match(r"^ {0,3}#{1,6}(?:[ \t]+(.*))?$", line)
        if not heading:
            continue
        value = re.sub(r"[ \t]+#+[ \t]*$", "", heading.group(1) or "").replace("\x00", "")
        base = slug(rendered_markdown_label(value))
        candidate = base
        suffix = 0
        while candidate in used:
            suffix += 1
            candidate = base + "-" + str(suffix)
        used.add(candidate)
        result.add(candidate)
    return result


def sha256(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def safe_local_file(root: Path, value: Path) -> Path:
    source = value if value.is_absolute() else root / value
    if not stat.S_ISREG(source.lstat().st_mode):
        raise ValueError("Evidence input is not a canonical regular file")
    target = source.resolve(strict=True)
    if source.resolve(strict=False) != source or not target.is_relative_to(root):
        raise ValueError("Evidence input escapes repository or uses indirection")
    return target


def require_ancestor(repo: Path, before: str, after: str) -> None:
    ancestry = subprocess.run(
        ["git", "--no-optional-locks", "-C", str(repo), "merge-base", "--is-ancestor", before, after],
        env=GIT_ENV, capture_output=True,
    )
    require(ancestry.returncode == 0, "Wiki publication is not descended from its recorded baseline")


def validate_wiki_domain(
    before_tree: dict[str, tuple[str, str, str]],
    after_tree: dict[str, tuple[str, str, str]],
    existing_pages: list[dict[str, object]],
    source_pages: list[dict[str, object]],
) -> None:
    before_regular = {name for name, entry in before_tree.items() if regular(entry)}
    after_regular = {name for name, entry in after_tree.items() if regular(entry)}
    require(len(before_regular) == len(before_tree), "Wiki baseline contains a non-regular entry")
    require(len(after_regular) == len(after_tree), "Published Wiki contains a non-regular entry")
    recorded_existing = {relative_path(str(item["path"])) for item in existing_pages}
    source_page_names = {PurePosixPath(relative_path(str(item["path"]))).name for item in source_pages}
    require(recorded_existing == before_regular, "Wiki baseline inventory is incomplete")
    require(after_regular == before_regular | source_page_names, "Published Wiki has missing or unclassified pages")


def verify_existing_pages(
    wiki: Path,
    before_tree: dict[str, tuple[str, str, str]],
    after_tree: dict[str, tuple[str, str, str]],
    items: list[dict[str, object]],
) -> int:
    preserved = 0
    for item in items:
        relative = relative_path(str(item["path"]))
        previous_raw = blob(wiki, before_tree, relative)
        require(sha256(previous_raw) == item["sha256_before"], "Wiki baseline digest mismatch: " + relative)
        current = blob(wiki, after_tree, relative)
        if item["preserved_byte_identical"]:
            require(current == previous_raw, "Preserved Wiki page changed: " + relative)
            preserved += 1
        else:
            previous = previous_raw.decode("utf-8", "strict").splitlines()
            current_lines = iter(current.decode("utf-8", "strict").splitlines())
            require(all(any(value == line for value in current_lines) for line in previous), "Wiki baseline lines not preserved: " + relative)
    return preserved


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--wiki", type=Path, required=True)
    parser.add_argument("--record", type=Path, required=True)
    parser.add_argument("--published", type=Path, required=True)
    args = parser.parse_args()
    root = args.root.resolve(strict=True)
    wiki = args.wiki.resolve(strict=True)
    record_path = safe_local_file(root, args.record)
    published_path = safe_local_file(root, args.published)
    record = json.loads(record_path.read_text())
    published = json.loads(published_path.read_text())
    source_commit = canonical_commit(root, published["source_candidate"])
    original_commit = canonical_commit(root, record["repository_source"])
    wiki_before = canonical_commit(wiki, published["wiki_before"])
    wiki_after = canonical_commit(wiki, published["wiki_after"])
    require(record["wiki_before"] == wiki_before, "Wiki baseline mismatch")
    require(published["wiki_before"] == wiki_before, "Published Wiki baseline mismatch")
    require_ancestor(wiki, wiki_before, wiki_after)
    source_tree, original_tree = tree(root, source_commit), tree(root, original_commit)
    before_tree, after_tree = tree(wiki, wiki_before), tree(wiki, wiki_after)
    published_files = {item["path"]: item for item in published["published_files"]}
    validate_wiki_domain(before_tree, after_tree, record["existing_wiki_pages"], record["source_pages"])
    source_page_names = {PurePosixPath(relative_path(str(item["path"]))).name for item in record["source_pages"]}
    require(len(source_page_names) == len(record["source_pages"]), "Published source page names collide")
    require(len(published_files) == len(published["published_files"]), "Published receipt has duplicate paths")
    require(set(published_files) == source_page_names, "Published receipt page inventory mismatch")
    checked = fragments = 0
    for item in record["source_pages"]:
        source_path = relative_path(item["path"])
        raw = blob(root, source_tree, source_path)
        require(sha256(raw) == item["sha256"], "Source page digest mismatch: " + source_path)
        wiki_path = PurePosixPath(source_path).name
        wiki_raw = blob(wiki, after_tree, wiki_path)
        require(wiki_raw == raw, "Published Wiki page differs: " + wiki_path)
        published_item = published_files.get(wiki_path)
        require(bool(published_item), "Published page absent from receipt: " + wiki_path)
        require(sha256(wiki_raw) == published_item["sha256"], "Published digest mismatch: " + wiki_path)
        require(after_tree[wiki_path][2] == published_item["remote_blob"], "Published blob mismatch: " + wiki_path)
        if wiki_path in record["new_concept_pages"]:
            previous = blob(root, original_tree, source_path).decode("utf-8", "strict")
            require(sha256(previous.encode()) == item["original_source_sha256"], "Original source digest mismatch")
            expected = previous.replace("https://github.com/simonvanieperen/BaryonOS", "https://github.com/BaryonOS/BaryonOS")
            if wiki_path in {"FAQ.md", "Repository-Guide.md"}:
                expected = expected.replace("[LICENSE-PLAN]", "[LICENSING]").replace("/main/LICENSE-PLAN.md", "/main/LICENSING.md")
            require(raw.decode("utf-8", "strict") == expected.rstrip() + "\n", "Prepared source transformation mismatch")
        text = raw.decode("utf-8", "strict")
        for destination in link_destinations(text):
            url = urlsplit(destination)
            decoded_path = strict_unquote(url.path)
            fragment = strict_unquote(url.fragment)
            if url.netloc == "github.com" and decoded_path.startswith("/BaryonOS/BaryonOS/wiki/"):
                target = relative_path(decoded_path.split("/wiki/", 1)[1] + ".md")
                target_text = blob(wiki, after_tree, target).decode("utf-8", "strict")
                if fragment:
                    require(fragment in anchors(target_text), "Missing Wiki fragment: " + target + "#" + fragment)
                    fragments += 1
                checked += 1
            elif url.netloc == "github.com" and decoded_path.startswith("/BaryonOS/BaryonOS/blob/main/"):
                target = relative_path(decoded_path.split("/blob/main/", 1)[1])
                target_text = blob(root, source_tree, target).decode("utf-8", "strict")
                if fragment:
                    require(fragment in anchors(target_text), "Missing repository fragment: " + target + "#" + fragment)
                    fragments += 1
                checked += 1
            elif url.netloc == "github.com" and decoded_path.startswith("/BaryonOS/BaryonOS/tree/main/"):
                target = relative_path(decoded_path.split("/tree/main/", 1)[1]).rstrip("/")
                require(any(name.startswith(target + "/") for name in source_tree), "Missing repository tree: " + target)
                checked += 1
            elif not url.scheme and not url.netloc and not decoded_path and fragment:
                require(fragment in anchors(text), "Missing same-page Wiki fragment: " + wiki_path + "#" + fragment)
                fragments += 1
                checked += 1
            elif not url.scheme and decoded_path and not decoded_path.startswith("/"):
                target = relative_path(decoded_path + ".md")
                target_text = blob(wiki, after_tree, target).decode("utf-8", "strict")
                if fragment:
                    require(fragment in anchors(target_text), "Missing relative Wiki fragment: " + target)
                    fragments += 1
                checked += 1
    preserved = verify_existing_pages(wiki, before_tree, after_tree, record["existing_wiki_pages"])
    print(json.dumps({
        "repository_source": source_commit,
        "wiki_before": wiki_before,
        "wiki_after": wiki_after,
        "source_pages": len(record["source_pages"]),
        "existing_pages_preserved_byte_identical": preserved,
        "validated_local_targets": checked,
        "validated_fragments": fragments,
        "assertions_used": False,
        "exit_code": 0,
    }, sort_keys=True))


if __name__ == "__main__":
    main()
