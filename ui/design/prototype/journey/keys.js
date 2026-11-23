// Pure shortcut table and matcher. A keydown with repeat never activates (ds-1.1 B8).
// Browser-reserved keys (Ctrl+W, Ctrl+Tab, Ctrl+L) are deliberately not bound.

const sig = (e) => [e.ctrlKey && 'Ctrl', e.altKey && 'Alt', e.shiftKey && 'Shift', e.metaKey && 'Meta'].filter(Boolean).join('+')

// [modifiers, key-or-code, command, worksInTextFields]
export const TABLE = [
  ['Ctrl', 'Escape', 'menu', true],
  ['Meta', 'Meta', 'menu', false],
  ['Ctrl', 'k', 'search', true],
  ['Alt', 'Backquote', 'switchNext', true],
  ['Alt+Shift', 'Backquote', 'switchPrev', true],
  ['Alt', 'F4', 'close', true],
  ['Alt', 'w', 'close', true],
  ['Alt+Shift', 'ArrowLeft', 'snapLeft', true],
  ['Alt+Shift', 'ArrowRight', 'snapRight', true],
  ['Alt+Shift', 'ArrowUp', 'maximise', true],
  ['Alt+Shift', 'ArrowDown', 'restore', true],
  ['Shift', 'F10', 'windowMenu', false],
  ['', 'F6', 'regionNext', true],
  ['Shift', 'F6', 'regionPrev', true],
  ['Ctrl+Alt', 'l', 'lock', true],
  ['Ctrl+Alt', 'End', 'system', true],
  ['', 'F1', 'help', true],
]

const keyOf = (e) => (e.code === 'Backquote' ? 'Backquote' : String(e.key).length === 1 ? e.key.toLowerCase() : e.key)

// match(e, {inText}) -> command or null
export function match(e, { inText = false } = {}) {
  if (e.repeat) return null
  const s = sig(e)
  const k = keyOf(e)
  for (const [mods, key, cmd, txt] of TABLE) {
    if (mods === s && key === k && (!inText || txt)) return cmd
  }
  return null
}

// Lock screen: any key except navigation and modifiers wakes the sign-in surface
const NOWAKE = new Set(['Tab', 'Shift', 'Control', 'Alt', 'Meta', 'F1', 'F6', 'CapsLock', 'ContextMenu'])
export const wakes = (e) => !e.repeat && !NOWAKE.has(e.key) && !e.ctrlKey && !e.altKey && !e.metaKey

// ds-1.1 B8 (WCAG 2.3.1), as in gallery.js: one activation per physical key press. An auto-repeated Enter or Space on a
// control is swallowed, and so is a repeated Enter in a single-line field (it would submit the form again). Arrow keys and
// text editing keep their repeat. The same guard runs for every page element (journey.js, capture phase).
const REPEAT_TARGET = 'button, [role="button"], [role="switch"], [role="menuitem"], [role="tab"], [role="option"], [role="listbox"], summary, input'
export const swallowRepeat = (e) => !!e.repeat && (e.key === 'Enter' || (e.key === ' ' && !e.target?.matches?.('input:not([type=checkbox]):not([type=radio]):not([type=button]):not([type=submit]):not([type=reset])')))
  && !!e.target?.matches?.(REPEAT_TARGET)
