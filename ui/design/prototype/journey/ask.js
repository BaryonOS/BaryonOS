// UX-E slice B, station 7: the journey's live permission prompt. It presents contract.js ask() requests in #jr-ask with the
// renderer the gallery uses (ask-view.js), and mirrors the gallery's controller: Toestaan and Weigeren equal, "Keuze onthouden"
// unchecked at every opening, focus on the question, the OQ-1 confirm step in the same dialog, Escape or closing without a
// choice = fail-closed cancel, focus back to the opener. DESIGN-SYSTEM.md §11.1. PROTOTYPE ONLY: nothing is enforced.
import { icon, setPrompt, appReason, ask as contractAsk } from '../contract.js'
import { renderAsk, syncAsk, what, APP, TR } from '../ask-view.js'

export function initAsk(ctx) {
  const dlg = document.getElementById('jr-ask')
  const $ = (s) => dlg.querySelector(s)
  let live = null, opener = null, failNext = false, last = null
  const settle = (res) => { const l = live; live = null; l?.resolve(res) }
  const hydrate = () => { for (const ph of dlg.querySelectorAll('i[data-icon]')) { const svg = icon(ph.dataset.icon); if (ph.className) svg.classList.add(...ph.className.split(/\s+/).filter(Boolean)); ph.replaceWith(svg) } }
  // MF-1 as in the gallery: in a short viewport or at 200 % the outcome sentences move to the end of the scrolling body
  const short = globalThis.matchMedia?.('(max-height: 40rem)')
  function placeOutcomes() {
    const ul = dlg.querySelector('.ds-ask-main .ds-ask-outcomes'); if (!ul) return
    if (short?.matches || document.documentElement.dataset.scale === '2') $('.ds-ask-main .ds-ask-body').append(ul)
    else $('.ds-ask-main .ds-ask-decide').insertBefore(ul, $('.ds-ask-main .ds-ask-choices'))
  }
  short?.addEventListener?.('change', placeOutcomes)

  setPrompt((req, view) => new Promise((resolve) => {
    // Simulatie "Volgende vraag kan niet worden getoond": present throws, contract.js answers via 'error' (fail closed)
    if (failNext) { failNext = false; throw new Error('prompt not shown (simulation)') }
    dlg.innerHTML = renderAsk(req, { p: 'jr-ask', k: ctx.kern(), view })
    hydrate()
    for (const n of dlg.querySelectorAll('[data-reason]')) n.textContent = appReason(req.reason) ?? '' // OQ-6: text, never markup
    placeOutcomes()
    const inner = $('.ds-ask')
    inner.dataset.kern = '' // follows theme changes like every Kern surface (journey.js applyAxes)
    inner.setAttribute('role', 'none'); inner.removeAttribute('aria-labelledby'); inner.removeAttribute('aria-describedby')
    dlg.setAttribute('aria-labelledby', 'jr-ask-t'); dlg.setAttribute('aria-describedby', 'jr-ask-f')
    live = { resolve, req, view }
    opener = document.activeElement
    const f = $('[data-initial-focus]')
    f.autofocus = true
    dlg.returnValue = ''
    dlg.showModal()
    f.focus() // the question, never a choice (MF-4)
  }))
  dlg.addEventListener('close', () => {
    if (dlg.open) return
    settle({ choice: 'cancel' }) // Escape or closing without a choice: fail closed, nothing stored (no-op once answered)
    if (opener?.isConnected) opener.focus(); else ctx.shell?.focusFallback()
    opener = null
  })
  dlg.addEventListener('change', (e) => { if (e.target.matches('[data-remember]') && live) syncAsk($('.ds-ask'), live.req, live.view) })
  dlg.addEventListener('click', (e) => {
    const choice = e.target.closest('[data-choice]')
    if (choice && live) {
      const on = $('[data-remember]').checked
      if (choice.dataset.choice === 'allow' && on && live.view.confirmAllow) { // OQ-1: the extra confirmation, in the same dialog
        $('.ds-ask-main').hidden = true; $('.ds-ask-confirm').hidden = false
        dlg.setAttribute('aria-labelledby', 'jr-ask-ct'); dlg.setAttribute('aria-describedby', 'jr-ask-cd')
        return $('[data-confirm-focus]').focus()
      }
      settle({ choice: choice.dataset.choice, remember: on }); return dlg.close()
    }
    const conf = e.target.closest('[data-confirm]')
    if (conf && live) {
      if (conf.dataset.confirm === 'ok') { settle({ choice: 'allow', remember: true, confirmed: true }); return dlg.close() }
      $('.ds-ask-confirm').hidden = true; $('.ds-ask-main').hidden = false
      dlg.setAttribute('aria-labelledby', 'jr-ask-t'); dlg.setAttribute('aria-describedby', 'jr-ask-f')
      return $('[data-initial-focus]').focus()
    }
  })

  // ask from an in-app gesture. Returns the contract result plus the quiet-note text the app shows in its own window (never
  // a focus-stealing dialog). `gesture` is false only for the Simulatie "App vraagt automatisch opnieuw" (R2 quiet denial).
  async function request(r, { gesture = true } = {}) {
    last = r
    const res = await contractAsk(r, { gesture })
    const a = APP[r.app] ?? r.app, w = what(r), R = TR.result, Q = TR.quiet
    const text = res.via === 'remembered' ? (res.answer === 'allow' ? Q.covered(a, w) : Q.deniedRemembered(a, w))
      : res.via === 'covered' ? R.allow['app-session'](a, w)
      : res.via === 'temporary-deny' ? Q.tempDeny(a, w, 1)
      : res.via === 'cancel' ? R.cancel(a)
      : res.via === 'error' ? R.error(a)
      : res.remembered ? R.remembered(a, w, TR.grant.decision[res.answer])
      : res.answer === 'allow' ? R.allow[res.until](a, w) : R.deny(a)
    return { ...res, text }
  }
  return {
    request,
    failNext: () => { failNext = true },
    repeatLast: () => (last ? request(last, { gesture: false }) : null),
    hasLast: () => !!last,
    isOpen: () => dlg.open,
    cancelForTest: () => { if (dlg.open) dlg.close() },
  }
}
