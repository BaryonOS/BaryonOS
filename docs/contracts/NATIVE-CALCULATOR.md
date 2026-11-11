# Native calculator: component contract

NATIVE-CALCULATOR-1, base `5e126b74bba2452bfc68b42caa400c61d471378c`.
This component delivers a real no_std CPL3 app, a general decimal arithmetic core,
typed INP1/SUR1 clients and its own native controls in the existing
Materiaal design. It is part of FR-A08 from
[FULL-TEST-RELEASE](../FULL-TEST-RELEASE.md); component tests and an ELF build
do not sign off full native app/release acceptance.

## Arithmetic contract, fixed in advance

There was no existing arithmetic engine on the checked apps branch/ecfeab4.
The new bounded parser is ordinary Rust arithmetic logic, not cryptography,
a float/C/host backend or a table of answers. The same library is used by
`calculator-app` and the separate host reference driver.

- At most 256 ASCII characters. Spaces/tabs are allowed between tokens. At most 32
  combined levels of parentheses and unary `+`/`-`; binary operator loops
  do not grow the stack per character.
- `+`, `-`, `*`, `/`, parentheses and unary signs. Multiplication/division take precedence over
  addition/subtraction; equal precedence is left-associative.
- Comma and point are both decimal separators, not thousands separators. No
  exponent, percent or scientific-precision claim.
- Every literal and every intermediate result must lie between
  −9.999.999.999,999999999 and +9.999.999.999,999999999. An out-of-range literal or
  intermediate result is rejected, even if a later operation could
  compensate. There is no silent saturation, wrapping or use of Infinity.
- The fixed scale is 10^9. Addition/subtraction are exact on this grid.
  Literal digits after the ninth decimal and products/quotients are rounded
  to the nearest grid point; exactly half goes to the **even**
  last digit, for both signs. Actually discarded non-zero value sets
  the rounding indicator; earlier rounding stays visible throughout the calculation.
- Division by zero gives a concrete Dutch error. Zero after underflow has
  no minus sign. The display uses a comma and omits superfluous zeros.

`|raw| ≤ 10^19−1` guarantees `|a×b| < 10^38 < i128::MAX`; for division the
multiplied numerator is smaller than 10^28. All operators additionally use
checked operations and check their result. Positive/negative maxima,
both half-even directions, roundup overflow and small valid results after
large raw products are actually tested. This is a bounded
calculator, not a universal exact-rational or arbitrary-precision engine.

## Input, errors and widgets

The expression is stored locally with caret and selection; it is fully
preserved on syntax, range, length and division errors. The previous valid
result is also preserved, with the visible label `Vorig` ("Previous"). That result is not
presented as the answer to erroneous or modified input.

Keyboard: digits, normal US physical operator combinations, parentheses, point/
comma, numpad with NumLock, Enter or `=` evaluate, Backspace/Delete,
Left/Right/Home/End, Shift selection, Ctrl+A, Tab/ShiftTab and Enter/Space on a
focused button. Escape cancels the current input and keeps the earlier
result; `C` and the C button reset input/result. Enter/Tab/Escape/C repeat
is not treated as a series of new deliberate actions. Releases do not calculate.

The same commands are used by the 5×4 buttons. `Wis` ("Clear") deletes the
selection or the previous character, C resets, `=` calculates and the other buttons insert
the character actually shown. A button only acts after a real left-mouse down
and up on the same button. Outside assignedRect, lost focus, input cancel or
overflow invalidates the capture; no clamping to an incidental button.
Click/drag in the expression places/selects real input. Long input scrolls
horizontally with the caret. The calculation itself is a bounded synchronous parser;
there is no pretended background task with a fictitious cancel status.

Layout/hit test/tile render share the same geometry. 100/200% from 320×240
logical, within the existing maximum 1920×1080 physical profile. Compact
windows use the same font family and tokens with suitable controls;
no control disappears to simplify acceptance. The single existing
`ui/native/src/design.rs`/`ui/prototype/tokens.css` remains the token source.
The existing approved Noto Sans Mono bitmap 0.3.2 is used directly with
the same feature profile. Licences/notices remain under
`ui/native/licenses`; provenance and SHA are in
[NATIVE-MATERIAAL](../implementation/NATIVE-MATERIAAL.md).

The app paints only bounded 32×8-pixel tiles in its own RAM. Typography,
radii, colours, hover, selection and focus follow Materiaal. Bitmap font coverage
is the existing Basic Latin/Latin1/Specials coverage, no general shaping/
screen-reader claim. There are no product animations, lorem ipsum or fake buttons.
Host examples from the preview driver are explicitly RAM render trials;
real QMP/native visual and control verification remains mandatory.

## Services, grants and lifetime

The calculator needs no volume, document, Key, entropy, network,
raw display, raw input, launch or file grants. The only six handles:
registry-SEND, control-RECV, surface-SEND/reply-RECV, INP1-SEND/reply-RECV.
Number/PID/epoch in the startup grants no authority; the real grant and
service ingress determine the rights. Handle aliases and known old document
metadata are rejected. The root must actually install exclusively these six
endpoint leaves and use its own reply endpoints without a RECV alias.

| Startup word | Meaning |
|---|---|
|0|boot mode; the actual SDK/kernel ABI is queried separately via `native::check_abi()` |
|88/96|Init registry-SEND / own control-RECV |
|176/184|own PID / Init PID |
|544/552/560/568|SUR1-SEND / own reply-RECV / compositor PID / epoch |
|576/584/592/600/608|INP1-SEND / own reply-RECV / policy PID / epoch / client ID |
|616|scale factor 1 or 2 |

Known document fields 48/56/64/72/80/168 are 0. The constructor reads real,
authenticated SUR1-INFO geometry; startup provides no framebuffer address or
pre-filled pixels. INP1 PEEK/ACK uses the existing exclusive typed
Session. One step processes at most 4 offers and 8 SUR1 tile transactions. A half
or stale frame does not make `ready` true. ACK supersession cancels captures;
transport loss makes input/surface failure visible and gives no new
fictitious ACK or re-grant.

Native entry uses CLD, checks the existing SysV RSP%16=8 and jumps
to Rust. No heap allocator, no stack enlargement: the existing 64KiB and the
ordinary W^X loader. Readiness is signalled with SYSTEM tag `0x53595354454d0001`, op 1;
start is authenticated Init op 2. Stop op 5 closes both typed owners,
only then sends DONE 3 and closes registry/control. Even on a close error,
all involved owners are attempted. STOP writes or saves no files.
STOP before GO also closes the four not-yet-handed-over service leaves before
DONE/registry/control cleanup follows; no frame is then declared ready.
The usermodel counter does not wrap; exhaustion becomes a terminal error.

## Diagnostics without pre-filled success

All words below come from actual model/client behaviour, are
private userdata and not an app→kernel privilege. Ready is cleared first on
publication and set last. Root must snapshot a complete current generation
before any process cleanup and keep the real PID on slot reuse.

| Word | Actual fact |
|---|---|
|40/240|clean STOP done 1 / any fail step |
|400|entire current frame ready plus real INP1 response; no arithmetic-answer claim |
|408/416|complete frames / successfully committed tiles |
|424/432/440|validated key/mouse offers / real ACKs |
|448/456|service errors / presented generation |
|464/472|successful calculations / concrete arithmetic errors |
|480/488/496|expression length / current result / rounding flag |
|504/512|caret / local visual focus state; no broker FocusACK |
|520/528/536|raw i128 result low64/high64 two's complement / result present |
|632|scale factor |

The result is `raw/10^9`; there is no static expected answer value in the
native entry. Personal calculations are not sent to OS telemetry or a
host service. Native test observers may only record the known test data
for their bounded trials; private userdata is not permission
for general logging.

## Reproducible component check

```sh
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-calculator-host-target python3 -B tools/dev.py cargo +1.90.0 test --locked --offline --manifest-path sdk/examples/calculator/Cargo.toml
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-calculator-host-target python3 -B tools/dev.py cargo +1.90.0 clippy --locked --offline --manifest-path sdk/examples/calculator/Cargo.toml --all-targets --features host-oracle -- -D warnings
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-calculator-host-target python3 -B tools/dev.py cargo +1.90.0 build --locked --offline --manifest-path sdk/examples/calculator/Cargo.toml --features host-oracle --bin calculator-oracle
python3 -B sdk/examples/calculator/tests/decimal_oracle.py --binary /tmp/nieuw-os-calculator-host-target/debug/calculator-oracle
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-calculator-none-target python3 -B tools/dev.py python3 -B sdk/examples/calculator/build.py
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-calculator-none-target python3 -B tools/dev.py python3 -B sdk/examples/calculator/build.py clippy
```

The Python Decimal reference uses 100-digit host precision and half-even per
literal/operator, plus the fixed grid/range boundary. It reads the real
Rust host driver results for deterministic varied expressions; it is
never a native backend. The host chain uses the real capability engine,
INP1 service, SUR1Receiver and typed clients. Only the recording RAM display
counterpart is explicitly a host model. Lost/negative replies, outside click,
focus generation, cleanup and bootstrap tampering are separate negative trials.

The own lock contains only existing local libraries and the already approved
Noto pin, without a new runtime library/download/build script. Root/SDK locks,
shared UI, kernel and build scripts are not changed here. The builder
delivers `NOS_CALCULATOR_ELF` and inspection/SHA, with `native_runtime_performed=false`.

Root then integrates the real kernel grants, launcher/Init, multiple
programs, compositor/focus policy and native QMP scenarios. Still required are
actually typing digits/parentheses/decimals, correction, overflow/÷0/recovery,
keyboard/mouse, 100/200%, focus switch and clean stop on the integrated
image with exact source/ELF/image hashes. Shared clipboard and the other full
release gates remain their own mandatory open acceptance criteria. A
host check, preview or successful native target build does not replace that execution.
