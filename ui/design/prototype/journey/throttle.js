// Pure change queue: at most one apply per `gap` ms; the latest value wins (ds-1.1 B8,
// WCAG 2.3.1: holding an arrow key on a theme radio cannot flash the whole screen).
export function createThrottle(apply, { gap = 400, now = () => Date.now(), later = (fn, ms) => setTimeout(fn, ms) } = {}) {
  let last = -Infinity
  let pending = null
  let timer = false
  const flush = () => { timer = false; last = now(); const v = pending; pending = null; apply(v) }
  return (v) => {
    pending = v
    if (timer) return
    const wait = last + gap - now()
    if (wait <= 0) flush()
    else { timer = true; later(flush, wait) }
  }
}
