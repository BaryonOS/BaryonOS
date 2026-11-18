"""Read-only validation of this source-bound Wiki publication slice."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
from urllib.parse import unquote, urlsplit


GIT_ENV = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
GIT_ENV.update(GIT_NO_LAZY_FETCH="1", GIT_TERMINAL_PROMPT="0")


def require(condition, detail):
    if not condition:
        raise ValueError(detail)


def confined(base, relative):
    target = (base / relative).resolve()
    require(target.is_relative_to(base), (base, relative))
    return target


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--wiki", type=Path, required=True)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[3]
    record = json.loads(Path(__file__).with_name("publication.json").read_text())
    wiki = args.wiki.resolve()
    checked = 0
    fragments = 0
    for item in record["source_pages"]:
        source = confined(root, item["path"])
        raw = source.read_bytes()
        require(hashlib.sha256(raw).hexdigest() == item["sha256"], source)
        require(confined(wiki, source.name).read_bytes() == raw, source)
        if source.name in record["new_concept_pages"]:
            previous_source = subprocess.check_output(["git", "-C", str(root), "show", record["repository_source"] + ":" + item["path"]], text=True, env=GIT_ENV)
            require(hashlib.sha256(previous_source.encode()).hexdigest() == item["original_source_sha256"], source)
            expected = previous_source.replace("https://github.com/simonvanieperen/BaryonOS", "https://github.com/BaryonOS/BaryonOS")
            if source.name in {"FAQ.md", "Repository-Guide.md"}:
                expected = expected.replace("[LICENSE-PLAN]", "[LICENSING]").replace("/main/LICENSE-PLAN.md", "/main/LICENSING.md")
            require(raw.decode() == expected.rstrip() + "\n", source)
        for match in re.finditer(r"\[[^\]]+\]\(([^)]+)\)", raw.decode()):
            url = urlsplit(match.group(1))
            path = unquote(url.path)
            if url.netloc == "github.com" and path.startswith("/BaryonOS/BaryonOS/wiki/"):
                target = confined(wiki, path.split("/wiki/", 1)[1] + ".md")
                require(target.is_file(), (source, target))
                checked += 1
            elif url.netloc == "github.com" and path.startswith("/BaryonOS/BaryonOS/blob/main/"):
                target = confined(root, path.split("/blob/main/", 1)[1])
                require(target.is_file(), (source, target))
                checked += 1
                if url.fragment:
                    text = target.read_text()
                    anchors = set(re.findall(r'<a\s+id="([^"]+)"', text))
                    for heading in re.findall(r"^#{1,6}\s+(.+)$", text, re.M):
                        slug = re.sub(r"[^\w\- ]", "", heading.lower()).replace(" ", "-")
                        anchors.add(slug)
                    require(unquote(url.fragment) in anchors, (source, target, url.fragment))
                    fragments += 1
            elif url.netloc == "github.com" and path.startswith("/BaryonOS/BaryonOS/tree/main/"):
                target = confined(root, path.split("/tree/main/", 1)[1])
                require(target.is_dir(), (source, target))
                checked += 1
            elif not url.scheme and path and not path.startswith("/"):
                require(confined(wiki, path + ".md").is_file(), (source, path))
                checked += 1
    preserved = 0
    for item in record["existing_wiki_pages"]:
        if item["preserved_byte_identical"]:
            require(hashlib.sha256(confined(wiki, item["path"]).read_bytes()).hexdigest() == item["sha256_before"], item)
            preserved += 1
        else:
            previous = subprocess.check_output(["git", "-C", str(wiki), "show", record["wiki_before"] + ":" + item["path"]], text=True, env=GIT_ENV)
            new = iter(confined(wiki, item["path"]).read_text().splitlines())
            require(all(any(value == line for value in new) for line in previous.splitlines()), item)
    print(json.dumps({"source_pages": len(record["source_pages"]), "new_concept_pages": len(record["new_concept_pages"]), "existing_pages_preserved_byte_identical": preserved, "validated_local_targets": checked, "validated_fragments": fragments, "exit_code": 0}))


if __name__ == "__main__":
    main()
