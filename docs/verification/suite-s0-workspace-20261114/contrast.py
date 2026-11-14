"""S0 colour evidence (repaired): WCAG 2.2 contrast with state fills for WORKSPACE-REQUIREMENTS.md.

Run:  python3 -I contrast.py > contrast.txt
Exit 0 means: the script ran, every self-check passed and the S0 verifier's nine failing
figures reproduce (section 8). It does NOT mean every pair passes; the BELOW counts say that.

Contrast: WCAG 2.2 relative luminance (sRGB, 0.04045 threshold), (L1 + 0.05) / (L2 + 0.05),
floored to two decimals so a reported pass is never a rounding artefact.
Chroma: CIELAB C*ab, D65 white. Overlays are composited to 8-bit sRGB with ties rounded to
even (Python round); section 8b repeats them with ties rounded up and asserts no verdict changes.

Inputs are hard-coded from Orbitaal v1, ui/design/concepts/orbitaal/index.html line 40 (.dk),
line 49 (.lt) and line 320 (.hc, used as "dk hc", so state fills inherit from .dk).
All neutral values are PROVISIONAL: derived from v1 while the v2 board is unaccepted.
Final values are computed in the ds-1.0 tokens stage with the same acceptance check.
"""
import math
import sys
from fractions import Fraction


def hex_rgb(h):
    h = h.lstrip("#")
    return tuple(int(h[i:i + 2], 16) for i in (0, 2, 4))


def lin(c8):
    c = c8 / 255
    return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4


def lum(h):
    r, g, b = (lin(c) for c in hex_rgb(h))
    return 0.2126 * r + 0.7152 * g + 0.0722 * b


def raw(a, b):
    la, lb = sorted((lum(a), lum(b)), reverse=True)
    return (la + 0.05) / (lb + 0.05)


def ratio(a, b):
    return math.floor(raw(a, b) * 100) / 100


def over(fg_hex, alpha, bg_hex):
    """Composite an rgba overlay on an opaque background, 8-bit result."""
    f, b = hex_rgb(fg_hex), hex_rgb(bg_hex)
    return "#" + "".join(f"{round(alpha * x + (1 - alpha) * y):02X}" for x, y in zip(f, b))


def over_up(fg_hex, alpha, bg_hex):
    """over() with ties rounded up, in exact arithmetic (rounding sensitivity, section 8b)."""
    f, b = hex_rgb(fg_hex), hex_rgb(bg_hex)
    a = Fraction(str(alpha))
    return "#" + "".join(f"{math.floor(a * x + (1 - a) * y + Fraction(1, 2)):02X}" for x, y in zip(f, b))


def lab(h):
    r, g, b = (lin(c) for c in hex_rgb(h))
    x = 0.4124 * r + 0.3576 * g + 0.1805 * b
    y = 0.2126 * r + 0.7152 * g + 0.0722 * b
    z = 0.0193 * r + 0.1192 * g + 0.9505 * b

    def f(t):
        return t ** (1 / 3) if t > (6 / 29) ** 3 else t / (3 * (6 / 29) ** 2) + 4 / 29

    fx, fy, fz = (f(v / w) for v, w in zip((x, y, z), (0.95047, 1.0, 1.08883)))
    L, a, bb = 116 * fy - 16, 500 * (fx - fy), 200 * (fy - fz)
    return L, a, bb, math.hypot(a, bb)


def twin(h):
    """Achromatic grey with the same relative luminance (nearest 8-bit step)."""
    t = lum(h)
    g = min(range(256), key=lambda v: abs(lum(f"#{v:02X}{v:02X}{v:02X}") - t))
    return f"#{g:02X}{g:02X}{g:02X}"


# Self-checks against known values (WCAG extremes; CONCEPTS.md records 1.07:1 for v1 s1/s2).
assert ratio("#000000", "#FFFFFF") == 21.0
assert ratio("#0B0E1B", "#12162A") == 1.07
assert abs(lab("#777777")[0] - 50.0) < 0.5 and lab("#777777")[3] < 0.01
assert over("#FFFFFF", 0.5, "#000000") == "#808080"

# ---- Orbitaal v1 (index.html lines 40, 49, 320). State fills are (base colour, alpha). ----
V1 = {
    "dark": dict(s0="#070912", s1="#0D1020", s2="#12162A", s3="#1A1F38", sin="#080A16",
                 ln="#232A47", ln2="#5D6890", tx1="#EEF1F8", tx2="#A9B2CB", tx3="#8C95B2",
                 red="#FF5C50", blue="#5AA2FF", green="#42D27F", silver="#C9D3E6", focus="#A8CAFF",
                 selbg=("#5AA2FF", .15), hov=("#AABCFF", .07), hov2=("#AABCFF", .12),
                 rim_a=("#E2E9FF", .46), rim_b=("#96A5DC", .10), rim_c=("#96A5DC", .20),
                 sec_bg=("#AABCFF", .08), pill_bg=("#AABCFF", .06), glowc=("#8CB4FF", .38),
                 pri_bg=("#F4F6FA", "#D9DFEA"), pri_bg_h=("#FFFFFF", "#E8ECF4"), pri_tx="#0A0D1A"),
    "light": dict(s0="#E4E9F2", s1="#EDF0F7", s2="#FBFCFE", s3="#E3E8F1", sin="#FFFFFF",
                  ln="#D8DEE9", ln2="#78819A", tx1="#111726", tx2="#424B62", tx3="#58627D",
                  red="#CC3328", blue="#1061D6", green="#127A43", silver="#5B667F", focus="#1061D6",
                  selbg=("#1061D6", .10), hov=("#182C60", .05), hov2=("#182C60", .09),
                  rim_a=("#FFFFFF", 1.0), rim_b=("#465A8C", .16), rim_c=("#465A8C", .30),
                  sec_bg=("#FFFFFF", 1.0), pill_bg=("#FFFFFF", 1.0), glowc=("#1061D6", .22),
                  pri_bg=("#232B42", "#121828"), pri_bg_h=("#2E3853", "#1A2134"), pri_tx="#FFFFFF"),
}
# Window shadows --wsh (two layers) and --wsh-off: colour and alphas, index.html line 44 (.dk) and line 53 (.lt).
WSH = {"dark": ("#000000", (.58, .42, .45)), "light": ("#24366C", (.22, .14, .16))}
HC = dict(base="#000000", line="#FFFFFF", tx1="#FFFFFF", tx2="#FFFFFF", tx3="#F2F2F2",
          red="#FF8A80", blue="#8EC5FF", green="#7CF0A8", focus="#FFE34D")
BASES = ("s1", "s2", "s3", "sin")
BASE_NAME = {"s0": "base(s0)", "s1": "panel(s1)", "s2": "window(s2)", "s3": "raised(s3)", "sin": "inset(sin)"}
COLOURS = ("s0", "s1", "s2", "s3", "sin", "ln", "ln2", "tx1", "tx2", "tx3")

# ---- Provisional neutral twins (D3 rule): same luminance, C*ab = 0. ----
PURE = {th: {k: twin(V1[th][k]) for k in COLOURS} for th in V1}
NEU = {th: dict(PURE[th]) for th in PURE}
NEU["dark"]["ln2"] = "#747474"  # draft deviation (pure twin #696969 is 2.93 on raised); kept provisional, failures reported
for th in NEU:
    NEU[th]["focus"] = NEU[th]["tx1"]  # focus.ring takes text.primary
SURROUND = {"dark": NEU["dark"]["s0"], "light": "#777777", "hc": "#000000"}

# Neutral state fills. "sel" = D3 rule (v1 selbg twinned at the v1 alpha), used by the document.
# "sel.v2" = achromatic overlay at the unaccepted v2 board alpha (white 8.5 % / black 6.5 %),
# kept only because the draft and the verifier used it.
NSTATE = {th: {"hov": (twin(V1[th]["hov"][0]), V1[th]["hov"][1]),
               "hov2": (twin(V1[th]["hov2"][0]), V1[th]["hov2"][1]),
               "sel": (twin(V1[th]["selbg"][0]), V1[th]["selbg"][1])} for th in V1}
NSTATE["dark"]["sel.v2"] = ("#FFFFFF", .085)
NSTATE["light"]["sel.v2"] = ("#000000", .065)

FG = (("text.primary", "tx1", 4.5), ("text.secondary", "tx2", 4.5), ("text.tertiary", "tx3", 4.5),
      ("line.strong", "ln2", 3.0), ("focus.ring", "focus", 3.0),
      ("place.device", "green", 3.0), ("place.nas", "blue", 3.0), ("place.internet", "red", 3.0))

below = {"neutral": [], "neutral.v2sel": [], "v1": [], "hc": []}
seen = {}


def row(section, label, theme, fgname, fg, base, state, bg, req):
    r = ratio(fg, bg)
    ok = r >= req
    seen[(theme, fgname, base, state, section)] = (fg, bg, r)
    if not ok:
        below[section].append((theme, fgname, fg, base, state, bg, r, req))
    print(f"{label:7} {theme:5} {fgname:15} {fg} on {BASE_NAME.get(base, base):10} {state:7} {bg}: "
          f"{r:5.2f} [{req}] {'ok' if ok else 'BELOW'}")
    return r


print("== 1. CIELAB C*ab of v1 chrome and its neutral twins ==")
for th in V1:
    for k in COLOURS:
        note = f" -> provisional {NEU[th][k]} (lifted; pure twin {ratio(PURE[th][k], PURE[th]['s3']):.2f} on raised)" \
            if NEU[th][k] != PURE[th][k] else ""
        print(f"{th:5} {k:4} v1 {V1[th][k]} C*={lab(V1[th][k])[3]:4.1f} -> twin {PURE[th][k]} "
              f"C*={lab(PURE[th][k])[3]:4.2f}{note}")
print("surround", SURROUND, {k: round(lab(v)[0], 1) for k, v in SURROUND.items()})

print("\n== 2. Twin bound: |v1 ratio - twin ratio| over every fg x bg pair (unfloored) ==")
worst = (0, None)
for th in V1:
    pure = PURE[th]  # unlifted twins, so the bound is about twinning only
    for fk in ("tx1", "tx2", "tx3", "ln2", "ln"):
        for bk in ("s0",) + BASES:
            d = abs(raw(V1[th][fk], V1[th][bk]) - raw(pure[fk], pure[bk]))
            if d > worst[0]:
                worst = (d, (th, fk, bk, raw(V1[th][fk], V1[th][bk]), raw(pure[fk], pure[bk])))
print(f"max difference {worst[0]:.4f} at {worst[1][0]} {worst[1][1]}/{worst[1][2]}: "
      f"v1 {worst[1][3]:.3f}, twin {worst[1][4]:.3f}")
BOUND = worst[0]
assert 0.05 < BOUND < 0.1, BOUND  # the draft's "within 0.05" is false; the bound stays below 0.1

print("\n== 3. PROVISIONAL neutral twins: every fg on every allowed base, rest + state fills ==")
print("states: rest; hov / hov2 = v1 hover fills twinned at v1 alpha; sel = v1 selbg twinned at v1 alpha (D3 rule)")
for th in ("dark", "light"):
    n = NEU[th]
    print(f"-- {th}: hov {NSTATE[th]['hov']}, hov2 {NSTATE[th]['hov2']}, sel {NSTATE[th]['sel']}")
    for fgname, key, req in FG:
        fg = n[key] if key in n else V1[th][key]  # place hues are unchanged by neutral mode
        for b in BASES:
            row("neutral", "neutral", th, fgname, fg, b, "rest", n[b], req)
            for st in ("hov", "hov2", "sel"):
                c, a = NSTATE[th][st]
                row("neutral", "neutral", th, fgname, fg, b, st, over(c, a, n[b]), req)

print("\n== 3b. Same, with the unaccepted v2 selection alpha (draft and verifier figures) ==")
for th in ("dark", "light"):
    n = NEU[th]
    c, a = NSTATE[th]["sel.v2"]
    for fgname, key, req in FG[:5]:
        for b in BASES:
            row("neutral.v2sel", "v2sel", th, fgname, n[key], b, "sel.v2", over(c, a, n[b]), req)

print("\n== 4. Inherited: Orbitaal v1 as drawn (chromatic), same matrix ==")
for th in ("dark", "light"):
    t = V1[th]
    for fgname, key, req in FG:
        for b in BASES:
            row("v1", "v1", th, fgname, t[key], b, "rest", t[b], req)
            for st, fill in (("hov", t["hov"]), ("hov2", t["hov2"]), ("sel", t["selbg"])):
                row("v1", "v1", th, fgname, t[key], b, st, over(fill[0], fill[1], t[b]), req)

print("\n== 5. High contrast (unchanged by neutral mode; state fills inherit from .dk) ==")
for fgname, key, req in (("text.primary", "tx1", 4.5), ("text.tertiary", "tx3", 4.5), ("line.strong", "line", 3.0),
                         ("focus.ring", "focus", 3.0), ("place.device", "green", 3.0),
                         ("place.nas", "blue", 3.0), ("place.internet", "red", 3.0)):
    row("hc", "hc", "hc", fgname, HC[key], "all", "rest", HC["base"], req)
    for st in ("hov", "hov2", "selbg"):
        c, a = V1["dark"][st]
        row("hc", "hc", "hc", fgname, HC[key], "all", st, over(c, a, HC["base"]), req)

print("\n== 6. Surround: only the focus ring, plus the high-contrast canvas edge line ==")
for th, f_ in (("dark", NEU["dark"]["tx1"]), ("light", NEU["light"]["tx1"]), ("hc", HC["focus"])):
    print(f"{th:5} focus.ring {f_} on surround {SURROUND[th]}: {ratio(f_, SURROUND[th]):5.2f} [3.0]")
print(f"hc    canvas edge line #FFFFFF on surround #000000: {ratio('#FFFFFF', '#000000'):5.2f} [3.0]")
assert ratio(NEU["light"]["tx1"], SURROUND["light"]) >= 3.0 and ratio(HC["focus"], "#000000") >= 3.0

print("\n== 6b. status.error as TEXT (TYPOGRAPHY 8.3 'status.error text'), 4.5 required ==")
print("v1 shares the place.internet red; D1 resolves status.error to text.primary instead")
below["status-red-as-text"] = []
for th in ("dark", "light"):
    n = NEU[th]
    for b in BASES:
        for st in ("rest", "hov", "hov2", "sel"):
            bg = n[b] if st == "rest" else over(*NSTATE[th][st], n[b])
            row("status-red-as-text", "status", th, "red-as-text", V1[th]["red"], b, st, bg, 4.5)
    print(f"{th:5} status.error = text.primary {n['tx1']}: min "
          f"{min(ratio(n['tx1'], over(*NSTATE[th][s], n[b])) for s in ('hov', 'hov2', 'sel') for b in BASES):.2f} [4.5]")

print("\n== 7. Other v1 values inside the app window: twin at the same alpha, or removed ==")
for th in ("dark", "light"):
    t = V1[th]
    for k in ("selbg", "hov", "hov2", "rim_a", "rim_b", "rim_c", "sec_bg", "pill_bg"):
        c, a = t[k]
        print(f"{th:5} {k:7} {c} @ {a * 100:g} % -> twin {twin(c)} @ {a * 100:g} %")
    print(f"{th:5} glowc   {t['glowc'][0]} @ {t['glowc'][1] * 100:g} % -> removed")
    print(f"{th:5} silver  {t['silver']} -> twin {twin(t['silver'])}")
    tx = twin(t["pri_tx"])
    for k in ("pri_bg", "pri_bg_h"):
        stops = [twin(s) for s in t[k]]
        rs = [ratio(tx, s) for s in stops]
        print(f"{th:5} {k:8} {t[k]} -> stops {stops}; pri-tx twin {tx}: {rs} [4.5] {'ok' if min(rs) >= 4.5 else 'BELOW'}")
        assert min(rs) >= 4.5
    c, alphas = WSH[th]
    print(f"{th:5} wsh/wsh-off {c} @ {' / '.join(f'{a * 100:g} %' for a in alphas)} -> twin {twin(c)} "
          f"at the same alphas (C*ab {lab(c)[3]:.1f} -> {lab(twin(c))[3]:.2f})")

print("\n== 7b. Button fills at rest: --sec-bg / --pill-bg twins (not yet bases of the acceptance check, D3) ==")
print("hover on a secondary button replaces the fill with --hov2 (index.html line 142), covered in section 3")
btn_below = []
for th in ("dark", "light"):
    n = NEU[th]
    for k in ("sec_bg", "pill_bg"):
        c, a = V1[th][k]
        for b in BASES:
            bg = over(twin(c), a, n[b])
            rs = [(nm, ratio(n[key] if key in n else V1[th][key], bg), req) for nm, key, req in FG]
            btn_below += [(th, k, b, bg, nm, r) for nm, r, req in rs if r < req]
            print(f"{th:5} {k:7} over {BASE_NAME[b]:10} {bg}: "
                  + ", ".join(f"{nm} {r:.2f}{'' if r >= req else ' BELOW'}" for nm, r, req in rs))
print(f"button-fill BELOW (reported here, outside the 12 of section 10): {btn_below}")
assert btn_below == [("dark", "sec_bg", "s3", "#2E2E2E", "line.strong", 2.90)]

print("\n== 8. Reproduction of the S0 verifier's figures (asserted) ==")
D, L = NEU["dark"], NEU["light"]
checks = [
    ("v1 dark ln2 #5D6890 on v1 raised #1A1F38", ratio("#5D6890", "#1A1F38"), 2.96),
    ("dark #747474 on sel row over raised #343434", ratio(D["ln2"], over("#FFFFFF", .085, D["s3"])), 2.66),
    ("light #818181 on sel row over panel #E0E0E0", ratio(L["ln2"], over("#000000", .065, L["s1"])), 2.95),
    ("light #818181 on sel row over raised #D9D9D9", ratio(L["ln2"], over("#000000", .065, L["s3"])), 2.75),
    ("dark #969696 on sel row over raised #343434", ratio(D["tx3"], over("#FFFFFF", .085, D["s3"])), 4.20),
    ("light #626262 on sel row over raised #D9D9D9", ratio(L["tx3"], over("#000000", .065, L["s3"])), 4.32),
    ("dark #747474 on sel twin #A0A0A0@15% over window #2C2C2C", ratio(D["ln2"], over("#A0A0A0", .15, D["s2"])), 2.98),
    ("light #818181 on hover twin #2F2F2F@9% over raised #D7D7D7", ratio(L["ln2"], over("#2F2F2F", .09, L["s3"])), 2.70),
    ("light #626262 on hover twin #2F2F2F@9% over raised #D7D7D7", ratio(L["tx3"], over("#2F2F2F", .09, L["s3"])), 4.23),
]
assert over("#FFFFFF", .085, D["s3"]) == "#343434" and over("#000000", .065, L["s1"]) == "#E0E0E0"
assert over("#000000", .065, L["s3"]) == "#D9D9D9" and over("#A0A0A0", .15, D["s2"]) == "#2C2C2C"
assert twin(V1["dark"]["selbg"][0]) == "#A0A0A0" and twin(V1["light"]["hov2"][0]) == "#2F2F2F"
assert over("#2F2F2F", .09, L["s3"]) == "#D7D7D7" and L["ln2"] == "#818181" and D["tx3"] == "#969696"
for label, got, want in checks:
    print(f"{label}: {got:.2f} (verifier {want:.2f}) {'reproduced' if got == want else 'MISMATCH'}")
    assert got == want, label

print("\n== 8b. Rounding sensitivity: every composited fill above, with ties rounded up instead of to even ==")
fills = [("neutral", th, NEU[th][b], c, a) for th in NEU for c, a in NSTATE[th].values() for b in BASES]
fills += [("neutral", th, NEU[th][b], twin(V1[th][k][0]), V1[th][k][1])
          for th in V1 for k in ("sec_bg", "pill_bg") for b in BASES]
fills += [("v1", th, V1[th][b], *V1[th][k]) for th in V1 for k in ("hov", "hov2", "selbg") for b in BASES]
fills += [("hc", "hc", HC["base"], *V1["dark"][k]) for k in ("hov", "hov2", "selbg")]
moved = [f_ for f_ in fills if over(f_[3], f_[4], f_[2]) != over_up(f_[3], f_[4], f_[2])]
assert all(sec == "v1" for sec, *_ in moved)  # neutral, button-fill and hc composites are rounding-independent
for sec, th, base, c, a in moved:
    even, up = over(c, a, base), over_up(c, a, base)
    print(f"{sec} {th} fill {c} @ {a * 100:g} % over {base}: {even} -> {up}")
    for fgname, key, req in FG:
        re_, ru = ratio(V1[th][key], even), ratio(V1[th][key], up)
        print(f"   {fgname:15} {V1[th][key]} {re_:5.2f} -> {ru:5.2f} [{req}] {'ok' if ru >= req else 'BELOW'}")
        assert (re_ >= req) == (ru >= req), "a verdict depends on the rounding mode"
print(f"{len(moved)} of {len(fills)} composited fills move by one 8-bit step; no verdict changes")

print("\n== 9. Minima per theme and base at rest (PROVISIONAL token table) ==")
for th in ("dark", "light"):
    for fgname, key, req in FG[:5]:
        vals = {b: seen[(th, fgname, b, "rest", "neutral")][2] for b in BASES}
        m = min(vals.values())
        print(f"{th:5} {fgname:15} " + " ".join(f"{BASE_NAME[b]} {v:5.2f}" for b, v in vals.items()) + f"  min {m:5.2f}")

print("\n== 10. BELOW rows (failures; none is presented as passing) ==")
for sec, rows_ in below.items():
    print(f"-- {sec}: {len(rows_)} BELOW")
    for th, fgname, fg, base, st, bg, r, req in rows_:
        print(f"   {th:5} {fgname:15} {fg} on {BASE_NAME.get(base, base):10} {st:7} {bg}: {r:5.2f} < {req}")
print(f"\nSUMMARY twin-bound {BOUND:.3f}; BELOW neutral {len(below['neutral'])}, neutral.v2sel "
      f"{len(below['neutral.v2sel'])}, v1 inherited {len(below['v1'])}, hc {len(below['hc'])}, "
      f"red status as text {len(below['status-red-as-text'])}. "
      "Acceptance check (every pair >= threshold on every allowed base with state fills): "
      f"{'PASS' if not below['neutral'] else 'FAIL'} for the provisional v1-derived twins.")
sys.exit(0)
