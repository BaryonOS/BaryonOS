"""Recorder/acceptance negatives with a controlled host clock; no VM evidence."""
from dataclasses import replace
import hashlib
import sys
from pathlib import Path
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from window_latency import (CONFIRMED_PHASE, MAX_U64, Context, Plan,
                            Recorder, SampleToken, Stratum)


def digest(text):
    return hashlib.sha256(text.encode()).hexdigest()


FRAME = digest("complete expected public test framebuffer")
CONTEXT = Context(pid=4, compositor_pid=2, service_generation=1,
                  input_epoch=8, surface_epoch=9)


class Clock:
    def __init__(self):
        self.now = 0

    def __call__(self):
        return self.now


def plan(*, budget=19, required=20, extra=()):
    return Plan(source_sha256=digest("source"),
        profile={"name": "host-recorder-test", "backend": "whpx", "vcpus": 1},
        workload={"actions": ["key-a"], "repetitions": required, "warmup": 0},
        budgets_ns={"input": budget},
        strata=(Stratum("key-a", "input", FRAME,
                        (("input_acks", 2), ("generation", 1)), required),) + extra)


def before(index):
    return {"input_acks": index * 2, "generation": index}


def after(index):
    return {"input_acks": index * 2 + 2, "generation": index + 1}


def start(recorder, declared, index=0):
    return recorder.start("key-a", evidence=declared.evidence,
                          context=CONTEXT, counters=before(index))


def finish(recorder, declared, token, index=0, **changes):
    values = dict(evidence=declared.evidence, context=CONTEXT,
                  counters=after(index), observed_frame_sha256=FRAME,
                  phase=CONFIRMED_PHASE)
    values.update(changes)
    return recorder.finish(token, **values)


class LatencyRecorderTests(unittest.TestCase):
    def test_nearest_rank_budgets_minimum_and_independent_action_strata(self):
        declared, clock = plan(), Clock()
        recorder = Recorder(declared, clock=clock)
        # Reverse order distinguishes sorting/nearest rank from arrival order.
        for index, duration in enumerate(range(20, 0, -1)):
            token = start(recorder, declared, index)
            clock.now += duration
            finish(recorder, declared, token, index)
            if index == 18:
                self.assertFalse(recorder.report()["accepted"])
                self.assertTrue(recorder.report()["strata"]["key-a"]["insufficient"])
        report = recorder.report()
        self.assertEqual((report["strata"]["key-a"]["p50_ns"],
                          report["strata"]["key-a"]["p95_ns"],
                          report["strata"]["key-a"]["max_ns"]), (10, 19, 20))
        self.assertTrue(report["accepted"])
        self.assertFalse(report["guest_or_physical_scanout_latency"])

        slow = plan(budget=18)
        too_slow = Recorder(slow, clock=clock)
        for index in range(20):
            token = start(too_slow, slow, index)
            clock.now += 19
            finish(too_slow, slow, token, index)
        self.assertFalse(too_slow.report()["accepted"])
        self.assertFalse(too_slow.report()["strata"]["key-a"]["budget_met"])

        two = plan(extra=(Stratum("key-backspace", "input", FRAME,
                                  (("input_acks", 2),)),))
        incomplete = Recorder(two, clock=clock)
        for index in range(20):
            token = start(incomplete, two, index)
            clock.now += 1
            finish(incomplete, two, token, index)
        self.assertFalse(incomplete.report()["accepted"])
        self.assertTrue(incomplete.report()["strata"]["key-backspace"]["insufficient"])

    def test_failures_superseded_and_partial_oracles_never_become_green(self):
        for mode in ("failed", "superseded", "input-ack", "wrong-frame"):
            with self.subTest(mode=mode):
                declared, clock = plan(budget=100), Clock()
                recorder = Recorder(declared, clock=clock)
                token = start(recorder, declared)
                clock.now += 1
                if mode in ("failed", "superseded"):
                    recorder.fail(token, "actual observation did not complete",
                                  superseded=mode == "superseded")
                else:
                    changes = {"phase": "input-ack"} if mode == "input-ack" else {
                        "observed_frame_sha256": digest("partial or different frame")}
                    with self.assertRaises(ValueError):
                        finish(recorder, declared, token, **changes)
                for index in range(1, 20):
                    token = start(recorder, declared, index)
                    clock.now += 1
                    finish(recorder, declared, token, index)
                result = recorder.report()
                self.assertFalse(result["accepted"])
                self.assertEqual(len(result["samples"]), 20)
                self.assertEqual(result["strata"]["key-a"]["confirmed"], 19)
                with self.assertRaises(ValueError):
                    start(recorder, declared, 20)
                self.assertFalse(recorder.report()["accepted"])

    def test_current_source_profile_workload_and_process_epoch_are_bound(self):
        declared = plan()
        mismatches = [{"evidence": replace(declared.evidence, **{field: digest(field)})}
                      for field in ("source_sha256", "profile_sha256", "workload_sha256")]
        mismatches += [{"context": replace(CONTEXT, **{field: value + 1})}
                       for field, value in CONTEXT.__dict__.items()]
        for changes in mismatches:
            with self.subTest(changes=changes):
                clock = Clock()
                recorder = Recorder(declared, clock=clock)
                token = start(recorder, declared)
                clock.now += 1
                with self.assertRaises(ValueError):
                    finish(recorder, declared, token, **changes)
                result = recorder.report()
                self.assertEqual(result["samples"][0]["status"], "failed")
                self.assertFalse(result["accepted"])
        tcg = Plan(source_sha256=declared.evidence.source_sha256,
            profile={"name": "host-recorder-test", "backend": "tcg", "vcpus": 1},
            workload=declared.as_dict()["workload"], budgets_ns={"input": 19},
            strata=declared._strata)
        with self.assertRaises(ValueError):
            Recorder(declared).start("key-a", evidence=tcg.evidence,
                                     context=CONTEXT, counters=before(0))

    def test_single_flight_counter_progress_replay_and_overflow_fail_closed(self):
        declared, clock = plan(), Clock()
        recorder = Recorder(declared, clock=clock)
        token = start(recorder, declared)
        self.assertFalse(recorder.report()["accepted"])
        with self.assertRaises(ValueError):
            start(recorder, declared)
        with self.assertRaises(ValueError):
            finish(recorder, declared, SampleToken(token.sample_id))
        finish(recorder, declared, token)
        with self.assertRaises(ValueError):
            start(recorder, declared, 0)  # A stale baseline cannot replay a frame.
        self.assertFalse(recorder.report()["accepted"])

        for counters in ({"input_acks": 0, "generation": 0},  # No action.
                         {"input_acks": 4, "generation": 1},  # Extra input.
                         {"input_acks": 2},  # Changed counter set.
                         {"input_acks": True, "generation": 1}):
            recorder = Recorder(declared, clock=clock)
            token = start(recorder, declared)
            with self.assertRaises(ValueError):
                finish(recorder, declared, token, counters=counters)
            self.assertFalse(recorder.report()["accepted"])
        with self.assertRaises(ValueError):
            Recorder(declared).start("key-a", evidence=declared.evidence,
                context=CONTEXT, counters={"input_acks": MAX_U64 - 1, "generation": 0})

    def test_nonmonotonic_clock_and_returned_data_cannot_rewrite_evidence(self):
        declared, clock = plan(), Clock()
        recorder = Recorder(declared, clock=clock)
        clock.now = 100
        token = start(recorder, declared)
        clock.now = 99
        with self.assertRaises(ValueError):
            finish(recorder, declared, token)
        self.assertEqual(recorder.report()["samples"][0]["status"], "failed")
        self.assertFalse(recorder.report()["accepted"])

        recorder, clock = Recorder(declared, clock=clock), clock
        clock.now = 100
        token = start(recorder, declared)
        clock.now = 101
        returned = finish(recorder, declared, token)
        returned["context"]["pid"] = 0
        returned["counters_after"]["generation"] = 0
        report = recorder.report()
        report["samples"][0]["context"]["pid"] = 0
        report["plan"]["budgets_ns"]["input"] = MAX_U64
        self.assertEqual(recorder.report()["samples"][0]["context"]["pid"], CONTEXT.pid)
        self.assertEqual(recorder.report()["samples"][0]["counters_after"]["generation"], 1)
        self.assertEqual(recorder.report()["plan"]["budgets_ns"]["input"], 19)
        clock.now = 100
        with self.assertRaises(ValueError):
            start(recorder, declared, 1)
        self.assertFalse(recorder.report()["accepted"])

    def test_plan_freezes_public_json_and_requires_predeclared_sample_floor(self):
        profile = {"backend": "tcg", "nested": ["public fixture"]}
        declared = Plan(source_sha256=digest("source"), profile=profile,
            workload={"repetitions": 20}, budgets_ns={"input": 100},
            strata=(Stratum("key-a", "input", FRAME, (("input_acks", 2),)),))
        old_hash = declared.sha256
        profile["backend"] = "whpx"
        profile["nested"].append("changed")
        self.assertEqual(declared.sha256, old_hash)
        self.assertEqual(declared.as_dict()["profile"]["backend"], "tcg")
        for required in (0, 19, True):
            with self.assertRaises(ValueError):
                plan(required=required)
        for invalid in (float("nan"), 1.5, Path("not-public-json")):
            with self.assertRaises(ValueError):
                Plan(source_sha256=digest("source"), profile={"invalid": invalid},
                     workload={"repetitions": 20}, budgets_ns={"input": 100},
                     strata=declared._strata)


if __name__ == "__main__":
    unittest.main()
