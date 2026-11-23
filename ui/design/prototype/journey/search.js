// Pure local search index over apps, files, settings and actions. No DOM, no network.
export const GROUPS = ['apps', 'files', 'settings', 'actions', 'assistant']

export const fold = (s) => String(s).normalize('NFD').replace(/[̀-ͯ]/g, '').replace(/['’]/g, '').toLowerCase()

function matches(text, q) {
  const t = fold(text)
  return t.startsWith(q) || t.split(/[\s›,()-]+/).some((w) => w.startsWith(q))
}

// items: [{group, id, label, sub?, terms?}]
export function buildIndex({ T, DATA, files, actions, name = (id) => T.apps[id] ?? id }) {
  const items = []
  for (const a of DATA.apps) items.push({ group: 'apps', id: a.id, label: name(a.id) })
  const coll = Object.fromEntries(DATA.collections.map((c) => [c.id, c.name]))
  for (const f of files) items.push({ group: 'files', id: f.id, label: f.name, sub: coll[f.collection] ?? '', place: f.storage, terms: [coll[f.collection] ?? ''] })
  const cat = T.instellingen.categories
  for (const [id, label] of Object.entries(cat)) items.push({ group: 'settings', id, label, sub: 'Instellingen' })
  for (const [axis, [label]] of Object.entries(T.journey.a11y.axes)) items.push({ group: 'settings', id: `toegankelijkheid:${axis}`, label: `${cat.toegankelijkheid} › ${label}`, page: 'toegankelijkheid' })
  items.push({ group: 'settings', id: 'weergave:theme', label: `${cat.weergave} › ${T.journey.shell.themes.contrast}`, page: 'weergave' })
  for (const id of actions) items.push({ group: 'actions', id, label: T.journey.search.actions[id] })
  return items
}

// -> [{group, items, total}] in fixed order, at most `per` per group; assistant row when q non-empty
export function query(index, raw, { internet = false, per = 5 } = {}) {
  const q = fold(raw.trim())
  if (!q) return []
  const out = []
  for (const g of GROUPS) {
    if (g === 'assistant') { out.push({ group: g, items: [{ group: g, id: 'assistant', label: raw.trim() }], total: 1 }); continue }
    const hit = index.filter((it) => it.group === g && (matches(it.label, q) || (it.terms ?? []).some((t) => matches(t, q))))
    if (hit.length) out.push({ group: g, items: hit.slice(0, per), total: hit.length })
  }
  if (internet) out.push({ group: 'internet', items: [], total: 0 })
  return out
}
