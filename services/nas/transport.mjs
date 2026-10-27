/**
 * N02: bounded loopback host experiment, not a native network stack or NAS API.
 * The protocol below frames ciphertext; cryptographic negotiation is TLS 1.3.
 * Keep engine.authority and the adapter inside the trusted host harness.
 */
import tls from 'node:tls';
import { constants, createHash, X509Certificate } from 'node:crypto';
import { parseEnvelope, MAX_ENVELOPE_BYTES } from './crypto.mjs';
import { NasError, fail, requireId } from './errors.mjs';

export const TLS_PROFILE = Object.freeze({
  version: 'TLSv1.3', group: 'X25519MLKEM768', cipher: 'TLS_AES_256_GCM_SHA384',
  alpn: 'nieuw-os-nas-host/2', host: '127.0.0.1',
  maxConnections: 4, maxFrames: 8, maxFrameBytes: MAX_ENVELOPE_BYTES,
  handshakeTimeoutMs: 1_500, frameTimeoutMs: 750, connectionTimeoutMs: 5_000,
});
const secureOptions = constants.SSL_OP_NO_TICKET | constants.SSL_OP_NO_RENEGOTIATION;

export function certificatePin(pemOrDer) {
  return createHash('sha256').update(new X509Certificate(pemOrDer).raw).digest('hex');
}

function credentialOptions(credentials) {
  if (!credentials || !['key', 'cert', 'ca'].every((key) => Buffer.isBuffer(credentials[key]))) fail('INVALID_TLS_CREDENTIALS');
  if (new X509Certificate(credentials.cert).publicKey.asymmetricKeyType !== 'ed25519') fail('UNSUPPORTED_AUTHENTICATION');
  return { key: credentials.key, cert: credentials.cert, ca: credentials.ca,
    minVersion: TLS_PROFILE.version, maxVersion: TLS_PROFILE.version,
    ecdhCurve: TLS_PROFILE.group, ciphers: TLS_PROFILE.cipher, secureOptions,
    rejectUnauthorized: true };
}

function validateSocket(socket, expectedPeerPin, { client = false, serverName = undefined } = {}) {
  if (!socket.authorized || socket.destroyed || socket.isSessionReused()) fail('TRANSPORT_AUTHENTICATION_FAILED');
  if (socket.getProtocol() !== TLS_PROFILE.version || socket.alpnProtocol !== TLS_PROFILE.alpn ||
      socket.getCipher()?.standardName !== TLS_PROFILE.cipher) fail('TRANSPORT_PROFILE_MISMATCH');
  const peer = socket.getPeerCertificate();
  if (!peer.raw || certificatePin(peer.raw) !== expectedPeerPin ||
      new X509Certificate(peer.raw).publicKey.asymmetricKeyType !== 'ed25519') fail('TRANSPORT_PEER_MISMATCH');
  if (client) {
    if (tls.checkServerIdentity(serverName, peer)) fail('TRANSPORT_HOSTNAME_MISMATCH');
    // Node exposes the negotiated group on CLIENT sockets only. Fail closed if
    // this runtime cannot report it; configured preferences alone are no proof.
    if (socket.getEphemeralKeyInfo()?.name !== TLS_PROFILE.group) fail('TRANSPORT_GROUP_MISMATCH');
  }
  socket.disableRenegotiation();
}

function framed(payload) {
  const header = Buffer.alloc(4); header.writeUInt32BE(payload.length);
  return Buffer.concat([header, payload]);
}

/** One bounded payload allocation; timer starts with the FIRST frame byte. */
function readFrames(socket, onFrame, shutdown) {
  let header = Buffer.alloc(4), headerBytes = 0, payload = null, payloadBytes = 0, timer = null;
  const clear = () => { clearTimeout(timer); timer = null; payload = null; };
  socket.once('close', clear);
  socket.on('data', (chunk) => {
    let cursor = 0;
    while (cursor < chunk.length && !socket.destroyed) {
      if (!timer) timer = setTimeout(() => shutdown('TRANSPORT_FRAME_TIMEOUT'), TLS_PROFILE.frameTimeoutMs);
      if (!payload) {
        const length = Math.min(4 - headerBytes, chunk.length - cursor);
        chunk.copy(header, headerBytes, cursor, cursor + length); headerBytes += length; cursor += length;
        if (headerBytes !== 4) continue;
        const size = header.readUInt32BE();
        if (size < 1 || size > TLS_PROFILE.maxFrameBytes) { shutdown('TRANSPORT_FRAME_LIMIT'); return; }
        payload = Buffer.alloc(size); payloadBytes = 0;
      }
      const length = Math.min(payload.length - payloadBytes, chunk.length - cursor);
      chunk.copy(payload, payloadBytes, cursor, cursor + length); payloadBytes += length; cursor += length;
      if (payloadBytes === payload.length) {
        const complete = payload;
        clear(); headerBytes = 0; payloadBytes = 0;
        try { onFrame(complete); } catch { shutdown('TRANSPORT_INVALID_FRAME'); return; }
      }
    }
  });
}

/** Real TLS server, bound ONLY to 127.0.0.1. Echoes validated ciphertext in RAM. */
export async function createLoopbackNasServer({ credentials, allowedClientPin, namespace }) {
  requireId(allowedClientPin, true); requireId(namespace, true);
  const sockets = new Set();
  const counters = { connections: 0, authenticatedConnections: 0, acceptedFrames: 0,
    acceptedCiphertextBytes: 0, rejectedConnections: 0, closed: false };
  const server = tls.createServer({ ...credentialOptions(credentials), requestCert: true,
    handshakeTimeout: TLS_PROFILE.handshakeTimeoutMs,
    ALPNCallback: ({ protocols }) => protocols.includes(TLS_PROFILE.alpn) ? TLS_PROFILE.alpn : undefined }, (socket) => {
    let frames = 0;
    const shutdown = () => { socket.destroy(); };
    socket.on('error', shutdown);
    try { validateSocket(socket, allowedClientPin); }
    catch { counters.rejectedConnections++; shutdown(); return; }
    counters.authenticatedConnections++;
    readFrames(socket, (bytes) => {
      if (++frames > TLS_PROFILE.maxFrames) { shutdown(); return; }
      const envelope = parseEnvelope(bytes);
      if (envelope.namespace !== namespace) { shutdown(); return; }
      counters.acceptedFrames++; counters.acceptedCiphertextBytes += bytes.length;
      // The host never receives a document key or decrypts the envelope. This
      // echo is not a store acknowledgement, synchronization or disk durability.
      socket.write(framed(bytes));
    }, shutdown);
  });
  server.maxConnections = TLS_PROFILE.maxConnections;
  server.on('connection', (socket) => {
    sockets.add(socket); counters.connections++;
    const absoluteDeadline = setTimeout(() => socket.destroy(), TLS_PROFILE.connectionTimeoutMs);
    socket.once('close', () => { sockets.delete(socket); clearTimeout(absoluteDeadline); });
    socket.on('error', () => socket.destroy());
  });
  // No session cache is populated and no ticket is accepted as authority.
  server.on('newSession', (_id, _data, done) => done());
  server.on('resumeSession', (_id, done) => done(null, null));
  server.on('tlsClientError', (_error, socket) => { counters.rejectedConnections++; socket.destroy(); });
  await new Promise((resolve, reject) => {
    server.once('error', reject);
    server.listen({ host: TLS_PROFILE.host, port: 0, exclusive: true }, resolve);
  });
  // Later errors close this disposable experiment instead of escaping as an
  // unhandled event or keeping a partially functioning authenticated service.
  server.on('error', () => { for (const socket of sockets) socket.destroy(); });
  const port = server.address().port;
  let closing;
  return Object.freeze({ port,
    evidence: () => Object.freeze({ ...counters, activeConnections: sockets.size,
      hostLoopbackOnly: true, configuredGroup: TLS_PROFILE.group,
      peerAuthentication: 'classical-ed25519-certificate', contentKeysOnServer: false }),
    close() {
      if (!closing) {
        counters.closed = true;
        for (const socket of sockets) socket.destroy();
        closing = new Promise((resolve) => server.close(resolve));
      }
      return closing;
    },
  });
}

/**
 * This adapter owns the real socket and opaque policy fact; neither is exported.
 * networkScope/localNetworkId are DECLARED harness inputs, not LAN/WAN detection.
 */
export async function connectLoopbackNas({ port, credentials, expectedPeerPin, serverName,
  engine, app, networkScope, localNetworkId = null }) {
  requireId(expectedPeerPin, true);
  if (!Number.isInteger(port) || port < 1 || port > 65535 || typeof serverName !== 'string' ||
      !/^[a-z0-9][a-z0-9.-]{0,252}$/.test(serverName)) fail('INVALID_TRANSPORT_DESTINATION');
  const preflight = engine.authorizeNasConnection({ app, peerIdentity: expectedPeerPin,
    networkScope, localNetworkId, route: 'direct' });
  if (!preflight.allowed) fail(preflight.reason);
  const options = credentialOptions(credentials);
  return new Promise((resolve, reject) => {
    let fact, pending, established = false, closed = false, sentFrames = 0;
    const socket = tls.connect({ ...options, host: TLS_PROFILE.host, port, servername: serverName,
      ALPNProtocols: [TLS_PROFILE.alpn],
      // Retain normal CA + hostname validation AND bind the certificate pin.
      checkServerIdentity: (hostname, certificate) => tls.checkServerIdentity(hostname, certificate) ||
        (certificate.raw && certificatePin(certificate.raw) === expectedPeerPin ? undefined : new NasError('TRANSPORT_PEER_MISMATCH')) });
    const connectTimer = setTimeout(() => shutdown('TRANSPORT_HANDSHAKE_TIMEOUT'), TLS_PROFILE.handshakeTimeoutMs);
    const lifetime = setTimeout(() => shutdown('TRANSPORT_LIFETIME_LIMIT'), TLS_PROFILE.connectionTimeoutMs);
    function shutdown(code = 'TRANSPORT_CLOSED') {
      if (closed) return;
      closed = true;
      clearTimeout(connectTimer); clearTimeout(lifetime);
      if (fact) engine.authority.forgetTransport(fact);
      if (pending) { clearTimeout(pending.timer); pending.reject(new NasError(code)); pending = null; }
      if (!established) reject(new NasError(code));
      socket.destroy();
    }
    function authorize() {
      if (closed || socket.destroyed) fail('TRANSPORT_CLOSED');
      const decision = engine.check({ app, resource: 'nas', transport: fact });
      if (!decision.allowed) { shutdown(decision.reason); fail(decision.reason); }
    }
    socket.on('error', () => shutdown('TRANSPORT_AUTHENTICATION_FAILED'));
    socket.once('end', () => shutdown()); socket.once('close', () => shutdown());
    socket.once('secureConnect', () => {
      try {
        validateSocket(socket, expectedPeerPin, { client: true, serverName });
        fact = engine.authority.attestVerifiedTransport({ app, peerIdentity: expectedPeerPin,
          networkScope, localNetworkId, route: 'direct' });
        authorize();
        const evidence = Object.freeze({ hostLoopbackOnly: true, protocol: socket.getProtocol(),
          group: socket.getEphemeralKeyInfo().name, cipher: socket.getCipher().standardName,
          alpn: socket.alpnProtocol, sessionReused: socket.isSessionReused(),
          peerAuthentication: 'classical-ed25519-certificate', peerIdentity: expectedPeerPin,
          networkScope, networkScopeSource: 'explicit-loopback-harness-assumption' });
        readFrames(socket, (bytes) => {
          authorize();
          if (!pending) { shutdown('TRANSPORT_UNEXPECTED_RESPONSE'); return; }
          parseEnvelope(bytes);
          const operation = pending; pending = null; clearTimeout(operation.timer);
          if (!bytes.equals(operation.bytes)) { operation.reject(new NasError('TRANSPORT_RESPONSE_MISMATCH')); shutdown(); return; }
          operation.resolve(bytes);
        }, shutdown);
        clearTimeout(connectTimer); established = true;
        resolve(Object.freeze({
          evidence: () => Object.freeze({ ...evidence, closed, sentFrames }),
          async sendCiphertext(bytes) {
            authorize();
            parseEnvelope(bytes);
            if (pending) fail('TRANSPORT_BUSY');
            if (sentFrames >= TLS_PROFILE.maxFrames) { shutdown('TRANSPORT_FRAME_LIMIT'); fail('TRANSPORT_FRAME_LIMIT'); }
            const snapshot = Buffer.from(bytes); sentFrames++;
            return new Promise((resolveFrame, rejectFrame) => {
              pending = { bytes: snapshot, resolve: resolveFrame, reject: rejectFrame,
                timer: setTimeout(() => shutdown('TRANSPORT_RESPONSE_TIMEOUT'), TLS_PROFILE.frameTimeoutMs) };
              socket.write(framed(snapshot), (error) => { if (error) shutdown('TRANSPORT_WRITE_FAILED'); });
            });
          },
          close: () => shutdown(),
        }));
      } catch { shutdown('TRANSPORT_AUTHENTICATION_FAILED'); }
    });
  });
}
