import { createFixtureState, COLLECTIONS, NAS_IDENTITY } from './fixtures.js';
import * as broker from './broker.js';

export const selectedObject = (state) => state.objects.find((object) => object.id === state.selectedObjectId);
export const currentVersion = (object) => object?.versions.at(-1);
export const activeGrants = (state) => state.grants.filter((grant) => grant.state === 'active');

const errors = {
  DENIED: 'Geen toegang. Vraag opnieuw toestemming voor dit document.',
  NOT_FOUND: 'Dit voorbeeld is niet gevonden. Ga terug naar Mijn materiaal.',
  STALE_REVISION: 'Het document heeft een nieuwere versie. Je concept blijft bewaard; sluit het document en open het opnieuw.',
  NAS_UNAVAILABLE: 'De eigen NAS is niet beschikbaar. Je kunt lokaal verder werken.',
  IDENTITY_MISMATCH: 'De identiteit is niet bevestigd. Synchronisatie is gestopt; controleer de verbinding opnieuw.',
  MODEL_MISSING: 'Er is geen lokaal model beschikbaar. Je kunt zonder assistent verder werken.',
  RESOURCE_LIMIT: 'Onvoldoende lokale capaciteit. Je kunt zonder assistent verder werken.',
  INVALID_PACKAGE: 'De voorbeeldhandtekening klopt niet. Het pakket is niet geïnstalleerd.',
};

function notify(state, text, kind = 'success') { state.message = { text, kind }; }
function checked(state, result) {
  if (!result.ok) notify(state, errors[result.code] || 'De handeling kon niet worden uitgevoerd.', 'error');
  return result.ok;
}

function endObjectSession(state) {
  state.grants = state.grants.map((grant) => grant.state === 'active' ? { ...grant, state: 'expired' } : grant);
  state.selectedObjectId = null;
  state.grantRequest = null;
  state.activeGrantId = null;
  state.mode = 'read';
  state.restoreRevision = null;
}

function appendRevision(state, object, version) {
  object.versions.push(version);
  object.revision = version.id;
  object.localState = 'saved';
  // Local save never claims the NAS acknowledged a copy.
  object.nasState = state.nas.state === 'unpaired' ? 'local' : 'pending';
  notify(state, `Versie ${version.id} · Op dit apparaat opgeslagen.`);
}

export function reducer(previous, event) {
  if (event.type === 'openObject' && previous.selectedObjectId === event.id) return previous;
  const state = structuredClone(previous);
  state.message = null;
  switch (event.type) {
    case 'onboardingStep': state.onboarding.step = Math.max(1, Math.min(3, event.step)); break;
    case 'profileName': state.onboarding.name = String(event.name).trim().slice(0, 40) || 'Lokaal profiel'; break;
    case 'reducedMotion': state.onboarding.reducedMotion = Boolean(event.value); break;
    case 'finishOnboarding': state.onboarding.completed = true; state.onboarding.step = 3; break;
    case 'openObject': {
      const result = broker.openObject(state, event.id);
      if (!checked(state, result)) { endObjectSession(state); break; }
      if (state.selectedObjectId !== event.id) endObjectSession(state);
      state.selectedObjectId = event.id;
      state.lastOpenedId = event.id;
      break;
    }
    case 'closeObject': endObjectSession(state); break;
    case 'newDocument': {
      endObjectSession(state);
      state.counter += 1;
      const id = `obj-new-${state.counter}`;
      state.objects.push({ id, title: `Nieuw document ${state.counter}`, mediaType: 'text', collectionId: 'Vandaag', revision: 1, localState: 'saved', nasState: 'local', versions: [{ id: 1, label: 'Nieuw document', text: '' }] });
      state.selectedObjectId = id; state.lastOpenedId = id;
      notify(state, 'Nieuw document op dit apparaat. Kies Bewerken om te beginnen.');
      break;
    }
    case 'requestEdit': {
      const object = selectedObject(state);
      if (!object) { checked(state, { ok: false, code: 'NOT_FOUND' }); break; }
      const internet = event.toolId === 'webvenster' ? 'demo.invalid' : false;
      const result = broker.requestGrant(state, { toolId: event.toolId || 'notities', objectIds: [object.id], operations: internet ? ['read'] : ['read', 'write'], lease: 'session', internet });
      if (checked(state, result)) { state.grantRequest = result.value; state.lastGrantState = 'pending'; }
      break;
    }
    case 'grantSession': {
      const keepEditorGrant = broker.authorize(state, state.activeGrantId, state.selectedObjectId, 'write').ok;
      const result = broker.grant(state);
      if (!checked(state, result)) break;
      state.counter += 1;
      state.grants.push(result.value);
      if (result.value.operations.includes('write') || !keepEditorGrant) state.activeGrantId = result.value.id;
      state.grantRequest = null; state.lastGrantState = 'allowed';
      const object = selectedObject(state);
      if (result.value.operations.includes('write')) {
        state.mode = 'edit';
        if (state.draft?.objectId !== object.id) state.draft = { objectId: object.id, text: currentVersion(object).text, baseRevision: object.revision, cursor: 0 };
      } else if (!keepEditorGrant) {
        state.mode = 'read';
      }
      notify(state, `${state.tools.find((tool) => tool.id === result.value.toolId).name} · alleen dit document. Toegang stopt als je het sluit.`);
      break;
    }
    case 'denyGrant': broker.deny(); state.grantRequest = null; state.lastGrantState = 'denied'; notify(state, 'Toegang geweigerd. Je document is ongewijzigd.', 'info'); break;
    case 'revokeGrant': {
      const result = broker.revoke(state, event.id);
      if (!checked(state, result)) break;
      state.grants = state.grants.map((grant) => grant.id === event.id ? { ...grant, state: 'revoked' } : grant);
      if (state.activeGrantId === event.id) { state.activeGrantId = null; state.mode = 'read'; }
      state.lastGrantState = 'revoked';
      notify(state, 'Toegang ingetrokken. Nieuwe bewerkingen zijn geblokkeerd; eerder gemaakte kopieën worden niet gewist.', 'info');
      break;
    }
    case 'updateDraft': {
      if (state.mode !== 'edit' || !state.draft || !broker.authorize(state, state.activeGrantId, state.selectedObjectId, 'write').ok) break;
      state.draft.text = String(event.text).slice(0, 20000);
      state.draft.cursor = Math.max(0, Math.min(event.cursor || 0, state.draft.text.length));
      break;
    }
    case 'beginSave': if (state.mode === 'edit') state.mode = 'saving'; break;
    case 'saveDraft': {
      if (!state.draft) break;
      const result = broker.saveRevision(state, { ...state.draft, grantId: state.activeGrantId });
      state.mode = broker.authorize(state, state.activeGrantId, state.selectedObjectId, 'write').ok ? 'edit' : 'read';
      if (!checked(state, result)) break;
      const object = selectedObject(state);
      appendRevision(state, object, result.value);
      state.draft.baseRevision = object.revision;
      break;
    }
    case 'previewRestore': {
      const object = selectedObject(state);
      if (object?.versions.some((version) => version.id === event.revision)) { state.restoreRevision = event.revision; state.mode = 'preview'; }
      break;
    }
    case 'cancelRestore': state.restoreRevision = null; state.mode = 'read'; break;
    case 'restoreRevision': {
      const object = selectedObject(state);
      if (!object) break;
      const result = broker.restoreAsNewRevision(state, { objectId: object.id, revision: state.restoreRevision, confirmed: true, baseRevision: event.baseRevision });
      if (!checked(state, result)) break;
      appendRevision(state, object, result.value); state.mode = 'read'; state.restoreRevision = null;
      state.draft = null;
      break;
    }
    case 'pairNas': { const result = broker.pairNas(state); if (checked(state, result)) state.nas = result.value; break; }
    case 'confirmNas': {
      const result = broker.pairNas(state, { confirmed: event.confirmed, identity: event.identity });
      if (checked(state, result)) { state.nas = result.value; notify(state, 'Thuis-NAS gekoppeld in deze simulatie. Er zijn geen bestanden verstuurd.'); }
      break;
    }
    case 'nasCondition': {
      const result = broker.setNasCondition(state, event.condition);
      if (checked(state, result)) { state.nas = result.value; notify(state, event.condition === 'online' ? 'De voorbeeldverbinding is hersteld.' : 'NAS-status gewijzigd. Lokaal werken blijft mogelijk.', 'info'); }
      break;
    }
    case 'beginInstall': state.install = { state: 'checking', toolId: event.id }; break;
    case 'installFixture': {
      const result = broker.installFixture(state, event.id);
      state.install = { state: result.ok ? 'installed' : 'error', toolId: event.id };
      if (checked(state, result)) { state.tools.find((tool) => tool.id === event.id).installed = true; notify(state, 'Beeldbewerker geïnstalleerd in de demo. Er zijn geen toegangsrechten verleend.'); }
      break;
    }
    case 'removeTool': {
      const tool = state.tools.find((item) => item.id === event.id && item.removable);
      if (!tool) break;
      tool.installed = false; state.install = { state: 'removed', toolId: tool.id };
      state.grants = state.grants.map((grant) => grant.toolId === tool.id ? { ...grant, state: 'revoked' } : grant);
      if (!broker.authorize(state, state.activeGrantId, state.selectedObjectId, 'write').ok) { state.mode = 'read'; state.activeGrantId = null; }
      notify(state, 'Voorbeeldpakket verwijderd. Je materiaal blijft bewaard.'); break;
    }
    case 'invalidPackage': state.conditions.invalidPackage = Boolean(event.value); break;
    case 'aiCondition': {
      if (!['ready', 'missingModel', 'lowMemory'].includes(event.condition)) break;
      state.ai.availability = event.condition; state.ai.state = event.condition;
      state.ai.response = ''; state.ai.runId += 1; break;
    }
    case 'aiContext': {
      if (!state.objects.some((object) => object.id === event.id)) break;
      const context = new Set(state.ai.context);
      if (event.selected) context.add(event.id); else context.delete(event.id);
      state.ai.context = [...context].slice(0, 3); state.ai.response = '';
      state.ai.state = state.ai.availability; state.ai.runId += 1; break;
    }
    case 'startSummary': {
      const result = broker.summarizeFixture(state, state.ai.context);
      if (checked(state, result)) { state.ai.state = 'running'; state.ai.response = ''; state.ai.runId += 1; }
      break;
    }
    case 'finishSummary': {
      if (state.ai.state !== 'running' || state.ai.runId !== event.runId) break;
      const result = broker.summarizeFixture(state, state.ai.context);
      if (checked(state, result)) { state.ai.state = 'result'; state.ai.response = result.value; notify(state, 'Voorbeeldsamenvatting gereed. Dit is een vaste simulatie, geen modeluitvoer.'); }
      break;
    }
    case 'cancelSummary': state.ai.state = 'cancelled'; state.ai.runId += 1; state.ai.response = ''; notify(state, 'Samenvatting geannuleerd.', 'info'); break;
    case 'clearMessage': state.message = null; break;
    default: return previous;
  }
  return state;
}

// Refresh resumes only local fixture content, never an active grant or pending task.
export function resumeState(serialized) {
  try {
    const stored = JSON.parse(serialized);
    if (stored?.schemaVersion !== 1 || !Array.isArray(stored.objects) || !stored.objects.length || stored.objects.length > 100
      || !stored.onboarding || !stored.ai || !stored.nas || !Array.isArray(stored.tools)) return createFixtureState();
    for (const object of stored.objects) {
      if (typeof object.id !== 'string' || !/^obj-(?:\d{3}|new-\d+)$/.test(object.id)
        || typeof object.title !== 'string' || object.title.length > 200 || !COLLECTIONS.includes(object.collectionId)
        || !['text', 'image'].includes(object.mediaType) || !Array.isArray(object.versions) || !object.versions.length
        || object.versions.length > 500 || !Number.isSafeInteger(object.revision)
        || object.versions.some((version, index) => version.id !== index + 1 || typeof version.text !== 'string'
          || version.text.length > 20000 || typeof version.label !== 'string' || version.label.length > 200)
        || object.versions.at(-1).id !== object.revision) return createFixtureState();
    }
    if (new Set(stored.objects.map((object) => object.id)).size !== stored.objects.length) return createFixtureState();
    const fresh = createFixtureState();
    // Build a canonical state; never copy markup-bearing fields or tool permissions
    // wholesale from storage. Storage is editable by the browser user/extensions.
    const state = fresh;
    state.objects = stored.objects.map((object) => ({
      id: object.id, title: object.title, collectionId: object.collectionId, mediaType: object.mediaType,
      revision: object.revision, localState: 'saved', nasState: object.nasState === 'pending' ? 'pending' : 'local',
      versions: object.versions.map((version) => ({ id: version.id, text: version.text, label: version.label })),
    }));
    state.counter = Number.isSafeInteger(stored.counter) && stored.counter >= 0 && stored.counter < 1000000 ? stored.counter : 0;
    state.onboarding = {
      completed: stored.onboarding.completed === true,
      step: [1, 2, 3].includes(stored.onboarding.step) ? stored.onboarding.step : 1,
      name: typeof stored.onboarding.name === 'string' ? stored.onboarding.name.slice(0, 40) : 'Lokaal profiel',
      reducedMotion: stored.onboarding.reducedMotion === true,
    };
    state.tools = fresh.tools.map((tool) => ({ ...tool, installed: !tool.removable || stored.tools.find((entry) => entry?.id === tool.id)?.installed === true }));
    const hasObject = (id) => state.objects.some((object) => object.id === id);
    state.lastOpenedId = hasObject(stored.lastOpenedId) ? stored.lastOpenedId : state.objects[0].id;
    if (stored.draft && hasObject(stored.draft.objectId) && typeof stored.draft.text === 'string' && stored.draft.text.length <= 20000
      && Number.isSafeInteger(stored.draft.baseRevision) && stored.draft.baseRevision > 0) {
      state.draft = { objectId: stored.draft.objectId, text: stored.draft.text, baseRevision: stored.draft.baseRevision,
        cursor: Number.isSafeInteger(stored.draft.cursor) ? Math.max(0, Math.min(stored.draft.cursor, stored.draft.text.length)) : 0 };
    }
    // Retain only bounded historical grants, with all active leases expired.
    state.grants = Array.isArray(stored.grants) ? stored.grants.filter((grant) => grant && /^grant-\d+$/.test(grant.id)
      && state.tools.some((tool) => tool.id === grant.toolId) && Array.isArray(grant.objectIds) && grant.objectIds.length === 1
      && hasObject(grant.objectIds[0]) && Array.isArray(grant.operations) && grant.operations.every((operation) => ['read', 'write'].includes(operation)))
      .slice(-500).map((grant) => ({ id: grant.id, toolId: grant.toolId, objectIds: [...grant.objectIds], operations: [...grant.operations],
        lease: 'session', internet: grant.toolId === 'webvenster' && grant.internet === 'demo.invalid' ? 'demo.invalid' : false,
        expiresOn: 'objectClose', state: grant.state === 'revoked' ? 'revoked' : 'expired' })) : [];
    const nasCondition = ['unpaired', 'offline', 'networkChanged', 'identityMismatch', 'online'].includes(stored.nas.state) ? stored.nas.state : 'unpaired';
    state.nas = { ...fresh.nas, state: nasCondition === 'online' && stored.nas.identity !== NAS_IDENTITY ? 'unpaired' : nasCondition,
      identity: stored.nas.identity === NAS_IDENTITY ? NAS_IDENTITY : null };
    const availability = ['ready', 'missingModel', 'lowMemory'].includes(stored.ai.availability) ? stored.ai.availability : 'ready';
    state.ai = { ...fresh.ai, availability, state: availability,
      context: Array.isArray(stored.ai.context) ? [...new Set(stored.ai.context.filter(hasObject))].slice(0, 3) : [state.objects[0].id] };
    state.conditions.invalidPackage = stored.conditions?.invalidPackage === true;
    return state;
  } catch { return createFixtureState(); }
}
