// Run 2r: a remembered Toestaan answers only a request for which remembering Toestaan is offered (OQ-1), while the key
// stays exactly five fields (OQ-4). Prototype only; nothing here is enforced by BaryonOS.
// Run: node --test --experimental-test-isolation=none ui/design/prototype/tests/*.test.mjs
import { test } from 'node:test'
import assert from 'node:assert/strict'
import { createPermissions } from '../contract.js'

const FIVE = { app: 'browser', right: 'send-copy', data: 'tuinplan', destination: 't.example', scope: 'document' }
const IRREVERSIBLE = { kind: 'action', ...FIVE, irreversible: true }

function setup(...answers) {
  const P = createPermissions(), shown = []
  P.setPrompt((req, view) => { shown.push({ req, view }); return Promise.resolve(answers.shift() ?? { choice: 'cancel' }) })
  return { P, shown }
}

for (const [name, earlier] of [['ongoing send-copy', { kind: 'ongoing', ...FIVE }], ['reversible send-copy action', { kind: 'action', ...FIVE, irreversible: false }]]) {
  test(`OQ-1: a remembered Toestaan for a ${name} never answers an irreversible single external send with the same five fields`, async () => {
    const { P, shown } = setup({ choice: 'allow', remember: true }, { choice: 'cancel' })
    const a = await P.ask(earlier)
    assert.equal(a.via, 'prompt'); assert.equal(a.remembered, true)
    const b = await P.ask(IRREVERSIBLE)
    assert.equal(shown.length, 2, 'the irreversible send is asked')
    assert.notEqual(b.via, 'remembered')
    assert.equal(b.answer, 'deny') // the scripted prompt was cancelled: fail closed
    assert.equal(shown[1].view.rememberAllow, false, 'remembering Toestaan is not offered for it')
    // the earlier request itself is still answered by its remembered Toestaan
    assert.equal((await P.ask(earlier)).via, 'remembered')
    assert.equal(shown.length, 2)
  })
}

test('OQ-1: Toestaan on the fall-through prompt (checkbox checked) allows that one send and leaves the stored Toestaan as it was', async () => {
  const { P, shown } = setup({ choice: 'allow', remember: true }, { choice: 'allow', remember: true })
  await P.ask({ kind: 'ongoing', ...FIVE })
  const b = await P.ask(IRREVERSIBLE)
  assert.deepEqual([b.answer, b.via, b.remembered, b.until], ['allow', 'prompt', false, 'action'])
  const g = P.grants.remembered()
  assert.equal(g.length, 1); assert.equal(g[0].decision, 'allow'); assert.equal(g[0].kind, 'ongoing')
  await P.ask(IRREVERSIBLE)
  assert.equal(shown.length, 3, 'the next irreversible send is asked again')
})

test('a legitimately remembered Toestaan still answers without a prompt, including a confirmed combined document-and-internet right', async () => {
  const READ = { kind: 'ongoing', app: 'notities', right: 'read', data: 'tuinplan', destination: null, scope: 'document' }
  const COMBO = { kind: 'ongoing', app: 'browser', right: 'read-connect', data: 'tuinplan', destination: 'tuinplanner.example', scope: 'document' }
  const REV = { kind: 'action', ...FIVE, irreversible: false }
  const { P, shown } = setup({ choice: 'allow', remember: true }, { choice: 'allow', remember: true, confirmed: true }, { choice: 'allow', remember: true })
  for (const r of [READ, COMBO, REV]) assert.equal((await P.ask(r)).remembered, true)
  assert.equal(shown.length, 3)
  for (const r of [READ, COMBO, REV]) {
    const res = await P.ask(r)
    assert.deepEqual([res.answer, res.via], ['allow', 'remembered'])
  }
  assert.equal(shown.length, 3, 'no prompt for a valid remembered grant')
})

test('Weigeren with remembering on the fall-through prompt replaces the stored Toestaan for those five fields with a remembered Weigeren', async () => {
  // The five fields are the identity of the request (OQ-4), so one remembered decision exists per key. The newer refusal
  // replaces the earlier Toestaan: access only narrows, never silently broadens. The earlier ongoing request is now refused too.
  const { P, shown } = setup({ choice: 'allow', remember: true }, { choice: 'deny', remember: true })
  const ongoing = { kind: 'ongoing', ...FIVE }
  await P.ask(ongoing)
  const b = await P.ask(IRREVERSIBLE)
  assert.deepEqual([b.answer, b.via, b.remembered], ['deny', 'prompt', true])
  const g = P.grants.remembered()
  assert.equal(g.length, 1); assert.equal(g[0].decision, 'deny')
  for (const r of [IRREVERSIBLE, ongoing]) { const { answer, via } = await P.ask(r); assert.deepEqual([answer, via], ['deny', 'remembered']) }
  assert.equal(shown.length, 2)
  // a different destination is a different request and is asked
  await P.ask({ ...IRREVERSIBLE, destination: 'ander.example' })
  assert.equal(shown.length, 3)
})

test('Appmachtigingen: a remembered Weigeren cannot be changed to Toestaan when the change is made for an irreversible single send', async () => {
  const { P } = setup({ choice: 'deny', remember: true })
  await P.ask({ kind: 'ongoing', ...FIVE })
  assert.equal(P.grants.change({ ...IRREVERSIBLE }, 'allow'), false)
  assert.equal(P.grants.remembered()[0].decision, 'deny')
  // changed for the stored (ongoing) request it is allowed, and the irreversible send is still asked afterwards
  assert.equal(P.grants.change(P.grants.remembered()[0], 'allow'), true)
  let asked = 0; P.setPrompt(() => { asked++; return Promise.resolve({ choice: 'cancel' }) })
  assert.notEqual((await P.ask(IRREVERSIBLE)).via, 'remembered')
  assert.equal(asked, 1)
})
