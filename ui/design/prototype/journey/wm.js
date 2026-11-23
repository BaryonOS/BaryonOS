// Pure window-stack model. No DOM. stack[0] is the active (most recently used) window;
// DOM order and focus order follow this array. Minimised windows keep their MRU slot.

export const SINGLE = new Set(['bestanden', 'instellingen', 'rekenmachine', 'notities'])

export function createWm({ emit = () => {} } = {}) {
  let seq = 0
  const wins = [] // MRU order
  const m = {
    stackMode: false,
    list: () => wins.slice(),
    visible: () => wins.filter((w) => !w.min),
    active: () => wins.find((w) => !w.min) ?? null,
    get: (id) => wins.find((w) => w.id === id) ?? null,
    open(app) {
      const ex = SINGLE.has(app) && wins.find((w) => w.app === app)
      if (ex) { m.focus(ex.id); return ex }
      const w = { id: `w${++seq}`, app, min: false, max: false, snap: null }
      wins.unshift(w)
      emit('open', w)
      return w
    },
    focus(id) {
      const i = wins.findIndex((w) => w.id === id)
      if (i < 0) return null
      const [w] = wins.splice(i, 1)
      w.min = false
      wins.unshift(w)
      emit('focus', w)
      return w
    },
    // hiding ends nothing (OQ-5): no appClosed
    minimise(id) {
      const i = wins.findIndex((w) => w.id === id)
      if (i < 0) return null
      const [w] = wins.splice(i, 1)
      w.min = true
      wins.push(w)
      emit('minimise', w)
      return m.active()
    },
    close(id) {
      const i = wins.findIndex((w) => w.id === id)
      if (i < 0) return null
      const [w] = wins.splice(i, 1)
      emit('close', w)
      if (!wins.some((x) => x.app === w.app)) emit('appClosed', w.app)
      return m.active()
    },
    maximise(id) { const w = m.get(id); if (w && !m.stackMode) { w.max = !w.max; w.snap = null; emit('layout', w) } return w },
    snap(id, side) { const w = m.get(id); if (w && !m.stackMode) { w.snap = side; w.max = false; emit('layout', w) } return w },
    isMax: (w) => m.stackMode || w.max,
    // switcher order: MRU, minimised included; step from the active window
    cycle(from, dir = 1) {
      const n = wins.length
      if (!n) return null
      const i = Math.max(0, wins.findIndex((w) => w.id === from))
      return wins[(i + dir + n) % n]
    },
  }
  return m
}
