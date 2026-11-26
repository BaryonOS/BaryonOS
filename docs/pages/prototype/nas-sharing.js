// Deterministic product mock. No keys, cryptography, network, or real NAS files.
// Native clients/services must enforce these policies outside the browser.
export const FAMILY_NAS_IDENTITY = 'NAS-DEMO-FAMILIE-01';
export const INVITE_LIFETIME_MS = 10 * 60 * 1000;

export function createNasSharingState() {
  return {
    view: 'own', serial: 0, invitation: null, pendingRemoval: null,
    received: { state: 'none', identity: null, internetIdentity: null, quotaGiB: 25, epoch: 0 },
    hosting: { enabled: false, adminRole: 'inactive', capacityGiB: 100, guests: [] },
  };
}

const ok = (value, message) => ({ ok: true, value, message });
const fail = (code) => ({ ok: false, code });
const validTime = (now) => Number.isSafeInteger(now) && now >= 0 && now <= Number.MAX_SAFE_INTEGER - INVITE_LIFETIME_MS;
const guestById = (state, id) => state.hosting.guests.find((guest) => guest.id === id);

export function receivedNasAccess(state, { identity, network = 'lan', epoch } = {}) {
  const received = state.received;
  if (received.state !== 'paired' || epoch !== received.epoch) return fail('NAS_UNAVAILABLE');
  if (identity !== received.identity || identity !== FAMILY_NAS_IDENTITY) return fail('IDENTITY_MISMATCH');
  if (!['lan', 'internet'].includes(network)) return fail('DENIED');
  if (network === 'internet' && received.internetIdentity !== identity) return fail('REMOTE_NOT_ALLOWED');
  return ok({ identity, epoch, network });
}

export function guestMetadata(state) {
  // The management surface exposes allocation metadata only, never document names,
  // user content, plaintext keys, thumbnails or a document-opening capability.
  return state.hosting.guests.map(({ id, label, quotaGiB, usedGiB, state: status, epoch }) =>
    ({ id, label, quotaGiB, usedGiB, state: status, epoch }));
}

export function changeNasSharing(previous, action, { localAdminAvailable = false } = {}) {
  if (!action || typeof action.type !== 'string') return fail('DENIED');
  const state = structuredClone(previous);
  switch (action.type) {
    case 'view':
      if (!['own', 'receive', 'offer'].includes(action.view)) return fail('DENIED');
      state.view = action.view; state.pendingRemoval = null;
      return ok(state);
    case 'exampleInvitation':
      if (!validTime(action.now) || state.received.state === 'paired') return fail('DENIED');
      state.serial += 1;
      state.invitation = { id: `family-invite-${state.serial}`, identity: FAMILY_NAS_IDENTITY,
        expiresAt: action.now + INVITE_LIFETIME_MS, used: false };
      state.received = { state: 'verifying', identity: null, internetIdentity: null, quotaGiB: 25, epoch: state.received.epoch + 1 };
      return ok(state, 'Voorbeelduitnodiging geladen. Vergelijk de identiteit met de afzender.');
    case 'confirmInvitation': {
      const invite = state.invitation;
      if (!validTime(action.now) || !invite || invite.used || action.now >= invite.expiresAt) return fail('INVITE_EXPIRED');
      if (state.received.state !== 'verifying' || action.confirmed !== true || action.lossAccepted !== true || action.identity !== invite.identity
        || action.identity !== FAMILY_NAS_IDENTITY || action.invitationId !== invite.id) return fail('IDENTITY_MISMATCH');
      invite.used = true;
      state.received.state = 'paired'; state.received.identity = invite.identity;
      return ok(state, 'Ruimte gekoppeld in de simulatie. Internettoegang staat nog uit.');
    }
    case 'cancelInvitation':
      state.invitation = null;
      state.received = { ...state.received, state: 'none', identity: null, internetIdentity: null, epoch: state.received.epoch + 1 };
      return ok(state);
    case 'internet':
      if ((action.allowed === true && state.received.state !== 'paired') || !state.received.identity || action.identity !== state.received.identity) return fail('IDENTITY_MISMATCH');
      if (action.allowed !== true && action.allowed !== false) return fail('DENIED');
      state.received.internetIdentity = action.allowed ? state.received.identity : null;
      return ok(state, action.allowed ? 'Internet is alleen voor deze bevestigde NAS toegestaan in de simulatie.' : 'Internettoegang ingetrokken. Koppeling blijft alleen lokaal beschikbaar.');
    case 'receivedCondition':
      if (!['offline', 'online', 'identityMismatch', 'revoked'].includes(action.condition)) return fail('DENIED');
      if (action.condition === 'online') {
        if (state.received.state !== 'offline' || state.received.identity !== FAMILY_NAS_IDENTITY) return fail('IDENTITY_MISMATCH');
        state.received.state = 'paired';
      } else if (action.condition === 'offline') {
        if (state.received.state !== 'paired') return fail('NAS_UNAVAILABLE');
        state.received.state = 'offline';
      } else {
        state.received.state = action.condition;
        state.received.identity = null; state.received.internetIdentity = null;
        state.received.epoch += 1;
      }
      return ok(state, action.condition === 'revoked'
        ? 'De beheerder heeft de ruimte ingetrokken. Lokale documenten blijven bewaard; oude uploads zijn gestopt.'
        : 'Voorbeeldverbinding gewijzigd. Lokale documenten blijven beschikbaar.');
    case 'enableHosting':
      if (!localAdminAvailable || action.confirmed !== true) return fail('LOCAL_ADMIN_REQUIRED');
      // Explicit local management role in this mock. A data pairing by itself
      // grants no administration, and remote administration is not implemented.
      state.hosting.enabled = true; state.hosting.adminRole = 'local-demo';
      return ok(state, 'Lokale voorbeeldbeheerrol geactiveerd. Je kunt ruimte op deze NAS aanbieden.');
    case 'inviteGuest': {
      if (!localAdminAvailable || !state.hosting.enabled || state.hosting.adminRole !== 'local-demo') return fail('LOCAL_ADMIN_REQUIRED');
      const label = String(action.label || '').trim();
      if (!label || label.length > 40 || !Number.isSafeInteger(action.quotaGiB) || action.quotaGiB < 1) return fail('INVALID_ALLOCATION');
      const reserved = state.hosting.guests.filter((guest) => guest.state !== 'removed').reduce((total, guest) => total + guest.quotaGiB, 0);
      if (action.quotaGiB > state.hosting.capacityGiB - reserved) return fail('NAS_QUOTA');
      if (state.hosting.guests.length >= 50) {
        const oldRemoved = state.hosting.guests.findIndex((guest) => guest.state === 'removed');
        if (oldRemoved === -1) return fail('NAS_QUOTA');
        state.hosting.guests.splice(oldRemoved, 1);
        // Unknown ids are rejected too; dropping old display history must never
        // reuse a namespace or turn a stale upload into a new allocation.
      }
      state.serial += 1;
      state.hosting.guests.push({ id: `guest-${state.serial}`, label, quotaGiB: action.quotaGiB,
        usedGiB: 0, state: 'invited', epoch: 1, invitation: `DEMO-RUIMTE-${state.serial}` });
      return ok(state, 'Voorbeelduitnodiging gemaakt. Er is geen bericht verstuurd.');
    }
    case 'guestUsage': {
      // Represents a response from a future authenticated service, not real I/O.
      const guest = guestById(state, action.id);
      if (!localAdminAvailable || !state.hosting.enabled || state.hosting.adminRole !== 'local-demo' || !guest || guest.state === 'removed' || guest.epoch !== action.epoch) return fail('DENIED');
      if (!Number.isSafeInteger(action.usedGiB) || action.usedGiB < 0 || action.usedGiB > guest.quotaGiB) return fail('RESOURCE_LIMIT');
      guest.state = 'active'; guest.usedGiB = action.usedGiB;
      return ok(state, 'Fictief ruimtegebruik bijgewerkt; documentinhoud blijft buiten het beheerscherm.');
    }
    case 'requestRemoval': {
      const guest = guestById(state, action.id);
      if (!localAdminAvailable || !state.hosting.enabled || state.hosting.adminRole !== 'local-demo' || !guest || guest.state === 'removed') return fail('DENIED');
      state.pendingRemoval = { id: guest.id, epoch: guest.epoch };
      return ok(state);
    }
    case 'cancelRemoval': state.pendingRemoval = null; return ok(state);
    case 'removeGuest': {
      const guest = guestById(state, action.id);
      if (!localAdminAvailable || !state.hosting.enabled || state.hosting.adminRole !== 'local-demo' || !guest || guest.state === 'removed' || action.confirmed !== true
        || state.pendingRemoval?.id !== guest.id || state.pendingRemoval.epoch !== guest.epoch) return fail('DENIED');
      // Admin confirmation only. No tenant approval, decryption key or user grant.
      guest.epoch += 1; guest.state = 'removed'; guest.usedGiB = 0;
      guest.quotaGiB = 0; guest.invitation = null; state.pendingRemoval = null;
      return ok(state, 'Toegang ingetrokken en de NAS-kopie gewist in de simulatie. Oude uploads kunnen deze ruimte niet heropenen.');
    }
    default: return fail('DENIED');
  }
}

export function resumeNasSharing(stored) {
  const fresh = createNasSharingState();
  if (!stored || !Array.isArray(stored.hosting?.guests)) return fresh;
  fresh.view = ['own', 'receive', 'offer'].includes(stored.view) ? stored.view : 'own';
  // All pairing/internet/admin confirmations expire on refresh. Only bounded
  // management history survives; browser storage never creates active authority.
  const seen = new Set();
  for (const entry of stored.hosting.guests.slice(0, 50)) {
    if (!entry || !/^guest-[1-9]\d{0,5}$/.test(entry.id) || seen.has(entry.id)
      || typeof entry.label !== 'string' || entry.label.length > 40
      || !Number.isSafeInteger(entry.epoch) || entry.epoch < 1 || entry.epoch > 1000000
      || !Number.isSafeInteger(entry.quotaGiB) || entry.quotaGiB < 0 || entry.quotaGiB > 100
      || !Number.isSafeInteger(entry.usedGiB) || entry.usedGiB < 0 || entry.usedGiB > entry.quotaGiB
      || !['invited', 'active', 'removed'].includes(entry.state)) continue;
    const total = fresh.hosting.guests.reduce((sum, guest) => sum + guest.quotaGiB, 0);
    if (entry.state !== 'removed' && total + entry.quotaGiB > fresh.hosting.capacityGiB) continue;
    seen.add(entry.id);
    fresh.hosting.guests.push({ id: entry.id, label: entry.label, quotaGiB: entry.state === 'removed' ? 0 : entry.quotaGiB,
      usedGiB: entry.state === 'removed' ? 0 : entry.usedGiB, state: entry.state, epoch: entry.epoch + 1, invitation: null });
    fresh.serial = Math.max(fresh.serial, Number(entry.id.slice(6)));
  }
  return fresh;
}
