// UX-E slice B, E11: owner file-open model of 23 November 2026. Positive, negative and revocation tests on the pure broker
// (journey/fileaccess.js) over a fresh contract.js permission instance, plus static checks that the journey wires it.
import { test } from 'node:test'
import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { createPermissions } from '../contract.js'
import { createFileAccess, realInput, saveShortcut } from '../journey/fileaccess.js'

const TRUSTED = Symbol('real-user-gesture') // stands in for a real input event through an injected predicate; the default is tested below
const setup = (answer = { choice: 'deny' }) => {
  const prompts = []
  const p = createPermissions()
  p.setPrompt((r) => { prompts.push(r); return Promise.resolve(answer) })
  const fa = createFileAccess(p, { isUserGesture: (e) => e === TRUSTED })
  return { p, fa, prompts, temp: () => p.grants.temporary(), rem: () => p.grants.remembered() }
}
const has = (list, app, right, data) => list.some((g) => g.app === app && g.right === right && g.data === data && g.scope === 'document' && g.destination === null)

test('positive: an explicit open gives a temporary read on exactly that document, without a prompt', () => {
  const { fa, prompts, temp } = setup()
  assert.equal(fa.shell.open('notities', 'tuinplan', TRUSTED), 'read')
  assert.ok(has(temp(), 'notities', 'read', 'tuinplan'))
  assert.equal(temp().length, 1)
  assert.equal(prompts.length, 0)
  const n = fa.forApp('notities')
  assert.ok(n.canRead('tuinplan'))
  assert.equal(n.level('tuinplan'), 'read')
})

test('negative: opening never grants readWrite and a write after open fails closed', () => {
  const { fa, temp } = setup()
  fa.shell.open('notities', 'tuinplan', TRUSTED)
  assert.ok(!has(temp(), 'notities', 'readWrite', 'tuinplan'))
  let wrote = false
  assert.deepEqual(fa.forApp('notities').write('tuinplan', () => { wrote = true }), { ok: false, reason: 'no-write-grant' })
  assert.equal(wrote, false)
})

test('positive: a trusted edit gives readWrite on that document only; further saves in the session never prompt', async () => {
  const { p, fa, prompts, temp } = setup()
  fa.shell.open('notities', 'tuinplan', TRUSTED)
  fa.shell.open('notities', 'lofoten', TRUSTED)
  assert.equal(fa.shell.edit('notities', 'tuinplan', TRUSTED), true)
  assert.ok(has(temp(), 'notities', 'readWrite', 'tuinplan'))
  assert.ok(!has(temp(), 'notities', 'readWrite', 'lofoten'))
  const n = fa.forApp('notities')
  let saves = 0
  for (let i = 0; i < 3; i++) assert.equal(n.write('tuinplan', () => saves++).ok, true)
  assert.equal(saves, 3)
  assert.equal(n.write('lofoten', () => saves++).ok, false)
  // a repeated identical contract request for the granted key is covered, never a new prompt
  const r = await p.ask({ kind: 'ongoing', app: 'notities', right: 'readWrite', data: 'tuinplan', destination: null, scope: 'document' }, { gesture: true })
  assert.equal(r.via, 'covered')
  assert.equal(prompts.length, 0)
})

test('negative: synthetic or app-originated escalation is refused', () => {
  const { fa, temp } = setup()
  fa.shell.open('notities', 'tuinplan', TRUSTED)
  for (const ev of [undefined, null, {}, { isTrusted: false }, { isTrusted: 'true' }, { isTrusted: 1 }]) assert.equal(fa.shell.edit('notities', 'tuinplan', ev), false)
  const n = fa.forApp('notities')
  assert.deepEqual(n.requestEdit('tuinplan'), { ok: false, reason: 'trusted-surface-only' })
  assert.equal(typeof n.edit, 'undefined')
  assert.ok(Object.isFrozen(n))
  assert.throws(() => { n.edit = () => true })
  assert.ok(!has(temp(), 'notities', 'readWrite', 'tuinplan'))
  // a trusted edit of a document the user never opened is refused as well
  assert.equal(fa.shell.edit('notities', 'boodschappen', TRUSTED), false)
  assert.ok(!has(temp(), 'notities', 'readWrite', 'boodschappen'))
})

test('negative: read on file A gives nothing on file B; separate authorization goes through ask and its prompt', async () => {
  const { p, fa, prompts } = setup({ choice: 'deny' })
  fa.shell.open('notities', 'tuinplan', TRUSTED)
  const n = fa.forApp('notities')
  assert.equal(n.canRead('boodschappen'), false)
  assert.equal(n.write('boodschappen', () => {}).ok, false)
  const r = await p.ask({ kind: 'ongoing', app: 'notities', right: 'read', data: 'boodschappen', destination: null, scope: 'document' }, { gesture: true })
  assert.equal(prompts.length, 1)
  assert.equal(r.answer, 'deny')
  assert.equal(n.canRead('boodschappen'), false)
})

test("negative: no delegation: app X's grant does not serve app Y", () => {
  const { fa } = setup()
  fa.shell.open('notities', 'tuinplan', TRUSTED)
  fa.shell.edit('notities', 'tuinplan', TRUSTED)
  for (const app of ['fotos', 'browser', 'tekst']) {
    const y = fa.forApp(app)
    assert.equal(y.canRead('tuinplan'), false, app)
    assert.equal(y.write('tuinplan', () => {}).ok, false, app)
  }
  assert.equal(fa.shell.edit('fotos', 'tuinplan', TRUSTED), false) // Y cannot be escalated on a file the user handed to X
})

test('negative: neither open nor edit creates a remembered (persistent) grant', () => {
  const { fa, rem } = setup()
  fa.shell.open('notities', 'tuinplan', TRUSTED)
  fa.shell.edit('notities', 'tuinplan', TRUSTED)
  assert.equal(rem().length, 0)
})

test('revocation: closing the document removes its grants, keeps others, and the next write fails closed', () => {
  const { p, fa, temp } = setup()
  p.grants.seedTemporary({ kind: 'ongoing', app: 'notities', right: 'read', data: 'boodschappen', destination: null, scope: 'document' }) // e.g. allowed by prompt
  fa.shell.open('notities', 'tuinplan', TRUSTED); fa.shell.edit('notities', 'tuinplan', TRUSTED)
  fa.shell.open('notities', 'lofoten', TRUSTED)
  fa.shell.close('notities', 'tuinplan')
  assert.ok(!has(temp(), 'notities', 'read', 'tuinplan') && !has(temp(), 'notities', 'readWrite', 'tuinplan'))
  assert.ok(has(temp(), 'notities', 'read', 'lofoten'))
  assert.ok(has(temp(), 'notities', 'read', 'boodschappen'))
  assert.equal(fa.forApp('notities').write('tuinplan', () => {}).ok, false)
  assert.equal(fa.shell.edit('notities', 'tuinplan', TRUSTED), false) // reopening is a new explicit open first
})

test('revocation: app close, user revoke and sign-out each remove the grant; a later write fails closed', () => {
  for (const how of ['appClosed', 'revoke', 'logout']) {
    const { fa, temp } = setup()
    fa.shell.open('notities', 'tuinplan', TRUSTED); fa.shell.edit('notities', 'tuinplan', TRUSTED)
    fa.shell.open('fotos', 'moestuin', TRUSTED)
    if (how === 'appClosed') fa.shell.appClosed('notities')
    if (how === 'revoke') fa.shell.revoke('notities', 'tuinplan')
    if (how === 'logout') fa.shell.logout()
    assert.ok(!temp().some((g) => g.app === 'notities' && g.data === 'tuinplan'), how)
    assert.equal(fa.forApp('notities').write('tuinplan', () => {}).ok, false, how)
    assert.ok(!fa.shell.list().some((x) => x.app === 'notities'), how)
    assert.equal(fa.forApp('fotos').canRead('moestuin'), how !== 'logout', how) // only sign-out ends other apps' grants
  }
})

test('the contract ending the app session directly also ends the broker level (no stale write)', () => {
  const { p, fa } = setup()
  fa.shell.open('notities', 'tuinplan', TRUSTED); fa.shell.edit('notities', 'tuinplan', TRUSTED)
  p.grants.appClosed('notities')
  assert.equal(fa.forApp('notities').canWrite('tuinplan'), false)
  assert.equal(fa.forApp('notities').canRead('tuinplan'), false)
})

test('static: journey wiring follows the model (no readWrite seed, no ask for write, edit only from the shell, isTrusted)', () => {
  const rd = (f) => readFileSync(new URL(`../${f}`, import.meta.url), 'utf8')
  const apps = rd('journey/apps.js'), shell = rd('journey/shell.js'), full = rd('journey.js')
  // the ?test=1 hook (sweep only) may seed states directly; the page outside that block may not
  const boot = full.slice(0, full.indexOf("if (new URLSearchParams(location.search).has('test'))"))
  assert.ok(boot.length > 1000)
  for (const [n, s] of [['apps', apps], ['shell', shell], ['journey', boot]]) {
    assert.doesNotMatch(s, /right: 'readWrite'/, `${n} seeds or asks readWrite directly`)
    assert.doesNotMatch(s, /seedTemporary/, `${n} seeds grants outside the broker`)
  }
  assert.doesNotMatch(apps, /fileAccess\.shell|\.shell\.(open|edit|close|revoke|logout|appClosed)\(|\bedit\(/, 'apps never reach the shell facet')
  assert.match(shell, /fileAccess\.shell\.edit\([^)]*\be\)/, 'the shell passes the real event to edit')
  assert.match(boot, /fileAccess\.shell\.logout\(\)/, 'sign-out ends file grants')
  assert.match(shell, /async function openFile\(id, ev\) \{[^]*?if \(!isUserGesture\(ev\)\)/, 'opens need a real user gesture (checked again in the broker)')
  assert.match(rd('journey/access.js'), /realInput\(e\) \|\| \(TEST_OPEN !== null && e === TEST_OPEN\)/, 'the page predicate: a real input event, or the ?test=1 token only')
  assert.match(boot, /grants\.logout\(\)/, 'bootstrap ends the page-load seed')
})

test('negative (repair 1): the default gesture rule refuses forged objects and script-made events for open and edit', () => {
  const p = createPermissions()
  const fa = createFileAccess(p) // default predicate: a real, browser-trusted input Event only
  const forged = [undefined, null, {}, { isTrusted: true }, { isTrusted: true, type: 'click' }, new Event('click'), Object.create(Event.prototype)]
  for (const ev of forged) {
    assert.equal(realInput(ev), false)
    assert.equal(fa.shell.open('notities', 'tuinplan', ev), null, 'open needs a real user gesture')
  }
  assert.equal(p.grants.temporary().length, 0)
  // even with a document handed over through an injected trusted path, the default broker never escalates on a forgery
  const q = createPermissions(), G = Symbol('g')
  const fb = createFileAccess(q, { isUserGesture: (e) => e === G })
  fb.shell.open('notities', 'tuinplan', G)
  assert.equal(fb.shell.edit('notities', 'tuinplan', { isTrusted: true }), false)
  assert.ok(!has(q.grants.temporary(), 'notities', 'readWrite', 'tuinplan'))
})

test('static (repair 1): apps hold only the app facet; every shell open route passes the input event', () => {
  const rd = (f) => readFileSync(new URL(`../${f}`, import.meta.url), 'utf8')
  for (const f of ['journey/apps.js', 'journey/stations.js']) {
    const s = rd(f)
    assert.doesNotMatch(s, /\bfileAccess\b/, `${f} must import appAccess only`)
  }
  const shell = rd('journey/shell.js')
  // one helper builds every open handler, so a missing event argument cannot recur per route
  assert.doesNotMatch(shell, /openFile\(\s*[\w.]+\s*\)/, 'an openFile call without the event')
  assert.match(shell, /fileAccess\.shell\.open\('notities', id, ev\)/, 'the broker receives the gesture itself')
})

// B repair 2: closing or switching a document must not touch decisions outside the file-open path (verifier W8, W9).
const docReq = (app, data, right = 'read') => ({ kind: 'ongoing', app, right, data, destination: null, scope: 'document' })
test('side effects (repair 2): a remembered allow or deny on (app, read, doc) survives open and close of that document', async () => {
  for (const choice of ['allow', 'deny']) {
    const { p, fa, rem } = setup({ choice, remember: true })
    assert.equal((await p.ask(docReq('notities', 'tuinplan'), { gesture: true })).remembered, true)
    fa.shell.open('notities', 'tuinplan', TRUSTED)
    fa.shell.edit('notities', 'tuinplan', TRUSTED)
    fa.shell.close('notities', 'tuinplan')
    fa.shell.open('notities', 'boodschappen', TRUSTED)
    fa.shell.revoke('notities', 'boodschappen')
    assert.equal(rem().length, 1, `${choice}: the remembered decision is still listed`)
    assert.equal(rem()[0].decision, choice)
    const again = await p.ask(docReq('notities', 'tuinplan'), { gesture: true })
    assert.equal(again.via, 'remembered', `${choice}: the next request is answered by the remembered decision, without a prompt`)
  }
})
test('side effects (repair 2): another app keeps its temporary grants and denials when a document of a different app closes', async () => {
  const { p, fa } = setup({ choice: 'deny' })
  await p.ask({ kind: 'ongoing', app: 'browser', right: 'connect', data: 'site', destination: 'tuinplanner.example', scope: 'domain' }, { gesture: true })
  fa.shell.open('notities', 'tuinplan', TRUSTED)
  fa.shell.close('notities', 'tuinplan')
  assert.equal(p.grants.temporaryDenials().filter((g) => g.app === 'browser').length, 1)
})
// B repair 3: ending one document uses contract grants.endTemporary (44ef6030), so the app's own R2 denial survives.
test('side effects (repair 2): the same app keeps its R2 temporary denial when one of its documents closes', async () => {
  const { p, fa } = setup({ choice: 'deny' })
  fa.shell.open('notities', 'tuinplan', TRUSTED)
  await p.ask(docReq('notities', 'boodschappen'), { gesture: true })
  fa.shell.close('notities', 'tuinplan')
  assert.equal(p.grants.temporaryDenials().length, 1)
})

// B repair 3, lead decision 1: ending one document emits only end-temporary events, never app-closed, and touches no other key
test('side effects (repair 3): closing a document ends only its own keys through endTemporary; no app-closed event', async () => {
  const ev = []
  const p = createPermissions({ emit: (t, d) => ev.push(d.op) })
  p.setPrompt(() => Promise.resolve({ choice: 'allow' }))
  const fa = createFileAccess(p, { isUserGesture: (e) => e === TRUSTED })
  fa.shell.open('notities', 'tuinplan', TRUSTED); fa.shell.edit('notities', 'tuinplan', TRUSTED)
  await p.ask(docReq('notities', 'boodschappen'), { gesture: true })
  const before = p.grants.temporary().filter((g) => g.data !== 'tuinplan')
  ev.length = 0
  fa.shell.close('notities', 'tuinplan')
  assert.deepEqual(ev, ['end-temporary', 'end-temporary'])
  assert.deepEqual(p.grants.temporary(), before)
})

// B repair 3, lead decision 2: a document read the user allowed in the prompt is revocable from the grants view
test('revocation (repair 3): Intrekken on a prompt-allowed read ends it; the next read prompts again; nothing else changes', async () => {
  const { p, fa, prompts, temp, rem } = setup({ choice: 'allow' })
  await p.ask({ kind: 'ongoing', app: 'browser', right: 'connect', data: 'site', destination: 'tuinplanner.example', scope: 'domain' }, { gesture: true })
  p.setPrompt((r) => { prompts.push(r); return Promise.resolve({ choice: r.app === 'fotos' ? 'deny' : 'allow' }) })
  await p.ask({ kind: 'ongoing', app: 'fotos', right: 'camera', data: 'camera', destination: null, scope: 'while-open' }, { gesture: true })
  assert.equal((await p.ask(docReq('notities', 'boodschappen'), { gesture: true })).answer, 'allow')
  fa.shell.open('notities', 'tuinplan', TRUSTED)
  const n = prompts.length
  assert.equal((await p.ask(docReq('notities', 'boodschappen'), { gesture: true })).via, 'covered', 'covered while held')
  assert.equal(prompts.length, n)
  fa.shell.revoke('notities', 'boodschappen') // the grants view calls this for a row that is not an opened file
  assert.ok(!has(temp(), 'notities', 'read', 'boodschappen'))
  assert.ok(has(temp(), 'notities', 'read', 'tuinplan'), 'the opened document keeps its read')
  assert.ok(temp().some((g) => g.app === 'browser' && g.right === 'connect'), 'another app keeps its connect grant')
  assert.equal(p.grants.temporaryDenials().length, 1, 'the R2 camera denial of Fotos is untouched')
  assert.equal(rem().length, 0)
  await p.ask(docReq('notities', 'boodschappen'), { gesture: true })
  assert.equal(prompts.length, n + 1, 'the next read asks again')
})

// B repair 3, lead decision 3: Ctrl+S never escalates; it saves only when readWrite is already held
test('negative (repair 3): the Save shortcut never escalates a read-only document; it saves only with readWrite', () => {
  assert.equal(saveShortcut(null), 'none')
  assert.equal(saveShortcut('read'), 'point-to-edit')
  assert.equal(saveShortcut('readWrite'), 'save')
  const { fa } = setup()
  fa.shell.open('notities', 'tuinplan', TRUSTED)
  assert.equal(saveShortcut(fa.shell.level('notities', 'tuinplan')), 'point-to-edit')
  assert.equal(fa.forApp('notities').write('tuinplan', () => {}).ok, false)
  fa.shell.edit('notities', 'tuinplan', TRUSTED)
  assert.equal(saveShortcut(fa.shell.level('notities', 'tuinplan')), 'save')
})
test('static (repair 3): the window keydown routes Ctrl+S through saveShortcut and never reaches editAction; edit needs a frame control', () => {
  const shell = readFileSync(new URL('../journey/shell.js', import.meta.url), 'utf8')
  const at = shell.indexOf("e.key.toLowerCase() === 's'")
  const kd = shell.slice(shell.lastIndexOf('keydown:', at), shell.indexOf('focusin:', at)) // the window frame's Ctrl+S handler
  assert.ok(kd.length > 40, 'window keydown handler found')
  assert.match(kd, /saveKey\(w\.id\)/)
  assert.doesNotMatch(kd, /editAction|saveAction|fileAccess\.shell\.edit/, 'the keyboard path never escalates')
  const sk = shell.slice(shell.indexOf('function saveKey(id) {'), shell.indexOf('function saveAction(id, e) {'))
  assert.ok(sk.length > 40, 'saveKey found')
  assert.match(sk, /saveShortcut\(lv\)/)
  assert.doesNotMatch(sk, /editAction|saveAction|fileAccess\.shell\.edit/, 'saveKey never escalates')
  assert.match(sk, /'point-to-edit'\) \{ nodes\.get\(id\)\?\.node\.querySelector\('\.jr-edit'\)\?\.focus\(\)/, 'a read-only Ctrl+S moves focus to Bewerken')
  assert.match(shell, /function editAction\(id, e\) \{[^]*?fromFrame\(e\)/, 'editAction accepts only a frame control activation')
  assert.match(shell, /const fromFrame = \(e\) => [^\n]*\.jr-frame-acts/)
})
