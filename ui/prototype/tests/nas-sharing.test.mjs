import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { createFixtureState, NAS_IDENTITY } from '../fixtures.js';
import { reducer, resumeState } from '../state.js';
import * as broker from '../broker.js';
import { createNasSharingState, changeNasSharing, receivedNasAccess, guestMetadata, FAMILY_NAS_IDENTITY, INVITE_LIFETIME_MS } from '../nas-sharing.js';

const share = (state, action) => reducer(state, { type: 'nasSharing', action });
function pairedOwner() {
  let state = reducer(createFixtureState(), { type: 'pairNas' });
  return reducer(state, { type: 'confirmNas', confirmed: true, identity: NAS_IDENTITY });
}
function received() {
  let state = share(createFixtureState(), { type: 'exampleInvitation', now: 1000 });
  return share(state, { type: 'confirmInvitation', invitationId: state.nasSharing.invitation.id,
    confirmed: true, lossAccepted: true, identity: FAMILY_NAS_IDENTITY, now: 1001 });
}
function hosting() {
  let state = share(pairedOwner(), { type: 'enableHosting', confirmed: true });
  return share(state, { type: 'inviteGuest', label: 'Voorbeeldgast', quotaGiB: 20 });
}

test('NAS-UI-01: discovery alone grants no connection; confirmation and identity are required', () => {
  const initial = createNasSharingState();
  assert.equal(receivedNasAccess(initial, { identity: FAMILY_NAS_IDENTITY, epoch: 0 }).ok, false);
  const pending = changeNasSharing(initial, { type: 'exampleInvitation', now: 1000 }).value;
  const confirm = { type: 'confirmInvitation', invitationId: pending.invitation.id, now: 1001,
    identity: FAMILY_NAS_IDENTITY, confirmed: true, lossAccepted: true };
  for (const bad of [{ confirmed: false }, { lossAccepted: false }, { identity: 'different-nas' }, { invitationId: 'other' }]) {
    assert.equal(changeNasSharing(pending, { ...confirm, ...bad }).ok, false);
  }
  const paired = changeNasSharing(pending, confirm).value;
  assert.equal(paired.received.state, 'paired');
  assert.equal(paired.received.internetIdentity, null);
  assert.equal(receivedNasAccess(paired, { identity: FAMILY_NAS_IDENTITY, epoch: paired.received.epoch }).ok, true);
});

test('NAS-UI-02: invitations expire and cannot be replayed after acceptance', () => {
  const pending = changeNasSharing(createNasSharingState(), { type: 'exampleInvitation', now: 1000 }).value;
  const confirm = { type: 'confirmInvitation', invitationId: pending.invitation.id, now: 1001,
    identity: FAMILY_NAS_IDENTITY, confirmed: true, lossAccepted: true };
  assert.equal(changeNasSharing(pending, { ...confirm, now: 1000 + INVITE_LIFETIME_MS }).code, 'INVITE_EXPIRED');
  assert.equal(changeNasSharing(changeNasSharing(pending, confirm).value, confirm).code, 'INVITE_EXPIRED');
});

test('NAS-UI-03: remote opt-in is specific to the confirmed peer; identity failure revokes it', () => {
  let state = received();
  const epoch = state.nasSharing.received.epoch;
  const request = { identity: FAMILY_NAS_IDENTITY, network: 'internet', epoch };
  assert.equal(receivedNasAccess(state.nasSharing, request).code, 'REMOTE_NOT_ALLOWED');
  state = share(state, { type: 'internet', identity: FAMILY_NAS_IDENTITY, allowed: true });
  assert.equal(receivedNasAccess(state.nasSharing, request).ok, true);
  assert.equal(receivedNasAccess(state.nasSharing, { ...request, identity: 'other' }).code, 'IDENTITY_MISMATCH');
  assert.equal(receivedNasAccess(state.nasSharing, { ...request, network: 'relay' }).code, 'DENIED');
  state = share(state, { type: 'receivedCondition', condition: 'identityMismatch' });
  assert.equal(state.nasSharing.received.internetIdentity, null);
  assert.equal(receivedNasAccess(state.nasSharing, request).ok, false);
  assert.equal(changeNasSharing(state.nasSharing, { type: 'receivedCondition', condition: 'online' }).ok, false);
});

test('NAS-UI-04: owner NAS can reconnect remotely only after explicit bound permission', () => {
  let state = pairedOwner();
  assert.equal(broker.connectNasInternet(state, NAS_IDENTITY).ok, false);
  state = reducer(state, { type: 'nasInternet', identity: NAS_IDENTITY, allowed: true });
  state = reducer(state, { type: 'nasCondition', condition: 'networkChanged' });
  assert.equal(state.nas.state, 'networkChanged');
  assert.equal(broker.connectNasInternet(state, 'other').ok, false);
  state = reducer(state, { type: 'nasInternetConnect', identity: NAS_IDENTITY });
  assert.equal(state.nas.state, 'online');
  assert.equal(state.nas.network, 'internet');
  state = reducer(state, { type: 'nasInternet', identity: NAS_IDENTITY, allowed: false });
  assert.equal(state.nas.state, 'offline');
  assert.equal(broker.setNasCondition(state, 'online').ok, false);
  assert.equal(broker.connectNasInternet(state, NAS_IDENTITY).ok, false);
});

test('NAS-UI-04b: remote permission can be revoked while the known NAS is offline', () => {
  let state = received();
  state = share(state, { type: 'internet', identity: FAMILY_NAS_IDENTITY, allowed: true });
  state = share(state, { type: 'receivedCondition', condition: 'offline' });
  state = share(state, { type: 'internet', identity: FAMILY_NAS_IDENTITY, allowed: false });
  assert.equal(state.nasSharing.received.internetIdentity, null);
  state = share(state, { type: 'receivedCondition', condition: 'online' });
  assert.equal(receivedNasAccess(state.nasSharing, { identity: FAMILY_NAS_IDENTITY, epoch: state.nasSharing.received.epoch, network: 'internet' }).ok, false);
});

test('NAS-UI-04c: verified route observations switch seamlessly using existing consent and retire local admin', () => {
  let state = hosting();
  const guest = state.nasSharing.hosting.guests[0];
  state = share(state, { type: 'requestRemoval', id: guest.id });
  state = reducer(state, { type: 'nasInternet', identity: NAS_IDENTITY, allowed: true });
  const observation = { type: 'observeNasRoute', identity: NAS_IDENTITY, reachable: true, verified: true };
  state = reducer(state, { ...observation, network: 'internet' });
  assert.equal(state.nas.state, 'online');
  assert.equal(state.nas.network, 'internet');
  assert.equal(state.nas.internetIdentity, NAS_IDENTITY);
  assert.equal(state.nasSharing.hosting.adminRole, 'inactive');
  assert.equal(state.nasSharing.pendingRemoval, null);
  state = reducer(state, { ...observation, network: 'lan', trustedLan: true });
  assert.equal(state.nas.state, 'online');
  assert.equal(state.nas.network, 'lan');
  assert.equal(state.nasSharing.hosting.adminRole, 'inactive');
  assert.equal(state.nas.internetIdentity, NAS_IDENTITY);
  assert.equal(share(state, { type: 'removeGuest', id: guest.id, confirmed: true }).nasSharing.hosting.guests[0].state, 'invited');
});

test('NAS-UI-04d: route observations cannot invent consent, trust or reachability; failures preserve local editing', () => {
  const observation = { type: 'observeNasRoute', network: 'internet', identity: NAS_IDENTITY, reachable: true, verified: true };
  assert.equal(reducer(createFixtureState(), observation).nas.state, 'unpaired');
  const noPermission = reducer(pairedOwner(), observation);
  assert.equal(noPermission.nas.state, 'offline');
  assert.equal(noPermission.nas.internetIdentity, null);
  assert.equal(reducer(noPermission, { ...observation, network: 'lan', trustedLan: true }).nas.state, 'online');

  let approved = hosting();
  approved = share(approved, { type: 'requestRemoval', id: approved.nasSharing.hosting.guests[0].id });
  approved = reducer(approved, { type: 'nasInternet', identity: NAS_IDENTITY, allowed: true });
  for (const change of [{ reachable: false }, { verified: false }, { network: 'relay' }, { network: 'lan', trustedLan: false }, { identity: 'other-nas' }]) {
    let failed = reducer(approved, { ...observation, ...change });
    assert.notEqual(failed.nas.state, 'online');
    assert.equal(failed.nasSharing.hosting.adminRole, 'inactive');
    assert.equal(failed.nasSharing.pendingRemoval, null);
    if (change.identity) {
      assert.equal(failed.nas.identity, null);
      assert.equal(failed.nas.internetIdentity, null);
      assert.equal(reducer(failed, observation).nas.state, 'identityMismatch');
    }
    failed = reducer(failed, { type: 'openObject', id: 'obj-001' });
    failed = reducer(failed, { type: 'requestEdit', toolId: 'notities' });
    failed = reducer(failed, { type: 'grantSession' });
    failed = reducer(failed, { type: 'updateDraft', text: 'Lokaal doorwerken', cursor: 0 });
    failed = reducer(failed, { type: 'saveDraft' });
    assert.equal(failed.objects.find((object) => object.id === 'obj-001').versions.at(-1).text, 'Lokaal doorwerken');
    assert.match(failed.message.text, /Op dit apparaat opgeslagen/);
  }
  const revoked = reducer(approved, { type: 'nasInternet', identity: NAS_IDENTITY, allowed: false });
  assert.equal(reducer(revoked, observation).nas.state, 'offline');
  assert.equal(reducer(revoked, observation).nas.internetIdentity, null);
});

test('NAS-UI-05: offering space requires own NAS, explicit setup, and available quota', () => {
  let state = share(createFixtureState(), { type: 'enableHosting', confirmed: true });
  assert.equal(state.nasSharing.hosting.enabled, false);
  state = hosting();
  assert.equal(state.nasSharing.hosting.guests.length, 1);
  for (const quotaGiB of [0, -1, 1.5, NaN, 81]) {
    const next = share(state, { type: 'inviteGuest', label: 'Extra gast', quotaGiB });
    assert.equal(next.nasSharing.hosting.guests.length, 1);
  }
  state = share(state, { type: 'inviteGuest', label: 'Extra gast', quotaGiB: 80 });
  assert.equal(state.nasSharing.hosting.guests.length, 2);
});

test('NAS-UI-05b: data pairing grants no admin role and internet routes cannot enable, invite or remove', () => {
  let dataOnly = pairedOwner();
  assert.equal(dataOnly.nasSharing.hosting.adminRole, 'inactive');
  assert.equal(share(dataOnly, { type: 'inviteGuest', label: 'No admin', quotaGiB: 10 }).nasSharing.hosting.guests.length, 0);
  dataOnly = reducer(dataOnly, { type: 'nasInternet', identity: NAS_IDENTITY, allowed: true });
  dataOnly = reducer(dataOnly, { type: 'nasInternetConnect', identity: NAS_IDENTITY });
  dataOnly = share(dataOnly, { type: 'enableHosting', confirmed: true });
  assert.equal(dataOnly.nasSharing.hosting.enabled, false);
  assert.equal(dataOnly.nasSharing.hosting.adminRole, 'inactive');

  let state = hosting();
  const guest = state.nasSharing.hosting.guests[0];
  state = share(state, { type: 'requestRemoval', id: guest.id });
  assert.equal(changeNasSharing(state.nasSharing, { type: 'removeGuest', id: guest.id, confirmed: true }, { localAdminAvailable: false }).ok, false);
  state = reducer(state, { type: 'nasInternet', identity: NAS_IDENTITY, allowed: true });
  state = reducer(state, { type: 'nasInternetConnect', identity: NAS_IDENTITY });
  assert.equal(state.nasSharing.hosting.adminRole, 'inactive');
  assert.equal(state.nasSharing.pendingRemoval, null);
  const before = structuredClone(state.nasSharing.hosting.guests);
  for (const action of [
    { type: 'enableHosting', confirmed: true },
    { type: 'inviteGuest', label: 'Remote guest', quotaGiB: 10 },
    { type: 'requestRemoval', id: guest.id },
    { type: 'removeGuest', id: guest.id, confirmed: true },
  ]) {
    state = share(state, action);
    assert.equal(state.nasSharing.hosting.adminRole, 'inactive');
    assert.deepEqual(state.nasSharing.hosting.guests, before);
  }
});

test('NAS-UI-06: admin removal needs admin confirmation, never guest consent or a decryption key', () => {
  let state = hosting();
  const guest = state.nasSharing.hosting.guests[0];
  const documents = structuredClone(state.objects);
  assert.equal(share(state, { type: 'removeGuest', id: guest.id, confirmed: true }).nasSharing.hosting.guests[0].state, 'invited');
  state = share(state, { type: 'requestRemoval', id: guest.id });
  const declined = share(state, { type: 'removeGuest', id: guest.id, confirmed: false });
  assert.equal(declined.nasSharing.hosting.guests[0].state, 'invited');
  state = share(state, { type: 'removeGuest', id: guest.id, confirmed: true, tenantConsent: false });
  assert.equal(state.nasSharing.hosting.guests[0].state, 'removed');
  assert.equal(state.nasSharing.hosting.guests[0].quotaGiB, 0);
  assert.equal(state.nasSharing.hosting.guests[0].epoch, guest.epoch + 1);
  assert.deepEqual(state.objects, documents);
});

test('NAS-UI-07: stale upload completions cannot reopen evicted space; new invite gets a different id', () => {
  let state = hosting();
  const guest = state.nasSharing.hosting.guests[0];
  state = share(state, { type: 'guestUsage', id: guest.id, epoch: guest.epoch, usedGiB: 5 });
  assert.equal(state.nasSharing.hosting.guests[0].usedGiB, 5);
  state = share(state, { type: 'requestRemoval', id: guest.id });
  state = share(state, { type: 'removeGuest', id: guest.id, confirmed: true });
  state = share(state, { type: 'guestUsage', id: guest.id, epoch: guest.epoch, usedGiB: 5 });
  assert.equal(state.nasSharing.hosting.guests[0].state, 'removed');
  assert.equal(state.nasSharing.hosting.guests[0].usedGiB, 0);
  state = share(state, { type: 'inviteGuest', label: 'Dezelfde gast', quotaGiB: 100 });
  assert.notEqual(state.nasSharing.hosting.guests[1].id, guest.id);
  assert.equal(state.nasSharing.hosting.guests[0].state, 'removed');
});

test('NAS-UI-08: admin sees only allocation metadata, never guest document fields', () => {
  const state = hosting();
  Object.assign(state.nasSharing.hosting.guests[0], { documents: ['private.txt'], key: 'secret', plaintext: 'private content' });
  const metadata = guestMetadata(state.nasSharing);
  assert.deepEqual(Object.keys(metadata[0]).sort(), ['epoch', 'id', 'label', 'quotaGiB', 'state', 'usedGiB']);
  assert.doesNotMatch(JSON.stringify(metadata), /private|secret/);
});

test('NAS-UI-09: receiving revocation blocks stale requests and preserves local documents and AI boundary', () => {
  let state = received();
  const before = structuredClone(state.objects);
  const epoch = state.nasSharing.received.epoch;
  state = share(state, { type: 'receivedCondition', condition: 'revoked' });
  assert.equal(receivedNasAccess(state.nasSharing, { identity: FAMILY_NAS_IDENTITY, epoch }).ok, false);
  assert.deepEqual(state.objects, before);
  assert.equal(state.ai.availability, 'ready');
  assert.equal(state.grants.length, 0);
});

test('NAS-UI-10: refresh expires internet and admin permissions, preserving bounded deletion history', () => {
  let state = hosting();
  const guest = state.nasSharing.hosting.guests[0];
  state = share(state, { type: 'requestRemoval', id: guest.id });
  state = share(state, { type: 'removeGuest', id: guest.id, confirmed: true });
  state = reducer(state, { type: 'nasInternet', identity: NAS_IDENTITY, allowed: true });
  state.nasSharing.received = { state: 'paired', identity: FAMILY_NAS_IDENTITY, internetIdentity: FAMILY_NAS_IDENTITY, epoch: 1, quotaGiB: 1000 };
  const resumed = resumeState(JSON.stringify(state));
  assert.equal(resumed.nas.internetIdentity, null);
  assert.equal(resumed.nasSharing.received.state, 'none');
  assert.equal(resumed.nasSharing.hosting.enabled, false);
  assert.equal(resumed.nasSharing.hosting.guests[0].state, 'removed');
  assert.equal(resumed.nasSharing.hosting.guests[0].quotaGiB, 0);
});

test('NAS-UI-11: UI mock stays network-free and retains explicit simulation labels', async () => {
  const source = await readFile(new URL('../nas-sharing.js', import.meta.url), 'utf8');
  assert.doesNotMatch(source, /\bfetch\s*\(|\bXMLHttpRequest\b|\bWebSocket\b|\bsendBeacon\s*\(|\bEventSource\b/);
  const app = await readFile(new URL('../app.js', import.meta.url), 'utf8');
  assert.match(app, /Simulatie: geen NAS-verbinding, encryptie of echte verwijdering/);
  assert.match(app, /Trek in en wis NAS-kopie/);
});
