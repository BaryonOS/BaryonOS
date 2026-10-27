import test from 'node:test';
import assert from 'node:assert/strict';
import { createPolicyEngine } from '../policy.mjs';

const peerIdentity = 'a'.repeat(64);
const otherPeer = 'b'.repeat(64);
const objectId = '1'.repeat(32);
function setup(kind = 'nas') {
  const engine = createPolicyEngine(); const app = engine.authority.installApp({ appId: 'synthetic.app', kind });
  return { ...engine, app };
}

test('connection preflight requires existing grants without minting authenticated transport', () => {
  const { authority, check, authorizeNasConnection, app } = setup();
  const request = { app, peerIdentity, networkScope: 'lan', localNetworkId: 'home' };
  assert.equal(authorizeNasConnection(request).reason, 'UNPINNED_PEER');
  authority.pinNasPeer(app, { peerIdentity, localNetworkId: 'home' });
  const permission = authorizeNasConnection(request);
  assert.equal(permission.allowed, true);
  assert.equal(check({ app, resource: 'nas', transport: permission }).reason, 'UNVERIFIED_TRANSPORT');
  assert.equal(check({ app, resource: 'nas', transport: request }).reason, 'UNVERIFIED_TRANSPORT');
  assert.equal(authorizeNasConnection({ ...request, networkScope: 'internet' }).reason, 'PEER_INTERNET_NOT_GRANTED');
  authority.grantNasDirectInternet(app, peerIdentity);
  assert.equal(authorizeNasConnection({ ...request, networkScope: 'internet' }).allowed, true);
  authority.revokeInternet(app);
  assert.equal(authorizeNasConnection({ ...request, networkScope: 'internet' }).allowed, false);
  assert.equal(authorizeNasConnection(request).allowed, true);
  authority.revokeNasPeer(app, peerIdentity);
  assert.equal(authorizeNasConnection(request).allowed, false);
});

test('connection preflight rejects invalid routes, wrong identities, AI and stale app handles', () => {
  const { authority, authorizeNasConnection, app } = setup();
  authority.pinNasPeer(app, { peerIdentity, localNetworkId: 'home' });
  authority.grantNasDirectInternet(app, peerIdentity);
  const request = { app, peerIdentity, networkScope: 'lan', localNetworkId: 'home' };
  for (const change of [
    { peerIdentity: otherPeer }, { peerIdentity: '' }, { localNetworkId: 'public' },
    { networkScope: 'unknown' }, { networkScope: undefined }, { route: 'relay' }, { route: 'redirect' },
  ]) assert.equal(authorizeNasConnection({ ...request, ...change }).allowed, false);
  for (const kind of ['ai', 'app', 'browser', 'model-downloader', 'update-downloader']) {
    const other = authority.installApp({ appId: `preflight.${kind}`, kind });
    assert.equal(authorizeNasConnection({ ...request, app: other }).allowed, false);
  }
  assert.equal(authorizeNasConnection().reason, 'UNKNOWN_APP');
  assert.equal(authorizeNasConnection({ ...request, app: {} }).reason, 'UNKNOWN_APP');
  authority.uninstallApp(app);
  assert.equal(authorizeNasConnection(request).reason, 'UNKNOWN_APP');
});

test('re-pairing cannot revive an old transport even when no check occurred during revocation', () => {
  const { authority, check, app } = setup();
  authority.pinNasPeer(app, { peerIdentity, localNetworkId: 'home' });
  const old = authority.attestVerifiedTransport({ app, peerIdentity, networkScope: 'lan', localNetworkId: 'home' });
  authority.revokeNasPeer(app, peerIdentity);
  authority.pinNasPeer(app, { peerIdentity, localNetworkId: 'home' });
  assert.equal(check({ app, resource: 'nas', transport: old }).reason, 'STALE_TRANSPORT_AUTHORITY');
  const fresh = authority.attestVerifiedTransport({ app, peerIdentity, networkScope: 'lan', localNetworkId: 'home' });
  assert.equal(check({ app, resource: 'nas', transport: fresh }).allowed, true);
});

test('internet revoke and regrant require a fresh remote transport but preserve the local route', () => {
  const { authority, check, app } = setup();
  authority.pinNasPeer(app, { peerIdentity, localNetworkId: 'home' });
  authority.grantNasDirectInternet(app, peerIdentity);
  const local = authority.attestVerifiedTransport({ app, peerIdentity, networkScope: 'lan', localNetworkId: 'home' });
  const remote = authority.attestVerifiedTransport({ app, peerIdentity, networkScope: 'internet' });
  authority.revokeInternet(app);
  authority.grantNasDirectInternet(app, peerIdentity);
  assert.equal(check({ app, resource: 'nas', transport: remote }).reason, 'STALE_TRANSPORT_AUTHORITY');
  assert.equal(check({ app, resource: 'nas', transport: local }).allowed, true);
  const fresh = authority.attestVerifiedTransport({ app, peerIdentity, networkScope: 'internet' });
  assert.equal(check({ app, resource: 'nas', transport: fresh }).allowed, true);
});

test('transport facts bind to one app and a pairing that exists at attestation time', () => {
  const { authority, check, app } = setup();
  assert.throws(() => authority.attestVerifiedTransport({ peerIdentity, networkScope: 'internet' }), { code: 'UNKNOWN_APP' });
  const early = authority.attestVerifiedTransport({ app, peerIdentity, networkScope: 'lan', localNetworkId: 'home' });
  authority.pinNasPeer(app, { peerIdentity, localNetworkId: 'home' });
  assert.equal(check({ app, resource: 'nas', transport: early }).reason, 'STALE_TRANSPORT_AUTHORITY');
  const fact = authority.attestVerifiedTransport({ app, peerIdentity, networkScope: 'lan', localNetworkId: 'home' });
  const other = authority.installApp({ appId: 'nas.other', kind: 'nas' });
  authority.pinNasPeer(other, { peerIdentity, localNetworkId: 'home' });
  assert.equal(check({ app: other, resource: 'nas', transport: fact }).reason, 'TRANSPORT_APP_MISMATCH');
  // Re-pairing a different peer must not invalidate this peer's live connection.
  authority.pinNasPeer(app, { peerIdentity: otherPeer, localNetworkId: 'home' });
  authority.revokeNasPeer(app, otherPeer);
  assert.equal(check({ app, resource: 'nas', transport: fact }).allowed, true);
});

test('installation grants no internet, LAN, documents, NAS or administrator rights', () => {
  for (const kind of ['app', 'browser', 'nas', 'ai', 'model-downloader', 'update-downloader']) {
    const { check, app } = setup(kind);
    for (const resource of ['internet', 'lan', 'documents', 'nas', 'admin']) assert.equal(check({ app, resource, objectId }).allowed, false);
  }
});

test('browser explicit internet consent does not grant general documents or NAS access', () => {
  const { authority, check, app } = setup('browser'); authority.grantInternet(app);
  assert.equal(check({ app, resource: 'internet' }).allowed, true);
  assert.equal(check({ app, resource: 'documents', objectId }).allowed, false);
  assert.equal(check({ app, resource: 'nas' }).allowed, false);
  authority.grantDocument(app, objectId, 'read');
  assert.equal(check({ app, resource: 'documents', objectId }).allowed, true);
  assert.equal(check({ app, resource: 'documents', objectId, mode: 'write' }).allowed, false);
  assert.equal(check({ app, resource: 'documents', objectId: '2'.repeat(32) }).allowed, false);
});

test('local system AI cannot receive internet or NAS authority', () => {
  const { authority, check, app } = setup('ai');
  assert.throws(() => authority.grantInternet(app), { code: 'AI_NETWORK_DENIED' });
  assert.throws(() => authority.pinNasPeer(app, { peerIdentity, localNetworkId: 'home' }), { code: 'NAS_GRANT_DENIED' });
  for (const resource of ['internet', 'lan', 'nas']) assert.equal(check({ app, resource }).reason, 'AI_NETWORK_DENIED');
});

test('NAS requires the pinned identity and explicitly selected local network', () => {
  const { authority, check, app } = setup(); authority.pinNasPeer(app, { peerIdentity, localNetworkId: 'home' });
  const good = authority.attestVerifiedTransport({ app, peerIdentity, networkScope: 'lan', localNetworkId: 'home' });
  const evil = authority.attestVerifiedTransport({ app, peerIdentity: otherPeer, networkScope: 'lan', localNetworkId: 'home' });
  const moved = authority.attestVerifiedTransport({ app, peerIdentity, networkScope: 'lan', localNetworkId: 'public' });
  assert.equal(check({ app, resource: 'nas', transport: good }).allowed, true);
  assert.equal(check({ app, resource: 'nas', transport: evil }).reason, 'UNPINNED_PEER');
  assert.equal(check({ app, resource: 'nas', transport: moved }).reason, 'LOCAL_NETWORK_CHANGED');
});

test('direct internet to a paired NAS requires separate peer-bound consent', () => {
  const { authority, check, app } = setup(); authority.pinNasPeer(app, { peerIdentity, localNetworkId: 'home' });
  const remote = authority.attestVerifiedTransport({ app, peerIdentity, networkScope: 'internet' });
  assert.equal(check({ app, resource: 'nas', transport: remote }).reason, 'PEER_INTERNET_NOT_GRANTED');
  assert.throws(() => authority.grantInternet(app), { code: 'USE_PEER_BOUND_CONSENT' });
  assert.throws(() => authority.grantNasDirectInternet(app, otherPeer), { code: 'NAS_NOT_PAIRED' });
  authority.grantNasDirectInternet(app, peerIdentity);
  assert.equal(check({ app, resource: 'nas', transport: remote }).allowed, true);
  assert.equal(check({ app, resource: 'internet' }).allowed, false);
  const wrong = authority.attestVerifiedTransport({ app, peerIdentity: otherPeer, networkScope: 'internet' });
  assert.equal(check({ app, resource: 'nas', transport: wrong }).allowed, false);
});

test('vendor relay and redirects stay denied even with direct-internet permission', () => {
  const { authority, check, app } = setup(); authority.pinNasPeer(app, { peerIdentity, localNetworkId: 'home' });
  authority.grantNasDirectInternet(app, peerIdentity);
  for (const route of ['relay', 'redirect']) {
    const fact = authority.attestVerifiedTransport({ app, peerIdentity, networkScope: 'internet', route });
    assert.equal(check({ app, resource: 'nas', transport: fact }).reason, 'RELAY_OR_REDIRECT_DENIED');
  }
});

test('client-supplied verified:true or cloned transport facts never count as authenticated transport', () => {
  const { authority, check, app } = setup(); authority.pinNasPeer(app, { peerIdentity, localNetworkId: 'home' });
  assert.equal(check({ app, resource: 'nas', transport: { verified: true, peerIdentity, networkScope: 'lan', localNetworkId: 'home' } }).reason, 'UNVERIFIED_TRANSPORT');
  const fact = authority.attestVerifiedTransport({ app, peerIdentity, networkScope: 'lan', localNetworkId: 'home' });
  assert.equal(check({ app, resource: 'nas', transport: structuredClone(fact) }).allowed, false);
  authority.forgetTransport(fact);
  assert.equal(check({ app, resource: 'nas', transport: fact }).allowed, false);
});

test('revoking a peer or internet right also invalidates decisions using previously valid facts', () => {
  const { authority, check, app } = setup(); authority.pinNasPeer(app, { peerIdentity, localNetworkId: 'home' });
  authority.grantNasDirectInternet(app, peerIdentity);
  const fact = authority.attestVerifiedTransport({ app, peerIdentity, networkScope: 'internet' });
  authority.revokeInternet(app);
  assert.equal(check({ app, resource: 'nas', transport: fact }).allowed, false);
  authority.grantNasDirectInternet(app, peerIdentity); authority.revokeNasPeer(app, peerIdentity);
  assert.equal(check({ app, resource: 'nas', transport: fact }).allowed, false);
  authority.pinNasPeer(app, { peerIdentity, localNetworkId: 'home' });
  assert.equal(check({ app, resource: 'nas', transport: fact }).reason, 'PEER_INTERNET_NOT_GRANTED');
});

test('model and update downloader network rights never grant personal document or NAS authority', () => {
  for (const kind of ['model-downloader', 'update-downloader']) {
    const { authority, check, app } = setup(kind); authority.grantInternet(app);
    assert.throws(() => authority.grantDocument(app, objectId), { code: 'DOCUMENT_GRANT_DENIED' });
    assert.equal(check({ app, resource: 'documents', objectId }).allowed, false);
    assert.equal(check({ app, resource: 'nas' }).allowed, false);
  }
});

test('stale app handles, forged handles and unknown rights fail closed', () => {
  const { authority, check, app } = setup('browser'); authority.grantInternet(app); authority.uninstallApp(app);
  assert.equal(check({ app, resource: 'internet' }).allowed, false);
  assert.equal(check({ app: { kind: 'browser', internet: true }, resource: 'internet' }).allowed, false);
  const reinstalled = authority.installApp({ appId: 'synthetic.app', kind: 'browser' });
  assert.equal(check({ app: reinstalled, resource: 'internet' }).allowed, false);
  assert.equal(check({ app: reinstalled, resource: 'raw-socket' }).reason, 'DEFAULT_DENY');
});

test('one pairing and internet consent automatically select LAN to internet to LAN', () => {
  const { authority, planNasRoute, app } = setup();
  authority.pinNasPeer(app, { peerIdentity, localNetworkId: 'home' });
  authority.grantNasDirectInternet(app, peerIdentity);
  const lan = authority.attestVerifiedTransport({ app, peerIdentity, networkScope: 'lan', localNetworkId: 'home' });
  const internet = authority.attestVerifiedTransport({ app, peerIdentity, networkScope: 'internet' });
  // Candidate ordering cannot make a remote route outrank an allowed local route.
  const atHome = planNasRoute({ app, peerIdentity, transports: [internet, lan] });
  assert.equal(atHome.status, 'ready'); assert.equal(atHome.route, 'lan'); assert.equal(atHome.transport, lan);
  const away = planNasRoute({ app, peerIdentity, transports: [internet] });
  assert.equal(away.status, 'ready'); assert.equal(away.route, 'internet'); assert.equal(away.transport, internet);
  const backHome = planNasRoute({ app, peerIdentity, transports: [internet, lan] });
  assert.equal(backHome.route, 'lan'); assert.equal(backHome.transport, lan);
  // No repeated pin or grant has occurred during any transition.
  assert.deepEqual(planNasRoute({ app, peerIdentity, transports: [] }), {
    status: 'paused', route: null, transport: null, reason: 'NO_VERIFIED_ROUTE',
  });
  assert.equal(planNasRoute({ app, peerIdentity, transports: [internet] }).route, 'internet');
});

test('route planner applies revoked rights immediately while previously verified facts remain', () => {
  const { authority, check, planNasRoute, app } = setup();
  authority.pinNasPeer(app, { peerIdentity, localNetworkId: 'home' });
  authority.grantNasDirectInternet(app, peerIdentity);
  const lan = authority.attestVerifiedTransport({ app, peerIdentity, networkScope: 'lan', localNetworkId: 'home' });
  const internet = authority.attestVerifiedTransport({ app, peerIdentity, networkScope: 'internet' });
  const oldDecision = planNasRoute({ app, peerIdentity, transports: [internet] });
  authority.revokeInternet(app);
  assert.equal(planNasRoute({ app, peerIdentity, transports: [internet] }).status, 'paused');
  assert.equal(planNasRoute({ app, peerIdentity, transports: [internet] }).reason, 'PEER_INTERNET_NOT_GRANTED');
  assert.equal(check({ app, resource: 'nas', transport: oldDecision.transport }).allowed, false);
  assert.equal(planNasRoute({ app, peerIdentity, transports: [internet, lan] }).route, 'lan');
  authority.revokeNasPeer(app, peerIdentity);
  const revoked = planNasRoute({ app, peerIdentity, transports: [internet, lan] });
  assert.equal(revoked.status, 'paused'); assert.equal(revoked.reason, 'NAS_NOT_PAIRED'); assert.equal(revoked.transport, null);
  assert.equal(check({ app, resource: 'nas', transport: lan }).allowed, false);
});

test('automatic selection never accepts a wrong peer, forged fact, relay, redirect or moved LAN', () => {
  const { authority, planNasRoute, app } = setup();
  authority.pinNasPeer(app, { peerIdentity, localNetworkId: 'home' });
  authority.grantNasDirectInternet(app, peerIdentity);
  authority.pinNasPeer(app, { peerIdentity: otherPeer, localNetworkId: 'home' });
  authority.grantNasDirectInternet(app, otherPeer);
  const otherLan = authority.attestVerifiedTransport({ app, peerIdentity: otherPeer, networkScope: 'lan', localNetworkId: 'home' });
  const wrongNetwork = authority.attestVerifiedTransport({ app, peerIdentity, networkScope: 'lan', localNetworkId: 'public' });
  const relay = authority.attestVerifiedTransport({ app, peerIdentity, networkScope: 'internet', route: 'relay' });
  const redirect = authority.attestVerifiedTransport({ app, peerIdentity, networkScope: 'internet', route: 'redirect' });
  const forged = { verified: true, peerIdentity, networkScope: 'lan', localNetworkId: 'home' };
  const invalid = [otherLan, wrongNetwork, relay, redirect, forged];
  assert.equal(planNasRoute({ app, peerIdentity, transports: invalid }).status, 'paused');
  const internet = authority.attestVerifiedTransport({ app, peerIdentity, networkScope: 'internet' });
  assert.equal(planNasRoute({ app, peerIdentity, transports: [...invalid, internet] }).transport, internet);
  authority.forgetTransport(internet);
  assert.equal(planNasRoute({ app, peerIdentity, transports: [...invalid, internet] }).status, 'paused');
});

test('automatic selection never creates missing consent and bounds its candidate input', () => {
  const { authority, planNasRoute, app } = setup();
  authority.pinNasPeer(app, { peerIdentity, localNetworkId: 'home' });
  const internet = authority.attestVerifiedTransport({ app, peerIdentity, networkScope: 'internet' });
  assert.equal(planNasRoute({ app, peerIdentity, transports: [internet] }).reason, 'PEER_INTERNET_NOT_GRANTED');
  assert.equal(planNasRoute({ app, peerIdentity, transports: [internet] }).status, 'paused');
  assert.equal(planNasRoute({ app, peerIdentity, transports: Array(65).fill(internet) }).reason, 'INVALID_ROUTE_SET');
  assert.equal(planNasRoute({ app, peerIdentity, transports: {} }).reason, 'INVALID_ROUTE_SET');
  assert.equal(planNasRoute().reason, 'UNKNOWN_APP');
  const ai = authority.installApp({ appId: 'synthetic.ai', kind: 'ai' });
  assert.equal(planNasRoute({ app: ai, peerIdentity, transports: [internet] }).reason, 'AI_NETWORK_DENIED');
  authority.uninstallApp(app);
  assert.equal(planNasRoute({ app, peerIdentity, transports: [internet] }).reason, 'UNKNOWN_APP');
});
