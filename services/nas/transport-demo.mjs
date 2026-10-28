import { createDocumentCipher } from './crypto.mjs';
import { createPolicyEngine } from './policy.mjs';
import { createLoopbackNasServer, connectLoopbackNas } from './transport.mjs';
import { createTlsFixture } from './tests/tls-fixture.mjs';

const fixture = await createTlsFixture();
let server, client;
try {
  const namespace = '1'.repeat(64), objectId = '2'.repeat(32);
  const engine = createPolicyEngine();
  const app = engine.authority.installApp({ appId: 'nas.transport.demo', kind: 'nas' });
  engine.authority.pinNasPeer(app, { peerIdentity: fixture.serverPin, localNetworkId: 'loopback-fixture' });
  server = await createLoopbackNasServer({ credentials: fixture.server, allowedClientPin: fixture.clientPin, namespace });
  client = await connectLoopbackNas({ port: server.port, credentials: fixture.client,
    expectedPeerPin: fixture.serverPin, serverName: fixture.serverName, engine, app,
    networkScope: 'lan', localNetworkId: 'loopback-fixture' });
  const cipher = createDocumentCipher(namespace);
  const encrypted = cipher.encrypt({ objectId, revision: 1, name: 'synthetic.txt', content: 'synthetic private content' });
  const response = await client.sendCiphertext(encrypted);
  const restored = cipher.decrypt({ objectId, revision: 1 }, response);
  if (restored.name !== 'synthetic.txt' || restored.content !== 'synthetic private content') throw new Error('DEMO_FAILED');
  const connection = client.evidence();
  engine.authority.revokeNasPeer(app, fixture.serverPin);
  let revoked = false;
  try { await client.sendCiphertext(encrypted); } catch (error) { revoked = error.code === 'UNPINNED_PEER'; }
  if (!revoked) throw new Error('DEMO_REVOCATION_FAILED');
  // No keys, certificates, document names/content or envelope bytes in evidence.
  console.log(JSON.stringify({ status: 'passed', node: process.versions.node, openssl: process.versions.openssl,
    connection, server: server.evidence(), clientOnlyDecryption: true, revokedSendBlocked: revoked,
    scope: 'Real loopback host TLS; no native OS, WAN reachability, PQ authentication or disk durability claim' }, null, 2));
} finally { client?.close(); await server?.close(); await fixture.close(); }
