import assert from 'node:assert/strict';
import fs from 'node:fs';
import { NasStore } from './store.mjs';
import { createDocumentCipher } from './crypto.mjs';
import { createPolicyEngine } from './policy.mjs';

const { store, adminCapability } = NasStore.createSynthetic();
try {
  const offer = store.createInvitation(adminCapability);
  // Demonstrates the explicit confirmation input; this does not authenticate a network peer.
  const user = store.acceptInvitation({ ...offer, confirmedPeerIdentity: offer.peerIdentity });
  const session = store.openSession(user);
  const cipher = createDocumentCipher(user.namespace);
  const objectId = 'd'.repeat(32);
  const expected = { objectId, revision: 1 };
  const input = { name: 'Synthetische demonstratie.txt', content: 'Alleen demonstratiegegevens.' };
  const envelope = cipher.encrypt({ ...expected, ...input });
  store.commitUpload(session, store.stageUpload(session, envelope));
  assert.deepEqual(cipher.decrypt(expected, store.readObject(session, objectId)), input);

  const { authority, check } = createPolicyEngine();
  const app = authority.installApp({ appId: 'demo.nas', kind: 'nas' });
  authority.pinNasPeer(app, { peerIdentity: offer.peerIdentity, localNetworkId: 'home' });
  const assumedTransport = authority.attestVerifiedTransport({ app, peerIdentity: offer.peerIdentity, networkScope: 'internet' });
  assert.equal(check({ app, resource: 'nas', transport: assumedTransport }).allowed, false);
  authority.grantNasDirectInternet(app, offer.peerIdentity);
  assert.equal(check({ app, resource: 'nas', transport: assumedTransport }).allowed, true);

  const staged = store.stageUpload(session, cipher.encrypt({ objectId, revision: 2, ...input }));
  store.evictNamespace(adminCapability, user.namespace);
  assert.throws(() => store.commitUpload(session, staged), { code: 'INVALID_SESSION' });
  assert.equal(store.inspectUsage(adminCapability)[0].storedBytes, 0);
  process.stdout.write(JSON.stringify({ prototype: 'N01 Node-hostprototype', result: 'passed',
    checks: ['authenticated-encryption-roundtrip', 'real-ciphertext-storage', 'explicit-peer-internet-policy',
      'administrator-eviction-without-user-key', 'stale-upload-denied'],
    boundary: 'Geen TLS, sockets, echte NAS of native OS-integratie; uitsluitend synthetische data.' }, null, 2) + '\n');
} finally {
  store.close();
  fs.rmSync(store.directory, { recursive: true, force: true });
}
