// Contract tests for the UX-C permission model (prototype only; nothing here is enforced by BaryonOS).
// Written from the owner decision of 15 November 2026 (items 1-8), readings R2 and R3 and the owner decisions of
// 18-19 November 2026 (OQ-1..OQ-6, MF-1..MF-6). No assertion is reused from the superseded C1 contract test.
// Run: node --test --experimental-test-isolation=none ui/design/prototype/tests/*.test.mjs
import { test } from 'node:test'
import assert from 'node:assert/strict'
import { createPermissions, appReason, keyOf, KEY } from '../contract.js'

const READ = { kind: 'ongoing', app: 'notities', right: 'read', data: 'tuinplan', destination: null, scope: 'document' }
const SAVE = { kind: 'action', app: 'notities', right: 'save-copy', data: 'tuinplan', destination: null, scope: 'document' }
const SEND = { kind: 'action', app: 'browser', right: 'send-copy', data: 'tuinplan', destination: 'tuinplanner.example', scope: 'document', irreversible: true }
const COMBO = { kind: 'ongoing', app: 'browser', right: 'read-connect', data: 'tuinplan', destination: 'tuinplanner.example', scope: 'document' }
const CAMERA = { kind: 'ongoing', app: 'fotos', right: 'camera', data: 'camera-voor', destination: null, scope: 'while-open' }

// a scripted prompt: answers in order, records every view it was shown
function setup(...answers) {
  const P = createPermissions(), shown = []
  P.setPrompt((req, view) => { shown.push({ req, view }); return Promise.resolve(answers.shift() ?? { choice: 'cancel' }) })
  return { P, shown }
}
const allow = (remember = false, confirmed) => ({ choice: 'allow', remember, confirmed })
const deny = (remember = false) => ({ choice: 'deny', remember })

test('owner 1 + R3: every prompt opens with remembering off; the model never stores without an explicit remember', async () => {
  const { P, shown } = setup(allow(), allow())
  await P.ask(SAVE); await P.ask(SAVE)
  assert.equal(shown.length, 2)
  assert.equal(P.grants.remembered().length, 0)
})

test('owner 2 + O-G12: Toestaan without remembering on an ongoing right lasts until the app closes', async () => {
  const { P, shown } = setup(allow())
  assert.deepEqual(await P.ask(READ), { answer: 'allow', via: 'prompt', remembered: false, until: 'app-session' })
  assert.equal((await P.ask(READ)).via, 'covered')
  assert.equal(shown.length, 1)
  P.grants.appClosed('notities')
  await P.ask(READ)
  assert.equal(shown.length, 2)
})

test('owner 2 + O-G4: Toestaan without remembering on a single action covers that action only', async () => {
  const { P, shown } = setup(allow(), deny())
  assert.equal((await P.ask(SAVE)).until, 'action')
  await P.ask(SAVE)
  assert.equal(shown.length, 2)
})

test('owner 3 + R2: Weigeren without remembering records no lasting denial and refuses automatic re-requests quietly until the app closes', async () => {
  const events = []
  const P = createPermissions({ emit: (t, d) => events.push(d) })
  let n = 0; P.setPrompt(() => { n++; return Promise.resolve(deny()) })
  await P.ask(READ)
  assert.equal(P.grants.remembered().length, 0)
  const r = await P.ask(READ)
  assert.deepEqual([r.answer, r.via, n], ['deny', 'temporary-deny', 1])
  assert.equal(r.retries, 1, 'first quiet refusal reports its count to the app note')
  assert.equal((await P.ask(READ)).retries, 2, 'the result carries the growing retry count')
  assert.equal(events.filter((e) => e.op === 'quiet-deny').at(-1).retries, 2, 'retry count for the quiet shell note')
  await P.ask(READ, { gesture: true }) // a new deliberate user gesture asks again
  assert.equal(n, 2)
  P.grants.appClosed('notities')
  assert.equal(P.grants.temporaryDenials().length, 0)
})

test('R2 single action: a temporary denial covers automatic identical re-requests in the session', async () => {
  const { P, shown } = setup(deny())
  await P.ask(SAVE)
  assert.equal((await P.ask(SAVE)).via, 'temporary-deny')
  assert.equal(shown.length, 1)
})

test('owner 4 + O-G2: with remembering, Toestaan and Weigeren are both stored for future identical requests', async () => {
  const a = setup(allow(true)); await a.P.ask(READ)
  assert.deepEqual(await a.P.ask(READ), { answer: 'allow', via: 'remembered', remembered: true, until: 'remembered' })
  const d = setup(deny(true)); await d.P.ask(READ)
  assert.equal((await d.P.ask(READ, { gesture: true })).answer, 'deny')
  assert.equal(d.shown.length, 1)
})

test('owner 5 + OQ-4: the strict five-field key; changing any one field is a new question', async () => {
  assert.deepEqual(KEY, ['app', 'right', 'data', 'destination', 'scope'])
  const variants = [
    { ...READ, app: 'fotos' }, { ...READ, right: 'readWrite' }, { ...READ, data: 'plattegrond' }, { ...READ, scope: 'collection' },
    { ...COMBO, destination: 'zaadruil.example' },
  ]
  const { P, shown } = setup(allow(true), allow(true), ...variants.map(() => deny()))
  await P.ask(READ); await P.ask(COMBO, {}).catch(() => {}) // COMBO stored? only with confirmation (see OQ-1 test)
  for (const v of variants) await P.ask(v)
  assert.equal(shown.length, 2 + variants.length)
})

test('owner 5 + MF-3: data is one exact item, never a class or pattern; camera and microphone are separate rights', async () => {
  const P = createPermissions()
  await assert.rejects(P.ask({ ...READ, data: '*.pdf' }), TypeError)
  await assert.rejects(P.ask({ ...CAMERA, right: 'camera+microphone' }), TypeError)
  assert.notEqual(keyOf(CAMERA), keyOf({ ...CAMERA, right: 'microphone' }))
})

test('owner 6: a valid grant never re-prompts; no anti-nag throttling across distinct keys (OQ-4)', async () => {
  const { P, shown } = setup(allow(true), deny(), deny())
  await P.ask(READ)
  for (let i = 0; i < 5; i++) assert.equal((await P.ask(READ)).answer, 'allow')
  await P.ask({ ...READ, data: 'plattegrond' }) // denied temporarily
  await P.ask({ ...READ, data: 'boodschappen' }) // a distinct legitimate request still prompts
  assert.equal(shown.length, 3)
})

test('owner 7: lasting choices are visible, changeable and revocable; revocation removes exactly the matching key', async () => {
  const { P } = setup(allow(true), deny(true))
  await P.ask(READ); await P.ask({ ...READ, data: 'plattegrond' })
  assert.equal(P.grants.remembered().length, 2)
  assert.equal(P.grants.change(READ, 'deny'), true)
  assert.equal((await P.ask(READ)).answer, 'deny')
  assert.equal(P.grants.revoke({ ...READ, scope: 'collection' }), null)
  assert.equal(P.grants.revoke(READ).data, 'tuinplan')
  assert.deepEqual(P.grants.remembered().map((g) => g.data), ['plattegrond'])
})

test('owner 8 + MF-1: each result states its real lifetime', async () => {
  const { P } = setup(allow(), allow(), deny())
  assert.equal((await P.ask(READ)).until, 'app-session')
  assert.equal((await P.ask(SAVE)).until, 'action')
  assert.equal((await P.ask(CAMERA)).until, 'app-session') // a temporary denial also lasts until the app closes
})

test('duplicate-grant race: identical concurrent requests share one prompt and store one decision', async () => {
  const P = createPermissions(); let n = 0, release
  P.setPrompt(() => { n++; return new Promise((r) => { release = r }) })
  const a = P.ask(READ), b = P.ask(READ)
  release(allow(true))
  assert.deepEqual(await a, await b)
  assert.equal(n, 1)
  assert.equal(P.grants.remembered().length, 1)
})

test('owner decision 1: Escape or closing cancels fail-closed, also with the checkbox checked; a new deliberate action may ask again', async () => {
  for (const res of [{ choice: 'cancel', remember: true }, { choice: undefined }, null]) {
    const { P, shown } = setup(res, allow())
    const r = await P.ask(READ)
    assert.deepEqual(r, { answer: 'deny', via: 'cancel', remembered: false, until: 'app-session' })
    assert.equal(P.grants.remembered().length, 0)
    assert.equal(P.grants.temporary().length, 0)
    assert.equal((await P.ask(READ)).via, 'temporary-deny')
    assert.equal((await P.ask(READ, { gesture: true })).answer, 'allow')
    assert.equal(shown.length, 2)
  }
})

test('OQ-1: no remembered Toestaan for an irreversible single external send; that one send is allowed and nothing is stored', async () => {
  const { P, shown } = setup(allow(true), allow())
  const r = await P.ask(SEND)
  assert.equal(shown[0].view.rememberAllow, false)
  assert.deepEqual(r, { answer: 'allow', via: 'prompt', remembered: false, until: 'action' })
  assert.equal(P.grants.remembered().length, 0)
  await P.ask(SEND)
  assert.equal(shown.length, 2)
})

test('OQ-1: a remembered Weigeren for an external send is keyed to the exact request', async () => {
  const { P, shown } = setup(deny(true), deny())
  await P.ask(SEND)
  assert.equal((await P.ask(SEND, { gesture: true })).via, 'remembered')
  await P.ask({ ...SEND, destination: 'zaadruil.example' })
  assert.equal(shown.length, 2)
  assert.equal(P.grants.change(SEND, 'allow'), false)
})

test('OQ-1: a combined document-and-internet Toestaan is remembered only with destination, bounded data and scope and the confirmation', async () => {
  const no = setup(allow(true))
  assert.equal(no.P.grants.remembered().length, 0)
  assert.equal((await no.P.ask(COMBO)).remembered, false)
  assert.equal(no.shown[0].view.confirmAllow, true)
  const wide = setup(allow(true, true))
  assert.equal((await wide.P.ask({ ...COMBO, scope: 'collection' })).remembered, false)
  const yes = setup(allow(true, true))
  assert.equal((await yes.P.ask(COMBO)).remembered, true)
  assert.equal(yes.P.grants.change(COMBO, 'allow'), false, 'changing to Toestaan outside the prompt would skip the confirmation')
})

test('OQ-5: a temporary grant ends on app termination, logout and session switch, survives window hide, has no timeout', async () => {
  for (const end of ['appClosed', 'logout', 'sessionSwitch']) {
    const { P } = setup(allow())
    await P.ask(READ)
    P.grants.windowHidden('notities')
    await new Promise((r) => setTimeout(r, 30))
    assert.equal((await P.ask(READ)).via, 'covered')
    P.grants[end]('notities')
    assert.equal(P.grants.temporary().length, 0, end)
  }
  const { P } = setup(allow())
  await P.ask(READ)
  P.grants.appClosed('fotos') // another app's end changes nothing
  assert.equal(P.grants.temporary().length, 1)
})

test('OQ-6: an app reason over 120 characters or non-plain is not shown; markup is plain text', async () => {
  assert.equal(appReason('x'.repeat(120)), 'x'.repeat(120))
  assert.equal(appReason('x'.repeat(121)), null)
  for (const bad of ['', 'a\nb', 'a‮b', 'a\u0000b', 42, null]) assert.equal(appReason(bad), null)
  assert.equal(appReason('<b>Nu</b>'), '<b>Nu</b>')
  const { P, shown } = setup(deny())
  await P.ask({ ...READ, reason: 'y'.repeat(200) })
  assert.equal(shown[0].view.reason, null)
})

test('MF-5: compact only for an identical, earlier temporarily allowed, low-risk request; full prompt for every exclusion', async () => {
  const { P, shown } = setup(allow(), allow(), allow(), deny(), allow(), allow(), allow(), allow())
  await P.ask(SAVE)
  await P.ask(SAVE)
  assert.equal(shown[1].view.compact, true, 'identical low-risk repeat')
  await P.ask({ ...SAVE, context: 'nieuw' })
  assert.equal(shown[2].view.compact, false, 'changed circumstance')
  await P.ask({ ...SAVE, data: 'plattegrond' })
  assert.equal(shown[3].view.compact, false, 'changed permission (other key)')
  await P.ask(SEND); await P.ask(SEND)
  assert.equal(shown[5].view.compact, false, 'external send')
  await P.ask(CAMERA); P.grants.appClosed('fotos'); await P.ask(CAMERA)
  assert.equal(shown[7].view.compact, false, 'camera')
  P.grants.logout()
  assert.equal(P.grants.eligible(SAVE), false, 'the eligibility record is volatile')
})

test('MF-5: a later refusal, cancellation, remembered answer or revocation ends compact eligibility', async () => {
  // allow, then temporary Weigeren, then a deliberate re-ask: full prompt
  let { P, shown } = setup(allow(), deny(), allow())
  await P.ask(SAVE); await P.ask(SAVE); await P.ask(SAVE, { gesture: true })
  assert.deepEqual(shown.map((s) => s.view.compact), [false, true, false], 'after a temporary Weigeren')
  // allow, then remembered Weigeren, then revoke: full prompt
  ;({ P, shown } = setup(allow(), deny(true), allow()))
  await P.ask(SAVE); await P.ask(SAVE); P.grants.revoke(SAVE); await P.ask(SAVE)
  assert.deepEqual(shown.map((s) => s.view.compact), [false, true, false], 'after a revoked remembered Weigeren')
  // allow, then Escape, then a deliberate re-ask: full prompt
  ;({ P, shown } = setup(allow(), { choice: 'cancel' }, allow()))
  await P.ask(SAVE); await P.ask(SAVE); await P.ask(SAVE, { gesture: true })
  assert.deepEqual(shown.map((s) => s.view.compact), [false, true, false], 'after a cancellation')
  // allow (read, ongoing), then remembered Toestaan via change from a remembered Weigeren, then revoke: full prompt
  ;({ P, shown } = setup(allow(), deny(true), allow()))
  await P.ask(READ); P.grants.appClosed('notities'); await P.ask(READ)
  assert.equal(P.grants.change(READ, 'allow'), true); P.grants.revoke(READ); await P.ask(READ)
  assert.equal(shown[2].view.compact, false, 'after change and revoke')
})

test('R3: remembering never carries over; after a remembered answer the next distinct request is not remembered', async () => {
  const OTHER = { ...READ, data: 'boodschappen' }
  for (const [first, second] of [[allow(true), allow()], [deny(true), deny()], [allow(true), deny()], [deny(true), allow()]]) {
    const { P } = setup(first, second)
    assert.equal((await P.ask(READ)).remembered, true)
    const r = await P.ask(OTHER)
    assert.equal(r.remembered, false)
    assert.equal(P.grants.remembered().some((g) => keyOf(g) === keyOf(OTHER)), false)
    assert.equal(P.grants.remembered().length, 1)
  }
})

test('OQ-4: a distinct request that arrives while a prompt is open waits its turn; it never cancels or silently denies the open one', async () => {
  const P = createPermissions(), opened = []
  P.setPrompt((req) => new Promise((resolve) => opened.push({ req, resolve })))
  const OTHER = { ...READ, data: 'boodschappen' }
  const first = P.ask(READ), second = P.ask(OTHER), third = P.ask(SAVE)
  await new Promise((r) => setTimeout(r, 0))
  assert.equal(opened.length, 1, 'one prompt at a time')
  opened[0].resolve(allow())
  assert.equal((await first).answer, 'allow')
  await new Promise((r) => setTimeout(r, 0))
  assert.equal(opened.length, 2); assert.equal(opened[1].req.data, 'boodschappen')
  opened[1].resolve(deny())
  assert.equal((await second).via, 'prompt')
  await new Promise((r) => setTimeout(r, 0))
  opened[2].resolve(allow())
  assert.equal((await third).answer, 'allow')
  assert.equal(P.grants.temporaryDenials().length, 1, 'only the explicit Weigeren is recorded')
})

test('a prompt that cannot be shown fails closed without a temporary denial, so the next request asks normally', async () => {
  const P = createPermissions(); let n = 0
  P.setPrompt(() => { n++; if (n === 1) throw new Error('no copy'); return Promise.resolve(allow()) })
  assert.deepEqual(await P.ask(READ), { answer: 'deny', via: 'error', remembered: false, until: 'action' })
  assert.equal(P.grants.temporaryDenials().length, 0)
  assert.equal(P.grants.remembered().length, 0)
  assert.equal((await P.ask(READ)).via, 'prompt', 'no R2 suppression after an error')
})
