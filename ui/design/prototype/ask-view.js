// Permission prompt and Appmachtigingen markup, factored verbatim out of ui/design/system/gallery.js (UX-E slice B) so the
// gallery and the journey render the same .ds-ask prompt and .ds-grants rows. Pure string builders plus syncAsk; the gallery
// keeps its own specimens, live controller and sample requests. DESIGN-SYSTEM.md §11.1. PROTOTYPE ONLY.
import { icon, T, files, grants, appReason, rememberAllowOffered, needsConfirm } from './contract.js'
const $ = (s, el = document) => el.querySelector(s)

export const TR = T.trusted
export const APP = { notities: T.apps.notities, browser: T.apps.webvenster, fotos: T.apps.fotos }
export const APP_ICON = { notities: 'notities', browser: 'browser', fotos: 'fotos' }
export const INSTALLED = { notities: '3 nov.', browser: '12 nov.', fotos: '3 nov.' } // gallery sample data (not strings.js copy; O-D1)
export const esc = (s) => String(s).replace(/[&<>"']/g, (ch) => `&#${ch.charCodeAt(0)};`)
export const KEPT = { local: ['device'], nas: ['nas'], both: ['device', 'nas'] }
export const badge = (place, label) => `<span class="ds-place" data-place="${place}"><i class="ds-node" data-shape="${place}" aria-hidden="true"></i><i data-icon="${place}"></i>${esc(label)}</span>`
export const SENSOR = { camera: 'Camera', microphone: 'Microfoon', location: 'Locatie' } // device sensors: the item is the sensor itself
export const itemName = (r) => SENSOR[r.right] ?? files.get(r.data)?.name ?? r.data
export const places = (r) => (SENSOR[r.right] ? ['device'] : KEPT[files.get(r.data)?.storage] ?? ['device'])
export const what = (r) => TR.what[r.right](itemName(r), r.destination)
export const words = (r) => { const G = TR.grant; return [APP[r.app] ?? r.app, G.rights[r.right] ?? r.right, itemName(r), r.destination ?? TR.key.none, G.scopes[r.scope] ?? r.scope] }
// the five-field key as chips: the same markup in the prompt's disclosure and in every Appmachtigingen row
export const keyChips = (r) => `<dl class="ds-key">${['app', 'right', 'data', 'destination', 'scope'].map((f, i) => `<div><dt>${TR.key.labels[f]}</dt><dd>${esc(words(r)[i])}</dd></div>`).join('')}</dl>`
// state sentence and outcome sentences for the remember checkbox state (MF-1: each states its real lifetime)
export function lines(r, view, on) {
  const a = APP[r.app], w = what(r), O = TR.outcome
  if (!on) return { state: r.kind === 'action' ? TR.state.offAction(a) : TR.state.off(a), allow: O.allow[r.kind](a, w), deny: O.deny[r.kind](a) }
  if (!view.rememberAllow) return { state: TR.state.onDenyOnly, allow: O.allow.denyOnly(a, w), deny: O.deny.remembered(a) }
  if (view.confirmAllow) return { state: TR.state.onConfirm, allow: O.allow.confirm(a, w), deny: O.deny.remembered(a) }
  return { state: TR.state.on, allow: O.allow.remembered(a, w), deny: O.deny.remembered(a) }
}
export const viewOf = (r, compact = false) => ({ compact, rememberAllow: rememberAllowOffered(r), confirmAllow: needsConfirm(r), reason: appReason(r.reason) })

// a leading icon per shell fact names its consequence class (in the order of TR.facts); the text stays the carrier
export const FACT_ICON = {
  read: ['internet', 'toegang', 'nas'], readWrite: ['draw', 'internet', 'toegang', 'nas'],
  connect: ['internet', 'web', 'toegang'], 'save-copy': ['device', 'internet'], 'send-copy': ['warn', 'web'],
  'read-connect': ['internet', 'warn', 'toegang'], camera: ['internet', 'toegang', 'microfoon'], microphone: ['internet', 'toegang', 'camera'],
}
// one renderer for the live prompt and every specimen (p = id prefix, k = Kern theme)
export function renderAsk(r, { p, k, view = viewOf(r), on = false, spec = '', preset = false, confirm = false }) {
  const a = APP[r.app], L = lines(r, view, on), cmp = TR.compare[r.right], held = r.right === 'read-connect'
  const F = TR.facts[r.right]
  const FI = FACT_ICON[r.right] ?? []
  const facts = (['read', 'readWrite'].includes(r.right) ? F(a, T.instellingen.opslag.nas, places(r).includes('nas')) : F(a, ...(r.right === 'read-connect' ? [itemName(r), r.destination] : [r.destination])))
    .map((t, i) => [FI[i] ?? 'dot', t]).filter(([, t]) => t)
  const src = r.right === 'connect' ? `<li class="ds-ask-stop" data-held><span class="ds-ask-stop-name">${esc(TR.path.appData(a))}</span></li>` // no document: the route starts at what the app already holds
    : `<li class="ds-ask-stop"${held ? ' data-held' : ''}><span class="ds-ask-stop-name">${esc(itemName(r))}</span>${places(r).map((pl) => badge(pl, T.instellingen.opslag[pl])).join('')}${held ? `<span class="ds-ask-held">${esc(a)} ${TR.path.already}</span>` : ''}</li>`
  const app = `<li class="ds-ask-stop" data-stop="app"><span class="ds-ask-link" aria-hidden="true"><i data-icon="chev"></i></span><span class="ds-ask-app" aria-hidden="true"><i data-icon="${APP_ICON[r.app]}"></i></span><span class="ds-ask-stop-name">${esc(a)}</span></li>`
  const dest = r.destination ? `<li class="ds-ask-stop" data-place="internet"><span class="ds-ask-link" aria-hidden="true"><i data-icon="chev"></i></span><span class="ds-ask-stop-name">${esc(r.destination)}</span>${badge('internet', T.instellingen.privacy.internet)}</li>` : ''
  const details = `<ol class="ds-ask-path" aria-label="${TR.path.label}">${src}${app}${dest}</ol>`
    + `<p class="ds-ask-holds">${esc(TR.path.holds(r.destination ?? a))}</p>`
    + (cmp ? `<div class="ds-ask-explain"><p>${esc(cmp.line)}</p>${cmp.why ? `<details><summary>${esc(cmp.why)}</summary><p>${esc(cmp.whyText)}</p></details>` : ''}</div>` : '')
    + `<details class="ds-ask-keybox"><summary>${TR.key.disclosure}</summary>${keyChips(r)}</details>`
  const body = `<div class="ds-ask-main"><div class="ds-ask-body">`
    + (spec ? `<p class="gal-spec">${esc(spec)}</p>` : '')
    + `<p class="ds-ask-proto"><i data-icon="warn"></i><span>${TR.prototype}</span></p>`
    + (preset ? `<p class="ds-ask-proto" data-preset><i data-icon="warn"></i><span>Testvoorinstelling: deze vraag opende met Keuze onthouden aan. Normaal staat het bij elke vraag uit.</span></p>` : '')
    + `<h3 class="ds-ask-title" id="${p}-t" tabindex="-1" data-initial-focus>${esc(TR.question[r.right](a, itemName(r), r.destination))}</h3>`
    + `<p class="ds-ask-from"><span class="ds-ask-app" aria-hidden="true"><i data-icon="${APP_ICON[r.app]}"></i></span><span><strong>${esc(a)}</strong> · ${esc(TR.provenance(INSTALLED[r.app]))}</span></p>`
    + (view.compact ? `<p class="ds-ask-again">${esc(TR.compact.note(a))}</p>` : '')
    + `<ul class="ds-ask-facts" id="${p}-f">${facts.map(([ic, f]) => `<li><span class="ds-ask-fact-icon" aria-hidden="true"><i data-icon="${ic}"></i></span><span>${esc(f)}</span></li>`).join('')}</ul>`
    + (view.reason ? `<p class="ds-ask-reason"><span class="ds-ask-reason-who">${esc(TR.reason(a))}</span> <span data-reason></span><span class="ds-ask-reason-tag">${TR.reasonTag}</span></p>` : '')
    + (view.compact ? `<details class="ds-ask-more"><summary>${TR.compact.details}</summary>${details}</details>` : details)
    + `</div><div class="ds-ask-decide"><div class="ds-ask-lifetime">`
    + `<label class="ds-check ds-ask-remember"><input type="checkbox" data-remember${on ? ' checked' : ''} aria-describedby="${p}-s">${TR.remember}</label>`
    + `<p class="ds-ask-state" id="${p}-s" data-on="${on}"><i data-icon="${on ? 'altijd' : 'klok'}"></i><span>${esc(L.state)}</span></p></div>`
    + `<ul class="ds-ask-outcomes"><li id="${p}-oa">${esc(L.allow)}</li><li id="${p}-od">${esc(L.deny)}</li></ul>` // MF-1: next to the answers, never scrolled away
    + `<div class="ds-ask-choices" role="group" aria-label="${TR.choicesLabel}">`
    + `<button class="ds-btn" data-variant="secondary" data-size="large" data-choice="allow" aria-describedby="${p}-oa">${TR.choices.allow}</button>`
    + `<button class="ds-btn" data-variant="secondary" data-size="large" data-choice="deny" aria-describedby="${p}-od">${TR.choices.deny}</button></div></div></div>`
  const conf = needsConfirm(r) ? `<div class="ds-ask-confirm"${confirm ? '' : ' hidden'}><div class="ds-ask-body">`
    + (spec ? `<p class="gal-spec">${esc(spec)}</p>` : '')
    + `<p class="ds-ask-proto"><i data-icon="warn"></i><span>${TR.prototype}</span></p>`
    + `<h3 class="ds-ask-title" id="${p}-ct" tabindex="-1" data-confirm-focus>${TR.confirm.title}</h3><p class="ds-ask-confirm-text" id="${p}-cd">${esc(TR.confirm.text(a, itemName(r), r.destination))}</p>${keyChips(r)}</div>`
    + `<div class="ds-ask-decide"><div class="ds-ask-choices"><button class="ds-btn" data-variant="secondary" data-size="large" data-confirm="back">${TR.confirm.back}</button><button class="ds-btn" data-variant="secondary" data-size="large" data-confirm="ok">${TR.confirm.ok}</button></div></div></div>` : ''
  return `<div class="ds-ask" data-theme="${k}" data-kind="${r.kind}"${view.compact ? ' data-compact' : ''}${confirm ? ' data-step="confirm"' : ''} role="alertdialog" aria-labelledby="${p}-${confirm ? 'ct' : 't'}" aria-describedby="${p}-${confirm ? 'cd' : 'f'}">`
    + `<div class="ds-ask-band"><i data-icon="lock"></i><span>${TR.origin}</span></div>${confirm ? body.replace('class="ds-ask-main"', 'class="ds-ask-main" hidden') : body}${conf}</div>`
}
export function syncAsk(ask, r, view) { // the checkbox changed: state sentence, icon and both outcome sentences follow
  const on = $('[data-remember]', ask).checked, L = lines(r, view, on), st = $('.ds-ask-state', ask)
  st.dataset.on = on; $('span', st).textContent = L.state
  st.querySelector('svg').replaceWith(icon(on ? 'altijd' : 'klok'))
  $('.ds-ask-outcomes > :first-child', ask).textContent = L.allow
  $('.ds-ask-outcomes > :last-child', ask).textContent = L.deny
}
// Appmachtigingen rows (the .ds-grants list): one markup for the gallery and the journey
export function grantRows(list) {
  const G = TR.grant
  return list.length ? list.map((g, i) => {
      const key = words(g).join(', '), to = g.decision === 'allow' ? 'deny' : 'allow'
      const change = to === 'deny' || grants.canChangeToAllow(g) ? `<button class="ds-btn" data-variant="secondary" data-change="${i}" data-to="${to}" aria-label="${esc(G.changeName(G.change[to], key))}">${G.change[to]}</button>` : ''
      return `<li class="ds-grant" data-decision="${g.decision}"><span class="ds-grant-app" aria-hidden="true"><i data-icon="${APP_ICON[g.app] ?? 'dot'}"></i></span>`
        + `<div><p class="ds-grant-name">${esc(APP[g.app] ?? g.app)} · ${esc(what(g))}</p><div class="ds-grant-key"><span>${G.precise}</span>${keyChips(g)}</div>`
        + `<p class="ds-grant-reach">${g.destination ? badge('internet', `${T.instellingen.privacy.internet} · ${g.destination}`) : places(g).map((pl) => badge(pl, T.instellingen.opslag[pl])).join('')}<span class="ds-grant-until" data-decision="${g.decision}"><i data-icon="altijd"></i>${G.decision[g.decision]}</span></p></div>`
        + `<div class="ds-grant-actions">${change}<button class="ds-btn" data-variant="secondary" data-forget="${i}" aria-label="${esc(G.forgetName(key))}">${G.forget}</button></div></li>`
    }).join('') : `<li><p class="ds-grant-empty" tabindex="-1">${G.empty}</p></li>`
}
