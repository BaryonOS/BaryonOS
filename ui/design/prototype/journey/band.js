// Prototype band (outside the simulated OS): Rondleiding, Simulatie, Status, Sneltoetsen.
import { T, el, bus } from '../contract.js'
import { STATIONS, isBuilt, built, SIM_OPTIONS } from './state.js'

export function initBand(ctx) {
  const J = ctx.J
  const band = document.querySelector('.jr-band')
  const panels = {}
  const buttons = {}
  const head = (id, title) => el('div', { class: 'jr-panel-head' },
    el('h2', { id: `jr-${id}-h` }, title),
    el('button', { class: 'ds-btn', 'data-variant': 'quiet', type: 'button', on: { click: () => toggle(id, false) } }, J.band.close))

  function toggle(id, force) {
    const open = force ?? panels[id].hidden
    panels[id].hidden = !open
    buttons[id].setAttribute('aria-expanded', String(open))
    if (id === 'tour') try { sessionStorage.setItem('jr-tour', open ? 'open' : 'closed') } catch {}
    if (open) panels[id].querySelector('h2').focus()
    else if (panels[id].contains(document.activeElement) || document.activeElement === document.body) buttons[id].focus()
  }

  // Rondleiding
  const list = el('ol', { class: 'jr-tour-list' })
  const renderTour = () => {
    list.replaceChildren(...STATIONS.map((s) => {
      const t = J.stations[s.id]
      if (!isBuilt(s)) {
        return el('li', {}, el('div', { class: 'jr-tour-item', 'aria-disabled': 'true', tabindex: '0', 'aria-describedby': `jr-why-${s.id}` },
          el('span', { class: 'jr-tour-n' }, String(s.n)), el('span', { class: 'jr-tour-t' }, t.title),
          el('span', { class: 'jr-chip' }, J.tour.later),
          el('span', { class: 'jr-tour-what', id: `jr-why-${s.id}` }, J.tour.laterWhy)))
      }
      const here = ctx.station === s.id
      return el('li', {}, el('button', { class: 'jr-tour-item', type: 'button', 'aria-current': here ? 'step' : null, on: { click: () => ctx.go(s.id) } },
        el('span', { class: 'jr-tour-n' }, String(s.n)), el('span', { class: 'jr-tour-t' }, t.title),
        here ? el('span', { class: 'jr-chip', 'data-on': '' }, J.tour.here) : ctx.seen.has(s.id) ? el('span', { class: 'jr-chip' }, J.tour.seen) : null,
        el('span', { class: 'jr-tour-what' }, t.what)))
    }))
  }
  const next = () => {
    const b = built(); const i = b.findIndex((s) => s.id === ctx.station)
    if (i === b.length - 1) ctx.live(J.tour.done); else ctx.go(b[i + 1].id)
  }
  panels.tour = el('section', { class: 'jr-panel', id: 'jr-tour', 'aria-labelledby': 'jr-tour-h', hidden: true },
    head('tour', J.tour.title), el('p', { class: 'jr-lead' }, J.tour.intro), list,
    el('button', { class: 'ds-btn', 'data-variant': 'primary', type: 'button', on: { click: next } }, J.tour.next))

  // Simulatie: named groups, visible note; never inside the OS frame.
  const groups = { signin: [J.sim.signin, J.sim.nextInput, { correct: J.sim.correct, incorrect: J.sim.incorrect }],
    desk: [J.sim.desk, J.sim.continueList, { filled: J.sim.filled, empty: J.sim.empty }],
    launch: [J.sim.launch, J.sim.nextLaunch, { normal: J.sim.normal, slow: J.sim.slow, fail: J.sim.fail }],
    search: [J.sim.search, J.sim.index, { ready: J.sim.ready, updating: J.sim.updating, unavailable: J.sim.unavailable }],
    nas: [J.more.nas, J.more.nasLabel, { online: J.more.nasOnline, offline: J.more.nasOffline }] }
  const sets = Object.entries(groups).map(([g, [legend, label, opts]]) => el('fieldset', { class: 'ds-fieldset jr-sim-group' },
    el('legend', {}, legend),
    el('div', { class: 'ds-seg', role: 'radiogroup', 'aria-label': label }, SIM_OPTIONS[g].map((v) => el('label', { class: 'ds-radio' },
      el('input', { type: 'radio', name: `jr-sim-${g}`, value: v, checked: ctx.sim[g] === v, on: { change: () => ctx.setSim(g, v) } }), opts[v]))),
    g === 'signin' ? el('div', { class: 'jr-row' },
      el('button', { class: 'ds-btn', 'data-variant': 'secondary', type: 'button', on: { click: async () => { if (ctx.signedIn && !(await ctx.signOut())) return; ctx.session.oobe(1) } } }, J.sim.firstRun),
      el('button', { class: 'ds-btn', 'data-variant': 'secondary', type: 'button', id: 'jr-sim-lock', on: { click: () => ctx.signedIn && ctx.go('aanmelden') } }, J.sim.lockNow)) : null))
  bus.on('jr:sim', (g) => { for (const r of document.querySelectorAll(`input[name="jr-sim-${g}"]`)) r.checked = r.value === ctx.sim[g] })
  panels.sim = el('aside', { class: 'jr-panel', id: 'jr-sim', 'aria-label': J.sim.note, hidden: true },
    head('sim', J.sim.title),
    el('p', { class: 'jr-sim-note' }, el('strong', {}, J.sim.note), ' · ', J.sim.lead), el('div', { class: 'jr-sim-groups' }, sets))

  // Status: per built station, one table from STATIONS[*].feas (the sweep reads the same data)
  panels.status = el('section', { class: 'jr-panel', id: 'jr-status', 'aria-labelledby': 'jr-status-h', hidden: true },
    head('status', J.status.title), el('p', { class: 'jr-lead' }, J.status.lead, ' ', T.simulationLong, ' ', J.band.reloadNote),
    built().map((s) => el('div', { class: 'jr-table-wrap' }, el('table', { class: 'jr-table' },
      el('caption', {}, `${s.n}. ${J.stations[s.id].title}`),
      el('thead', {}, el('tr', {}, el('th', { scope: 'col' }, J.status.fn), el('th', { scope: 'col' }, J.status.state), el('th', { scope: 'col' }, J.status.source))),
      el('tbody', {}, s.feas.map(([fn, k, src]) => el('tr', {}, el('td', {}, fn), el('td', {}, ctx.feas(k)), el('td', {}, src))))))))

  // Sneltoetsen
  panels.keys = el('section', { class: 'jr-panel', id: 'jr-keys', 'aria-labelledby': 'jr-keys-h', hidden: true },
    head('keys', J.keys.title), el('p', { class: 'jr-lead' }, J.keys.lead),
    el('div', { class: 'jr-table-wrap' }, el('table', { class: 'jr-table' },
      el('thead', {}, el('tr', {}, el('th', { scope: 'col' }, J.keys.action), el('th', { scope: 'col' }, J.keys.native), el('th', { scope: 'col' }, J.keys.here))),
      el('tbody', {}, J.keys.rows.map(([a, n, h]) => el('tr', {}, el('td', {}, a), el('td', {}, n), el('td', {}, h === J.keys.onlyReal ? h : el('kbd', { class: 'ds-kbd' }, h))))))))

  // Bypass block (WCAG 2.4.1): the first stop jumps past the open panels to the first control of the simulated OS
  const skip = el('button', { class: 'ds-btn', 'data-variant': 'quiet', type: 'button', on: { click: () => [...document.querySelectorAll('#jr-os button, #jr-os input')].find((x) => x.offsetParent && !x.closest('[inert], [hidden]'))?.focus() } }, J.more.skip)
  const nav = el('div', { class: 'jr-band-tools' }, skip, ['tour', 'sim', 'status', 'keys'].map((id) => {
    buttons[id] = el('button', { class: 'ds-btn', 'data-variant': 'quiet', type: 'button', 'aria-expanded': 'false', 'aria-controls': `jr-${id}`, on: { click: () => toggle(id) } }, J.band[id === 'keys' ? 'keys' : id])
    return buttons[id]
  }))
  band.append(el('div', { class: 'jr-band-id' }, el('h1', {}, J.band.title), el('p', {}, T.simulation)), nav)
  band.after(panels.tour, panels.status, panels.keys, panels.sim)
  for (const p of Object.values(panels)) p.querySelector('h2').setAttribute('tabindex', '-1')

  renderTour()
  bus.on('jr:station', renderTour)
  bus.on('jr:session', () => { document.getElementById('jr-sim-lock').disabled = !ctx.signedIn })
  document.getElementById('jr-sim-lock').disabled = true
  let seen = null
  try { seen = sessionStorage.getItem('jr-tour') } catch {}
  if (seen !== 'closed') { panels.tour.hidden = false; buttons.tour.setAttribute('aria-expanded', 'true') }
  return { toggle }
}
