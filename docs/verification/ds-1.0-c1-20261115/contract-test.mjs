// Independent verifier test of ui/design/prototype/contract.js (UX-D-C1 repair-1, derived from verify-0). Run: node contract-test.mjs <worktree>
import assert from 'node:assert/strict'
import { pathToFileURL } from 'node:url'
import path from 'node:path'
const root = process.argv[2]
const C = await import(pathToFileURL(path.join(root, 'ui/design/prototype/contract.js')).href)
const { ask, setPrompt, grants, CHOICES, bus, T, destination } = C
const log = []
let n = 0
const ok = (cond, msg) => { assert.ok(cond, msg); n++; log.push('PASS ' + msg) }
let prompts = 0, next = 'deny'
setPrompt(async (req) => { prompts++; return next })
const events = []
bus.on('grants', (d) => events.push(d.op))

ok(JSON.stringify(CHOICES) === JSON.stringify({ ongoing: ['untilClose', 'always', 'deny'], action: ['once', 'always', 'deny'] }), 'CHOICES exactly ongoing untilClose|always|deny, action once|always|deny')
ok(JSON.stringify(T.trusted.choices.ongoing) === JSON.stringify({ untilClose: 'Toestaan tot afsluiten', always: 'Altijd toestaan', deny: 'Weigeren' }), 'strings ongoing labels')
ok(JSON.stringify(T.trusted.choices.action) === JSON.stringify({ once: 'Eenmalig toestaan', always: 'Altijd toestaan', deny: 'Weigeren' }), 'strings action labels')
const seed = grants.all()
log.push('seeded grants: ' + JSON.stringify(seed))

const A = { kind: 'ongoing', app: 'browser', resource: 'tuinplanner.example', right: 'connect', scope: 'domain' }
// untilClose
next = 'untilClose'; prompts = 0
ok(await ask(A) === 'untilClose' && prompts === 1, 'ongoing untilClose prompts once and returns untilClose')
ok(grants.covering(A)?.until === 'close', 'untilClose stored as until close')
ok(await ask(A) === 'untilClose' && prompts === 1, 'covered request does not prompt again (untilClose)')
// app closes -> untilClose lapses
grants.appClosed('browser')
ok(grants.covering(A) === null, 'appClosed ends the untilClose grant')
next = 'deny'
ok(await ask(A) === 'deny' && prompts === 2, 'after close the same request prompts again')
ok(grants.covering(A) === null, 'deny stores nothing')
// once on action
const B = { kind: 'action', app: 'browser', resource: 'tuinplan', right: 'send-copy', scope: 'document', to: 'tuinplanner.example' }
const before = grants.all().length
next = 'once'
ok(await ask(B) === 'once' && prompts === 3, 'action once prompts and returns once')
ok(grants.all().length === before && grants.covering(B) === null, 'once is never stored')
next = 'deny'
ok(await ask(B) === 'deny' && prompts === 4, 'next action request prompts again after once (next=deny)')
// wrong-kind answers are coerced to deny
next = 'untilClose'
ok(await ask(B) === 'deny' && grants.covering(B) === null, 'untilClose answer on action prompt coerced to deny, nothing stored')
next = 'once'
ok(await ask(A) === 'deny' && grants.covering(A) === null, 'once answer on ongoing prompt coerced to deny, nothing stored')
next = undefined
ok(await ask(A) === 'deny', 'Escape/undefined answer = deny')
next = 'session'
ok(await ask(A) === 'deny' && grants.covering(A) === null, "legacy 'session' answer = deny, nothing stored")
// always
next = 'always'; const p0 = prompts
ok(await ask(A) === 'always' && prompts === p0 + 1, 'always prompts once')
ok(grants.covering(A)?.until === 'always', 'always stored with until always')
ok(await ask(A) === 'always' && prompts === p0 + 1, 'covered always request does not prompt')
grants.appClosed('browser')
ok(grants.covering(A)?.until === 'always', 'always survives appClosed')
ok(await ask({ ...A, kind: 'action' }) === 'always' && prompts === p0 + 1, 'same key with kind action is covered by the always grant (kind not part of key)')
// broader / different key prompts again
next = 'deny'
for (const [k, v] of [['resource', 'andere.example'], ['right', 'readWrite'], ['scope', 'all'], ['app', 'notities']]) {
  const p = prompts
  const r = await ask({ ...A, [k]: v })
  ok(r === 'deny' && prompts === p + 1, `different ${k} (${v}) prompts again, no broadening`)
}
// list and revoke
const all = grants.all()
ok(all.some((g) => g.app === 'browser' && g.resource === A.resource && g.until === 'always'), 'grants.all lists the always grant')
all[0].until = 'tampered'
ok(grants.all()[0].until !== 'tampered', 'grants.all returns copies (no external mutation)')
const revoked = []
bus.on('grant:revoked', (g) => revoked.push(g))
const r = grants.revoke({ app: A.app, resource: A.resource, right: A.right, scope: A.scope })
ok(r && revoked.length === 1 && grants.covering(A) === null, 'revoke by key removes the grant and emits grant:revoked')
ok(grants.revoke(A) === null, 'revoking a missing key returns null')
next = 'deny'; const p1 = prompts
ok(await ask(A) === 'deny' && prompts === p1 + 1, 'after revoke the request prompts again')
// validation
await assert.rejects(ask({ ...A, kind: 'session' })); n++; log.push('PASS unknown kind rejected')
await assert.rejects(ask({ ...A, scope: '' })); n++; log.push('PASS missing key field rejected')
// seed normalisation
ok(seed.every((g) => ['app', 'resource', 'right', 'scope', 'until'].every((k) => g[k])), 'seeded grants normalised to the four-field key plus until')
ok(seed.every((g) => g.until === 'close' || g.until === 'always'), 'seeded grants only close|always (never once)')
// files keep no grants copy
ok(C.files.list().every((f) => !('grants' in f)), 'files carry no stale grants field')
// ---------- repair-1: destination is part of the key (blocker), covered result per kind (minor) ----------
const S1 = { ...B, to: 'tuinplanner.example' }, S2 = { ...B, to: 'andere-dienst.example' }
await assert.rejects(ask({ ...B, to: undefined }), /destination/); n++; log.push('PASS send-copy without a destination is rejected')
next = 'always'; let p2 = prompts
ok(await ask(S1) === 'always' && prompts === p2 + 1, 'always for send-copy to tuinplanner.example prompts once')
ok(grants.covering(S1)?.to === 'tuinplanner.example', 'stored grant carries the destination')
ok(await ask(S1) === 'always' && prompts === p2 + 1, 'same destination is covered without a prompt')
next = 'deny'
ok(grants.covering(S2) === null, 'grant for tuinplanner.example does not cover andere-dienst.example')
ok(await ask(S2) === 'deny' && prompts === p2 + 2, 'a second destination prompts again (no broadening)')
next = 'always'
ok(await ask(S2) === 'always' && prompts === p2 + 3, 'always for the second destination prompts and stores')
ok(grants.all().filter((g) => g.resource === 'tuinplan' && g.right === 'send-copy').length === 2, 'two send-copy grants, one per destination')
grants.revoke(S1)
ok(grants.covering(S1) === null && grants.covering(S2)?.to === 'andere-dienst.example', 'revoking one destination leaves the other')
ok(destination(S2) === 'andere-dienst.example' && destination(A) === 'tuinplanner.example' && destination({ right: 'read', resource: 'tuin' }) === null, 'destination(): send-copy -> to, connect -> resource, read -> null')
// a covered request returns the stored duration regardless of kind (documented in DESIGN-SYSTEM.md §11.1)
next = 'untilClose'; const C1 = { kind: 'ongoing', app: 'browser', resource: 'tuinplanner.example', right: 'connect', scope: 'domain' }
grants.revoke(C1); p2 = prompts
ok(await ask(C1) === 'untilClose' && prompts === p2 + 1, 'ongoing untilClose stored for connect')
ok(grants.covering({ ...C1, to: 'x.example' })?.resource === C1.resource, 'to is ignored for non-sending rights (the connect key is the domain resource)')
ok(await ask({ ...C1, kind: 'action' }) === 'untilClose' && prompts === p2 + 1, "covered action-kind request returns the stored 'untilClose' (stored duration, not remapped)")
log.push('events: ' + events.join(','))
log.push(`TOTAL ${n} assertions passed`)
console.log(log.join('\n'))
