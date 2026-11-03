"""Single-flight, image-confirmed host/VM intervals; this module runs no VM.

Call ``start`` immediately before submitting the first QMP input command. Call
``finish`` only after the complete expected framebuffer has passed its exact
oracle, with no intervening input. The interval includes QMP, scheduling,
observation and capture overhead. It is neither a guest timestamp nor physical
input-to-scanout latency. An input ACK or focus counter alone cannot finish it.

The caller remains responsible for obtaining real observations and checking
the full oracle. Hashes supplied to this recorder are bindings, not evidence
that it executed those checks. Every attempted sample is retained; failures
and superseded samples prevent acceptance even after subsequent successes.
"""

from dataclasses import asdict, dataclass
import hashlib
import json
import re
import time


MIN_SAMPLES = 20
MAX_SAMPLES = 100_000
MAX_U64 = (1 << 64) - 1
CONFIRMED_PHASE = "exact-full-frame-oracle"
_NAME = re.compile(r"[a-z][a-z0-9_-]{0,63}\Z")
_HASH = re.compile(r"[0-9a-f]{64}\Z")


def _uint(value, label, *, positive=False):
    if type(value) is not int or not (int(positive) <= value <= MAX_U64):
        raise ValueError(label + " must be a bounded unsigned integer")
    return value


def _name(value, label):
    if not isinstance(value, str) or not _NAME.fullmatch(value):
        raise ValueError("Invalid " + label)
    return value


def _digest(value, label):
    if not isinstance(value, str) or not _HASH.fullmatch(value):
        raise ValueError("Invalid " + label)
    return value


def _canonical(value):
    # Deliberately only public JSON data: no coercion, float/NaN, object hook,
    # file access or host-dependent object representation enters a binding.
    def check(item, depth=0):
        if depth > 16:
            raise ValueError("Binding JSON is too deeply nested")
        if item is None or type(item) is bool:
            return
        if type(item) is int and -MAX_U64 <= item <= MAX_U64:
            return
        if type(item) is str and len(item) <= 4096:
            return
        if type(item) is list:
            for child in item:
                check(child, depth + 1)
            return
        if type(item) is dict and all(type(key) is str for key in item):
            for key, child in item.items():
                check(key, depth + 1)
                check(child, depth + 1)
            return
        raise ValueError("Binding must contain bounded public JSON values")
    check(value)
    encoded = json.dumps(value, sort_keys=True, separators=(",", ":"),
                         ensure_ascii=True, allow_nan=False)
    if len(encoded) > 1024 * 1024:
        raise ValueError("Binding JSON is oversized")
    return encoded


def _sha(encoded):
    return hashlib.sha256(encoded.encode("ascii")).hexdigest()


def _copy(value):
    return json.loads(json.dumps(value, allow_nan=False))


def _counters(values):
    if type(values) is not dict or not values or len(values) > 64:
        raise ValueError("Expected a bounded nonempty counter dictionary")
    return { _name(key, "counter"): _uint(value, "counter")
             for key, value in sorted(values.items()) }


@dataclass(frozen=True)
class Evidence:
    source_sha256: str
    profile_sha256: str
    workload_sha256: str

    def __post_init__(self):
        for key, value in asdict(self).items():
            _digest(value, key)


@dataclass(frozen=True)
class Context:
    """Current target and presentation identities, obtained from real probes."""
    pid: int
    compositor_pid: int
    service_generation: int
    input_epoch: int
    surface_epoch: int

    def __post_init__(self):
        for key, value in asdict(self).items():
            _uint(value, key, positive=True)


@dataclass(frozen=True)
class Stratum:
    """One action/direction, its full expected image and exact counter deltas."""
    action: str
    endpoint: str
    expected_frame_sha256: str
    advances: tuple[tuple[str, int], ...]
    required_samples: int = MIN_SAMPLES

    def __post_init__(self):
        _name(self.action, "action")
        _name(self.endpoint, "endpoint")
        _digest(self.expected_frame_sha256, "expected frame hash")
        if type(self.required_samples) is not int or not MIN_SAMPLES <= self.required_samples <= MAX_SAMPLES:
            raise ValueError("At least twenty samples must be predeclared")
        if type(self.advances) is not tuple or not 1 <= len(self.advances) <= 64:
            raise ValueError("Declare at least one genuine advancing counter")
        names = []
        for item in self.advances:
            if type(item) is not tuple or len(item) != 2:
                raise ValueError("Invalid counter advance")
            names.append(_name(item[0], "counter"))
            _uint(item[1], "counter advance", positive=True)
        if len(names) != len(set(names)):
            raise ValueError("Duplicate counter advance")


class Plan:
    """Freeze this public plan and persist ``as_dict()`` before VM execution.

    Profile must identify the measured backend/configuration; workload must
    describe the fixed actions/repetitions. Budgets use integer nanoseconds and
    apply only to this declared profile and confirmed-frame endpoint. A TCG
    result cannot be relabelled as WHPX evidence.
    """
    def __init__(self, *, source_sha256, profile, workload, budgets_ns, strata):
        _digest(source_sha256, "source hash")
        if type(profile) is not dict or not profile or type(workload) is not dict or not workload:
            raise ValueError("Declare public profile and workload objects")
        profile_json, workload_json = _canonical(profile), _canonical(workload)
        if type(budgets_ns) is not dict or not budgets_ns:
            raise ValueError("Declare endpoint p95 budgets")
        budgets = {_name(key, "endpoint"): _uint(value, "p95 budget", positive=True)
                   for key, value in sorted(budgets_ns.items())}
        if type(strata) is not tuple or not strata or len(strata) > 64:
            raise ValueError("Declare a bounded tuple of strata")
        if any(type(item) is not Stratum or item.endpoint not in budgets for item in strata):
            raise ValueError("Every stratum needs a declared endpoint budget")
        if len({item.action for item in strata}) != len(strata):
            raise ValueError("Action names must identify unique strata")
        if set(budgets) != {item.endpoint for item in strata}:
            raise ValueError("Unused endpoint budget")
        if sum(item.required_samples for item in strata) > MAX_SAMPLES:
            raise ValueError("Too many planned samples")
        self._json = _canonical(dict(
            schema="nieuw-os-window-latency-plan-v1", source_sha256=source_sha256,
            source_fingerprint_version=3, profile=json.loads(profile_json),
            workload=json.loads(workload_json), budgets_ns=budgets,
            strata=[asdict(item) | {"advances": [list(pair) for pair in item.advances]}
                    for item in strata],
            quantiles="nearest-rank-ceil-p-times-n", minimum_samples=MIN_SAMPLES,
            endpoint=CONFIRMED_PHASE,
            clock="host-time.monotonic_ns", units="nanoseconds"))
        self._strata = strata
        self._evidence = Evidence(source_sha256, _sha(profile_json), _sha(workload_json))

    @property
    def sha256(self):
        return _sha(self._json)

    @property
    def evidence(self):
        return self._evidence

    def as_dict(self):
        return json.loads(self._json)


@dataclass(frozen=True)
class SampleToken:
    sample_id: int


def _quantile(values, percent):
    if not values:
        return None
    return sorted(values)[(percent * len(values) + 99) // 100 - 1]


class Recorder:
    """No retries, timeout changes, QMP, serial parsing or input injection here."""
    def __init__(self, plan, *, clock=time.monotonic_ns):
        if type(plan) is not Plan or not callable(clock):
            raise ValueError("Expected a plan and monotonic clock")
        # Own a serialized snapshot: later mutation of a caller's dictionaries
        # or Plan cannot change the identity/budget of already started samples.
        self._plan = plan.as_dict()
        self._plan_hash = plan.sha256
        self._evidence = plan.evidence
        self._strata = {item.action: item for item in plan._strata}
        self._clock = clock
        self._last_ns = None
        self._samples = []
        self._violations = []
        self._pending = None
        self._token = None
        self._observations = {}

    def _reject(self, reason):
        self._violations.append(reason)
        raise ValueError(reason)

    def _now(self):
        now = _uint(self._clock(), "host monotonic time")
        if self._last_ns is not None and now < self._last_ns:
            raise ValueError("Host monotonic clock moved backwards")
        self._last_ns = now
        return now

    def _identity(self, evidence, context):
        if type(evidence) is not Evidence or evidence != self._evidence:
            raise ValueError("Source/profile/workload binding changed")
        if type(context) is not Context:
            raise ValueError("Missing current process/epoch context")

    def start(self, action, *, evidence, context, counters):
        """Return a unique token just before the first input-command submit."""
        if self._pending is not None:
            self._reject("A sample is already in flight")
        try:
            self._identity(evidence, context)
            spec = self._strata.get(action)
            if spec is None:
                raise ValueError("Action was not predeclared")
            if sum(sample["action"] == action for sample in self._samples) >= spec.required_samples:
                raise ValueError("Predeclared action sample count exceeded")
            before = _counters(counters)
            previous = self._observations.get(context, {})
            if any(value < previous.get(name, 0) for name, value in before.items()):
                raise ValueError("Counter baseline predates an earlier confirmed sample")
            for name, delta in spec.advances:
                if name not in before or before[name] > MAX_U64 - delta:
                    raise ValueError("Missing or exhausted required counter")
            now = self._now()
        except (TypeError, ValueError) as error:
            self._reject(str(error))
        self._token = SampleToken(len(self._samples) + 1)
        self._pending = dict(sample_id=self._token.sample_id, action=action,
            endpoint=spec.endpoint, plan_sha256=self._plan_hash,
            evidence=asdict(evidence), context=asdict(context), counters_before=before,
            expected_frame_sha256=spec.expected_frame_sha256, submit_ns=now,
            status="pending")
        self._samples.append(self._pending)
        return self._token

    def _take(self, token):
        if self._pending is None or token is not self._token:
            self._reject("No matching single-flight sample")
        sample = self._pending
        self._pending = self._token = None
        return sample

    def finish(self, token, *, evidence, context, counters,
               observed_frame_sha256, phase):
        """Finish after exact *complete* image verification, never on input ACK.

        No additional injected input may occur between start and finish. The
        caller must fail a superseded sample rather than assign a later frame.
        """
        sample = self._take(token)
        try:
            now = self._now()
            sample["confirmed_ns"] = now
            self._identity(evidence, context)
            if asdict(context) != sample["context"]:
                raise ValueError("Process or presentation epoch changed")
            after = _counters(counters)
            sample["counters_after"] = after
            if set(after) != set(sample["counters_before"]):
                raise ValueError("Counter set changed")
            if any(after[key] < value for key, value in sample["counters_before"].items()):
                raise ValueError("Observed counter moved backwards")
            for name, delta in self._strata[sample["action"]].advances:
                if after[name] - sample["counters_before"][name] != delta:
                    raise ValueError("Required action progress differs from the exact declared delta")
            if type(phase) is not str or phase != CONFIRMED_PHASE:
                raise ValueError("Input ACK/partial phase is not a confirmed complete frame")
            _digest(observed_frame_sha256, "observed frame hash")
            sample["observed_frame_sha256"] = observed_frame_sha256
            if observed_frame_sha256 != sample["expected_frame_sha256"]:
                raise ValueError("Complete frame differs from the predeclared oracle")
        except (TypeError, ValueError) as error:
            sample.update(status="failed", reason=str(error))
            self._reject(str(error))
        sample.update(status="confirmed", interval_ns=now - sample["submit_ns"],
                      phase=phase)
        self._observations.setdefault(context, {}).update(after)
        return _copy(sample)

    def fail(self, token, reason, *, superseded=False):
        """Retain a failed observation; there is no success-by-retry path."""
        sample = self._take(token)
        sample.update(status="superseded" if superseded else "failed")
        try:
            if type(reason) is not str or not reason or len(reason) > 4096 or type(superseded) is not bool:
                raise ValueError("Invalid failure description")
            sample["reason"] = reason
            sample["ended_ns"] = self._now()
        except (TypeError, ValueError) as error:
            sample["reason"] = str(error)
            self._reject(str(error))

    def report(self):
        strata = {}
        for action, spec in self._strata.items():
            samples = [sample for sample in self._samples if sample["action"] == action]
            values = [sample["interval_ns"] for sample in samples if sample["status"] == "confirmed"]
            p95 = _quantile(values, 95)
            budget = self._plan["budgets_ns"][spec.endpoint]
            insufficient = len(values) < spec.required_samples
            clean = all(sample["status"] == "confirmed" for sample in samples)
            strata[action] = dict(endpoint=spec.endpoint, attempts=len(samples),
                confirmed=len(values), failed=sum(s["status"] == "failed" for s in samples),
                superseded=sum(s["status"] == "superseded" for s in samples),
                pending=sum(s["status"] == "pending" for s in samples),
                required_samples=spec.required_samples, insufficient=insufficient,
                quantile_population="confirmed samples only; every failed attempt prevents acceptance",
                p50_ns=_quantile(values, 50), p95_ns=p95, max_ns=max(values, default=None),
                p95_budget_ns=budget, budget_met=p95 is not None and p95 <= budget,
                accepted=not insufficient and clean and not self._violations
                         and self._pending is None and p95 is not None and p95 <= budget)
        # Round-trip also prevents callers mutating nested records in our state.
        return _copy(dict(
            schema="nieuw-os-window-latency-report-v1", plan_sha256=self._plan_hash,
            plan=self._plan, evidence=asdict(self._evidence),
            scope="declared-profile host input-submit to exact-full-frame confirmation; includes observation/capture overhead",
            acceptance_scope="recording contract and declared profile/stratum p95 budgets; not OS/hardware acceptance",
            guest_or_physical_scanout_latency=False, samples=self._samples,
            violations=self._violations, strata=strata,
            accepted=all(item["accepted"] for item in strata.values())))
