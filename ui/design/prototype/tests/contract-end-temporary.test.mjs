// Tests for grants.endTemporary, added by the lead for the owner file-open decision of 23 November 2026
// (revocation on document close and user revoke while the app keeps running). Prototype only; not enforced by BaryonOS.
// Run: node --test --experimental-test-isolation=none ui/design/prototype/tests/*.test.mjs
import { test } from 'node:test'
import assert from 'node:assert/strict'
import { createPermissions, keyOf } from '../contract.js'

const READ = { kind: 'ongoing', app: 'notities', right: 'read', data: 'tuinplan', destination: null, scope: 'document' }
const OTHER = { ...READ, data: 'reisplan' }
const BOOD = { ...READ, data: 'boodschappen' }
const FOTO = { ...READ, app: 'fotos' }

function setup(...answers) {
  const events = [], P = createPermissions({ emit: (t, d) => events.push(d) }), shown = []
  P.setPrompt((req, view) => { shown.push({ req, view }); return Promise.resolve(answers.shift() ?? { choice: 'cancel' }) })
  return { P, shown, events }
}

test('endTemporary ends exactly one temporary grant; the next request prompts again in full', async () => {
  const { P, shown, events } = setup({ choice: 'allow' }, { choice: 'allow' })
  await P.ask(READ)
  const g = P.grants.endTemporary(READ)
  assert.equal(g.data, 'tuinplan')
  assert.deepEqual(events.at(-1), { op: 'end-temporary', key: keyOf(READ) })
  assert.equal(P.grants.temporary().length, 0)
  assert.equal(P.grants.eligible(READ), false) // changed circumstance: no compact form
  await P.ask(READ)
  assert.equal(shown.length, 2)
  assert.equal(shown[1].view.compact, false)
})

test('endTemporary leaves other keys, other apps, R2 denials and remembered decisions untouched', async () => {
  const { P } = setup({ choice: 'deny' }, { choice: 'deny', remember: true })
  P.grants.seedTemporary(READ); P.grants.seedTemporary(OTHER); P.grants.seedTemporary(FOTO)
  await P.ask(BOOD) // temporary denial (R2)
  await P.ask({ ...BOOD, data: 'dagboek' }) // remembered Weigeren
  P.grants.endTemporary(READ)
  assert.deepEqual(P.grants.temporary().map((g) => `${g.app}/${g.data}`).sort(), ['fotos/tuinplan', 'notities/reisplan'])
  assert.equal(P.grants.temporaryDenials().length, 1)
  assert.equal(P.grants.remembered().length, 1)
  assert.equal((await P.ask(BOOD)).via, 'temporary-deny') // automatic re-request still refused quietly
})

test('endTemporary on a key without a temporary grant changes nothing and emits nothing', () => {
  const { P, events } = setup()
  P.grants.seedTemporary(READ)
  assert.equal(P.grants.endTemporary(OTHER), null)
  assert.equal(P.grants.endTemporary({ ...READ, right: 'readWrite' }), null)
  assert.equal(P.grants.temporary().length, 1)
  assert.equal(events.length, 0)
})
