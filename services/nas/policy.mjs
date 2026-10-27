import { fail, requireId, NAMESPACE } from './errors.mjs';

const allowedKinds = new Set(['app', 'browser', 'nas', 'ai', 'model-downloader', 'update-downloader']);
const networkResources = new Set(['internet', 'lan', 'nas']);
const deny = (reason) => Object.freeze({ allowed: false, reason });
const allow = () => Object.freeze({ allowed: true, reason: 'EXPLICIT_GRANT' });
const pauseRoute = (reason) => Object.freeze({ status: 'paused', route: null, transport: null, reason });

/**
 * Host-only authorization model. It creates NO sockets and authenticates NO TLS.
 * Keep `authority` in a future trusted OS/transport/UI adapter. Give apps only
 * their own opaque app handle and policy decisions. A JSON { verified: true } is no fact.
 */
export function createPolicyEngine() {
  const apps = new WeakMap();
  const transportFacts = new WeakMap();
  function appRecord(app) {
    const record = (typeof app === 'object' && app !== null) ? apps.get(app) : undefined;
    if (!record || !record.installed) fail('UNKNOWN_APP');
    return record;
  }
  function peer(value) {
    if (typeof value !== 'string' || !NAMESPACE.test(value)) fail('INVALID_PEER');
    return value;
  }
  function networkGrantable(record) { if (record.kind === 'ai') fail('AI_NETWORK_DENIED'); }
  function nasRouteDecision(record, { peerIdentity, networkScope, localNetworkId, route }) {
    if (record.kind === 'ai') return deny('AI_NETWORK_DENIED');
    if (record.kind !== 'nas') return deny('NAS_DENIED');
    if (typeof peerIdentity !== 'string' || !NAMESPACE.test(peerIdentity)) return deny('INVALID_PEER');
    if (!['lan', 'internet'].includes(networkScope)) return deny('INVALID_NETWORK_SCOPE');
    if (route !== 'direct') return deny('RELAY_OR_REDIRECT_DENIED');
    if (!record.peers.has(peerIdentity)) return deny('UNPINNED_PEER');
    if (networkScope === 'lan') {
      return record.peers.get(peerIdentity) === localNetworkId ? allow() : deny('LOCAL_NETWORK_CHANGED');
    }
    return record.internetPeers.has(peerIdentity) ? allow() : deny('PEER_INTERNET_NOT_GRANTED');
  }

  // Permission to ATTEMPT a connection, not evidence that a peer is authenticated.
  // A trusted adapter calls this before opening a socket and must still validate TLS,
  // mint a live transport fact, and call check() before every application write.
  function authorizeNasConnection({ app, peerIdentity, networkScope, localNetworkId = null, route = 'direct' } = {}) {
    let record;
    try { record = appRecord(app); } catch { return deny('UNKNOWN_APP'); }
    return nasRouteDecision(record, { peerIdentity, networkScope, localNetworkId, route });
  }

  const authority = Object.freeze({
    installApp({ appId, kind = 'app' }) {
      if (typeof appId !== 'string' || !/^[a-z0-9][a-z0-9.-]{0,63}$/.test(appId) || !allowedKinds.has(kind)) fail('INVALID_APP');
      const handle = Object.freeze({});
      apps.set(handle, { appId, kind, installed: true, internet: false, documents: new Map(),
        peers: new Map(), internetPeers: new Set() });
      return handle;
    },
    uninstallApp(app) {
      const record = appRecord(app); record.installed = false; record.internet = false;
      record.documents.clear(); record.peers.clear(); record.internetPeers.clear();
    },
    // Invocation by the trusted consent adapter represents a deliberate user action.
    grantInternet(app) {
      const record = appRecord(app); networkGrantable(record);
      if (record.kind === 'nas') fail('USE_PEER_BOUND_CONSENT');
      record.internet = true;
    },
    revokeInternet(app) { const record = appRecord(app); record.internet = false; record.internetPeers.clear(); },
    grantDocument(app, objectId, mode = 'read') {
      const record = appRecord(app); requireId(objectId);
      if (['nas', 'model-downloader', 'update-downloader'].includes(record.kind) || !['read', 'write'].includes(mode)) fail('DOCUMENT_GRANT_DENIED');
      record.documents.set(objectId, mode);
    },
    revokeDocument(app, objectId) { appRecord(app).documents.delete(objectId); },
    pinNasPeer(app, { peerIdentity, localNetworkId }) {
      const record = appRecord(app);
      if (record.kind !== 'nas') fail('NAS_GRANT_DENIED');
      peer(peerIdentity);
      if (typeof localNetworkId !== 'string' || !/^[a-zA-Z0-9_-]{1,64}$/.test(localNetworkId)) fail('INVALID_NETWORK');
      record.peers.set(peerIdentity, localNetworkId);
      record.internetPeers.delete(peerIdentity); // Pin/re-pin never implicitly restores internet consent.
    },
    grantNasDirectInternet(app, peerIdentity) {
      const record = appRecord(app); peer(peerIdentity);
      if (record.kind !== 'nas' || !record.peers.has(peerIdentity)) fail('NAS_NOT_PAIRED');
      record.internetPeers.add(peerIdentity);
    },
    revokeNasPeer(app, peerIdentity) {
      const record = appRecord(app); record.peers.delete(peerIdentity); record.internetPeers.delete(peerIdentity);
    },
    // Only a trusted future transport adapter may mint this after REAL validation.
    // In N01 tests these are declared assumptions, never measured TLS evidence.
    attestVerifiedTransport({ peerIdentity, networkScope, localNetworkId = null, route = 'direct' }) {
      peer(peerIdentity);
      if (!['lan', 'internet'].includes(networkScope) || !['direct', 'relay', 'redirect'].includes(route)) fail('INVALID_TRANSPORT_FACT');
      if (networkScope === 'lan' && (typeof localNetworkId !== 'string' || !/^[a-zA-Z0-9_-]{1,64}$/.test(localNetworkId))) fail('INVALID_NETWORK');
      const fact = Object.freeze({});
      transportFacts.set(fact, { peerIdentity, networkScope, localNetworkId, route });
      return fact;
    },
    forgetTransport(fact) { transportFacts.delete(fact); },
  });

  function check({ app, resource, objectId = undefined, mode = 'read', transport = undefined }) {
    let record;
    try { record = appRecord(app); } catch { return deny('UNKNOWN_APP'); }
    if (record.kind === 'ai' && networkResources.has(resource)) return deny('AI_NETWORK_DENIED');
    if (resource === 'documents') {
      const grant = record.documents.get(objectId);
      return (mode === 'read' && ['read', 'write'].includes(grant)) || (mode === 'write' && grant === 'write') ? allow() : deny('DOCUMENT_DENIED');
    }
    if (resource === 'internet') {
      if (record.kind === 'nas') return deny('USE_NAS_PEER_POLICY');
      return record.internet ? allow() : deny('INTERNET_NOT_GRANTED');
    }
    if (resource === 'nas') {
      if (record.kind !== 'nas') return deny('NAS_DENIED');
      const fact = typeof transport === 'object' && transport !== null ? transportFacts.get(transport) : undefined;
      if (!fact) return deny('UNVERIFIED_TRANSPORT');
      return nasRouteDecision(record, fact);
    }
    // General LAN, admin and unknown rights have no implicit route through this API.
    return deny('DEFAULT_DENY');
  }
  // Pure route selection: consume existing consent, never grant or reconnect by itself.
  // Call again when reachability or rights change; an old result is not a capability.
  function planNasRoute({ app, peerIdentity, transports = [] } = {}) {
    let record;
    try { record = appRecord(app); } catch { return pauseRoute('UNKNOWN_APP'); }
    if (record.kind === 'ai') return pauseRoute('AI_NETWORK_DENIED');
    if (record.kind !== 'nas') return pauseRoute('NAS_DENIED');
    if (typeof peerIdentity !== 'string' || !NAMESPACE.test(peerIdentity)) return pauseRoute('INVALID_PEER');
    if (!record.peers.has(peerIdentity)) return pauseRoute('NAS_NOT_PAIRED');
    if (!Array.isArray(transports) || transports.length > 64) return pauseRoute('INVALID_ROUTE_SET');
    let directInternet = null;
    let internetNeedsPermission = false;
    for (const transport of transports) {
      const fact = typeof transport === 'object' && transport !== null ? transportFacts.get(transport) : undefined;
      // Never switch to another peer, even if that peer has a separate valid grant.
      if (!fact || fact.peerIdentity !== peerIdentity) continue;
      const decision = check({ app, resource: 'nas', transport });
      if (!decision.allowed) {
        if (decision.reason === 'PEER_INTERNET_NOT_GRANTED') internetNeedsPermission = true;
        continue;
      }
      const selected = Object.freeze({ status: 'ready', route: fact.networkScope, transport,
        reason: fact.networkScope === 'lan' ? 'VERIFIED_LAN' : 'VERIFIED_DIRECT_INTERNET' });
      if (fact.networkScope === 'lan') return selected;
      directInternet ??= selected;
    }
    return directInternet ?? pauseRoute(internetNeedsPermission ? 'PEER_INTERNET_NOT_GRANTED' : 'NO_VERIFIED_ROUTE');
  }
  return Object.freeze({ authority, check, planNasRoute, authorizeNasConnection });
}
