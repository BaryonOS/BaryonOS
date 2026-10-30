# TASK-V18-TLS-TIME-FAILCLOSED

**Status:** bounded abstract model and executable mutation check; no Rust or
TLS-stack changes. **Model baseline:** repository commit
`ca45a7dabe6a040921de03bdc2fad3e5045a75ba` (`simonvanieperen-upgraded-guide`).
The two inspected source commits are separate, frozen inputs and are not
claimed to be ancestors of that baseline.

## Property and modeled boundary

For the currently implemented time qualities and wrappers:

1. `None` is rejected as `MissingSource`.
2. Any `WallClockQuality::UnauthenticatedRtc`, for every tested timestamp
   class, is rejected as `UnauthenticatedRtc`; it cannot produce a
   `CertificateTime`.
3. The NET adapter cannot turn either rejection into a `rustls::UnixTime`.
   `CertificateClock::checked_now` preserves runtime-source failures as
   `Error::Source` and admission failures as `Error::Unavailable`.
4. `TimeProvider::current_time` has no value for either error class. Separate
   source inspection confirms the native reader maps a successful RTC read to
   `Some` and propagates a runtime error without a clock fallback; that
   low-level adapter is not independently executed by this abstract model.
5. A length-three sequence of missing, RTC, and source-error observations
   cannot change those per-call results. Repeated, increasing and decreasing
   RTC samples therefore do not acquire a new quality in this stateless
   boundary.

`CertificateTime` construction is a separate Rust privacy/type-system fact:
the source declares a private field and no public constructor. The model
records that source-bound assumption; it does not prove Rust visibility,
compile-fail doctests, or the compiled binary.

There is **no authenticated-success or handshake operation in the model**:
the inspected NET leaf has no such entrypoint, and this work does not claim
that a rustls handshake, certificate-chain verification, hostname check,
peer authentication, or native TLS connection was executed. `UnixTime`
means only that the adapter admitted a time value. The checked property is
the current fail-closed time gate, not a complete TLS authentication theorem.
Future authenticated time, freshness, uncertainty and persistent rollback
protection remain open and are deliberately not assigned invented semantics.

## Source binding and review basis

The model was derived by read-only inspection. SHA-256 values below are of
the exact file blobs at the named commits, not of the whole repository.
Line ranges are one-based in those blobs.

| Source | Commit | File SHA-256 | Relevant lines / facts |
|---|---|---|---|
| CORE | `9afabbd8266cd1230b7b89b6c70a56caa22f88ce` | `f135b4b848ba41510662ac25c69e7cc0b1e3629825f36eb697a0b98c82285f79` | `sdk/runtime/src/tls_time.rs:29-44` private `CertificateTime`; `46-67` exhaustive current-quality rejection |
| CORE contract | `9afabbd8266cd1230b7b89b6c70a56caa22f88ce` | `41dfebea098aeed16486a5e63e1b1f864dec674696749085c585a85e18936ceb` | `docs/contracts/TLS-TIME-QUALITY.md:19-56` invariant, no safe promotion, no handshake claim; `58-80` future time properties remain open |
| CORE tests | `9afabbd8266cd1230b7b89b6c70a56caa22f88ce` | `0216d00ade9f99110fc054acec88cd17a4a12a4aa3c33ebe239673ca54f0a536` | `sdk/runtime/tests/tls_time.rs`; missing, RTC extremes/plausible values, repeated/forward/backward, fallback attempt |
| NET | `4a0ce99ff6b88adcd0e2c6be0571b30390d8d1c8` | `a6f0cc80ca8a1ec1b2c910c9cd38452c07afaea2902134a3181b6d1e52733f1e` | `services/net-tls/src/time.rs:9-15` CORE-only conversion; `18-36` source-vs-unavailable error; `39-42` rustls `None`; `45-48` native read maps to `Some` without fallback |
| NET tests | `4a0ce99ff6b88adcd0e2c6be0571b30390d8d1c8` | `796ee4fcce0c85cef535baaca312d2104ce5ea264aee4fa0c262364fcaf9a809` | `services/net-tls/tests/time.rs`; missing source, RTC samples, and source-error outcomes |
| NET docs | `4a0ce99ff6b88adcd0e2c6be0571b30390d8d1c8` | `575bcc8f970b3ae6f42c35d373732316224ed10082a295046f0eb0793fe51030` | `services/net-tls/README.md:33-45` states error propagation, no fallback, and no existing TLS consumer |

Security/domain coordination: SEC (session `081d6f31`) agreed to this
fail-closed property without new time-trust semantics; NET (session
`039dbef8`) confirmed ownership of the exact NET leaf and confirmed there is
no authenticated-success entrypoint. The manager (session `89f84098`)
assigned this task and this exclusive model directory. CORE owner/session 14
(`fbf60554`) was asked to confirm the bounded source scope; no separate
acknowledgment was available at execution time. The exact CORE source and
contract were nevertheless inspected read-only. This correspondence gap
does not authorize changes to CORE, SDK, kernel, NET, shared progress files,
or product contracts.

## Model, actions, failures, and assumptions

`model.py` is a deterministic explicit-state enumerator using only the Python
standard library. An observation is one invocation's reader result:

- `Missing`: `Ok(None)`;
- `RTC(t)`: `Ok(Some(WallClockReading { quality: UnauthenticatedRtc, ... }))`;
- `SourceError(e)`: `Err(RuntimeError(e))`.

The finite alphabet contains missing, three opaque runtime-error codes, and
nine `i64` representatives: minimum, a low valid signed time, negative one,
epoch, one, plausible time minus/at/plus one second, and maximum. The core
function does not branch on timestamp; it matches `Option` and the only
current quality. Representative values exercise extremes and plausible
values without claiming to enumerate all `2^64` bit patterns.

The modeled wrapper actions are serialized per call and compose the
`CertificateClock` reader result, CORE admission, NET `checked_time`, NET
`checked_now`, and `TimeProvider::current_time`. There is no shared mutable
clock state in these functions. The checker enumerates all histories of zero through three
observations: alphabet size 13, **2,380 histories / 6,942 source-call
positions**. This bounded sequence exploration checks that order and
repetition do not create trust; it is not an SMP, race, latency, or liveness
model.

The deliberate bad variant `promote_rtc_mutant` incorrectly admits a
non-negative RTC sample as `CertificateTime`. The checker must return a
concrete violation rather than treating the empty success set as sufficient.
The witness uses plausible Unix seconds `1793361600` and shows the mutant
reaching both NET `UnixTime` and `TimeProvider::Some`.

Assumptions: the manually transcribed model transitions correspond to the
listed source lines and exact file hashes; the current source-quality set is
only `UnauthenticatedRtc`; calls are serialized at the wrapper boundary; and
the rustls trait adapter's `None` is modeled only as “no supplied time.”
No property is inferred about rustls handshake behavior from `None`.

## Reproduction and result

No dependency or model-checking tool was added. Run:

```sh
python3 -B -m unittest discover -s experiments/verification/tls-time-gate -p 'test_*.py' -v
python3 -B experiments/verification/tls-time-gate/model.py
```

The model command returns exit 0 only when the normal bounded exploration
passes and the intentionally faulty RTC-promotion variant is detected.
`test_model.py` also checks the exploration counts and the concrete mutant
counterexample. These are abstract-model results, not Rust unit tests,
source-code proof, refinement proof, executable-code proof, native TLS, or
human expert audit.
