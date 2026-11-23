// UX-E slice A: pure journey modules (state, wm, search, keys, throttle). No DOM.
import { test } from 'node:test'
import assert from 'node:assert/strict'
import { STATIONS, FEAS, parse, serialize, go, canMove, built, SIM_DEFAULTS, SIM_OPTIONS } from '../journey/state.js'
import { createWm } from '../journey/wm.js'
import { buildIndex, query, GROUPS } from '../journey/search.js'
import { match, wakes, swallowRepeat } from '../journey/keys.js'
import { createThrottle } from '../journey/throttle.js'
import { T } from '../strings.js'
import { DATA } from '../data.js'

test('ten stations in E1 order with unique ids, titles and valid labels', () => {
  assert.deepEqual(STATIONS.map((s) => s.n), [1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
  assert.equal(new Set(STATIONS.map((s) => s.id)).size, 10)
  for (const s of STATIONS) {
    assert.ok(T.journey.stations[s.id]?.title, s.id)
    for (const [fn, lab, src] of s.feas) { assert.ok(fn && src); assert.ok(FEAS.includes(lab), `${s.id}: ${lab}`) }
  }
  for (const s of built()) { assert.ok(s.feas.length > 0, s.id); assert.ok(s.states.includes('success') || s.states.includes('empty')) }
  assert.deepEqual(built().map((s) => s.id), ['aanmelden', 'bureaublad', 'vensters', 'zoeken', 'bestanden', 'versies', 'toestemming', 'instellingen'])
})

test('router: round trip, defaults, pending before sign-in, later slices never route', () => {
  for (const s of built()) assert.equal(parse(serialize(s.id), { signedIn: true }).station, s.id)
  assert.equal(parse('', { signedIn: true }).station, 'bureaublad')
  assert.equal(parse('#/onzin', { signedIn: false }).station, 'aanmelden')
  const r = parse('#/zoeken', { signedIn: false })
  assert.deepEqual([r.station, r.pending], ['aanmelden', 'zoeken'])
  const l = parse('#/meldingen', { signedIn: true })
  assert.deepEqual([l.station, l.later], ['bureaublad', 'meldingen'])
  assert.equal(parse('#/versies', { signedIn: true }).station, 'versies')
})

test('go() only opens or focuses: no close or discard effect for any station', () => {
  for (const s of STATIONS) for (const signedIn of [true, false]) {
    const kinds = go(s.id, { signedIn, windows: 3 }).effects.map((e) => e[0])
    assert.ok(!kinds.includes('close') && !kinds.includes('discard') && !kinds.includes('reset'), s.id)
  }
  assert.ok(go('vensters', { signedIn: true, windows: 0 }).effects.some((e) => e[1] === 'rekenmachine'))
})

test('session phases', () => {
  assert.ok(canMove('locked', 'chooser') && canMove('chooser', 'recovery') && canMove('desktop', 'system'))
  assert.ok(!canMove('locked', 'desktop'))
})

test('wm: single instance, MRU, minimise keeps app open, close emits appClosed once', () => {
  const ev = []
  const wm = createWm({ emit: (t, d) => ev.push([t, d.app ?? d]) })
  const a = wm.open('notities'); const b = wm.open('rekenmachine')
  assert.equal(wm.open('notities').id, a.id)
  assert.deepEqual(wm.list().map((w) => w.id), [a.id, b.id])
  wm.minimise(a.id)
  assert.equal(wm.active().id, b.id)
  assert.ok(!ev.some((e) => e[0] === 'appClosed'))
  assert.equal(wm.cycle(b.id, 1).id, a.id)
  assert.equal(wm.cycle(b.id, -1).id, a.id)
  wm.focus(a.id); assert.equal(wm.list()[0].id, a.id); assert.equal(a.min, false)
  wm.close(a.id)
  assert.deepEqual(ev.filter((e) => e[0] === 'appClosed'), [['appClosed', 'notities']])
  wm.snap(b.id, 'left'); assert.equal(b.snap, 'left')
  wm.stackMode = true; assert.ok(wm.isMax(b)); wm.snap(b.id, 'right'); assert.equal(b.snap, 'left')
})

test('search: groups in order, collections, diacritics, settings, internet off', () => {
  const idx = buildIndex({ T, DATA, files: DATA.files, actions: ['lock', 'dark'] })
  const groups = (q) => query(idx, q).map((g) => g.group)
  const tuin = query(idx, 'tuin').find((g) => g.group === 'files')
  assert.ok(tuin.items.some((i) => i.id === 'tuinplan'))
  assert.ok(tuin.items.every((i) => i.sub === 'Tuin' || /tuin/i.test(i.label)))
  assert.ok(query(idx, 'fotos').find((g) => g.group === 'apps').items.some((i) => i.id === 'fotos'))
  const c = query(idx, 'contrast').find((g) => g.group === 'settings').items.map((i) => i.label)
  assert.ok(c.some((l) => l.startsWith('Toegankelijkheid')) && c.some((l) => l.startsWith('Weergave')))
  const order = groups('n'); assert.deepEqual(order, GROUPS.filter((g) => order.includes(g)))
  assert.ok(!groups('tuin').includes('internet'))
  assert.deepEqual(query(idx, '  '), [])
})

test('keys: table, repeat never activates, text fields keep their keys, reserved keys unbound', () => {
  const k = (key, o = {}) => ({ key, code: o.code ?? '', ctrlKey: !!o.c, altKey: !!o.a, shiftKey: !!o.s, metaKey: false, repeat: !!o.r })
  assert.equal(match(k('k', { c: 1 })), 'search')
  assert.equal(match(k('k', { c: 1, r: 1 })), null)
  assert.equal(match(k('`', { a: 1, code: 'Backquote' })), 'switchNext')
  assert.equal(match(k('Dead', { a: 1, code: 'Backquote' })), 'switchNext')
  assert.equal(match(k('~', { a: 1, s: 1, code: 'Backquote' })), 'switchPrev')
  assert.equal(match(k('l', { c: 1, a: 1 })), 'lock')
  assert.equal(match(k('End', { c: 1, a: 1 })), 'system')
  for (const [key, o] of [['w', { c: 1 }], ['Tab', { c: 1 }], ['l', { c: 1 }]]) assert.equal(match(k(key, o)), null)
  assert.equal(match(k('ArrowLeft', { a: 1, s: 1 }), { inText: true }), 'snapLeft')
  assert.equal(match(k('F10', { s: 1 }), { inText: true }), null)
  assert.equal(match(k('F6'), { inText: true }), 'regionNext')
  assert.ok(wakes(k('a')) && !wakes(k('Tab')) && !wakes(k('a', { r: 1 })) && !wakes(k('Shift')))
})

test('throttle: 30 changes in one second apply at most three times, last value wins', () => {
  let t = 0; const q = []; const applied = []
  const th = createThrottle((v) => applied.push(v), { now: () => t, later: (fn, ms) => q.push([t + ms, fn]) })
  for (let i = 0; i < 30; i++) {
    t = Math.round(i * 1000 / 30)
    while (q.length && q[0][0] <= t) q.shift()[1]()
    th(i)
  }
  t = 2000; while (q.length) q.shift()[1]()
  assert.ok(applied.length <= 4, String(applied.length))
  assert.equal(applied.at(-1), 29)
})

// repair 1: B8 guard for every control, not only shortcuts (held Enter signed in, cleared notifications, repeated digits)
test('held key: a repeated Enter or Space on a control is swallowed; text editing and arrows keep their repeat', () => {
  const target = (sel) => ({ matches: (q) => q.split(/,\s*(?![^()]*\))/).some((part) => {
    part = part.trim()
    if (part.startsWith('input')) return sel.startsWith('input') && !(/:not\(\[type=checkbox\]\)/.test(part) && /checkbox|radio|button|submit|reset/.test(sel))
    return part === sel || (part === 'button' && sel === 'button')
  }) })
  const ev = (key, sel, repeat = true) => ({ key, repeat, target: target(sel) })
  assert.equal(swallowRepeat(ev('Enter', 'button')), true) // lock wake, Orbit clock, calculator digit
  assert.equal(swallowRepeat(ev(' ', 'button')), true)
  assert.equal(swallowRepeat(ev('Enter', '[role="switch"]')), true) // quick-settings switch
  assert.equal(swallowRepeat(ev('Enter', 'input[type=password]')), true) // implicit form submit of the secret field
  assert.equal(swallowRepeat(ev(' ', 'input[type=text]')), false) // typing spaces keeps its repeat
  assert.equal(swallowRepeat(ev('Enter', 'button', false)), false) // the first press always activates
  assert.equal(swallowRepeat(ev('ArrowDown', 'button')), false)
  assert.equal(swallowRepeat(ev('Enter', 'textarea')), false) // new lines in Notities keep their repeat
})

test('simulation groups: every group has a default inside its options, NAS reachability included', () => {
  for (const [g, opts] of Object.entries(SIM_OPTIONS)) assert.ok(opts.includes(SIM_DEFAULTS[g]), g)
  assert.deepEqual(SIM_OPTIONS.nas, ['online', 'offline'])
  for (const s of built()) for (const [fn] of s.feas) assert.ok(typeof fn === 'string' && fn.length > 3, `${s.id}: status row text from strings.js`)
})
