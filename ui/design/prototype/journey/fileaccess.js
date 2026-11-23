// Owner file-open model of 23 November 2026 (build/design-pipeline-checkpoint/owner-decision-file-open-20261123.md),
// UX-E slice B, item E11. Pure: no DOM; runs on any contract.js createPermissions() instance, so node tests use their own.
// PROTOTYPE ONLY: this page demonstrates the model; BaryonOS does not enforce it yet (Technisch haalbaar, nog niet gebouwd).
//
// Two facets, so an app can never hand itself a trusted flag:
//  - shell: the trusted, OS-drawn surfaces (Bestanden Openen, Zoeken, Verder waar je was, the window-bar Bewerken and
//    Opslaan, the grants view, sign-out). Only the shell opens, escalates, closes and revokes.
//  - forApp(app): what the app itself may do: read what the user handed it and write only after a trusted edit.
// Grants are recorded through the contract (grants.seedTemporary): scope 'document', destination null, never remembered.
// Anything else (another file, a broader scope, a remembered grant, any network right) goes through contract ask().
const req = (app, right, doc) => ({ kind: 'ongoing', app, right, data: doc, destination: null, scope: 'document' })
const DOC_RIGHTS = new Set(['read', 'readWrite'])

// isUserGesture decides what counts as a real user action on a trusted shell surface. The default accepts only a real input
// event (an Event the browser marks isTrusted); a plain object such as { isTrusted: true } or a script-made event is refused.
export const realInput = (e) => typeof Event === 'function' && e instanceof Event && e.isTrusted === true
export function createFileAccess(permissions, { isUserGesture = realInput } = {}) {
  const { grants } = permissions
  const open = new Map() // `${app}\n${doc}` -> 'read' | 'readWrite'; the shell's record of what the user handed over
  const key = (app, doc) => `${app}\n${doc}`
  const held = (app, right, doc) => grants.temporary().some((g) => g.app === app && g.right === right && g.data === doc && g.destination === null && g.scope === 'document')
  // Ending one document (lead decision of 23 Nov 2026, contract 44ef6030): grants.endTemporary ends exactly the document's
  // temporary read and readWrite on their five-field keys. Remembered decisions (grants.revoke is for those only), the app's
  // temporary denials (R2) and every other key stay as they are; the app keeps running.
  function end(app, doc) {
    for (const r of DOC_RIGHTS) grants.endTemporary(req(app, r, doc))
    open.delete(key(app, doc))
  }
  const level = (app, doc) => {
    const l = open.get(key(app, doc))
    if (l === 'readWrite' && held(app, 'readWrite', doc)) return 'readWrite'
    if (l && held(app, 'read', doc)) return 'read'
    return null
  }

  const shell = {
    // (a) an explicit user open: temporary read for exactly this document, no prompt, never readWrite
    open(app, doc, ev) {
      if (typeof app !== 'string' || typeof doc !== 'string' || !app || !doc || !isUserGesture(ev)) return null
      if (!open.has(key(app, doc))) open.set(key(app, doc), 'read')
      if (!held(app, 'read', doc)) grants.seedTemporary(req(app, 'read', doc))
      return level(app, doc)
    },
    // (b) a user editing or save action on a trusted shell surface. `ev` must pass isUserGesture (a real input event): an
    // event synthesised by script or a forged object, including from app content, is refused. Only an opened document escalates.
    edit(app, doc, ev) {
      if (!isUserGesture(ev) || !level(app, doc)) return false
      if (!held(app, 'readWrite', doc)) grants.seedTemporary(req(app, 'readWrite', doc))
      open.set(key(app, doc), 'readWrite')
      return true
    },
    // (c) revocation: closing the document, the app ending, the user revoking (also a read allowed in the prompt), sign-out
    close(app, doc) { if (open.has(key(app, doc)) || held(app, 'read', doc) || held(app, 'readWrite', doc)) end(app, doc) },
    revoke(app, doc) { shell.close(app, doc) },
    appClosed(app) {
      grants.appClosed(app)
      for (const k of [...open.keys()]) if (k.startsWith(`${app}\n`)) open.delete(k)
    },
    logout() { grants.logout(); open.clear() },
    list: () => [...open.keys()].map((k) => { const [app, doc] = k.split('\n'); return { app, doc, level: level(app, doc) } }).filter((x) => x.level),
    level,
  }

  // what an app can do with the files the user handed it; there is deliberately no way to grant or transfer anything here
  const forApp = (app) => Object.freeze({
    level: (doc) => level(app, doc),
    canRead: (doc) => level(app, doc) !== null,
    canWrite: (doc) => level(app, doc) === 'readWrite',
    // (b) an app-originated request for write access is refused: only the trusted shell surface escalates
    requestEdit: () => ({ ok: false, reason: 'trusted-surface-only' }),
    // a write fails closed unless the user's edit or save action granted readWrite for this document in this session
    write(doc, fn) {
      if (level(app, doc) !== 'readWrite') return { ok: false, reason: 'no-write-grant' }
      fn()
      return { ok: true }
    },
  })
  return { shell, forApp }
}

// E11 (b), lead decision 3: what the Save shortcut may do. Ctrl+S is never a trusted escalation: on a document the app holds
// read-only it only points the user at the shell's Bewerken control; it saves only when readWrite is already held.
export function saveShortcut(level) {
  return level === 'readWrite' ? 'save' : level === 'read' ? 'point-to-edit' : 'none'
}
