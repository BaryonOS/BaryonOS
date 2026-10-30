from __future__ import annotations

import unittest

from model import (
    OBSERVATIONS,
    PLAUSIBLE_SECONDS,
    Observation,
    SourceKind,
    TimeUnavailable,
    core_certificate_time,
    enumerate_histories,
    evaluate,
    find_rtc_promotion_counterexample,
    violations,
)


class TlsTimeGateModelTests(unittest.TestCase):
    def test_bounded_exploration_covers_all_current_input_classes(self) -> None:
        histories, calls, reachable = enumerate_histories()
        self.assertEqual(histories, 2380)
        self.assertEqual(calls, 6942)
        self.assertEqual(
            set(reachable),
            {"MissingSource", "ProviderNone", "SourceError", "UnauthenticatedRtc"},
        )
        self.assertEqual(len(OBSERVATIONS), 13)

    def test_missing_and_rtc_inputs_are_rejected_at_each_time_boundary(self) -> None:
        missing = Observation(SourceKind.MISSING)
        rtc = Observation(SourceKind.RTC, seconds=PLAUSIBLE_SECONDS)
        self.assertEqual(
            core_certificate_time(missing),
            (None, TimeUnavailable.MISSING_SOURCE),
        )
        self.assertEqual(
            core_certificate_time(rtc),
            (None, TimeUnavailable.UNAUTHENTICATED_RTC),
        )
        self.assertEqual(violations(missing, evaluate(missing)), ())
        self.assertEqual(violations(rtc, evaluate(rtc)), ())

    def test_source_error_is_preserved_and_provider_denies_time(self) -> None:
        observation = Observation(SourceKind.ERROR, error_code=-9)
        outcome = evaluate(observation)
        self.assertEqual(outcome.checked_error, ("Source", -9))
        self.assertIsNone(outcome.provider_seconds)
        self.assertEqual(violations(observation, outcome), ())

    def test_intentionally_wrong_rtc_promotion_has_concrete_counterexample(self) -> None:
        witness = find_rtc_promotion_counterexample()
        self.assertEqual(witness["input"]["kind"], "unauthenticated_rtc")
        self.assertEqual(witness["input"]["seconds"], PLAUSIBLE_SECONDS)
        self.assertEqual(
            witness["mutant_outcome"]["rustls_time_provider_seconds"],
            PLAUSIBLE_SECONDS,
        )
        self.assertIn(
            "UnauthenticatedRtc produced CertificateTime",
            witness["violations"],
        )
        self.assertIn(
            "failed time admission produced TimeProvider::Some",
            witness["violations"],
        )


if __name__ == "__main__":
    unittest.main()
