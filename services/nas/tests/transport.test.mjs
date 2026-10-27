import test, { before, after } from 'node:test';
import assert from 'node:assert/strict';
import tls from 'node:tls';
import net from 'node:net';
import { once } from 'node:events';
import { createDocumentCipher } from '../crypto.mjs';
import { createPolicyEngine } from '../policy.mjs';
import { createLoopbackNasServer, connectLoopbackNas, TLS_PROFILE } from '../transport.mjs';
import { createTlsFixture } from './tls-fixture.mjs';

const namespace = 'a'.repeat(64), objectId = 'b'.repeat(32), localNetworkId = 'synthetic-home';
let fixture;
before(async () => { fixture = await createTlsFixture(); });
after(async () => { await fixture?.close(); });
const document = { objectId, revision: 1, name: 'privé document', content: 'inhoud uitsluitend voor client' };
function encrypted(ns = namespace) { return createDocumentCipher(ns).encrypt(document); }
async function setup(t, { consent = true, networkScope = 'lan', pin = fixture.serverPin } = {}) {
  const engine = createPolicyEngine();
  const app = engine.authority.installApp({ appId: 'nas.transport.test', kind: 'nas' });
  if (consent) engine.authority.pinNasPeer(app, { peerIdentity: pin, localNetworkId });
  const server = await createLoopbackNasServer({ credentials: fixture.server,
    allowedClientPin: fixture.clientPin, namespace });
  t.after(() => server.close());
  const options = { port: server.port, credentials: fixture.client, expectedPeerPin: pin,
    serverName: fixture.serverName, engine, app, networkScope, localNetworkId };
  return { engine, app, server, options };
}
function frame(bytes) { const header = Buffer.alloc(4); header.writeUInt32BE(bytes.length); return Buffer.concat([header, bytes]); }

// Hostile fixture: bypasses the public adapter to test the real server boundary.
// A timeout is a TEST FAILURE, not evidence that the server rejected the client.
function rawAttempt(port, overrides = {}, onSecure = () => {}) {
  return new Promise((resolve, reject) => {
    let secure = false, received = 0, cleanup, errorCode = null;
    const socket = tls.connect({ ...fixture.client, host: TLS_PROFILE.host, port,
      servername: fixture.serverName, minVersion: 'TLSv1.3', maxVersion: 'TLSv1.3',
      ciphers: TLS_PROFILE.cipher, ecdhCurve: TLS_PROFILE.group,
      ALPNProtocols: [TLS_PROFILE.alpn], rejectUnauthorized: true, ...overrides });
    const deadline = setTimeout(() => { socket.destroy(); reject(new Error('RAW_TEST_TIMED_OUT')); }, 2_750);
    socket.on('error', (error) => { errorCode = error.code; });
    socket.on('data', (bytes) => { received += bytes.length; });
    socket.once('secureConnect', () => { secure = true; cleanup = onSecure(socket); });
    socket.once('close', () => { clearTimeout(deadline); if (typeof cleanup === 'function') cleanup(); resolve({ secure, received, errorCode }); });
  });
}

test('N02 real TLS measures hybrid group and returns only client-decryptable ciphertext', async (t) => {
  const { server, options } = await setup(t);
  const client = await connectLoopbackNas(options); t.after(() => client.close());
  const cipher = createDocumentCipher(namespace), envelope = cipher.encrypt(document);
  const response = await client.sendCiphertext(envelope);
  assert.deepEqual(cipher.decrypt({ objectId, revision: 1 }, response), { name: document.name, content: document.content });
  assert.equal(response.includes(Buffer.from(document.name)), false);
  assert.equal(response.includes(Buffer.from(document.content)), false);
  const evidence = client.evidence();
  assert.equal(evidence.group, 'X25519MLKEM768'); assert.equal(evidence.protocol, 'TLSv1.3');
  assert.equal(evidence.cipher, 'TLS_AES_256_GCM_SHA384'); assert.equal(evidence.alpn, TLS_PROFILE.alpn);
  assert.equal(evidence.sessionReused, false); assert.equal(evidence.hostLoopbackOnly, true);
  assert.equal(server.evidence().authenticatedConnections, 1); assert.equal(server.evidence().acceptedFrames, 1);
  assert.equal(server.evidence().contentKeysOnServer, false);
});

test('N02 preflight without pairing creates no socket', async (t) => {
  const { server, options } = await setup(t, { consent: false });
  await assert.rejects(connectLoopbackNas(options), /NAS_NOT_PAIRED|UNPINNED_PEER/);
  assert.equal(server.evidence().connections, 0);
});

test('N02 internet requires current peer consent before a socket exists', async (t) => {
  const { server, options, engine, app } = await setup(t, { networkScope: 'internet' });
  await assert.rejects(connectLoopbackNas(options), /PEER_INTERNET_NOT_GRANTED/);
  assert.equal(server.evidence().connections, 0);
  engine.authority.grantNasDirectInternet(app, fixture.serverPin);
  const client = await connectLoopbackNas(options); t.after(() => client.close());
  await client.sendCiphertext(encrypted());
  assert.equal(client.evidence().networkScopeSource, 'explicit-loopback-harness-assumption');
  engine.authority.revokeInternet(app);
  await assert.rejects(client.sendCiphertext(encrypted()), /PEER_INTERNET_NOT_GRANTED/);
  assert.equal(server.evidence().acceptedFrames, 1);
});

test('N02 changed local network and forged app cannot initiate a connection', async (t) => {
  const { server, options } = await setup(t);
  await assert.rejects(connectLoopbackNas({ ...options, localNetworkId: 'untrusted-home' }), /LOCAL_NETWORK_CHANGED/);
  await assert.rejects(connectLoopbackNas({ ...options, app: {} }), /UNKNOWN_APP/);
  assert.equal(server.evidence().connections, 0);
});

test('N02 incorrect NAS pin is rejected even with a trusted CA and hostname', async (t) => {
  const { server, options } = await setup(t, { pin: fixture.otherServerPin });
  await assert.rejects(connectLoopbackNas(options), /TRANSPORT_AUTHENTICATION_FAILED/);
  assert.equal(server.evidence().acceptedFrames, 0);
});

test('N02 certificate pin does not bypass normal hostname or CA checks', async (t) => {
  const { server, options } = await setup(t);
  await assert.rejects(connectLoopbackNas({ ...options, serverName: 'wrong.loopback.invalid' }), /TRANSPORT_AUTHENTICATION_FAILED/);
  await assert.rejects(connectLoopbackNas({ ...options, credentials: { ...fixture.client, ca: fixture.otherServer.cert } }), /TRANSPORT_AUTHENTICATION_FAILED/);
  assert.equal(server.evidence().acceptedFrames, 0);
});

test('N02 server rejects another CA-authorized client because its pin differs', async (t) => {
  const { server, options } = await setup(t);
  await assert.rejects(async () => {
    const client = await connectLoopbackNas({ ...options, credentials: fixture.stranger });
    try { await client.sendCiphertext(encrypted()); } finally { client.close(); }
  }, /TRANSPORT_/);
  assert.equal(server.evidence().acceptedFrames, 0);
  assert.equal(server.evidence().authenticatedConnections, 0);
});

test('N02 missing client certificate is rejected by real mutual TLS', async (t) => {
  const { server } = await setup(t);
  const result = await rawAttempt(server.port, { key: undefined, cert: undefined }, (socket) => socket.write(frame(encrypted())));
  assert.equal(result.received, 0); assert.equal(server.evidence().acceptedFrames, 0);
});

for (const [description, overrides] of [
  ['TLS 1.2 downgrade', { minVersion: 'TLSv1.2', maxVersion: 'TLSv1.2', ecdhCurve: 'X25519', ciphers: 'ECDHE-ECDSA-AES256-GCM-SHA384' }],
  ['classical-only key exchange', { ecdhCurve: 'X25519' }],
  ['other record cipher', { ciphers: 'TLS_AES_128_GCM_SHA256' }],
  ['wrong ALPN', { ALPNProtocols: ['other-protocol'] }],
  ['missing ALPN', { ALPNProtocols: [] }],
]) {
  test(`N02 rejects ${description} without application data`, async (t) => {
    const { server } = await setup(t);
    const result = await rawAttempt(server.port, overrides, (socket) => socket.write(frame(encrypted())));
    assert.equal(result.received, 0); assert.equal(server.evidence().acceptedFrames, 0);
    if (description !== 'missing ALPN') assert.equal(result.secure, false);
    if (description === 'TLS 1.2 downgrade') assert.equal(result.errorCode, 'ERR_SSL_TLSV1_ALERT_PROTOCOL_VERSION');
  });
}

test('N02 current peer revocation prevents a subsequent write on an established socket', async (t) => {
  const { server, options, engine, app } = await setup(t);
  const client = await connectLoopbackNas(options); t.after(() => client.close());
  await client.sendCiphertext(encrypted());
  engine.authority.revokeNasPeer(app, fixture.serverPin);
  await assert.rejects(client.sendCiphertext(encrypted()), /UNPINNED_PEER/);
  assert.equal(server.evidence().acceptedFrames, 1); assert.equal(client.evidence().closed, true);
});

test('N02 review regression: revocation between preflight and handshake blocks application data', async (t) => {
  const { server, options, engine, app } = await setup(t);
  // connectLoopbackNas runs preflight synchronously before returning its promise;
  // socket establishment and secureConnect occur later through the event loop.
  const connecting = connectLoopbackNas(options);
  engine.authority.revokeNasPeer(app, fixture.serverPin);
  await assert.rejects(connecting, /UNPINNED_PEER/);
  assert.equal(server.evidence().acceptedFrames, 0);
});

test('N02 old idle connection cannot revive after peer revocation and re-pairing', async (t) => {
  const { server, options, engine, app } = await setup(t);
  const old = await connectLoopbackNas(options); t.after(() => old.close());
  // No intervening send observes revocation: the epoch must invalidate the old
  // authority even when the same peer identity is deliberately pinned again.
  engine.authority.revokeNasPeer(app, fixture.serverPin);
  engine.authority.pinNasPeer(app, { peerIdentity: fixture.serverPin, localNetworkId });
  await assert.rejects(old.sendCiphertext(encrypted()), /STALE_TRANSPORT_AUTHORITY/);
  assert.equal(server.evidence().acceptedFrames, 0);
  const fresh = await connectLoopbackNas(options); t.after(() => fresh.close());
  await fresh.sendCiphertext(encrypted());
  assert.equal(server.evidence().acceptedFrames, 1);
});

test('N02 old internet connection stays revoked after new consent while LAN remains usable', async (t) => {
  const { server, options, engine, app } = await setup(t);
  engine.authority.grantNasDirectInternet(app, fixture.serverPin);
  const remoteOptions = { ...options, networkScope: 'internet' };
  const oldRemote = await connectLoopbackNas(remoteOptions); t.after(() => oldRemote.close());
  const lan = await connectLoopbackNas(options); t.after(() => lan.close());
  engine.authority.revokeInternet(app);
  engine.authority.grantNasDirectInternet(app, fixture.serverPin);
  await assert.rejects(oldRemote.sendCiphertext(encrypted()), /STALE_TRANSPORT_AUTHORITY/);
  await lan.sendCiphertext(encrypted());
  const freshRemote = await connectLoopbackNas(remoteOptions); t.after(() => freshRemote.close());
  await freshRemote.sendCiphertext(encrypted());
  assert.equal(server.evidence().acceptedFrames, 2);
});

test('N02 revoke during an in-flight request prevents a response reaching the app', async (t) => {
  const { options, engine, app } = await setup(t);
  const client = await connectLoopbackNas(options); t.after(() => client.close());
  const request = client.sendCiphertext(encrypted());
  engine.authority.revokeNasPeer(app, fixture.serverPin);
  await assert.rejects(request, /UNPINNED_PEER/);
  assert.equal(client.evidence().closed, true);
});

test('N02 closed socket forgets the opaque transport fact and cannot be reused', async (t) => {
  const { options, engine, app } = await setup(t);
  let observedFact;
  const adapterEngine = { ...engine, authority: { ...engine.authority,
    attestVerifiedTransport(input) { observedFact = engine.authority.attestVerifiedTransport(input); return observedFact; } } };
  const client = await connectLoopbackNas({ ...options, engine: adapterEngine });
  assert.equal(engine.check({ app, resource: 'nas', transport: observedFact }).allowed, true);
  client.close();
  assert.equal(engine.check({ app, resource: 'nas', transport: observedFact }).allowed, false);
  await assert.rejects(client.sendCiphertext(encrypted()), /TRANSPORT_CLOSED/);
});

test('N02 rejects plaintext, oversized requests and extra frames without extra server writes', async (t) => {
  const { server, options } = await setup(t);
  const client = await connectLoopbackNas(options); t.after(() => client.close());
  await assert.rejects(client.sendCiphertext(Buffer.from('private document')), /INVALID_/);
  await assert.rejects(client.sendCiphertext(Buffer.alloc(TLS_PROFILE.maxFrameBytes + 1)), /INVALID_/);
  const envelope = encrypted();
  for (let i = 0; i < TLS_PROFILE.maxFrames; i++) await client.sendCiphertext(envelope);
  await assert.rejects(client.sendCiphertext(envelope), /TRANSPORT_FRAME_LIMIT/);
  assert.equal(server.evidence().acceptedFrames, TLS_PROFILE.maxFrames);
});

for (const [description, payload] of [
  ['zero frame length', () => Buffer.alloc(4)],
  ['oversized frame length', () => { const h = Buffer.alloc(4); h.writeUInt32BE(TLS_PROFILE.maxFrameBytes + 1); return h; }],
  ['plaintext frame', () => frame(Buffer.from('plaintext is forbidden'))],
  ['different namespace', () => frame(encrypted('c'.repeat(64)))],
]) {
  test(`N02 hostile client cannot bypass ${description} validation`, async (t) => {
    const { server } = await setup(t);
    const result = await rawAttempt(server.port, {}, (socket) => socket.write(payload()));
    assert.equal(result.received, 0); assert.equal(server.evidence().acceptedFrames, 0);
  });
}

test('N02 server independently bounds frames from a hostile client', async (t) => {
  const { server } = await setup(t);
  const payload = frame(encrypted());
  await rawAttempt(server.port, {}, (socket) => socket.write(Buffer.concat(Array(TLS_PROFILE.maxFrames + 1).fill(payload))));
  assert.equal(server.evidence().acceptedFrames, TLS_PROFILE.maxFrames);
});

test('N02 slow frame drip cannot reset the absolute frame deadline', async (t) => {
  const { server } = await setup(t); const start = Date.now();
  await rawAttempt(server.port, {}, (socket) => {
    const payload = frame(encrypted()); let index = 0;
    socket.write(payload.subarray(index++, index));
    const timer = setInterval(() => { if (!socket.destroyed) socket.write(payload.subarray(index++, index)); }, 90);
    return () => clearInterval(timer);
  });
  assert.ok(Date.now() - start < TLS_PROFILE.frameTimeoutMs + 1_000);
  assert.equal(server.evidence().acceptedFrames, 0);
});

test('N02 pre-handshake sockets are bounded and closed at handshake timeout', async (t) => {
  const { server } = await setup(t); const sockets = [];
  for (let i = 0; i < TLS_PROFILE.maxConnections; i++) {
    const socket = net.connect({ host: TLS_PROFILE.host, port: server.port }); sockets.push(socket);
    socket.on('error', () => {}); await once(socket, 'connect');
  }
  t.after(() => { for (const socket of sockets) socket.destroy(); });
  const extra = net.connect({ host: TLS_PROFILE.host, port: server.port }); extra.on('error', () => {});
  const deadline = setTimeout(() => extra.destroy(), 500);
  await once(extra, 'close'); clearTimeout(deadline);
  assert.equal(server.evidence().connections, TLS_PROFILE.maxConnections);
  assert.equal(server.evidence().acceptedFrames, 0);
  await Promise.all(sockets.map((socket) => socket.destroyed ? undefined : once(socket, 'close')));
  assert.equal(server.evidence().authenticatedConnections, 0);
});

test('N02 offered TLS session state cannot resume a previous authenticated connection', async (t) => {
  const { server } = await setup(t);
  const envelope = frame(encrypted());
  function exchange(session) {
    return new Promise((resolve, reject) => {
      let ticket, result, received = 0, settled = false;
      const socket = tls.connect({ ...fixture.client, host: TLS_PROFILE.host, port: server.port,
        servername: fixture.serverName, minVersion: 'TLSv1.3', maxVersion: 'TLSv1.3',
        ciphers: TLS_PROFILE.cipher, ecdhCurve: TLS_PROFILE.group,
        ALPNProtocols: [TLS_PROFILE.alpn], rejectUnauthorized: true, session });
      const deadline = setTimeout(() => { socket.destroy(); reject(new Error('RESUMPTION_TEST_TIMED_OUT')); }, 1_500);
      socket.on('session', (value) => { ticket?.fill(0); ticket = Buffer.from(value); });
      socket.once('secureConnect', () => {
        result = { reused: socket.isSessionReused(), group: socket.getEphemeralKeyInfo()?.name };
        socket.write(envelope);
      });
      socket.on('data', (bytes) => {
        received += bytes.length;
        if (received === envelope.length) {
          settled = true; clearTimeout(deadline); socket.end(); resolve({ ...result, ticket });
        }
      });
      socket.on('error', () => { clearTimeout(deadline); ticket?.fill(0); reject(new Error('RESUMPTION_TEST_FAILED')); });
      socket.once('close', () => { clearTimeout(deadline); if (!settled) { ticket?.fill(0); reject(new Error('RESUMPTION_TEST_CLOSED')); } });
    });
  }
  const first = await exchange();
  let second;
  try {
    // TLS 1.3 may issue stateful tickets even with SSL_OP_NO_TICKET. The
    // server's resumeSession callback must still reject their cached authority.
    assert.ok(Buffer.isBuffer(first.ticket) && first.ticket.length > 0);
    second = await exchange(first.ticket);
    assert.equal(first.reused, false); assert.equal(second.reused, false);
    assert.equal(second.group, TLS_PROFILE.group);
    assert.equal(server.evidence().acceptedFrames, 2);
  } finally { first.ticket?.fill(0); second?.ticket?.fill(0); }
});

// Independent reviewer probes, retained as permanent negative boundary tests.
for (const replyKind of ['oversized', 'mismatched']) {
  test(`N02 review regression: authenticated hostile server ${replyKind} response invalidates the connection`,
    { timeout: 4_000 }, async (t) => {
      const engine = createPolicyEngine();
      const app = engine.authority.installApp({ appId: 'nas.hostile-server.test', kind: 'nas' });
      engine.authority.pinNasPeer(app, { peerIdentity: fixture.serverPin, localNetworkId });
      const sockets = new Set();
      const server = tls.createServer({ ...fixture.server, minVersion: 'TLSv1.3', maxVersion: 'TLSv1.3',
        ecdhCurve: TLS_PROFILE.group, ciphers: TLS_PROFILE.cipher, ALPNProtocols: [TLS_PROFILE.alpn],
        requestCert: true, rejectUnauthorized: true, handshakeTimeout: TLS_PROFILE.handshakeTimeoutMs }, (socket) => {
        socket.on('error', () => {});
        socket.once('data', () => {
          const reply = replyKind === 'oversized' ? null : createDocumentCipher(namespace)
            .encrypt({ ...document, revision: 2, content: 'different synthetic content' });
          const header = Buffer.alloc(4);
          header.writeUInt32BE(reply ? reply.length : TLS_PROFILE.maxFrameBytes + 1);
          socket.write(reply ? Buffer.concat([header, reply]) : header);
        });
      });
      server.on('connection', (socket) => { sockets.add(socket); socket.once('close', () => sockets.delete(socket)); });
      server.on('tlsClientError', (_error, socket) => socket.destroy());
      t.after(async () => { for (const socket of sockets) socket.destroy(); await new Promise((resolve) => server.close(resolve)); });
      server.listen(0, TLS_PROFILE.host); await once(server, 'listening');
      const client = await connectLoopbackNas({ port: server.address().port, credentials: fixture.client,
        expectedPeerPin: fixture.serverPin, serverName: fixture.serverName, engine, app,
        networkScope: 'lan', localNetworkId });
      t.after(() => client.close());
      await assert.rejects(client.sendCiphertext(encrypted()),
        replyKind === 'oversized' ? /TRANSPORT_FRAME_LIMIT/ : /TRANSPORT_RESPONSE_MISMATCH/);
      assert.equal(client.evidence().closed, true);
      await assert.rejects(client.sendCiphertext(encrypted()), /TRANSPORT_CLOSED/);
    });
}
