// Stations 2-4: desktop, Orbit, Menu, status, clock, Zoeken, switcher and windows.
// Windows follow wm.js: DOM order = MRU order = focus order; the active window is never moved.
import { T, DATA, el, icon, store, files, bus, motionMs, grants } from '../contract.js'
import { createWm } from './wm.js'
import { buildIndex, query } from './search.js'
import { ACTIONS } from './state.js'
import { renderApp, saveNote, noteDirty, discardNote, emptyState, noteDoc, setNoteDoc, canOpenInNotes } from './apps.js'
import { fileAccess, TEST_OPEN, isUserGesture, appName } from './access.js'
import { saveShortcut } from './fileaccess.js'
import { appEnded, netInUse } from './stations.js'

const APP_ICON = { bestanden: 'bestanden', notities: 'notities', instellingen: 'instellingen', haven: 'haven', browser: 'browser', fotos: 'fotos', rekenmachine: 'rekenmachine', muziek: 'muziek' }

export function initShell(ctx) {
  const J = ctx.J
  const desk = document.getElementById('jr-desk')
  const winHost = el('div', { class: 'jr-windows', id: 'jr-windows' })
  const orbit = el('nav', { class: 'ds-orbit', 'aria-label': T.orbit.name })
  const pops = el('div', { class: 'jr-pops' })
  const cont = el('section', { class: 'jr-continue', 'aria-labelledby': 'jr-cont-h' })
  // atmosphere only (neutral blade light, no place colour): place colours appear solely with shape, icon and text
  desk.append(el('div', { class: 'jr-field', 'aria-hidden': 'true' }), cont, winHost, orbit, pops)

  const nodes = new Map() // window id -> {node, body, app}
  const wm = createWm({ emit: (type, w) => {
    if (type === 'appClosed') { fileAccess.shell.appClosed(w); appEnded(w) } // OQ-5 end event: last window of the app closed; ends its file grants (E11 c)
  } })
  const title = appName
  // B repair 3 (lead decision 5): internet in use, by which running app and for which single destination (gesimuleerd)
  const netUses = () => netInUse(new Set(wm.list().map((w) => w.app)))
  const netText = () => netUses().map((u) => J.b.r3.netUse(title(u.app), u.dest)).join('; ')
  const place = (storage) => {
    const p = storage === 'local' ? 'device' : 'nas'
    return el('span', { class: 'ds-place', 'data-place': p }, el('i', { class: 'ds-node', 'data-shape': p, 'data-state': p === 'nas' && !store.get('nasOnline') ? 'off' : null, 'aria-hidden': 'true' }), icon(p, { size: 16 }), p === 'device' ? T.privacy.local : DATA.nas.name)
  }

  // --- Desktop: Verder waar je was ---
  function renderContinue() {
    const recent = ctx.sim.desk === 'empty' ? [] : files.list({ recent: true }).slice(0, 4)
    const h = new Date(`${DATA.now.iso}`).getHours()
    const greet = T.desktop.greeting[h < 12 ? 'morning' : h < 18 ? 'afternoon' : 'evening']
    const nas = !!store.get('nasOnline'), net = netText()
    // the three-node privacy instrument (ds-1.1 .ds-privacy): where work is, with shape, icon and text; internet stays off
    const col = (p, state, label) => el('div', { class: 'ds-privacy-col', 'data-place': p, 'data-state': state },
      el('i', { class: 'ds-node', 'data-shape': p, 'data-state': state === 'off' ? 'off' : null }), el('span', { class: 'ds-privacy-lbl' }, icon(p, { size: 16 }), label))
    const privacy = el('div', { class: 'jr-desk-privacy', 'data-sim-scope': '', 'data-sim-fn': '' },
      el('div', { class: 'ds-privacy', 'data-state': 'local', role: 'status', 'aria-label': net ? J.b.r3.privacy(nas, net) : J.more.privacy(nas) },
        el('div', { class: 'ds-privacy-track', 'aria-hidden': 'true' }, col('device', 'lit', J.more.device), col('nas', nas ? 'lit' : 'off', DATA.nas.name), col('internet', net ? 'lit' : 'off', J.more.internet)),
        el('div', { class: 'ds-privacy-text', 'aria-hidden': 'true' }, el('span', {}, J.more.privacyWhere(nas)), el('span', {}, net ? J.b.r3.privacyNet(net) : J.more.privacyNet))),
      ctx.feas('simulated'))
    // recent work on a trajectory, newest first: the lit dot is where you were last (Orbitaal v2 board, Bureaublad)
    cont.replaceChildren(el('div', { class: 'jr-desk-head' }, el('p', { class: 'jr-greet' }, `${greet}, ${DATA.user.name}`), privacy),
      el('div', { class: 'jr-plate' },
        el('h2', { id: 'jr-cont-h' }, T.desktop.continueTitle),
        recent.length ? el('ul', { class: 'jr-cont-list' }, recent.map((f) => el('li', {}, el('button', { class: 'jr-cont-item', type: 'button', on: { click: (e) => openFile(f.id, e) } },
          el('span', { class: 'jr-thumb', 'aria-hidden': 'true' }, icon(f.kind, { size: 24 })),
          el('span', { class: 'jr-cont-text' }, el('span', { class: 'jr-cont-name' }, f.name), el('span', { class: 'jr-cont-meta' }, `${J.more.kinds[f.kind] ?? ''} · ${f.changed}`)),
          place(f.storage)))))
          : el('p', { class: 'jr-empty-line' }, J.shell.continueEmpty)))
  }
  // E11 (a): every explicit user open (Bestanden Openen and Openen met, a Zoeken result, Verder waar je was, Menu recent, the
  // Openen... chooser) lands here and seeds a temporary read for exactly that document; never readWrite, never a prompt.
  // Router opens never seed. `ev` must be the real input event of the user's gesture; the broker checks it again
  // (fileaccess.js isUserGesture), so a script-made event or a forged object seeds nothing. TEST_OPEN: ?test=1 hook only.
  async function openFile(id, ev) {
    if (!canOpenInNotes(id)) return open('bestanden')
    if (!isUserGesture(ev)) { ctx.live(J.access.refusedOpen); return null }
    const prev = noteDoc()
    if (prev && prev !== id) { // the single Notities window switches document: the previous one is closed first
      if (noteDirty()) {
        const v = await ctx.confirm(J.access.dirtyClose(files.get(prev)?.name ?? prev))
        if (v === 'cancel') return null
        if (v === 'save' && !saveNote()) return null
        if (v === 'discard') discardNote()
      }
      fileAccess.shell.close('notities', prev)
    }
    if (!fileAccess.shell.open('notities', id, ev)) { ctx.live(J.access.refusedOpen); return null }
    if (id !== prev) setNoteDoc(id) // reopening the same document keeps its draft
    const w = open('notities')
    refresh('notities')
    return w
  }
  function refresh(app) {
    for (const [id, n] of nodes) if (n.app === app && !n.body.getAttribute('aria-busy') && !n.body.querySelector(':scope > .ds-empty[data-tone="error"][id$="-err"]')) { // never over a failed launch
      const a = document.activeElement, keep = n.body.contains(a) ? (a.id === 'jr-note' ? [a.selectionStart, a.selectionEnd] : []) : null
      renderApp(app, n.body, { ...ctx, open, openFile, args: {} })
      syncBar(id)
      if (keep) { const t = n.body.querySelector('#jr-note'); if (t) { t.focus(); if (keep.length) t.setSelectionRange(...keep) } } // a save never drops focus or caret
    }
  }
  // E11 (b): Bewerken and Opslaan are drawn by the shell in the window bar, outside the app content; they pass the real input
  // event to the broker, which refuses anything not isTrusted (a script-made click, including one from app content).
  function syncBar(id) {
    const n = nodes.get(id); if (!n || n.app !== 'notities') return
    const d = noteDoc(), lv = d ? fileAccess.shell.level('notities', d) : null
    const eb = n.node.querySelector('.jr-edit'), sb = n.node.querySelector('.jr-save'), on = n.node.querySelector('.jr-edit-on')
    // repair 1: Bewerken is a one-way action, not a toggle; once granted, a state chip replaces it until the file is closed
    eb.hidden = !lv || lv === 'readWrite'
    on.hidden = lv !== 'readWrite'
    sb.hidden = !lv
  }
  // B repair 3 (lead decision 3): only an activation of a control the shell draws in the window frame (Bewerken, Opslaan)
  // escalates; a keystroke, whatever element has focus, never does. TEST_OPEN: the ?test=1 hook only.
  const fromFrame = (e) => (TEST_OPEN !== null && e === TEST_OPEN) || !!e?.currentTarget?.closest?.('.jr-frame-acts')
  function editAction(id, e) {
    const d = noteDoc()
    if (!d || !fromFrame(e)) return false
    if (fileAccess.shell.level('notities', d) === 'readWrite') return true
    const ok = fileAccess.shell.edit('notities', d, e) // the broker refuses anything but a real user gesture
    ctx.live(ok ? J.access.edited(files.get(d)?.name ?? d) : J.access.refused)
    if (ok) {
      refresh('notities')
      // metaphor: the key turns (the chip slides out of where Bewerken was); reduced motion: duration tokens are 0 ms
      const chip = nodes.get(id)?.node.querySelector('.jr-edit-on')
      if (chip) { chip.dataset.anim = ''; setTimeout(() => chip.removeAttribute('data-anim'), 400) }
      nodes.get(id)?.node.querySelector('#jr-note')?.focus()
    }
    return ok
  }
  // Ctrl+S (E11 b, lead decision 3): saves only while readWrite is held; on a read-only document it moves focus to the
  // shell's Bewerken control and says why, so the escalation is always the user's own activation of that control.
  function saveKey(id) {
    const d = noteDoc(), lv = d ? fileAccess.shell.level('notities', d) : null, act = saveShortcut(lv)
    if (act === 'save') { if (noteDirty()) { if (saveNote()) { refresh('notities'); ctx.live(J.shell.saved) } else ctx.live(J.access.writeRefused(files.get(d)?.name ?? d)) } return }
    if (act === 'point-to-edit') { nodes.get(id)?.node.querySelector('.jr-edit')?.focus(); ctx.live(J.b.r3.ctrlSReadOnly(files.get(d)?.name ?? d)) }
  }
  function saveAction(id, e) {
    const d = noteDoc()
    if (!d || !editAction(id, e)) return
    if (!noteDirty()) return
    if (saveNote()) { refresh('notities'); ctx.live(J.shell.saved) } else ctx.live(J.access.writeRefused(files.get(d)?.name ?? d))
  }

  // The system file chooser (a powerbox): a modal dialog the shell draws outside every app's content. Opening it needs a user
  // gesture; the pick inside it is the open gesture that seeds read for that one file (E11 a). With `file`, it is Openen met.
  function chooser(e, { file = null } = {}) {
    if (!isUserGesture(e)) { ctx.live(J.access.refusedOpen); return }
    const dlg = document.getElementById('jr-chooser'), O = J.b.open, X = J.b.files
    const opener = document.activeElement
    let picked = false
    const pick = (id) => (ev) => { picked = true; dlg.close(); openFile(id, ev) }
    const items = file
      ? ['notities', 'browser', 'fotos'].map((app) => el('li', {}, app === 'notities' && canOpenInNotes(file)
        ? el('button', { class: 'ds-row', type: 'button', on: { click: pick(file) } }, icon(APP_ICON[app], { size: 18 }), el('span', { class: 'ds-row-name' }, title(app)))
        : el('div', { class: 'ds-row', 'aria-disabled': 'true' }, icon(APP_ICON[app], { size: 18 }), el('span', { class: 'ds-row-name' }, title(app)), el('span', { class: 'ds-row-meta' }, X.cannotOpen(title(app))))))
      : files.list().filter((f) => canOpenInNotes(f.id)).map((f) => el('li', {}, el('button', { class: 'ds-row', type: 'button', on: { click: pick(f.id) } }, icon(f.kind === 'list' ? 'list' : 'note', { size: 18 }), el('span', { class: 'ds-row-name' }, f.name), place(f.storage))))
    const cancel = el('button', { class: 'ds-btn', 'data-variant': 'secondary', type: 'button', on: { click: () => dlg.close() } }, O.cancel)
    dlg.replaceChildren(el('h2', { class: 'ds-dialog-title', id: 'jr-chooser-t' }, file ? X.openWithTitle(files.get(file)?.name ?? file) : O.chooserTitle),
      el('div', { class: 'ds-dialog-body' }, el('p', { class: 'ds-dialog-text', id: 'jr-chooser-d' }, file ? O.withLead : O.chooserLead), el('ul', { class: 'jr-plain jr-chooser-list' }, items)),
      el('div', { class: 'ds-dialog-actions' }, cancel))
    dlg.onclose = () => { if (!picked && opener?.isConnected) opener.focus() }
    dlg.showModal()
    ;(dlg.querySelector('.ds-dialog-body button') ?? cancel).focus()
  }
  ctx.chooser = chooser

  // --- Orbit ---
  const items = {}
  function renderOrbit() {
    const stack = wm.stackMode
    const running = new Set(wm.list().map((w) => w.app))
    const active = wm.active()?.app
    const apps = stack ? [] : [...new Set([...store.get('pinned'), ...running])]
    const net = netText()
    const item = (key, label, ic, onclick, extra = {}) => {
      const b = el('button', { class: 'ds-taskitem', type: 'button', 'aria-label': label, on: { click: onclick }, ...extra }, ic)
      items[key] = b
      return b
    }
    orbit.replaceChildren(...[
      item('menu', T.orbit.menu, el('img', { src: '../brand/mark-compact.svg', alt: '', class: 'jr-orbit-mark' }), (e) => togglePop('menu', e.currentTarget), { 'aria-haspopup': 'dialog', 'aria-expanded': String(openPop === 'menu') }),
      item('search', T.orbit.search, icon('zoeken', { size: 22 }), (e) => togglePop('search', e.currentTarget), { 'aria-expanded': String(openPop === 'search') }),
      stack ? item('windows', J.shell.windowsCount(wm.list().length), icon('window', { size: 22 }), (e) => togglePop('switcher', e.currentTarget), { 'aria-expanded': String(openPop === 'switcher') }) : null,
      el('span', { class: 'ds-orbit-sep', 'aria-hidden': 'true' }),
      ...apps.map((a) => {
        const st = a === active ? 'active' : running.has(a) ? 'running' : null
        return item(`app:${a}`, st === 'active' ? J.shell.active(title(a)) : st ? T.orbit.running(title(a)) : title(a), icon(APP_ICON[a] ?? 'apps', { size: 22 }),
          () => (st === 'active' ? minimise(wm.active().id) : open(a)), { 'data-state': st, 'aria-current': st === 'active' ? 'true' : null })
      }),
      apps.length ? el('span', { class: 'ds-orbit-sep', 'aria-hidden': 'true' }) : null,
      item('status', net ? J.b.r3.status(!!store.get('nasOnline'), net) : store.get('nasOnline') ? J.shell.status : J.shell.statusOffline,
        el('span', { class: 'jr-status-nodes' }, el('i', { class: 'ds-node', 'data-shape': 'device', 'data-place': 'device', 'aria-hidden': 'true' }),
          el('i', { class: 'ds-node', 'data-shape': 'nas', 'data-place': 'nas', 'data-state': store.get('nasOnline') ? null : 'off', 'aria-hidden': 'true' }),
          el('i', { class: 'ds-node', 'data-shape': 'internet', 'data-place': 'internet', 'data-state': net ? 'on' : 'off', 'aria-hidden': 'true' })),
        (e) => togglePop('quick', e.currentTarget), { 'aria-expanded': String(openPop === 'quick') }),
      item('clock', J.shell.clock(DATA.now.clock, DATA.now.date),
        el('span', { class: 'jr-clock-item' }, el('span', {}, DATA.now.clock), el('span', { class: 'jr-clock-date' }, DATA.now.short),
          store.get('notifications').length ? el('span', { class: 'ds-taskitem-badge', 'aria-hidden': 'true' }, String(store.get('notifications').length)) : null),
        (e) => togglePop('notes', e.currentTarget), { 'aria-expanded': String(openPop === 'notes'), 'data-wide': '' })].filter(Boolean))
    syncNet() // the desktop privacy instrument follows the same state (an app closing ends its internet use there too)
  }

  // --- Popovers: non-modal, one at a time, Escape or outside click closes, focus returns to the trigger ---
  let openPop = null
  let trigger = null
  let returnTo = null // the element focus came from when a shortcut opened the popover (a window field); Escape returns there
  const POP_ITEM = { menu: 'menu', search: 'search', quick: 'status', notes: 'clock', switcher: 'windows' }
  const popTitle = { menu: T.orbit.menu, search: T.orbit.search, quick: T.quick.title, notes: T.notifications.title, switcher: T.window.switcher }
  function togglePop(id, from, keep) {
    if (openPop === id && !keep) return closePopovers(true)
    const prev = document.activeElement
    const back = pops.contains(prev) ? returnTo : prev && prev !== document.body && !orbit.contains(prev) ? prev : null
    closePopovers(false)
    returnTo = back
    openPop = id
    trigger = Object.keys(items).find((k) => items[k] === from) ?? (items[POP_ITEM[id]] ? POP_ITEM[id] : 'menu')
    // after the rise (a transform), keep the focused field in view: it may have been scrolled to while the popover was still moving
    const p = el('section', { class: 'jr-pop', 'data-pop': id, role: 'dialog', 'aria-label': popTitle[id], tabindex: '-1', on: { keydown: popKeys, animationend: () => { const a = document.activeElement; if (p.contains(a)) a.scrollIntoView({ block: 'nearest' }) } } }, ...POP[id]())
    pops.replaceChildren(p)
    renderOrbit()
    // focus never drops to the page: a popover with nothing to operate (all notifications cleared) takes focus itself
    ;(p.querySelector('[data-initial-focus]') ?? p.querySelector('input, button, [tabindex="0"]') ?? p).focus()
  }
  function closePopovers(restore = true) {
    if (!openPop) return
    const back = returnTo
    returnTo = null
    openPop = null
    pops.replaceChildren()
    renderOrbit()
    if (restore) (back?.isConnected && !back.closest('[hidden], [inert]') ? back : items[trigger] ?? items.menu)?.focus()
  }
  function popKeys(e) {
    if (e.key === 'Escape' && !e.repeat) { e.preventDefault(); e.stopPropagation(); closePopovers(true) }
  }
  // Focus leaving the popover (Tab, Shift+Tab, F6) closes it, so nothing behind it can hold focus unseen (WCAG 2.4.11)
  document.addEventListener('focusin', (e) => { if (openPop && !e.target.closest('.jr-pop, dialog') && e.target !== items[trigger]) closePopovers(false) })
  document.addEventListener('pointerdown', (e) => { if (openPop && !e.target.closest('.jr-pop, .ds-orbit')) closePopovers(false) })

  const appBtn = (a) => el('button', { class: 'jr-app-tile', type: 'button', on: { click: () => { closePopovers(false); open(a) } } }, icon(APP_ICON[a] ?? 'apps', { size: 24 }), el('span', {}, title(a)))
  const POP = {
    menu: () => {
      const q = el('input', { class: 'ds-input', type: 'search', id: 'jr-menu-q', placeholder: T.menu.searchPlaceholder, 'data-initial-focus': '',
        on: { input: (e) => { const v = e.target.value; togglePop('search', items.search); searchFor(v) } } })
      const power = el('div', { class: 'jr-power', hidden: true, id: 'jr-power' },
        el('button', { class: 'ds-btn', 'data-variant': 'quiet', type: 'button', on: { click: () => { closePopovers(false); ctx.go('aanmelden') } } }, T.menu.sleep),
        el('button', { class: 'ds-btn', 'data-variant': 'quiet', type: 'button', on: { click: () => ctx.live(J.session.shutdownNote) } }, T.menu.shutdown),
        el('div', { class: 'jr-later', 'aria-disabled': 'true', tabindex: '0' }, T.menu.restartRecovery, el('span', { class: 'ds-menu-why' }, J.tour.laterWhy)))
      return [
        el('h2', { class: 'ds-vh' }, T.orbit.menu),
        el('div', { class: 'ds-field', 'data-variant': 'search' }, el('label', { class: 'ds-vh', for: 'jr-menu-q' }, T.orbit.search), el('span', { class: 'ds-input-wrap' }, icon('zoeken', { size: 18 }), q)),
        el('h3', { class: 'jr-pop-h' }, T.menu.pinned), el('div', { class: 'jr-app-grid' }, store.get('pinned').map(appBtn)),
        el('h3', { class: 'jr-pop-h' }, T.menu.recent), el('ul', { class: 'jr-plain' }, files.list({ recent: true }).slice(0, 3).map((f) => el('li', {}, el('button', { class: 'ds-row', type: 'button', on: { click: (e) => { closePopovers(false); openFile(f.id, e) } } }, icon('note', { size: 18 }), el('span', { class: 'ds-row-name' }, f.name), place(f.storage))))),
        el('h3', { class: 'jr-pop-h' }, T.menu.allApps), el('div', { class: 'jr-app-grid' }, DATA.apps.filter((a) => !a.pinned).map((a) => appBtn(a.id))),
        el('div', { class: 'jr-pop-foot' }, el('span', { class: 'jr-avatar', 'aria-hidden': 'true' }, DATA.user.initials), el('span', { class: 'jr-grow' }, DATA.user.full),
          el('button', { class: 'ds-btn', 'data-variant': 'secondary', type: 'button', on: { click: () => { closePopovers(false); ctx.go('aanmelden') } } }, icon('lock', { size: 16 }), T.menu.lock),
          el('button', { class: 'ds-btn', 'data-variant': 'secondary', type: 'button', 'aria-expanded': 'false', 'aria-controls': 'jr-power', on: { click: (e) => { power.hidden = !power.hidden; e.currentTarget.setAttribute('aria-expanded', String(!power.hidden)) } } }, icon('power', { size: 16 }), T.menu.power)),
        power,
      ]
    },
    search: () => {
      const q = el('input', { class: 'ds-input', type: 'search', id: 'jr-q', role: 'combobox', 'aria-expanded': 'true', 'aria-controls': 'jr-results', 'aria-autocomplete': 'list', 'data-initial-focus': '',
        on: { input: () => drawResults(), keydown: resultKeys } })
      return [
        el('h2', { class: 'ds-vh' }, T.orbit.search),
        el('div', { class: 'ds-field', 'data-variant': 'search' }, el('label', { class: 'ds-field-label', for: 'jr-q' }, J.search.label), el('span', { class: 'ds-input-wrap' }, icon('zoeken', { size: 18 }), q)),
        el('div', { class: 'jr-search-state', id: 'jr-search-state' }),
        el('div', { class: 'jr-search-body' },
          el('div', { class: 'jr-results', id: 'jr-results', role: 'listbox', 'aria-label': J.search.label }),
          el('aside', { class: 'jr-preview', id: 'jr-preview', 'aria-label': J.more.preview })),
        el('p', { class: 'jr-pop-note', 'data-sim-scope': '', 'data-sim-fn': '' }, T.search.internetOff, ' ', ctx.feas('feasible')),
        el('p', { class: 'jr-pop-note' }, T.search.keys),
      ]
    },
    quick: () => {
      const sw = (key, label) => el('button', { class: 'ds-switch', role: 'switch', type: 'button', 'aria-checked': String(!!store.get(key)), on: { click: (e) => { store.set(key, !store.get(key)); e.currentTarget.setAttribute('aria-checked', String(!!store.get(key))) } } }, el('span', { class: 'ds-switch-track', 'aria-hidden': 'true' }), label)
      const on = store.get('nasOnline')
      return [
        el('h2', { class: 'jr-pop-title' }, T.quick.title),
        el('p', { class: 'jr-pop-note', 'data-sim-scope': '', 'data-sim-fn': '' }, el('span', { class: 'ds-place', 'data-place': 'device' }, el('i', { class: 'ds-node', 'data-shape': 'device', 'aria-hidden': 'true' }), icon('device', { size: 16 }), T.privacy.local), ' ', ctx.feas('simulated')),
        el('p', { class: 'jr-pop-note jr-quick-net', 'data-sim-scope': '', 'data-sim-fn': '' }, el('span', { class: 'ds-place', 'data-place': 'internet' }, el('i', { class: 'ds-node', 'data-shape': 'internet', 'data-state': netText() ? 'on' : 'off', 'aria-hidden': 'true' }), icon('internet', { size: 16 }), J.more.internet),
          ' ', netText() ? J.b.r3.quickNet(netText()) : J.b.r3.quickNetOff, ' ', ctx.feas('simulated')),
        el('div', { class: 'jr-quick' }, sw('wifi', `${T.quick.wifi} · ${T.quick.wifiNetwork}`), sw('bluetooth', T.quick.bluetooth), sw('focusMode', T.quick.focus)),
        el('fieldset', { class: 'ds-fieldset' }, el('legend', {}, T.quick.theme), el('div', { class: 'ds-seg' }, J.a11y.axes.theme[1].map(([v, l]) => el('label', { class: 'ds-radio' },
          el('input', { type: 'radio', name: 'jr-quick-theme', value: v, checked: (ctx.axes.theme ?? '') === v, on: { change: () => ctx.setAxis('theme', v) } }), l)))),
        el('div', { class: 'jr-quick-nas', 'data-sim-scope': '', 'data-sim-fn': '' },
          el('p', { class: 'jr-row' }, el('span', { class: 'ds-place', 'data-place': 'nas' }, el('i', { class: 'ds-node', 'data-shape': 'nas', 'data-state': on ? null : 'off', 'aria-hidden': 'true' }), icon('nas', { size: 16 }), DATA.nas.name),
            el('span', {}, on ? J.more.nasReachable : J.more.nasUnreachable)),
          el('p', { class: 'ds-field-hint' }, J.more.nasWhere, ' ', ctx.feas('simulated'))),
        el('button', { class: 'ds-btn', 'data-variant': 'secondary', type: 'button', on: { click: () => { closePopovers(false); open('instellingen') } } }, icon('instellingen', { size: 16 }), T.quick.settings),
      ]
    },
    notes: () => {
      const list = store.get('notifications')
      return [
        el('div', { class: 'jr-pop-headrow' }, el('h2', { class: 'jr-pop-title', tabindex: '-1', 'data-initial-focus': '' }, T.notifications.title),
          list.length ? el('button', { class: 'ds-btn', 'data-variant': 'quiet', type: 'button', on: { click: () => { store.set('notifications', []); togglePop('notes', items[trigger], true) } } }, T.notifications.clearAll) : null),
        list.length ? el('ul', { class: 'jr-plain jr-notes' }, list.map((n) => el('li', { class: 'jr-note-item', 'data-sim-scope': '', 'data-sim-fn': '' },
          el('div', { class: 'jr-grow' }, el('h3', {}, n.title), el('p', {}, n.body), el('p', { class: 'jr-pop-note' }, `${title(n.app)} · ${n.when} `, ctx.feas('simulated'))),
          el('button', { class: 'ds-window-ctl', type: 'button', 'aria-label': `${T.notifications.dismiss}: ${n.title}`, on: { click: () => { store.set('notifications', store.get('notifications').filter((x) => x.id !== n.id)); togglePop('notes', items[trigger], true) } } }, icon('close', { size: 16 })))))
          : el('div', { class: 'jr-empty-line' }, el('p', {}, el('strong', {}, T.notifications.empty)), el('p', {}, T.notifications.emptyHint)),
        el('p', { class: 'jr-pop-note' }, J.shell.later),
      ]
    },
    switcher: () => {
      const ws = wm.list()
      if (!ws.length) return [el('h2', { class: 'jr-pop-title' }, T.window.switcher), el('p', { class: 'jr-empty-line', 'data-initial-focus': '', tabindex: '-1' }, T.window.noWindows)]
      sel = sel && ws.some((w) => w.id === sel) ? sel : ws[0].id
      const lb = el('div', { class: 'jr-switch-list', role: 'listbox', tabindex: '0', 'aria-label': T.window.switcher, 'aria-activedescendant': `jr-sw-${sel}`, 'data-initial-focus': '',
        on: { keydown: (e) => {
          if (e.repeat && e.key === 'Enter') return
          if (['ArrowRight', 'ArrowDown'].includes(e.key)) { e.preventDefault(); step(1) }
          if (['ArrowLeft', 'ArrowUp'].includes(e.key)) { e.preventDefault(); step(-1) }
          if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); choose() }
        } } },
      ws.map((w) => el('div', { class: 'jr-switch-card', role: 'option', id: `jr-sw-${w.id}`, 'aria-selected': String(w.id === sel), on: { click: () => { sel = w.id; choose() } } },
        icon(APP_ICON[w.app] ?? 'apps', { size: 28 }), el('span', {}, title(w.app)), w.min ? el('span', { class: 'jr-pop-note' }, T.window.minimize) : null)))
      return [el('h2', { class: 'jr-pop-title' }, T.window.switcher), lb, el('p', { class: 'jr-pop-note' }, J.keys.rows[2][2])]
    },
  }

  // --- Switcher ---
  let sel = null
  function step(dir) {
    const next = wm.cycle(sel ?? wm.active()?.id, dir)
    if (!next) return
    sel = next.id
    const lb = pops.querySelector('.jr-switch-list')
    lb?.setAttribute('aria-activedescendant', `jr-sw-${sel}`)
    for (const c of pops.querySelectorAll('.jr-switch-card')) c.setAttribute('aria-selected', String(c.id === `jr-sw-${sel}`))
  }
  function choose() { const id = sel; sel = null; closePopovers(false); if (id) { wm.focus(id); render(); focusWin(id); ctx.live(J.shell.focused(title(wm.get(id).app))) } }
  addEventListener('keyup', (e) => { if (e.key === 'Alt' && openPop === 'switcher' && altCycle) { altCycle = false; choose() } })
  let altCycle = false

  // --- Search ---
  let qText = ''
  let active = -1
  function searchFor(v) {
    if (openPop !== 'search') togglePop('search', items.search)
    const q = pops.querySelector('#jr-q')
    q.value = v
    q.focus()
    drawResults()
  }
  function drawResults() {
    const q = pops.querySelector('#jr-q'); if (!q) return
    qText = q.value
    const idx = buildIndex({ T, DATA, files: files.list(), actions: ACTIONS, name: appName })
    const st = pops.querySelector('#jr-search-state')
    const box = pops.querySelector('#jr-results')
    const mode = ctx.sim.search
    st.replaceChildren(); st.removeAttribute('data-sim-scope')
    let groups = query(idx, qText, { internet: store.get('internetSearch') })
    if (mode === 'updating' && qText.trim()) {
      st.dataset.simScope = ''
      st.append(el('p', { 'data-sim-fn': '' }, J.search.updating, ' ', ctx.feas('simulated')), el('div', { class: 'ds-progress', 'data-state': 'indeterminate', role: 'progressbar', 'aria-label': J.search.updating }, el('span', { class: 'ds-progress-bar' })))
      groups = groups.filter((g) => g.group !== 'files' || (g.items = g.items.slice(0, 1)))
    }
    if (mode === 'unavailable' && qText.trim()) {
      st.dataset.simScope = ''
      st.append(el('div', { class: 'ds-banner', 'data-tone': 'error', 'data-sim-fn': '' }, icon('warn', { size: 18 }), el('p', {}, J.search.error, ' ', ctx.feas('simulated'))),
        el('button', { class: 'ds-btn', 'data-variant': 'secondary', type: 'button', on: { click: () => { ctx.setSim('search', 'ready'); drawResults() } } }, J.shell.retry))
      groups = groups.filter((g) => g.group !== 'files')
    }
    const opts = []
    const opt = (it, label, extra) => {
      const id = `jr-r${opts.length}`
      opts.push({ id, it })
      const ic = { apps: APP_ICON[it.id] ?? 'apps', files: 'note', settings: 'instellingen', actions: 'fwd', assistant: 'assistent' }[it.group] ?? 'dot'
      return el('div', { class: 'ds-row', role: 'option', id, 'aria-selected': 'false', on: { click: (e) => run(it, e), pointerdown: (e) => e.preventDefault() } }, icon(ic, { size: 20 }), label, extra)
    }
    if (!qText.trim()) {
      box.replaceChildren(el('div', { role: 'group', 'aria-label': J.search.suggestions, class: 'jr-result-group' }, el('p', { class: 'jr-pop-h', 'aria-hidden': 'true' }, J.search.suggestions),
        files.list({ recent: true }).slice(0, 2).map((f) => opt({ group: 'files', id: f.id }, el('span', { class: 'ds-row-name' }, f.name), place(f.storage))),
        ['lock', 'dark', 'contrast'].map((a) => opt({ group: 'actions', id: a }, el('span', { class: 'ds-row-name' }, J.search.actions[a])))))
    } else if (!groups.some((g) => g.group !== 'assistant' && g.items.length)) {
      // an outage or a running index update is never blamed on the spelling
      const [head, hint] = mode === 'unavailable' ? [J.more.filesDown, J.more.filesDownHint] : mode === 'updating' ? [J.more.updatingNone(qText.trim()), J.more.updatingHint] : [T.search.none(qText.trim()), T.search.noneHint]
      box.replaceChildren(el('div', { class: 'jr-empty-line' }, el('p', {}, el('strong', {}, head)), el('p', {}, hint)),
        ...groups.filter((g) => g.group === 'assistant').map((g) => assistant(g, opt)))
    } else {
      box.replaceChildren(...groups.filter((g) => g.items.length).map((g) => g.group === 'assistant' ? assistant(g, opt)
        : el('div', { role: 'group', 'aria-labelledby': `jr-g-${g.group}`, class: 'jr-result-group' },
          el('p', { class: 'jr-pop-h', id: `jr-g-${g.group}` }, T.search.groups[g.group], ' · ', J.search.count(g.total)),
          g.items.map((it) => opt(it, el('span', { class: 'ds-row-name' }, it.label), it.group === 'files' ? [el('span', { class: 'ds-row-meta' }, it.sub), place(it.place)] : it.sub ? el('span', { class: 'ds-row-meta' }, it.sub) : null)))))
    }
    box._opts = opts
    preview(opts[0]?.it)
    active = -1
    q.removeAttribute('aria-activedescendant')
    if (document.activeElement === q) q.scrollIntoView({ block: 'nearest' }) // the popover grows upward as results arrive
    ctx.live(qText.trim() ? J.search.count(groups.reduce((n, g) => n + (g.group === 'assistant' ? 0 : g.total), 0)) : J.search.suggestions, '#jr-live')
  }
  const assistant = (g, opt) => el('div', { role: 'group', 'aria-labelledby': 'jr-g-as', class: 'jr-result-group', 'data-sim-scope': '' },
    el('p', { class: 'jr-pop-h', id: 'jr-g-as' }, T.search.groups.assistant, ' ', ctx.feas('simulated')),
    opt(g.items[0], el('span', { class: 'ds-row-name', 'data-sim-fn': '' }, T.search.assistantAnswer(g.items[0].label)), el('span', { class: 'ds-row-meta' }, T.search.assistantNote)))
  // Preview column (wide frames only, hidden in stack mode by journey.css): what the highlighted result is and where it lives
  function preview(it) {
    const pv = pops.querySelector('#jr-preview'); if (!pv) return
    if (!it || it.group === 'assistant') return pv.replaceChildren(el('p', { class: 'jr-pop-h' }, J.more.preview), el('p', { class: 'jr-pop-note' }, J.more.previewHint))
    const f = it.group === 'files' ? files.get(it.id) : null
    const ic = f ? f.kind : { apps: APP_ICON[it.id] ?? 'apps', settings: 'instellingen', actions: 'fwd' }[it.group] ?? 'dot'
    pv.replaceChildren(el('p', { class: 'jr-pop-h' }, J.more.preview),
      el('span', { class: 'jr-thumb', 'data-size': 'large', 'aria-hidden': 'true' }, icon(ic, { size: 32 })),
      el('p', { class: 'jr-pop-title' }, f ? f.name : it.label),
      el('p', { class: 'jr-pop-note' }, f ? [J.more.kinds[f.kind] ?? '', it.sub ? ` · ${it.sub}` : ''].join('') : { apps: J.more.groupApp, settings: J.more.groupSetting, actions: J.more.groupAction }[it.group] ?? ''),
      ...(f ? [el('p', { class: 'jr-preview-line' }, `${J.more.previewWhere} `, place(f.storage)), el('p', { class: 'jr-pop-note' }, `${J.more.previewChanged} ${f.changed}`)] : []))
  }
  function resultKeys(e) {
    const box = pops.querySelector('#jr-results'); const opts = box?._opts ?? []
    if (!opts.length || e.repeat && e.key === 'Enter') return
    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      e.preventDefault()
      active = (active + (e.key === 'ArrowDown' ? 1 : -1) + opts.length) % opts.length
      for (const o of opts) document.getElementById(o.id).setAttribute('aria-selected', 'false')
      const cur = document.getElementById(opts[active].id)
      cur.setAttribute('aria-selected', 'true'); cur.scrollIntoView({ block: 'nearest' })
      preview(opts[active].it)
      e.target.setAttribute('aria-activedescendant', opts[active].id)
    } else if (e.key === 'Enter' && active >= 0) { e.preventDefault(); run(opts[active].it, e) }
  }
  function run(it, e) {
    if (it.group === 'assistant') {
      const st = pops.querySelector('#jr-search-state')
      st.dataset.simScope = ''
      st.replaceChildren(el('div', { class: 'ds-banner', 'data-tone': 'info', 'data-sim-fn': '' }, icon('assistent', { size: 18 }), el('p', {}, J.search.assistantCard, ' ', ctx.feas('simulated'))))
      return
    }
    closePopovers(false)
    if (it.group === 'apps') open(it.id)
    else if (it.group === 'files') openFile(it.id, e)
    else if (it.group === 'settings') open('instellingen', { page: it.page ?? it.id.split(':')[0] })
    else if (it.id === 'lock') ctx.go('aanmelden')
    else if (it.id === 'nas') open('bestanden')
    else { ctx.setAxis('theme', it.id); ctx.live(J.r2.themeSet(J.search.actions[it.id] ?? it.id)) }
  }

  // E11 (c): the grants view (Instellingen > Privacy en beveiliging) lists and revokes file grants through these two.
  ctx.openFiles = () => fileAccess.shell.list()
  ctx.revokeFile = async (app, doc) => {
    if (app === 'notities' && doc === noteDoc() && noteDirty()) { // never drop a draft silently
      const v = await ctx.confirm(J.access.revokeAsk(files.get(doc)?.name ?? doc))
      if (v === 'cancel') return false
      if (v === 'save' && !saveNote()) return false
      if (v === 'discard') discardNote()
    }
    fileAccess.shell.revoke(app, doc)
    refresh(app)
    return true
  }

  // --- Windows ---
  let cascade = 0
  function open(app, args = {}) {
    if (app === 'bestanden' && (args.select || args.tab)) bus.emit('jr:files', args)
    const existing = wm.list().find((w) => w.app === app)
    if (existing && args.tile) nodes.get(existing.id).node.dataset.tile = args.tile
    // a route revisit focuses an open multi-window app instead of adding a window (go() only opens or focuses)
    if (args.reuse && existing) { wm.focus(existing.id); render(); focusWin(existing.id); ctx.live(J.shell.focused(title(app))); return existing }
    const w = wm.open(app)
    if (existing && existing.id === w.id) { render(); if (args.page) bus.emit('jr:settings', args.page); if (args.select || args.tab) refresh(app); focusWin(w.id); ctx.live(J.shell.focused(title(app))); return w }
    const from = items[`app:${app}`]?.getBoundingClientRect()
    const n = cascade++ % 5
    const node = el('section', { class: 'ds-window jr-win', role: 'region', 'aria-labelledby': `${w.id}-t`, 'data-app': app, 'data-tile': args.tile ?? null,
      style: `--jr-dyn-x: calc(4% + ${n * 32}px); --jr-dyn-y: ${16 + n * 28}px`,
      on: { keydown: (e) => { if (app === 'notities' && (e.ctrlKey || e.metaKey) && !e.altKey && e.key.toLowerCase() === 's') { e.preventDefault(); if (!e.repeat) saveKey(w.id) } },
        focusin: () => { if (wm.active()?.id !== w.id) { wm.focus(w.id); render() } }, pointerdown: () => { if (wm.active()?.id !== w.id) { wm.focus(w.id); render() } } } })
    const body = el('div', { class: 'ds-window-body' })
    const bar = el('header', { class: 'ds-window-bar', on: { dblclick: () => { wm.maximise(w.id); render() } } },
      el('span', { class: 'ds-menu-anchor jr-winmenu-anchor' },
        el('button', { class: 'jr-winicon', type: 'button', 'aria-label': J.shell.winMenu(title(app)), 'aria-haspopup': 'menu', 'aria-expanded': 'false', on: { click: (e) => winMenu(w.id, e.currentTarget) } }, icon(APP_ICON[app] ?? 'apps', { size: 18 }))),
      el('h2', { class: 'ds-window-title', id: `${w.id}-t`, tabindex: '-1' }, title(app)),
      el('div', { class: 'ds-window-ctls' },
        el('button', { class: 'ds-window-ctl', type: 'button', 'aria-label': T.window.minimize, on: { click: () => minimise(w.id) } }, icon('min', { size: 16 })),
        el('button', { class: 'ds-window-ctl jr-ctl-max', type: 'button', 'aria-label': T.window.maximize, on: { click: () => { wm.maximise(w.id); render() } } }, icon('max', { size: 16 })),
        el('button', { class: 'ds-window-ctl', type: 'button', 'aria-label': `${T.window.close} (Alt+W)`, on: { click: () => close(w.id) } }, icon('close', { size: 16 }))),
      // B repair 2: the shell's action row comes after the window controls in the DOM, so Tab follows the two visual rows
      app === 'notities' ? el('span', { class: 'jr-frame-acts' },
        el('button', { class: 'ds-btn jr-open', 'data-variant': 'quiet', type: 'button', 'aria-label': J.b.open.chooserTitle, on: { click: (e) => chooser(e) } }, icon('bestanden', { size: 16 }), el('span', { class: 'jr-open-t' }, J.b.open.dots)),
        el('button', { class: 'ds-btn jr-edit', 'data-variant': 'secondary', type: 'button', hidden: true, on: { click: (e) => editAction(w.id, e) } }, icon('toegang', { size: 16 }), J.access.edit),
        el('span', { class: 'jr-chip jr-edit-on', 'data-on': '', hidden: true }, icon('toegang', { size: 14 }), J.access.editing),
        el('button', { class: 'ds-btn jr-save', 'data-variant': 'secondary', type: 'button', hidden: true, on: { click: (e) => saveAction(w.id, e) } }, icon('check', { size: 16 }), J.access.save, el('kbd', { class: 'ds-kbd' }, 'Ctrl+S'))) : null)
    node.append(bar, body)
    winHost.prepend(node)
    nodes.set(w.id, { node, body, app })
    launch(w.id, args)
    render()
    focusWin(w.id) // before the grow transform: focus scrolls to where the window ends up, not to its start beside the Orbit
    // Metaphor: the window grows out of the Orbit item that launched it (reduced motion: instant).
    if (from && motionMs(220)) {
      const to = node.getBoundingClientRect()
      node.style.setProperty('--jr-dyn-dx', `${from.left + from.width / 2 - (to.left + to.width / 2)}px`)
      node.style.setProperty('--jr-dyn-dy', `${from.top - to.top}px`)
      node.classList.add('jr-grow-in')
      setTimeout(() => node.classList.remove('jr-grow-in'), 300)
    }
    ctx.live(J.shell.opened(title(app)))
    return w
  }
  function launch(id, args) {
    const { body, app } = nodes.get(id)
    const mode = ctx.sim.launch
    body.dataset.simScope = ''
    body.setAttribute('aria-busy', 'true')
    body.replaceChildren(el('p', { class: 'ds-vh' }, J.shell.loadingApp(title(app))),
      el('div', { class: 'ds-skeleton', 'aria-hidden': 'true' }, [70, 52, 64].map((wd) => el('div', { class: 'ds-skeleton-row' }, el('span', { class: 'ds-skeleton-line', 'data-shape': 'icon' }), el('span', { class: 'ds-skeleton-line', style: { width: `${wd}%` } })))),
      mode !== 'normal' ? el('p', { class: 'jr-pop-note', 'data-sim-fn': '' }, J.shell.loadingApp(title(app)), ' ', ctx.feas('simulated')) : null)
    if (mode === 'slow') return // held by the Simulatie panel until Normaal is chosen again
    setTimeout(() => {
      if (!nodes.has(id)) return
      body.removeAttribute('aria-busy')
      if (mode === 'fail') {
        body.replaceChildren(emptyState({ id: `${id}-err`, tone: 'error', role: 'alert', ic: APP_ICON[app] ?? 'apps', title: J.shell.failedApp(title(app)), hint: J.shell.failedHint, feas: ctx.feas('simulated'),
          actions: [el('button', { class: 'ds-btn', 'data-variant': 'primary', type: 'button', on: { click: () => { ctx.setSim('launch', 'normal'); launch(id, args) } } }, J.shell.retry)] }))
        return
      }
      body.removeAttribute('data-sim-scope')
      renderApp(app, body, { ...ctx, open, openFile, args })
      syncBar(id)
    }, 600)
  }
  bus.on('jr:sim', (g) => { if (g === 'launch' && ctx.sim.launch === 'normal') for (const [id, n] of nodes) if (n.body.getAttribute('aria-busy')) launch(id, {}) })

  function render() {
    const list = wm.list()
    const act = wm.active()
    list.forEach((w, i) => {
      const n = nodes.get(w.id); if (!n) return
      n.node.hidden = w.min || (wm.stackMode && w !== act) // stack mode: only the active window is shown
      n.node.dataset.state = wm.isMax(w) ? 'maximised' : w === act ? 'active' : 'inactive'
      n.node.dataset.snap = w.snap ?? ''
      n.node.style.zIndex = String(list.length - i)
      n.node.querySelector('.jr-ctl-max').hidden = wm.stackMode
      const mx = n.node.querySelector('.jr-ctl-max')
      mx.setAttribute('aria-label', w.max ? T.window.restore : T.window.maximize)
      mx.replaceChildren(icon(w.max ? 'window' : 'max', { size: 16 }))
      if (i > 0) winHost.append(n.node) // move the others; the active node stays put (focus and scroll survive)
    })
    renderOrbit()
  }
  function focusWin(id) {
    const n = nodes.get(id); if (!n) return
    ;(n.node.querySelector('[data-autofocus]') ?? n.node.querySelector('.ds-window-title')).focus()
  }
  function minimise(id) {
    const n = nodes.get(id); const app = n.app
    const done = () => { n.node.classList.remove('jr-fold'); wm.minimise(id); render(); const a = wm.active(); a ? focusWin(a.id) : items[`app:${app}`]?.focus(); ctx.live(J.shell.minimised(title(app))) }
    const to = items[`app:${app}`]?.getBoundingClientRect()
    if (to && motionMs(220)) {
      const r = n.node.getBoundingClientRect()
      n.node.style.setProperty('--jr-dyn-dx', `${to.left + to.width / 2 - (r.left + r.width / 2)}px`)
      n.node.style.setProperty('--jr-dyn-dy', `${to.top - r.top}px`)
      n.node.classList.add('jr-fold')
      setTimeout(done, 220)
    } else done()
  }
  async function close(id) {
    const n = nodes.get(id); if (!n) return
    if (n.app === 'notities' && noteDirty()) {
      const v = await ctx.confirm(J.access.dirtyClose(files.get(noteDoc())?.name ?? ''))
      if (v === 'cancel') return
      if (v === 'save' && !saveNote()) return
      if (v === 'discard') discardNote()
    }
    nodes.delete(id); n.node.remove(); wm.close(id); render()
    if (n.app === 'notities') setNoteDoc(null) // the document is closed with its window; appClosed ended its grants
    const a = wm.active(); a ? focusWin(a.id) : items.menu?.focus()
    ctx.live(J.shell.closed(title(n.app)))
  }
  function winMenu(id, btn) {
    const anchor = btn.parentElement
    anchor.querySelector('.ds-menu')?.remove()
    if (btn.getAttribute('aria-expanded') === 'true') { btn.setAttribute('aria-expanded', 'false'); return }
    const w = wm.get(id)
    const mi = (label, fn, kbd) => el('button', { class: 'ds-menu-item', role: 'menuitem', type: 'button', tabindex: '-1', on: { click: () => { shut(); fn() } } }, el('span', {}, label), kbd ? el('kbd', { class: 'ds-kbd' }, kbd) : null)
    const shut = (focus = false) => { menu.remove(); btn.setAttribute('aria-expanded', 'false'); if (focus) btn.focus() }
    const menu = el('div', { class: 'ds-menu', role: 'menu', 'aria-label': J.shell.winMenu(title(w.app)), on: { keydown: (e) => {
      const its = [...menu.querySelectorAll('[role=menuitem]')]; const i = its.indexOf(document.activeElement)
      if (e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); shut(true) }
      if (e.key === 'ArrowDown' || e.key === 'ArrowUp') { e.preventDefault(); its[(i + (e.key === 'ArrowDown' ? 1 : -1) + its.length) % its.length].focus() }
      if (e.key === 'Home' || e.key === 'End') { e.preventDefault(); its[e.key === 'Home' ? 0 : its.length - 1].focus() }
      if (e.key === 'Tab') shut()
    } } },
    mi(T.window.minimize, () => minimise(id)),
    wm.stackMode ? null : mi(w.max ? T.window.restore : T.window.maximize, () => { wm.maximise(id); render(); focusWin(id) }, 'Alt+Shift+↑'),
    wm.stackMode ? null : mi(J.shell.snapLeft, () => { wm.snap(id, 'left'); render(); focusWin(id) }),
    wm.stackMode ? null : mi(J.shell.snapRight, () => { wm.snap(id, 'right'); render(); focusWin(id) }),
    el('div', { class: 'ds-menu-sep', role: 'separator' }),
    mi(T.window.close, () => close(id), 'Alt+W'))
    anchor.append(menu)
    btn.setAttribute('aria-expanded', 'true')
    menu.querySelector('[role=menuitem]').focus()
  }

  // Stack mode below 48rem of OS-frame width (covers 320 CSS px and 200 % of 1440): one maximised window at a time.
  const os = document.getElementById('jr-os')
  new ResizeObserver(() => {
    const s = os.clientWidth < 768
    if (s !== wm.stackMode) { wm.stackMode = s; os.toggleAttribute('data-stack', s); render() }
  }).observe(os)

  function command(cmd, e) {
    const a = wm.active()
    if (cmd === 'menu') togglePop('menu', items.menu)
    else if (cmd === 'search') togglePop('search', items.search, true)
    else if (cmd === 'switchNext' || cmd === 'switchPrev') {
      if (openPop !== 'switcher') { sel = a?.id ?? null; altCycle = !!e.altKey; togglePop('switcher', items.windows ?? items.menu) }
      step(cmd === 'switchNext' ? 1 : -1)
    } else if (!a) return
    else if (cmd === 'close') close(a.id)
    else if (cmd === 'windowMenu') winMenu(a.id, nodes.get(a.id).node.querySelector('.jr-winicon'))
    else { if (cmd === 'snapLeft') wm.snap(a.id, 'left'); if (cmd === 'snapRight') wm.snap(a.id, 'right'); if (cmd === 'maximise' && !a.max) wm.maximise(a.id); if (cmd === 'restore') { if (a.max) wm.maximise(a.id); else wm.snap(a.id, null) } render(); focusWin(a.id) }
  }

  bus.on('store:nasOnline', () => { renderOrbit(); renderContinue(); if (openPop === 'quick') togglePop('quick', items.status, true) })
  bus.on('store:notifications', renderOrbit)
  // B repair 3 (lead decision 5): a connect grant starting or ending (allow, end-temporary, app-closed, logout) updates the
  // Orbit status and the desktop privacy instrument at once; focus on an Orbit item survives the redraw
  let netSig = ''
  const syncNet = () => { const s = netText(); if (s !== netSig) { netSig = s; renderContinue() } }
  bus.on('permission', () => { const k = Object.keys(items).find((x) => items[x] === document.activeElement); renderOrbit(); if (k) items[k]?.focus() })
  bus.on('jr:sim', (g) => { if (g === 'desk') renderContinue(); if (g === 'search') drawResults() })
  renderContinue(); renderOrbit()

  return { /* after sign-in focus starts on the Menu button */
    show() { renderContinue(); items.menu?.focus() }, open, openSearch: () => togglePop('search', items.search, true), pop: (id) => togglePop(id, items[id === 'quick' ? 'status' : id === 'notes' ? 'clock' : id], true), searchFor, closePopovers, command,
    openFileForTest: TEST_OPEN ? (id) => openFile(id, TEST_OPEN) : undefined,
    // ?test=1 only: the trusted Bewerken of the active Notities window, so the sweep can measure the edit state (E10)
    editForTest: TEST_OPEN ? () => { const n = [...nodes].find(([, x]) => x.app === 'notities'); return n ? editAction(n[0], TEST_OPEN) : false } : undefined,
    chooser,
    count: () => wm.list().length, dirty: noteDirty, saveAll: saveNote, discardAll: discardNote, openFile, refresh,
    closeAll() { for (const id of [...nodes.keys()]) { nodes.get(id).node.remove(); nodes.delete(id); wm.close(id) } setNoteDoc(null); closePopovers(false); render() },
    regionTarget: () => { const a = wm.active(); return a ? nodes.get(a.id).node.querySelector('.jr-winicon') : document.querySelector('.jr-continue button') },
    focusFallback: () => { const a = wm.active(); a ? focusWin(a.id) : items.menu?.focus() },
  }
}
