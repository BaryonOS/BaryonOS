// This broker is a deterministic UI mock, NOT an OS security boundary.
// Every protected mock operation rechecks its grant, scope and lifetime.
import { NAS_IDENTITY } from './fixtures.js';

const ok = (value = null) => ({ ok: true, value });
const fail = (code) => ({ ok: false, code });
const objectById = (state, id) => state.objects.find((object) => object.id === id);
const toolById = (state, id) => state.tools.find((tool) => tool.id === id);

export function listObjects(state, { query = '', collection = 'Alles' } = {}) {
  const needle = String(query).trim().toLocaleLowerCase('nl');
  return ok(state.objects.filter((object) =>
    (collection === 'Alles' || object.collectionId === collection)
    && (!needle || `${object.title} ${object.collectionId} ${object.mediaType === 'image' ? 'Afbeelding SVG' : 'Tekstdocument tekst'}`.toLocaleLowerCase('nl').includes(needle))));
}

export function openObject(state, id) {
  const object = objectById(state, id);
  return object ? ok(object) : fail('NOT_FOUND');
}

export function requestGrant(state, request) {
  const tool = toolById(state, request?.toolId);
  if (!tool?.installed || !Array.isArray(request?.objectIds) || request.objectIds.length !== 1
    || !Array.isArray(request.operations) || request.operations.length === 0
    || request.lease !== 'session') return fail('DENIED');
  const object = objectById(state, request.objectIds[0]);
  if (!object) return fail('NOT_FOUND');
  if (state.selectedObjectId !== object.id || !tool.types.includes(object.mediaType)) return fail('DENIED');
  if (request.operations.some((operation) => !['read', 'write'].includes(operation))) return fail('DENIED');
  if (request.internet !== false && !(tool.id === 'webvenster' && request.internet === 'demo.invalid')) return fail('DENIED');
  if (tool.id === 'webvenster' && request.operations.includes('write')) return fail('DENIED');
  return ok({ ...request, objectIds: [...request.objectIds], operations: [...new Set(request.operations)] });
}

export function grant(state) {
  const checked = requestGrant(state, state.grantRequest);
  if (!checked.ok) return checked;
  return ok({ ...checked.value, id: `grant-${state.counter + 1}`, expiresOn: 'objectClose', state: 'active' });
}

export const deny = () => ok();

export function revoke(state, grantId) {
  return state.grants.some((entry) => entry.id === grantId) ? ok(grantId) : fail('NOT_FOUND');
}

export function authorize(state, grantId, objectId, operation) {
  const current = state.grants.find((entry) => entry.id === grantId);
  if (!current || current.state !== 'active' || current.expiresOn !== 'objectClose'
    || state.selectedObjectId !== objectId || !current.objectIds.includes(objectId)
    || !current.operations.includes(operation) || !toolById(state, current.toolId)?.installed) return fail('DENIED');
  return ok(current);
}

export function saveRevision(state, { objectId, grantId, baseRevision, text }) {
  if (!authorize(state, grantId, objectId, 'write').ok) return fail('DENIED');
  const object = objectById(state, objectId);
  if (!object) return fail('NOT_FOUND');
  if (object.revision !== baseRevision) return fail('STALE_REVISION');
  if (typeof text !== 'string' || text.length > 20000) return fail('RESOURCE_LIMIT');
  return ok({ id: object.revision + 1, label: 'Bewerking opgeslagen', text });
}

export function restoreAsNewRevision(state, { objectId, revision, confirmed, baseRevision }) {
  const object = objectById(state, objectId);
  if (!object) return fail('NOT_FOUND');
  // Explicit object-owner action in this mock; no tool is granted write access.
  if (!confirmed || state.selectedObjectId !== objectId || state.restoreRevision !== revision) return fail('DENIED');
  if (object.revision !== baseRevision) return fail('STALE_REVISION');
  const old = object.versions.find((version) => version.id === revision);
  return old ? ok({ id: object.revision + 1, label: `Hersteld uit versie ${revision}`, text: old.text }) : fail('NOT_FOUND');
}

export function pairNas(state, { confirmed = false, identity = null } = {}) {
  if (!confirmed) return ok({ ...state.nas, state: 'verifying', internetIdentity: null });
  if (state.nas.state !== 'verifying' || identity !== NAS_IDENTITY) return fail('IDENTITY_MISMATCH');
  return ok({ ...state.nas, state: 'online', identity: NAS_IDENTITY, network: 'lan', internetIdentity: null });
}

export function setNasCondition(state, condition) {
  if (!['offline', 'networkChanged', 'identityMismatch', 'online', 'unpaired'].includes(condition)) return fail('DENIED');
  if (condition === 'online' && state.nas.identity !== NAS_IDENTITY) return fail('NAS_UNAVAILABLE');
  if (condition === 'online' && state.nas.network === 'internet' && state.nas.internetIdentity !== NAS_IDENTITY) return fail('DENIED');
  if (condition === 'online' && ['identityMismatch', 'networkChanged'].includes(state.nas.state)) return fail('IDENTITY_MISMATCH');
  const canReconnectRemotely = state.nas.identity === NAS_IDENTITY && state.nas.internetIdentity === NAS_IDENTITY;
  const clearIdentity = ['unpaired', 'identityMismatch'].includes(condition) || condition === 'networkChanged' && !canReconnectRemotely;
  const identity = clearIdentity ? null : state.nas.identity;
  return ok({ ...state.nas, state: condition, identity, internetIdentity: clearIdentity ? null : state.nas.internetIdentity });
}

export function setNasInternet(state, { identity, allowed } = {}) {
  if ((allowed === true && state.nas.state !== 'online') || state.nas.identity !== NAS_IDENTITY || identity !== state.nas.identity) return fail('IDENTITY_MISMATCH');
  if (allowed !== true && allowed !== false) return fail('DENIED');
  return ok({ ...state.nas, internetIdentity: allowed ? identity : null,
    state: !allowed && state.nas.network === 'internet' ? 'offline' : state.nas.state });
}

export function connectNasInternet(state, identity) {
  if (!['online', 'offline', 'networkChanged'].includes(state.nas.state) || identity !== NAS_IDENTITY
    || state.nas.identity !== identity || state.nas.internetIdentity !== identity) return fail('DENIED');
  // Fixture identity assertion only; this does not authenticate a real peer.
  return ok({ ...state.nas, state: 'online', network: 'internet' });
}

export function observeNasRoute(state, observation = {}) {
  // Browser fixture only: these flags stand in for an authenticated service
  // observation. They never establish pairing, internet consent or admin rights.
  const { network, identity, reachable, verified, trustedLan } = observation || {};
  const knownRoute = network === 'lan' || network === 'internet';
  const nas = { ...state.nas, ...(knownRoute ? { network } : {}) };
  if (!['online', 'offline', 'networkChanged'].includes(nas.state) || nas.identity !== NAS_IDENTITY) return ok(nas);
  // Accept the observation with a paused data state, rather than returning an
  // error that would leave an old online/LAN state and local admin session alive.
  if (!knownRoute || reachable !== true || verified !== true) return ok({ ...nas, state: 'offline' });
  if (identity !== nas.identity) return ok({ ...nas, state: 'identityMismatch', identity: null, internetIdentity: null });
  if (network === 'internet' && nas.internetIdentity !== identity
    || network === 'lan' && trustedLan !== true) return ok({ ...nas, state: 'offline' });
  return ok({ ...nas, state: 'online' });
}

export function installFixture(state, toolId) {
  const tool = toolById(state, toolId);
  if (!tool?.removable) return fail('NOT_FOUND');
  if (state.conditions.invalidPackage) return fail('INVALID_PACKAGE');
  return ok(toolId);
}

export function summarizeFixture(state, objectIds) {
  if (state.ai.availability === 'missingModel') return fail('MODEL_MISSING');
  if (state.ai.availability === 'lowMemory') return fail('RESOURCE_LIMIT');
  if (!Array.isArray(objectIds) || objectIds.length === 0 || objectIds.length > 3) return fail('DENIED');
  if (objectIds.some((id) => !objectById(state, id))) return fail('NOT_FOUND');
  // Only explicitly supplied objects are inspected. No index or network access.
  return ok(objectIds.map((id) => {
    const object = objectById(state, id);
    const text = object.versions.at(-1).text;
    const first = text.split('\n').filter(Boolean).slice(0, 2).join(' ');
    return `${object.title}: ${first}`;
  }).join('\n\n'));
}

