# BaryonOS brand mark

Status: **design**, version 1, 13 November 2026. Branch
`claude/baryon-os-ui-ux-design-152adb`. Version 1 is the shared baseline used
unchanged by all three visual concepts so they can be compared fairly. It is a
starting point, not a frozen design: after the direction is selected, the mark
is refined (geometry, symmetry, optical balance, proportions, materials, boot
symbol, launcher icon, monochrome system icon, small sizes and animated
identity) together with the design system, before native UI integration. The
three coloured particles and the interconnected triangle stay. The mark is generated deterministically
by `ui/design/brand/generate_mark.py`; all SVG files in `ui/design/brand/` are
build outputs of that script. The native renderer does not yet draw the mark;
see [NATIVE-FEASIBILITY.md](NATIVE-FEASIBILITY.md).

The BaryonOS name, logos and marks are project marks. MPL-2.0 grants no rights
to them ([LICENSING.md](../../LICENSING.md), "Trademarks").

## 1. Source and what was preserved

The project owner supplied the original logo as a raster image in the design
session of 13 November 2026. That image is not in the repository.
`mark-original.svg` is a **vector reconstruction** traced from measured
coordinates of that image. It is kept as a selectable option and should be
replaced by the source artwork once that artwork is committed.

The refined mark keeps every structural element of the original:

- three circular nodes, red at the top, blue at the right and green at the
  lower left;
- an angular, bonded triangle in silver that connects the nodes;
- three tapered blades that sweep along the outside of the triangle;
- a dark ground and a slight anticlockwise lean.

The refinement changes proportion and finish only:

| Aspect | Original | Refined |
|---|---|---|
| Geometry | Hand-drawn, irregular | Equilateral node triangle, one circumradius `R` drives every size |
| Lean | About −7° | Exactly −7° (`ROTATION`) |
| Node separation | Bond touches nodes | Clearance ring of 0.07 `R` carves the bond and blades around every node |
| Blades | Uneven wedges, one corner open | Three identical clockwise wedges, wide at the root, tip at the next clearance ring |
| Silver | Flat `#D5D9DE` | Diagonal three-stop gradient `#F4F6F9` → `#D7DCE3` → `#A7B0BC` |
| Node colours | `#FC4539`, `#087EFC`, `#31BC5C` | `#FF4A3D`, `#2585FF`, `#33C46B` on dark; `#D9342A`, `#1061D6`, `#14894A` on light |

## 2. Construction

All values are fractions of the node circumradius `R`.

| Constant | Value | Meaning |
|---|---|---|
| `ROTATION` | −7.0° | Lean of the node triangle; red sits at −97° |
| `NODE` | 0.22 | Node radius |
| `CLEAR` | 0.07 | Clearance ring around each node |
| `BOND_OUTER` | 0.80 | Circumradius of the bond triangle |
| `BOND` | 0.15 | Bond thickness |
| `GAP` | 0.07 | Gap between bond and blade |
| `BLADE` | 0.22 | Blade width at its root |

The compact variant multiplies `NODE` by 1.35, `CLEAR` by 1.3 and `BOND` by 1.6,
and drops the blades. Optical centring uses the bounding box of the nodes, not
the triangle centroid.

## 3. Files and use

| File | Use | Minimum size |
|---|---|---|
| `mark-primary-dark.svg` | Primary mark on deep space | 40 px |
| `mark-primary.svg` | Primary mark, transparent ground, for dark surfaces | 40 px |
| `mark-light.svg` | Primary mark for light surfaces (ink bond, deeper nodes) | 40 px |
| `mark-mono-silver.svg`, `mark-mono-ink.svg` | One-colour reproduction, engraving, monochrome displays | 24 px |
| `mark-compact.svg`, `mark-compact-mono.svg` | Reduced sizes: dock, tab, status area, favicon | 14 px; use below 40 px |
| `app-icon.svg` | Launcher and installer icon on a deep-space tile | 32 px |
| `lockup-dark.svg`, `lockup-light.svg` | Mark with wordmark, horizontal | 24 px mark height |
| `splash.svg` | Static boot frame; final frame of the convergence animation | — |
| `mark-original.svg` | Reconstruction of the original logo, kept as an option | 40 px |

Clear space around the mark equals one node diameter on every side. The mark
is never rotated beyond its fixed lean, outlined, recoloured per node outside
the variants above, placed on a busy image without a backing tile, or used as
a button that grants a capability.

## 4. Wordmark

"Baryon" is set in Archivo Semibold (weight 600) and "OS" in Archivo Light
(weight 300), both at 125 % width. The generator traces the glyphs to outline
paths at build time, so the SVG files do not depend on an installed font.
Archivo is OFL-licensed; provenance and hashes are in
[ui/design/fonts/FONTS.md](../../ui/design/fonts/FONTS.md). The product name
stays "BaryonOS" ([NAMING-AND-IA.md](NAMING-AND-IA.md)).

## 5. How the mark informs the system

The interface takes a grammar from the mark rather than copies of it.

- **Three bonded parts.** Composition favours three-part groupings: a window is
  title, content and status; Orbit is apps, running work and system status; the
  permission question is who, what and where. Three-part rhythm is a default,
  not a rule that overrides content.
- **Geometry.** The 60° triangle and the −7° lean appear only in identity
  moments: boot, sign-in, the menu button, empty states and progress. Ordinary
  controls stay rectilinear, so the mark stays special.
- **Clearance rings.** A node never touches its bond. The same rule gives
  focus rings and status dots a visible gap from their surface.
- **Icon logic.** System icons are drawn on a 24-unit grid with round joins.
  Where an icon needs an accent it uses a single node-dot, never the full
  triangle. The full mark is reserved for BaryonOS itself.
- **Colour.** Red, green and blue are accents for highlights, focus, motion,
  trusted indicators and differentiation between apps or workspaces. They are
  never large fills, and they never carry meaning alone: every coloured state
  also has a label, shape or position.
- **Motion.** In the convergence animation the three nodes arrive from three
  orbits, the bond draws, and the blades sweep once (see the design system,
  motion tokens). Loading states reuse the three nodes as a small orbit. Under
  reduced motion the final frame appears with a short fade only.
- **Trust.** Trusted system surfaces (sign-in, lock, permission questions) carry
  the compact mark in a fixed position. This makes them recognisable. It does
  not prove them genuine: an app can imitate pixels, so secrets are only entered
  through the session-service route
  ([DESKTOP-SHELL.md](../implementation/DESKTOP-SHELL.md) §1).

## 6. Regeneration

```bash
python3 -B ui/design/brand/generate_mark.py
```

Requires Python 3 with Pillow and NumPy for the wordmark tracing. The output is
deterministic for a given font file; the Archivo hash is pinned in FONTS.md.
