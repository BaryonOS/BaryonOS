#!/usr/bin/env python3
"""U05-T3 native local search runner (case 25 only; draft from step S2).

One cold QEMU boot of a test-volume-search build on a blank disposable volume
image. Both app slots run sdk/examples/search-volume's search-app over their
own volume domain. A pass needs all of the following:
- the full marker chain, ending in VOLUME:CASE:SEARCH:PASS, with no failure marker;
- a raw QEMU exit of 33 and no timeout;
- the after-image decoded by tools/volume_disk.py, in exactly the state the
  app flow leaves;
- the report words of both slots equal to the values that the independent
  oracle (tests/test_search_oracle.py) recomputes from that image;
- the negative-check mask complete, at least MIN_REQUESTS requests per slot,
  and exactly the two foreign probes per slot in SERVICE:REJECTED.

The runner is NOT runnable before the S3 wiring exists (kernel feature, case 25
in volume.rs including slot B's words in finish(), and the tools/build.py
override). It also reuses the QEMU launcher of tools/run_settings.py (#436).
Without these it exits non-zero before any boot. Evidence is local QEMU only;
see sdk/examples/search-volume/README.md for the limits.
"""
from __future__ import annotations
import argparse
from datetime import datetime, timezone
import json
from pathlib import Path
import sys
import time
import uuid
from common import BUILD, ROOT, save_json, sha256, source_hash
from run_volume import BOOT, FAILURES, controlled, numbers
import volume_disk

sys.path.insert(0, str(ROOT / "tests"))
import test_search_oracle as oracle  # noqa: E402  (shared independent oracle helpers)

FEATURE = "test-volume-search"
MARKERS = BOOT + ("VOLUME:MANIFEST:PASS", "VOLUME:READY", "VOLUME:CASE:SEARCH:PASS",
                  "VOLUME:RESOURCES:PASS", "VOLUME:PASS", "M1:PASS")
TOKENS = ("alpha", "beta")  # own token of domain 0 and domain 1
# Fixed query set of search-app, in its order; the last entry is the other domain's token.
COMMON = ("notes", "final", "draft", "budget", "archive", "projects")
ALL_NEGATIVES = (1 << 12) - 1
FOREIGN_PROBES = 2  # foreign STAT + LIST per slot; the only requests the service rejects
# Lower bound, not an exact count: 6 forced operations (each one mutation plus
# a LIST, 4 COLLECTION_STAT and a LIST), 3 uploads (BEGIN, APPEND, COMMIT),
# rename, delete, stale rename and the foreign probes.
MIN_REQUESTS = 48
WORDS = ("DOCS", "DIGEST", "RECOVERED_REVISION", "RECOVERED_COLLECTION", "RECOVERED_SECOND")
MAX_TIMEOUT = 600


def queries(domain: int) -> list[bytes]:
    return [q.encode() for q in (TOKENS[domain], *COMMON, TOKENS[1 - domain])]


def expected_state(domain: int):
    """(objects, collections) the app flow leaves: object -> (present, revision,
    name, collection); (domain, index) -> (present, revision, name)."""
    base, own = domain * volume_disk.PER_DOMAIN, TOKENS[domain]
    objects = {base + i: (False, 0, None, 0) for i in range(volume_disk.PER_DOMAIN)}
    objects[base] = (True, 2, f"{own} notes", 1)
    objects[base + 1] = (True, 2, f"{own} final", 0)
    objects[base + 2] = (False, 2, None, 0)
    collections = {(domain, i): (False, 0, None) for i in range(1, volume_disk.COLLECTIONS + 1)}
    collections[(domain, 1)] = (True, 2, "Archive")
    return objects, collections


def check_state(image_view: dict) -> None:
    """The analysed after-image is exactly the scripted end state of both domains."""
    for domain in range(volume_disk.DOMAINS):
        objects, collections = expected_state(domain)
        for obj, (present, revision, name, collection) in objects.items():
            item = image_view["objects"][obj]
            got = (item["present"], item["revision"],
                   item["versions"][0]["name"] if item["present"] else None,
                   item["collection"] if item["present"] else 0)
            if got != (present, revision, name, collection):
                raise ValueError(f"object {obj} is {got}, expected {(present, revision, name, collection)}")
        for item in image_view["collections"]:
            key = (item["domain"], item["index"])
            if key in collections:
                got = (item["present"], item["revision"], item["name"] if item["present"] else None)
                if got != collections[key]:
                    raise ValueError(f"collection {key} is {got}, expected {collections[key]}")


def oracle_words(image_view: dict, domain: int) -> tuple[int, int, int, int]:
    """(docs, digest, own hits, other hits) recomputed from the image alone."""
    labels = {c["index"]: c["name"].encode() for c in image_view["collections"]
              if c["domain"] == domain and c["present"]}
    records = [(o["object"], o["revision"], [o["versions"][0]["name"].encode(), labels.get(o["collection"], b"")])
               for o in image_view["objects"]
               if o["present"] and o["object"] // volume_disk.PER_DOMAIN == domain]
    value, counts = oracle.FNV_OFFSET, []
    for text in queries(domain):
        hits = [(obj, rev) for obj, rev, fields in records if oracle.matches(fields, text)]
        counts.append(len(hits))
        value = oracle.digest(value, hits)
    return counts[0], value, counts[0], counts[-1]


def verdict(stdout: str, raw_exit: int | None, timed_out: bool, image_view: dict) -> dict:
    """Fail closed: raise ValueError on the first unmet condition."""
    if controlled(stdout) != list(MARKERS):
        raise ValueError(f"marker sequence mismatch: {controlled(stdout)}")
    if any(marker in line for line in stdout.splitlines() for marker in FAILURES):
        raise ValueError("failure marker present")
    if raw_exit != 33 or timed_out:
        raise ValueError(f"raw exit {raw_exit}, timeout {timed_out}")
    evidence = numbers(stdout)
    check_state(image_view)
    reported = {}
    for domain, slot in enumerate("AB"):
        try:
            docs, digest, own, other, mask = (evidence[f"VOLUME:{slot}:{word}:"] for word in WORDS)
            requests = evidence[f"VOLUME:{slot}:REQUESTS:"]
        except KeyError as missing:
            raise ValueError(f"missing evidence {missing}") from None
        want = oracle_words(image_view, domain)
        if (docs, digest, own, other) != want:
            raise ValueError(f"slot {slot} words {(docs, digest, own, other)} != oracle {want}")
        if other != 0 or own != 2:
            raise ValueError(f"slot {slot} own/other hits {own}/{other}, expected 2/0")
        if mask != ALL_NEGATIVES:
            raise ValueError(f"slot {slot} negative mask {mask:#x} != {ALL_NEGATIVES:#x}")
        if requests < MIN_REQUESTS:
            raise ValueError(f"slot {slot} sent {requests} requests, expected at least {MIN_REQUESTS}")
        reported[slot] = dict(docs=docs, digest=digest, own_hits=own, other_hits=other,
                              negatives=mask, requests=requests)
    rejected = evidence.get("VOLUME:SERVICE:REJECTED:")
    if rejected != FOREIGN_PROBES * volume_disk.DOMAINS:
        raise ValueError(f"service rejected {rejected}, expected {FOREIGN_PROBES * volume_disk.DOMAINS}")
    return dict(evidence=evidence, reported=reported)


def require_wiring(root: Path = ROOT) -> None:
    """Refuse clearly when the shared S3 kernel/build wiring is absent."""
    def text(path):
        file = root / path
        return file.read_text(encoding="utf-8") if file.is_file() else ""
    cargo, volume, build = text("kernel/Cargo.toml"), text("kernel/src/m2/m3/volume.rs"), text("tools/build.py")
    missing = [what for what, ok in (
        (f"kernel/Cargo.toml feature {FEATURE}", f'{FEATURE} = ["native-volume"]' in cargo),
        ("kernel/src/m2/m3/volume.rs case 25 with marker VOLUME:CASE:SEARCH:PASS",
         "VOLUME:CASE:SEARCH:PASS" in volume),
        ("kernel/src/m2/m3/volume.rs finish() printing slot B's words for case 25",
         "VOLUME:B:RECOVERED_SECOND:" in volume),
        ("tools/build.py VOLUME_SEARCH_CASES and the search-app override",
         "VOLUME_SEARCH_CASES" in build and "search-volume" in build),
        ("tools/run_settings.py boot() (#436)", "def boot(" in text("tools/run_settings.py"))) if not ok]
    if missing:
        raise SystemExit("run_search: missing shared edit(s): " + "; ".join(missing)
                         + " (U05-T3 step S3; see sdk/examples/search-volume/README.md)")


def check_build(build_dir: Path, frozen: str) -> dict:
    manifest_path = build_dir / "manifest.json"
    if not manifest_path.is_file():
        raise SystemExit(f"run_search: no build in {build_dir}; run NOS_BUILD_DIR={build_dir} "
                         f"python3 -B tools/build.py --kernel-feature {FEATURE}")
    manifest = json.loads(manifest_path.read_text())
    if manifest.get("kernel_feature") != FEATURE or manifest.get("source_sha256") != frozen:
        raise SystemExit(f"run_search: {build_dir} is not a {FEATURE} build of the current source")
    return manifest


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--build", type=Path, default=BUILD, help=f"NOS_BUILD_DIR of a {FEATURE} build")
    parser.add_argument("--timeout", type=int, default=MAX_TIMEOUT)
    args = parser.parse_args()
    if not 0 < args.timeout <= MAX_TIMEOUT:
        raise SystemExit("invalid timeout")
    require_wiring()
    from run_settings import boot  # present only after #436 (checked above)
    frozen = source_hash()
    build_dir = args.build.resolve()
    manifest = check_build(build_dir, frozen)
    run_dir = build_dir / "run" / f"search-{uuid.uuid4().hex[:10]}"
    run_dir.mkdir(parents=True)
    image = run_dir / "volume.img"
    volume_disk.create_image(image, "empty")
    before = sha256(image)
    started = datetime.now(timezone.utc).isoformat()
    start = time.monotonic()
    command, version, rc, out, err, timed_out = boot(build_dir, run_dir, image, False, args.timeout,
                                                     manifest["kernel"]["sha256"])
    duration = time.monotonic() - start
    (run_dir / "serial.txt").write_text(out, encoding="utf-8")
    (run_dir / "stderr.txt").write_text(err, encoding="utf-8")
    report = dict(schema_version=1, case="search", kernel_feature=FEATURE, started_utc=started,
                  duration_s=duration, raw_qemu_exit=rc, timed_out=timed_out, command=command,
                  qemu_version=version, source_sha256=frozen, kernel_sha256=manifest["kernel"]["sha256"],
                  app=manifest.get("volume_docs"), before_sha256=before, after_sha256=sha256(image),
                  run_directory=str(run_dir), passed=False)
    try:
        report.update(verdict(out, rc, timed_out, volume_disk.analyse(image.read_bytes())))
        report["passed"] = source_hash() == frozen
        if not report["passed"]:
            raise ValueError("source changed during run")
    except Exception as error:  # recorded in the result, then fail closed
        report["error"] = f"{type(error).__name__}: {error}"
    save_json(run_dir / "result.json", report)
    print(f"U05-T3 search: passed={report['passed']} raw={rc} duration={duration:.1f}s {run_dir}", flush=True)
    if "error" in report:
        print(report["error"], flush=True)
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
