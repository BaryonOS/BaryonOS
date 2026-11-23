// UX-E slice A: static hygiene and copy checks over the journey files (reads files, no DOM).
import { test } from 'node:test'
import assert from 'node:assert/strict'
import { readFileSync, readdirSync, existsSync } from 'node:fs'
import { createHash } from 'node:crypto'
import { T } from '../strings.js'
import { STATIONS } from '../journey/state.js'

const P = new URL('../', import.meta.url)
const rd = (rel) => readFileSync(new URL(rel, P), 'utf8')
const jsFiles = ['journey.js', ...readdirSync(new URL('journey/', P)).filter((f) => f.endsWith('.js')).map((f) => `journey/${f}`),
  ...(existsSync(new URL('apps/', P)) ? readdirSync(new URL('apps/', P)).filter((f) => f.endsWith('.js')).map((f) => `apps/${f}`) : [])]
const css = rd('journey.css')
const html = rd('index.html') + rd('sweep.html')

const RESERVED = 'ds-workspace ds-menu-bar ds-toolbar ds-tool-rail ds-canvas ds-inspector ds-panel ds-timeline ds-data-grid ds-status-bar ds-document-tabs ds-command-palette ds-track ds-ruler ds-playhead ds-cell ds-number-field ds-layer-row ds-keycap ds-expander ds-export-dialog ds-object-chip ds-place-badge ds-version-indicator ds-link-badge ds-workspace-switcher ds-keymap-editor ds-version-timeline ds-color-picker ds-scope ds-meter'.split(' ')
const COLOUR = /#[0-9a-f]{3,8}\b|\b(rgba?|hsla?|oklch|oklab|lab|lch|hwb)\(|(?<![\w-])(white|black|red|green|blue|gray|grey|yellow|orange|purple|pink|silver|navy)(?![\w-])/i

test('no colour literal in journey.css, index.html, sweep.html or journey scripts', () => {
  const strip = (s) => s.replace(/\/\*[\s\S]*?\*\//g, '').replace(/^\s*\/\/.*$/gm, '')
  for (const [name, src] of [['journey.css', css], ['html', html.replace(/<!--[\s\S]*?-->/g, '')], ...jsFiles.map((f) => [f, rd(f)])]) {
    const body = strip(src).replace(/data-[a-z-]+="[^"]*"/g, '').replace(/'[a-z-]+-(blue|green|red)'/g, '')
    const m = (name.endsWith('.js') ? body.match(/style[^;\n]*(#[0-9a-f]{3,8}\b|rgba?\()/i) : body.match(COLOUR))
    assert.equal(m, null, `${name}: ${m && m[0]}`)
  }
})

test('every var(--x) in journey.css is defined in tokens.css, components.css or journey.css', () => {
  const defs = new Set()
  for (const src of [rd('../system/tokens.css'), rd('../system/components.css'), css]) for (const m of src.matchAll(/(--[\w-]+)\s*:/g)) defs.add(m[1])
  const used = [...css.matchAll(/var\((--[\w-]+)/g)].map((m) => m[1])
  const missing = [...new Set(used)].filter((v) => !defs.has(v) && !v.startsWith('--jr-dyn-'))
  assert.deepEqual(missing, [])
})

test('no ds rule redefined globally, no reserved S7 names, no reserved attributes, no assertive live region', () => {
  const selectors = css.replace(/\/\*[\s\S]*?\*\//g, '').split('{').map((s) => s.split('}').pop().trim()).filter(Boolean)
  for (const sel of selectors) for (const part of sel.split(',')) {
    const p = part.trim()
    if (p.startsWith('@') || /^(from|to|\d+%)$/.test(p)) continue
    assert.ok(!/^\.ds-/.test(p), `global ds selector: ${p}`)
  }
  for (const r of RESERVED) assert.ok(!new RegExp(`\\.${r}\\b`).test(css), r)
  const all = css + html + jsFiles.filter((f) => f !== 'journey/sweep.js').map(rd).join('\n') // the sweep searches for assertive regions
  assert.ok(!/aria-live=["']?assertive|'aria-live':\s*'assertive'/.test(all))
  assert.ok(!/data-(density|surface|workspace|disclosure)\b/.test(all.replace(/\/\/.*$/gm, '')))
})

const ALLOWED = /^[\x20-\x7e\n -ÿ–—‘’“”•…€›↑↓]*$/
const strings = []
const walk = (v) => {
  if (typeof v === 'string') strings.push(v)
  else if (typeof v === 'function') strings.push(String(v(2, 'Tuinplan')))
  else if (v && typeof v === 'object') for (const x of Object.values(v)) walk(x)
}
walk(T.journey)
for (const s of STATIONS) for (const [fn] of s.feas) strings.push(fn) // visible in the Status table

test('journey copy stays inside the type-1.0 glyph set, no exclamation marks, no OK/Ja buttons', () => {
  for (const s of strings) {
    assert.match(s, ALLOWED, `glyph outside the atlas: ${s}`)
    assert.ok(!s.includes('!'), s)
    assert.ok(!/^(OK|Ja)$/.test(s), s)
  }
})

test('claim stems appear only in strings marked as simulated or planned', () => {
  const STEMS = /versleutel|synchronis|beveilig|afgedwongen|opgeslagen op thuis-nas|hersteld/i
  for (const s of strings) if (STEMS.test(s)) assert.match(s, /gesimuleerd|gepland|simulatie/i, s)
})

test('strings.js outside the journey block is byte-identical to ec459716', () => {
  const s = rd('strings.js')
  const i = s.indexOf('  // journey:start'); const j = s.indexOf('  // journey:end\n') + '  // journey:end\n'.length
  assert.ok(i > 0 && j > i)
  assert.equal(createHash('sha256').update(s.slice(0, i) + s.slice(j)).digest('hex'), '393d16fc719a6e4fde9ee187da4015f06a9bd849346cfe4d27796a6d8ccdbd82')
})

test('secret rule: session.js never reads, stores, logs or emits the secret inputs', () => {
  const src = rd('journey/session.js')
  const reads = [...src.matchAll(/\b(secret|code|newSecret|oobeSecret)\w*\.value\b(?!\s*=\s*'')/g)].map((m) => m[0])
  assert.deepEqual(reads, [])
  assert.ok(!/localStorage|sessionStorage|console\.|bus\.emit|location\.(hash|search)\s*=/.test(src))
  assert.ok(/type: 'password'/.test(src) && /autocomplete: 'off'/.test(src))
})

// repair 2: the defect classes the verifier found stay closed, and the sweep keeps the checks that find them everywhere
test('layers that paint nothing pass the pointer through; the session surface is never a squeezed scroll container', () => {
  assert.match(css, /\.jr-windows \{[^}]*pointer-events: none/)
  assert.match(css, /\.jr-windows > \* \{ pointer-events: auto; \}/)
  assert.doesNotMatch(css, /\.jr-kern \{[^}]*overflow: (hidden|auto|scroll)/)
  for (const sel of ['.jr-signin', '.jr-system', '.jr-starting']) assert.doesNotMatch(css, new RegExp(`\\${sel} \\{[^}]*(align-content|place-items|place-content): center`), sel)
})

test('sweep fails on pointer-blocked controls, vertical clipping, stringified null and unsettled states', () => {
  const sw = rd('journey/sweep.js')
  for (const kind of ["'pointer-blocked'", "'clip-y'", "'null-text'", 'state did not settle', 'error body not shown']) assert.ok(sw.includes(kind), kind)
  assert.doesNotMatch(sw, /\.jr-kern/, 'no .jr-kern exemption')
})

test('the Sem and Andere gebruiker notes do not claim that every input works', () => {
  const J = T.journey.r2
  for (const s of [J.onlyNoor, J.otherLater, J.semRefused('Sem')]) assert.doesNotMatch(s, /werkt elke invoer|niet aangemeld/)
  assert.notEqual(J.unsavedHeading, J.dirtyCloseBody)
})
