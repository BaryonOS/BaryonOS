import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { createFixtureState, NAS_IDENTITY } from '../fixtures.js';
import { reducer, resumeState, selectedObject, activeGrants } from '../state.js';
import * as broker from '../broker.js';

function editable() {
  let state = reducer(createFixtureState(), { type: 'openObject', id: 'obj-001' });
  state = reducer(state, { type: 'requestEdit', toolId: 'notities' });
  return reducer(state, { type: 'grantSession' });
}

test('P2-01: onboarding steps and local profile complete without account', () => {
  let state = createFixtureState();
  state = reducer(state, { type: 'onboardingStep', step: 2 });
  state = reducer(state, { type: 'onboardingStep', step: 1 });
  assert.equal(state.onboarding.step, 1);
  state = reducer(state, { type: 'finishOnboarding' });
  assert.equal(state.onboarding.completed, true);
  assert.equal(activeGrants(state).length, 0);
});

test('P2-01: search, collection and empty fixture results', () => {
  const state = createFixtureState();
  assert.deepEqual(broker.listObjects(state, { query: 'TUIN' }).value.map((item) => item.id), ['obj-001']);
  assert.equal(broker.listObjects(state, { collection: 'Huis' }).value.length, 2);
  assert.equal(broker.listObjects(state, { collection: 'Archief' }).value.length, 0);
  assert.equal(broker.listObjects(state, { query: 'bestaat niet' }).value.length, 0);
});

test('P2-01: search also finds the visible document type within its collection', () => {
  const state = createFixtureState();
  assert.deepEqual(broker.listObjects(state, { query: 'AfBeElDiNg' }).value.map((item) => item.id), ['obj-002']);
  assert.deepEqual(broker.listObjects(state, { query: 'tekst' }).value.map((item) => item.id), ['obj-001', 'obj-003']);
  assert.deepEqual(broker.listObjects(state, { query: 'afbeelding', collection: 'Vandaag' }).value, []);
});

test('P2-02: requesting and denying access never modifies content or grants', () => {
  let state = reducer(createFixtureState(), { type: 'openObject', id: 'obj-001' });
  const original = structuredClone(state.objects);
  state = reducer(state, { type: 'requestEdit', toolId: 'notities' });
  assert.equal(activeGrants(state).length, 0);
  state = reducer(state, { type: 'denyGrant' });
  assert.deepEqual(state.objects, original);
  assert.equal(broker.saveRevision(state, { objectId: 'obj-001', grantId: 'invented', baseRevision: 3, text: 'wrong' }).code, 'DENIED');
});

test('P2-02: explicit grant stays scoped to one document, session and operations', () => {
  const state = editable();
  const grant = activeGrants(state)[0];
  assert.deepEqual(grant.objectIds, ['obj-001']);
  assert.equal(grant.internet, false);
  assert.equal(grant.expiresOn, 'objectClose');
  assert.equal(broker.authorize(state, grant.id, 'obj-003', 'write').code, 'DENIED');
  assert.equal(broker.authorize(state, grant.id, 'obj-001', 'admin').code, 'DENIED');
});

test('P2-02: forged broad and unknown requests fail closed', () => {
  const state = editable();
  for (const request of [
    null,
    { toolId: 'notities', objectIds: ['obj-001', 'obj-003'], operations: ['write'], lease: 'session', internet: false },
    { toolId: 'notities', objectIds: ['obj-001'], operations: ['admin'], lease: 'session', internet: false },
    { toolId: 'notities', objectIds: ['obj-001'], operations: ['read'], lease: 'permanent', internet: false },
    { toolId: 'notities', objectIds: ['obj-001'], operations: ['read'], lease: 'session', internet: 'example.com' },
  ]) assert.equal(broker.requestGrant(state, request).code, 'DENIED');
});

test('P2-02: revocation blocks subsequent saves and preserves local draft', () => {
  let state = editable();
  state = reducer(state, { type: 'updateDraft', text: 'Concept blijft bestaan', cursor: 4 });
  const grantId = state.activeGrantId;
  state = reducer(state, { type: 'revokeGrant', id: grantId });
  assert.equal(broker.saveRevision(state, { objectId: 'obj-001', grantId, baseRevision: 3, text: 'wrong' }).code, 'DENIED');
  state = reducer(state, { type: 'saveDraft' });
  assert.equal(selectedObject(state).revision, 3);
  assert.equal(state.draft.text, 'Concept blijft bestaan');
});

test('P2-02: closing and reopening expires the old grant, resumes draft only', () => {
  let state = editable();
  state = reducer(state, { type: 'updateDraft', text: 'Onderbroken werk', cursor: 5 });
  const old = state.activeGrantId;
  state = reducer(state, { type: 'closeObject' });
  state = reducer(state, { type: 'openObject', id: 'obj-001' });
  assert.equal(state.mode, 'read');
  assert.equal(state.draft.cursor, 5);
  assert.equal(broker.authorize(state, old, 'obj-001', 'write').code, 'DENIED');
  state = reducer(state, { type: 'requestEdit', toolId: 'notities' });
  state = reducer(state, { type: 'grantSession' });
  assert.equal(state.draft.text, 'Onderbroken werk');
  assert.notEqual(state.activeGrantId, old);
});

test('P2-02: refresh resumes fixtures but never an active lease', () => {
  const resumed = resumeState(JSON.stringify(editable()));
  assert.equal(activeGrants(resumed).length, 0);
  assert.equal(resumed.activeGrantId, null);
  assert.equal(resumed.selectedObjectId, null);
  assert.equal(resumed.mode, 'read');
  assert.deepEqual(resumeState('{invalid'), createFixtureState());
});

test('stored display values cannot inject version markup or alter tool permissions', () => {
  const stored = editable();
  stored.tools[0].name = '<img src=x onerror=alert(1)>';
  stored.tools[0].types = ['text', 'image'];
  const safe = resumeState(JSON.stringify(stored));
  assert.equal(safe.tools[0].name, 'Notities');
  assert.deepEqual(safe.tools[0].types, ['text']);
  stored.objects[0].revision = '<svg onload=alert(1)>';
  stored.objects[0].versions.at(-1).id = stored.objects[0].revision;
  assert.deepEqual(resumeState(JSON.stringify(stored)), createFixtureState());
});

test('P2-02: fixture installation grants zero rights and invalid signature blocks it', () => {
  let state = reducer(createFixtureState(), { type: 'invalidPackage', value: true });
  state = reducer(state, { type: 'installFixture', id: 'beeldbewerker' });
  assert.equal(state.install.state, 'error');
  assert.equal(state.tools.find((tool) => tool.id === 'beeldbewerker').installed, false);
  state = reducer(state, { type: 'invalidPackage', value: false });
  state = reducer(state, { type: 'installFixture', id: 'beeldbewerker' });
  assert.equal(state.tools.find((tool) => tool.id === 'beeldbewerker').installed, true);
  assert.equal(state.grants.length, 0);
});

test('P2-02: Webvenster requests explicit destination but receives no write grant', () => {
  let state = reducer(createFixtureState(), { type: 'openObject', id: 'obj-001' });
  state = reducer(state, { type: 'requestEdit', toolId: 'webvenster' });
  assert.equal(state.grantRequest.internet, 'demo.invalid');
  state = reducer(state, { type: 'grantSession' });
  assert.equal(broker.authorize(state, state.activeGrantId, 'obj-001', 'read').ok, true);
  assert.equal(broker.authorize(state, state.activeGrantId, 'obj-001', 'write').code, 'DENIED');
});

test('P2-02: granting Webvenster preserves a valid editor grant', () => {
  let state = editable();
  state = reducer(state, { type: 'updateDraft', text: 'Eerste concept', cursor: 14 });
  const editorGrantId = state.activeGrantId;
  state = reducer(state, { type: 'requestEdit', toolId: 'webvenster' });
  state = reducer(state, { type: 'grantSession' });

  assert.equal(state.mode, 'edit');
  assert.equal(state.activeGrantId, editorGrantId);
  const webGrant = activeGrants(state).find((grant) => grant.toolId === 'webvenster');
  assert.equal(broker.authorize(state, webGrant.id, 'obj-001', 'read').ok, true);
  assert.equal(broker.authorize(state, webGrant.id, 'obj-001', 'write').code, 'DENIED');

  state = reducer(state, { type: 'updateDraft', text: 'Bijgewerkt concept', cursor: 18 });
  state = reducer(state, { type: 'saveDraft' });
  assert.equal(selectedObject(state).revision, 4);
  assert.equal(selectedObject(state).versions.at(-1).text, 'Bijgewerkt concept');
});

test('P2-03: all NAS failure states keep local save working without a sync claim', () => {
  for (const condition of ['offline', 'networkChanged', 'identityMismatch']) {
    let state = reducer(editable(), { type: 'nasCondition', condition });
    state = reducer(state, { type: 'updateDraft', text: 'Lokaal bewaard', cursor: 0 });
    state = reducer(state, { type: 'saveDraft' });
    assert.equal(selectedObject(state).revision, 4);
    assert.equal(selectedObject(state).nasState, 'pending');
    assert.match(state.message.text, /Op dit apparaat opgeslagen/);
  }
});

test('P2-03: NAS identity cannot be trusted by a network-state toggle', () => {
  let state = reducer(createFixtureState(), { type: 'pairNas' });
  assert.equal(broker.pairNas(state, { confirmed: true, identity: 'attacker' }).code, 'IDENTITY_MISMATCH');
  state = reducer(state, { type: 'confirmNas', confirmed: true, identity: NAS_IDENTITY });
  assert.equal(state.nas.state, 'online');
  for (const condition of ['identityMismatch', 'networkChanged']) {
    let failed = reducer(state, { type: 'nasCondition', condition });
    assert.equal(failed.nas.identity, null);
    failed = reducer(failed, { type: 'nasCondition', condition: 'offline' });
    assert.equal(failed.nas.identity, null);
    assert.equal(broker.setNasCondition(failed, 'online').ok, false);
    failed = reducer(failed, { type: 'pairNas' });
    assert.equal(failed.nas.state, 'verifying');
    failed = reducer(failed, { type: 'confirmNas', confirmed: true, identity: NAS_IDENTITY });
    assert.equal(failed.nas.state, 'online');
    assert.equal(failed.nas.identity, NAS_IDENTITY);
  }
});

test('P2-04: missing local model or memory never produces a summary', () => {
  for (const [condition, code] of [['missingModel', 'MODEL_MISSING'], ['lowMemory', 'RESOURCE_LIMIT']]) {
    let state = reducer(createFixtureState(), { type: 'aiCondition', condition });
    assert.equal(broker.summarizeFixture(state, ['obj-001']).code, code);
    state = reducer(state, { type: 'startSummary' });
    assert.notEqual(state.ai.state, 'running');
    assert.equal(state.ai.response, '');
  }
});

test('P2-04: summary uses selected material only', () => {
  const state = createFixtureState();
  const response = broker.summarizeFixture(state, ['obj-003']);
  assert.equal(response.ok, true);
  assert.match(response.value, /Boodschappen/);
  assert.doesNotMatch(response.value, /Tuinplan|rozemarijn|schuur/i);
  assert.equal(broker.summarizeFixture(state, []).code, 'DENIED');
  assert.equal(broker.summarizeFixture(state, ['not-found']).code, 'NOT_FOUND');
});

test('P2-04: cancellation and changed context reject stale asynchronous completion', () => {
  let state = reducer(createFixtureState(), { type: 'startSummary' });
  const runId = state.ai.runId;
  state = reducer(state, { type: 'cancelSummary' });
  state = reducer(state, { type: 'finishSummary', runId });
  assert.equal(state.ai.state, 'cancelled');
  assert.equal(state.ai.response, '');
  state = reducer(state, { type: 'startSummary' });
  const secondId = state.ai.runId;
  state = reducer(state, { type: 'aiContext', id: 'obj-003', selected: true });
  state = reducer(state, { type: 'finishSummary', runId: secondId });
  assert.equal(state.ai.response, '');
});

test('P2-05: restore first previews, then appends without erasing history', () => {
  let state = reducer(createFixtureState(), { type: 'openObject', id: 'obj-001' });
  const versions = structuredClone(selectedObject(state).versions);
  assert.equal(broker.restoreAsNewRevision(state, { objectId: 'obj-001', revision: 1, confirmed: false, baseRevision: 3 }).code, 'DENIED');
  state = reducer(state, { type: 'previewRestore', revision: 1 });
  assert.equal(selectedObject(state).revision, 3);
  state = reducer(state, { type: 'restoreRevision', baseRevision: 3 });
  assert.equal(selectedObject(state).revision, 4);
  assert.deepEqual(selectedObject(state).versions.slice(0, 3), versions);
  assert.equal(selectedObject(state).versions[3].text, versions[0].text);
});

test('stale revision and oversized content cannot overwrite saved material', () => {
  const state = editable();
  const request = { objectId: 'obj-001', grantId: state.activeGrantId, baseRevision: 2, text: 'wrong' };
  assert.equal(broker.saveRevision(state, request).code, 'STALE_REVISION');
  assert.equal(broker.saveRevision(state, { ...request, baseRevision: 3, text: 'x'.repeat(20001) }).code, 'RESOURCE_LIMIT');
});

test('unknown object route never keeps a previous object or lease open', () => {
  const state = reducer(editable(), { type: 'openObject', id: 'missing' });
  assert.equal(state.selectedObjectId, null);
  assert.equal(activeGrants(state).length, 0);
});

test('reducer does not mutate its caller state', () => {
  const before = editable();
  const snapshot = structuredClone(before);
  const after = reducer(before, { type: 'updateDraft', text: 'nieuw', cursor: 2 });
  assert.deepEqual(before, snapshot);
  assert.notEqual(after.draft.text, before.draft.text);
});

test('draft keystrokes preserve saved history without deep-cloning it', () => {
  const before = editable();
  const beforeText = before.draft.text;
  const after = reducer(before, { type: 'updateDraft', text: 'nieuwe tekst', cursor: 6 });
  assert.notEqual(after, before);
  assert.notEqual(after.draft, before.draft);
  assert.equal(after.objects, before.objects);
  assert.equal(after.objects[0].versions, before.objects[0].versions);
  assert.equal(before.draft.text, beforeText);
  assert.equal(after.draft.text, 'nieuwe tekst');
});

test('P2-08: CSP blocks outbound connections and app code has no networking APIs', async () => {
  const html = await readFile(new URL('../index.html', import.meta.url), 'utf8');
  assert.match(html, /connect-src 'none'/);
  assert.match(html, /font-src 'none'/);
  assert.match(html, /form-action 'none'/);
  for (const name of ['app.js', 'state.js', 'broker.js', 'fixtures.js']) {
    const source = await readFile(new URL(`../${name}`, import.meta.url), 'utf8');
    assert.doesNotMatch(source, /\bfetch\s*\(|\bXMLHttpRequest\b|\bWebSocket\b|\bsendBeacon\s*\(|\bEventSource\b|\bserviceWorker\b/);
  }
});

test('P2-06: specified text and control color pairs meet their contrast thresholds', async () => {
  const css = await readFile(new URL('../tokens.css', import.meta.url), 'utf8');
  function value(name) { return css.match(new RegExp(`--${name}:\\s*(#[a-fA-F0-9]{6})`))[1]; }
  function luminance(color) {
    const rgb = [1, 3, 5].map((index) => parseInt(color.slice(index, index + 2), 16) / 255).map((v) => v <= .04045 ? v / 12.92 : ((v + .055) / 1.055) ** 2.4);
    return rgb[0] * .2126 + rgb[1] * .7152 + rgb[2] * .0722;
  }
  function ratio(a, b) { const x = luminance(a); const y = luminance(b); return (Math.max(x, y) + .05) / (Math.min(x, y) + .05); }
  for (const [foreground, background] of [['text', 'background'], ['muted', 'background'], ['surface', 'accent'], ['danger', 'surface'], ['warning', 'warning-bg']]) {
    assert.ok(ratio(value(foreground), value(background)) >= 4.5, `${foreground}/${background}`);
  }
  assert.ok(ratio(value('border'), value('surface')) >= 3);
});
