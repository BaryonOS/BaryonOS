// Pure journey model: station registry, router and session phases. No DOM.
// UX-E slice A. Simulation only: nothing here is enforced by BaryonOS.

import { T } from '../strings.js'

export const BUILT = 'A' // slices built so far, in order A < B < C
export const FEAS = ['now', 'feasible', 'research', 'simulated']

// feas rows: [function (Dutch, shown in the Status table; text in strings.js T.journey.more.statusRows), label key, source]
const R = T.journey.more.statusRows
const rows = (id, specs) => specs.map(([k, src], i) => [R[id][i], k, src])
export const STATIONS = [
  { n: 1, id: 'aanmelden', slice: 'A', states: ['empty', 'loading', 'error', 'success'], feas: rows('aanmelden', [
    ['now', 'PROFILES-AND-USERS §3'], ['feasible', 'PROFILES-AND-USERS invariant 8'], ['simulated', 'dit prototype'],
    ['feasible', 'PROFILES-AND-USERS §3 Recovery'], ['feasible', 'PROFILES-AND-USERS §3'], ['simulated', 'PROFILES-AND-USERS §2'],
  ]) },
  { n: 2, id: 'bureaublad', slice: 'A', states: ['empty', 'success'], feas: rows('bureaublad', [
    ['now', 'DESKTOP-SHELL §2'], ['feasible', 'DESKTOP-SHELL D3'], ['simulated', 'NAMING-AND-IA §5'], ['simulated', 'NAS-SHARING-AND-INTERNET'],
  ]) },
  { n: 3, id: 'vensters', slice: 'A', states: ['empty', 'loading', 'error', 'success'], feas: rows('vensters', [
    ['now', 'DESKTOP-SHELL §2'], ['now', 'DESKTOP-SHELL §2'], ['feasible', 'DESKTOP-SHELL D2'], ['simulated', 'contract.js files.save'],
    ['simulated', 'DESIGN-SYSTEM §11.1 OQ-5'],
  ]) },
  { n: 4, id: 'zoeken', slice: 'A', states: ['empty', 'loading', 'error', 'none', 'success'], feas: rows('zoeken', [
    ['now', 'NAMING-AND-IA §5'], ['feasible', 'DESKTOP-SHELL D4'], ['simulated', 'DESKTOP-SHELL §2'], ['feasible', 'NAS-SHARING-AND-INTERNET'],
  ]) },
  { n: 5, id: 'bestanden', slice: 'B', states: [], feas: [] },
  { n: 6, id: 'versies', slice: 'B', states: [], feas: [] },
  { n: 7, id: 'toestemming', slice: 'B', states: [], feas: [] },
  { n: 8, id: 'instellingen', slice: 'B', states: [], feas: [] },
  { n: 9, id: 'meldingen', slice: 'B', states: [], feas: [] },
  { n: 10, id: 'herstel', slice: 'C', states: [], feas: [] },
]

export const isBuilt = (st) => st.slice <= BUILT
export const byId = (id) => STATIONS.find((s) => s.id === id)
export const built = () => STATIONS.filter(isBuilt)

export const serialize = (id) => `#/${id}`

// parse(hash, {signedIn}) -> {station, pending, later}
//  - unknown/empty: bureaublad when signed in, aanmelden otherwise
//  - a later-slice station: stays where it is with later=id (never a broken route)
//  - before sign-in: everything except aanmelden and herstel routes to aanmelden, pending keeps the target
export function parse(hash, { signedIn = false } = {}) {
  const id = String(hash || '').replace(/^#\/?/, '')
  const st = byId(id)
  const home = signedIn ? 'bureaublad' : 'aanmelden'
  if (!st) return { station: home, pending: null, later: null }
  if (!isBuilt(st)) return { station: home, pending: null, later: st.id }
  if (!signedIn && st.id !== 'aanmelden' && st.id !== 'herstel') return { station: 'aanmelden', pending: st.id, later: null }
  return { station: st.id, pending: null, later: null }
}

// Effects the shell must run to show a station. go() only opens or focuses:
// it never closes a window, discards a draft or resets simulation state.
export function go(id, { signedIn = false, windows = 0 } = {}) {
  const r = parse(serialize(id), { signedIn })
  const fx = []
  if (r.later) return { ...r, effects: [['announceLater', r.later]] }
  if (r.station === 'aanmelden' && signedIn) fx.push(['lock'])
  else if (r.station === 'aanmelden') fx.push(['showLock'])
  if (r.station === 'bureaublad') fx.push(['closePopovers'])
  if (r.station === 'vensters') { fx.push(['open', 'notities']); if (windows < 2) fx.push(['open', 'rekenmachine']) }
  if (r.station === 'zoeken') fx.push(['openSearch'])
  fx.push(['announce', r.station])
  return { ...r, effects: fx }
}

// Session phases: locked -> chooser -> (recovery -> newsecret) -> starting -> desktop; oobe -> starting
const PHASES = {
  locked: ['chooser', 'oobe'],
  chooser: ['locked', 'recovery', 'starting', 'oobe'],
  recovery: ['chooser', 'newsecret'],
  newsecret: ['starting'],
  oobe: ['locked', 'starting'],
  starting: ['desktop'],
  desktop: ['locked', 'system', 'chooser'],
  system: ['desktop', 'locked', 'chooser'],
}
export const canMove = (from, to) => (PHASES[from] ?? []).includes(to)

// Simulation state (the band's Simulatie panel). Each group maps to one station's states.
export const SIM_DEFAULTS = { signin: 'correct', desk: 'filled', launch: 'normal', search: 'ready', nas: 'online' }
export const SIM_OPTIONS = {
  signin: ['correct', 'incorrect'],
  desk: ['filled', 'empty'],
  launch: ['normal', 'slow', 'fail'],
  search: ['ready', 'updating', 'unavailable'],
  nas: ['online', 'offline'],
}

// Fixed search actions (ids resolve to T.journey.search.actions)
export const ACTIONS = ['lock', 'dark', 'light', 'contrast', 'nas']
