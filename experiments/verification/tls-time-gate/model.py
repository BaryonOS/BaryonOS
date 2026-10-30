#!/usr/bin/env python3
"""Bounded abstract checker for the current Nieuw OS TLS certificate-time gate."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from itertools import product
import json


TRACE_BOUND = 3
PLAUSIBLE_SECONDS = 1_793_361_600
INT64_MIN = -(1 << 63)
INT64_MAX = (1 << 63) - 1


class SourceKind(Enum):
    MISSING = "missing"
    RTC = "unauthenticated_rtc"
    ERROR = "runtime_error"


class TimeUnavailable(Enum):
    MISSING_SOURCE = "MissingSource"
    UNAUTHENTICATED_RTC = "UnauthenticatedRtc"


@dataclass(frozen=True)
class Observation:
    kind: SourceKind
    seconds: int = 0
    error_code: int = 0


@dataclass(frozen=True)
class Outcome:
    core_seconds: int | None
    core_error: TimeUnavailable | None
    checked_seconds: int | None
    checked_error: tuple[str, int | str] | None
    provider_seconds: int | None


RTC_SECONDS = (
    INT64_MIN,
    -62_135_596_800,
    -1,
    0,
    1,
    PLAUSIBLE_SECONDS - 1,
    PLAUSIBLE_SECONDS,
    PLAUSIBLE_SECONDS + 1,
    INT64_MAX,
)
SOURCE_ERRORS = (-5, -9, -22)
OBSERVATIONS = (
    Observation(SourceKind.MISSING),
    *(Observation(SourceKind.RTC, seconds=value) for value in RTC_SECONDS),
    *(Observation(SourceKind.ERROR, error_code=code) for code in SOURCE_ERRORS),
)


def core_certificate_time(
    observation: Observation, *, promote_rtc_mutant: bool = False
) -> tuple[int | None, TimeUnavailable | None]:
    """Abstract CORE::certificate_time; the flag is an intentional bad mutant."""
    if observation.kind is SourceKind.MISSING:
        return None, TimeUnavailable.MISSING_SOURCE
    if observation.kind is SourceKind.RTC:
        if promote_rtc_mutant and observation.seconds >= 0:
            return observation.seconds, None
        return None, TimeUnavailable.UNAUTHENTICATED_RTC
    raise ValueError("runtime source errors bypass CORE::certificate_time")


def evaluate(
    observation: Observation, *, promote_rtc_mutant: bool = False
) -> Outcome:
    """Compose the CORE gate, NET wrapper, and rustls TimeProvider adapter."""
    if observation.kind is SourceKind.ERROR:
        error = ("Source", observation.error_code)
        return Outcome(None, None, None, error, None)

    core_seconds, core_error = core_certificate_time(
        observation, promote_rtc_mutant=promote_rtc_mutant
    )
    if core_error is not None:
        error = ("Unavailable", core_error.value)
        return Outcome(None, core_error, None, error, None)

    assert core_seconds is not None
    # checked_time converts admitted u64 seconds to rustls UnixTime.
    # current_time returns Some only when checked_now succeeds.
    return Outcome(core_seconds, None, core_seconds, None, core_seconds)


def violations(observation: Observation, outcome: Outcome) -> tuple[str, ...]:
    """Check observable gate properties, without modeling a TLS handshake."""
    failures: list[str] = []
    if observation.kind is SourceKind.MISSING:
        if outcome.core_error is not TimeUnavailable.MISSING_SOURCE:
            failures.append("missing source was not rejected by CORE")
        if outcome.checked_error != ("Unavailable", "MissingSource"):
            failures.append("missing source was not preserved as NET unavailability")
    elif observation.kind is SourceKind.RTC:
        if outcome.core_seconds is not None:
            failures.append("UnauthenticatedRtc produced CertificateTime")
        if outcome.core_error is not TimeUnavailable.UNAUTHENTICATED_RTC:
            failures.append("RTC quality was not rejected by CORE")
        if outcome.checked_seconds is not None:
            failures.append("UnauthenticatedRtc produced rustls UnixTime")
        if outcome.checked_error != ("Unavailable", "UnauthenticatedRtc"):
            failures.append("RTC rejection was not preserved by checked_now")
    else:
        if outcome.checked_error != ("Source", observation.error_code):
            failures.append("runtime source error was not preserved")
    if observation.kind is not SourceKind.ERROR and outcome.provider_seconds is not None:
        failures.append("failed time admission produced TimeProvider::Some")
    if observation.kind is SourceKind.ERROR and outcome.provider_seconds is not None:
        failures.append("source failure produced TimeProvider::Some")
    return tuple(failures)


def enumerate_histories(trace_bound: int = TRACE_BOUND) -> tuple[int, int, tuple[str, ...]]:
    """Exhaustively check every observation trace up to the configured bound."""
    histories = 0
    calls = 0
    reachable: set[str] = set()
    for length in range(trace_bound + 1):
        for history in product(OBSERVATIONS, repeat=length):
            histories += 1
            for observation in history:
                calls += 1
                outcome = evaluate(observation)
                failures = violations(observation, outcome)
                if failures:
                    raise AssertionError((history, observation, failures))
                if observation.kind is SourceKind.MISSING:
                    reachable.add("MissingSource")
                elif observation.kind is SourceKind.RTC:
                    reachable.add("UnauthenticatedRtc")
                else:
                    reachable.add("SourceError")
                if outcome.provider_seconds is None:
                    reachable.add("ProviderNone")
    required = {"MissingSource", "UnauthenticatedRtc", "SourceError", "ProviderNone"}
    if reachable != required:
        raise AssertionError(f"unexpected reachability set: {sorted(reachable)}")
    return histories, calls, tuple(sorted(reachable))


def find_rtc_promotion_counterexample() -> dict[str, object]:
    """Run the deliberately faulty RTC-promotion variant and return its witness."""
    witness = Observation(SourceKind.RTC, seconds=PLAUSIBLE_SECONDS)
    outcome = evaluate(witness, promote_rtc_mutant=True)
    failures = violations(witness, outcome)
    if not failures:
        raise AssertionError("bounded checker failed to detect RTC-promotion mutant")
    return {
        "input": {
            "kind": witness.kind.value,
            "seconds": witness.seconds,
        },
        "mutant_outcome": {
            "core_certificate_time_seconds": outcome.core_seconds,
            "net_checked_now_seconds": outcome.checked_seconds,
            "rustls_time_provider_seconds": outcome.provider_seconds,
        },
        "violations": list(failures),
    }


def main() -> int:
    histories, calls, reachable = enumerate_histories()
    counterexample = find_rtc_promotion_counterexample()
    print(
        json.dumps(
            {
                "model": "TASK-V18-TLS-TIME-FAILCLOSED",
                "result": "PASS",
                "trace_bound": TRACE_BOUND,
                "observation_alphabet_size": len(OBSERVATIONS),
                "histories_explored": histories,
                "source_calls_checked": calls,
                "reachable_outcomes": reachable,
                "rtc_promotion_mutant": "DETECTED",
                "mutant_counterexample": counterexample,
                "authenticated_handshake_modeled": False,
            },
            indent=2,
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
