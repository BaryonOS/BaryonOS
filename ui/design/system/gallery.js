// BaryonOS design system ds-1.0: component gallery behaviour. DESIGN ARTEFACT, 14 November 2026; not a working OS.
// Builds one specimen set per theme from <template id="tpl-panel"> ({p} = id prefix, {k} = Kern theme, the opposite
// theme per MF1), renders icons with contract.js icon(), lists token swatches, and wires keyboard behaviour:
// Tab/Shift+Tab (native), Space/Enter (native buttons), arrows for tabs, listbox rows and menu (radios are native),
// Escape closes the menu, dialogs and tooltips. Query: ?only=dark|light|contrast|scale|ref ?motion=reduce ?measure=1
import { icon, T } from '../prototype/contract.js'
import { ICONS } from './icons.js'

const q = new URLSearchParams(location.search)
const root = document.documentElement
const $ = (s, el = document) => el.querySelector(s)
const $$ = (s, el = document) => [...el.querySelectorAll(s)]
const THEMES = [['dark', 'Donker', 'light'], ['light', 'Licht', 'dark'], ['contrast', 'Hoog contrast', 'contrast']]
const KERN = Object.fromEntries(THEMES.map(([p, , k]) => [p, k]))
if (q.get('motion') === 'reduce') root.dataset.motion = 'reduce'

const fill = (id, p, k) => document.getElementById(id).innerHTML.replaceAll('{p}', p).replaceAll('{k}', k)
function hydrate(scope) {
  for (const ph of $$('i[data-icon]', scope)) {
    const svg = icon(ph.dataset.icon)
    if (ph.className) svg.classList.add(...ph.className.split(/\s+/).filter(Boolean))
    ph.replaceWith(svg)
  }
}

// ---------- build ----------
const host = $('#panels')
for (const [p, name, k] of THEMES) {
  const sec = document.createElement('section')
  Object.assign(sec, { className: 'gal-panel', id: `panel-${p}` })
  sec.dataset.theme = p
  sec.setAttribute('aria-labelledby', `h-${p}`)
  sec.innerHTML = `<h2 id="h-${p}">${name} <small>data-theme="${p}" · Kern in "${k}"</small></h2>` + fill('tpl-panel', p, k)
  $('[data-ask-host]', sec).innerHTML = fill('tpl-ask', p, k)
  host.append(sec)
}
$('#scale-host').innerHTML = fill('tpl-scale', 's', 'light')
hydrate(document)

const SW = {
  'Neutrale rollen': ['bg', 'surface-panel', 'surface', 'surface-raised', 'surface-inset', 'line', 'line-strong', 'text', 'text-muted', 'text-subtle', 'text-disabled', 'focus', 'selection', 'hover', 'hover-strong', 'blade', 'blade-glow', 'well', 'scrim', 'silver', 'rim-a', 'rim-b', 'rim-c', 'button-primary-bg', 'button-primary-bg-hover', 'button-primary-text', 'button-secondary-bg', 'pill-bg'].map(n => `--color-${n}`),
  'Status (S0 D1: tekstkleur, geen tint)': ['danger', 'warning', 'success'].map(n => `--color-${n}`),
  'Materialen': ['kern-bg', 'kern-tx', 'kern-tx2', 'kern-line', 'kern-rim', 'kern-warn-bg', 'kern-warn-line', 'kern-warn-ic', 'glass', 'glass-line', 'launch', 'launch-opaque', 'insp-bg', 'orbit-bg', 'tile-bg', 'tile-fg', 'tile-pc'].map(n => `--material-${n}`),
}
const PLACE = [['--place-device', 'Dit apparaat · groen'], ['--place-nas', 'NAS · blauw'], ['--place-internet', 'Internet · rood'], ['--place-internet-line', 'Internet, vraagrand']]
const ELEV = ['--elev-window', '--elev-window-inactive', '--elev-orbit', '--elev-plate', '--elev-launcher', '--elev-inspector', '--elev-trusted', '--material-tile-rim', '--material-tile-sh']
const swatch = (cs, name, label) => `<figure><span class="gal-chip" style="background: var(${name})"></span><figcaption>${label || name}<span>${label ? name + ' · ' : ''}${cs.getPropertyValue(name).trim()}</span></figcaption></figure>`
for (const sec of $$('#panels > .gal-panel')) {
  const cs = getComputedStyle(sec)
  $('[data-swatches="color"]', sec).innerHTML = Object.entries(SW).map(([h, list]) => `<p class="gal-sw-h">${h}</p>` + list.map(n => swatch(cs, n)).join('')).join('')
  $('[data-swatches="place"]', sec).innerHTML = PLACE.map(([n, l]) => swatch(cs, n, l)).join('')
  $('[data-swatches="elev"]', sec).innerHTML = ELEV.map(n => `<figure style="box-shadow: var(${n})">${n}</figure>`).join('')
}

// icon set and type scale (reference section)
const keys = Object.keys(ICONS)
$('#icon-count').textContent = `${keys.length} sleutels`
$('#icon-grid').append(...keys.map((n, i) => {
  const alias = keys.slice(0, i).find(o => ICONS[o] === ICONS[n])
  const f = document.createElement('figure')
  const c = document.createElement('figcaption')
  c.textContent = alias ? `${n} = ${alias}` : n
  f.append(icon(n, { size: 24 }), c)
  return f
}))
const TYPE = [['display', '09:41'], ['title', 'Tuinplan'], ['reading', 'Moestuin achterin: tomaten langs het hek, bonen tegen de schutting.'], ['body-strong', 'Webvenster vraagt internet'], ['body', 'Je werk staat op dit apparaat en op Thuis-NAS.'], ['body-dense', 'Huurcontract Spoorstraat 12 · 1,2 MB'], ['caption', 'Gewijzigd vandaag 09:12']]
const rcs = getComputedStyle(root)
$('#type-scale').innerHTML = TYPE.map(([r, s]) => {
  const v = (x) => rcs.getPropertyValue(`--type-${r}-${x}`).trim()
  const txt = /^\d\d:\d\d$/.test(s) ? `<span class="ds-num">${s}</span>` : s
  return `<div><span class="lab">--type-${r}-* · ${v('size')} / ${v('line')} · ${v('weight')}</span><span class="ds-t-${r}">${txt}</span></div>`
}).join('')

// ---------- behaviour ----------
const motion = $('#motion-toggle')
motion.setAttribute('aria-checked', root.dataset.motion === 'reduce')

const menuOf = (btn) => document.getElementById(btn.getAttribute('aria-controls'))
function setMenu(btn, open, focus) {
  const menu = menuOf(btn)
  menu.hidden = !open
  btn.setAttribute('aria-expanded', open)
  const items = $$('[role="menuitem"]', menu)
  if (open && focus) (focus === 'last' ? items.at(-1) : items[0]).focus()
}
function selectOption(opt) {
  const list = opt.closest('[role="listbox"]')
  for (const o of $$('[role="option"]', list)) { o.setAttribute('aria-selected', o === opt); o.tabIndex = o === opt ? 0 : -1 }
  opt.focus()
}
function activateTab(tab) {
  for (const t of $$('[role="tab"]', tab.closest('[role="tablist"]'))) {
    const on = t === tab
    t.setAttribute('aria-selected', on); t.tabIndex = on ? 0 : -1
    document.getElementById(t.getAttribute('aria-controls')).hidden = !on
  }
  tab.focus()
}
const step = (list, cur, key) => {
  const i = list.indexOf(cur), n = list.length
  return { ArrowDown: list[(i + 1) % n], ArrowRight: list[(i + 1) % n], ArrowUp: list[(i - 1 + n) % n], ArrowLeft: list[(i - 1 + n) % n], Home: list[0], End: list[n - 1] }[key]
}

// permission result: the panel's toast reports what happened (strings.js trusted.*)
const RESULT = {
  once: () => T.trusted.allowedOnce('Webvenster', 'tuinplanner.example'),
  session: () => `Webvenster mag tuinplanner.example gebruiken tot je de app sluit.`,
  deny: () => T.trusted.denied('Webvenster'),
}
function report(panel, choice) {
  const toast = $('.ds-toast', panel)
  if (!toast) return
  $('.ds-toast-title', toast).textContent = 'Systeem'
  $('.ds-toast-body', toast).textContent = RESULT[choice]()
  toast.hidden = false
  const show = $('[data-toast-show]', panel); if (show) show.hidden = true
}

let opener = null
const askDlg = $('#dlg-ask'), delDlg = $('#dlg-delete')
function openDialog(dlg, from, theme) {
  opener = from
  dlg.dataset.theme = theme
  dlg.showModal()
  // focus starts on the safest choice: Weigeren (deny) or Annuleren
  ;($('[data-choice="deny"]', dlg) || $('[data-safe]', dlg)).focus()
}
for (const dlg of [askDlg, delDlg]) {
  dlg.addEventListener('cancel', () => { dlg.returnValue = dlg === askDlg ? 'deny' : 'cancel' })
  dlg.addEventListener('close', () => {
    if (dlg === askDlg && opener) report(opener.closest('.gal-panel'), dlg.returnValue || 'deny')
    if (opener) opener.focus()
    opener = null
  })
}

document.addEventListener('click', (e) => {
  const t = e.target
  const sw = t.closest('[role="switch"]')
  if (sw && !sw.disabled) {
    const on = sw.getAttribute('aria-checked') !== 'true'
    sw.setAttribute('aria-checked', on)
    if (sw === motion) { if (on) root.dataset.motion = 'reduce'; else delete root.dataset.motion }
    return
  }
  const opt = t.closest('[role="option"]'); if (opt) return selectOption(opt)
  const tab = t.closest('[role="tab"]'); if (tab) return activateTab(tab)
  const trig = t.closest('[aria-haspopup="menu"]:not([role="menuitem"])')
  if (trig) return setMenu(trig, trig.getAttribute('aria-expanded') !== 'true', e.detail === 0 ? 'first' : null)
  const item = t.closest('[role="menuitem"]')
  if (item) {
    if (item.getAttribute('aria-disabled') === 'true' || item.hasAttribute('aria-haspopup')) return
    const btn = document.querySelector(`[aria-controls="${item.closest('[role="menu"]').id}"]`)
    setMenu(btn, false); return btn.focus()
  }
  const choice = t.closest('[data-choice]')
  if (choice) {
    const dlg = choice.closest('dialog')
    if (dlg) return dlg.close(choice.dataset.choice)
    return report(choice.closest('.gal-panel'), choice.dataset.choice)
  }
  if (t.closest('dialog button[value]')) return t.closest('dialog').close(t.closest('button').value)
  const openAsk = t.closest('[data-open-ask]')
  if (openAsk) {
    const p = openAsk.closest('.gal-panel').dataset.theme
    askDlg.innerHTML = fill('tpl-ask', `m-${p}`, KERN[p]); hydrate(askDlg)
    return openDialog(askDlg, openAsk, p)
  }
  const openDel = t.closest('[data-open-delete]')
  if (openDel) return openDialog(delDlg, openDel, openDel.closest('.gal-panel').dataset.theme)
  const close = t.closest('[data-toast-close]')
  if (close) {
    const card = close.closest('.gal-card'); close.closest('.ds-toast').hidden = true
    const show = $('[data-toast-show]', card); show.hidden = false; return show.focus()
  }
  const show = t.closest('[data-toast-show]')
  if (show) { const card = show.closest('.gal-card'); $('.ds-toast', card).hidden = false; show.hidden = true; $('[data-toast-close]', card).focus() }
})

document.addEventListener('keydown', (e) => {
  const t = e.target
  const k = e.key
  const menu = t.closest('[role="menu"]')
  if (menu) {
    const btn = document.querySelector(`[aria-controls="${menu.id}"]`)
    if (k === 'Escape') { e.preventDefault(); setMenu(btn, false); return btn.focus() }
    if (k === 'Tab') return setMenu(btn, false)
    const next = step($$('[role="menuitem"]', menu), t.closest('[role="menuitem"]'), k)
    if (next) { e.preventDefault(); next.focus() }
    return
  }
  if (t.matches('[aria-haspopup="menu"]:not([role="menuitem"])')) {
    if (k === 'ArrowDown' || k === 'ArrowUp') { e.preventDefault(); return setMenu(t, true, k === 'ArrowUp' ? 'last' : 'first') }
    if (k === 'Escape' && t.getAttribute('aria-expanded') === 'true') { e.preventDefault(); return setMenu(t, false) }
  }
  if (t.matches('[role="tab"]')) {
    const next = step($$('[role="tab"]', t.closest('[role="tablist"]')), t, k)
    if (next) { e.preventDefault(); activateTab(next) }
    return
  }
  if (t.matches('[role="option"]')) {
    const next = (k === 'ArrowLeft' || k === 'ArrowRight') ? null : step($$('[role="option"]', t.closest('[role="listbox"]')), t, k)
    if (next) { e.preventDefault(); selectOption(next) }
    if (k === ' ') { e.preventDefault(); selectOption(t) }
    return
  }
  if (k === 'Escape') {
    const a = t.closest('.ds-tip-anchor'); if (a) a.dataset.dismissed = ''
  }
})
const undismiss = (e) => { const a = e.target.closest?.('.ds-tip-anchor'); if (a) delete a.dataset.dismissed }
document.addEventListener('focusout', undismiss)
document.addEventListener('mouseout', undismiss)

// ---------- query: ?only and ?measure ----------
const ONLY = { dark: '#panel-dark', light: '#panel-light', contrast: '#panel-contrast', scale: '#panel-scale', ref: '#panel-ref' }
if (ONLY[q.get('only')]) { root.dataset.only = q.get('only'); $(ONLY[q.get('only')]).dataset.show = '' }

const rgb = (s) => {
  s = s.trim()
  if (s.startsWith('#')) { const h = s.length === 4 ? [...s.slice(1)].map(c => c + c).join('') : s.slice(1, 7); return [0, 2, 4].map(i => parseInt(h.slice(i, i + 2), 16)) }
  const m = s.match(/[\d.]+/g); return m ? m.slice(0, 3).map(Number) : null
}
const lum = (c) => c.map(v => { v /= 255; return v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4 }).reduce((a, v, i) => a + v * [0.2126, 0.7152, 0.0722][i], 0)
const ratio = (a, b) => { const [x, y] = [lum(a), lum(b)].sort((m, n) => n - m); return +((x + 0.05) / (y + 0.05)).toFixed(2) }
function baseOf(el) {
  for (let n = el.parentElement; n; n = n.parentElement) {
    if (n.classList.contains('ds-ask')) return rgb(getComputedStyle(n).getPropertyValue('--material-kern-bg'))
    const c = getComputedStyle(n).backgroundColor
    if (c && !/rgba\(.*,\s*0\)$/.test(c) && c !== 'transparent') return rgb(c)
  }
  return rgb(getComputedStyle(document.body).backgroundColor)
}

async function measure() {
  await document.fonts.ready
  await new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r)))
  const out = { errors: window.__errs }
  out.fonts = [...document.fonts].map(f => `${f.family} ${f.weight} ${f.status}`)
  out.interLoaded = [...document.fonts].some(f => f.family.replace(/"/g, '') === 'Inter' && f.status === 'loaded')
  const used = $$('svg.icon').map(s => [...s.classList].find(c => c.startsWith('icon-')).slice(5))
  out.icons = { keys: keys.length, rendered: used.length, distinctUsed: new Set(used).size, unresolved: [...new Set(used.filter(n => !(n in ICONS)))], placeholdersLeft: $$('i[data-icon]').length }
  out.unfilledPlaceholders = ['#panels', '#scale-host'].some(s => /\{[pk]\}/.test($(s).innerHTML))
  out.scrollWidth = root.scrollWidth
  const small = []
  let count = 0
  for (const el of $$('button, input, select, textarea, a[href], [role="option"], [role="tab"], [role="menuitem"]')) {
    if (el.closest('[inert], template') || el.disabled || !el.getClientRects().length) continue
    const box = (el.matches('input[type="checkbox"], input[type="radio"]') && el.closest('label')) || el
    const r = box.getBoundingClientRect(); count++
    if (r.width < 23.99 || r.height < 23.99) small.push(`${el.tagName}.${el.className} ${r.width.toFixed(1)}x${r.height.toFixed(1)}`)
  }
  out.targets = { checked: count, under24: small }
  // computed type sweep: every visible text element (zoomed strip excluded) must be Inter in a type-1.0 role (size/line/weight)
  const TY = { 13: ['18/400'], 15: ['22/400', '20/400', '22/600'], 16: ['26/400'], 22: ['28/600'], 48: ['56/400'] }
  const off = new Set(); let tn = 0
  for (const el of $$('body *')) {
    if (el.closest('template, [data-scale], #out, svg, dialog:not([open])') || !el.getClientRects().length) continue
    if (!el.matches('input:not([type="checkbox"], [type="radio"])') && ![...el.childNodes].some(c => c.nodeType === 3 && c.textContent.trim())) continue
    const c = getComputedStyle(el), sz = Math.round(parseFloat(c.fontSize)), lh = Math.round(parseFloat(c.lineHeight)), w = +c.fontWeight; tn++
    if (!/^"?Inter/.test(c.fontFamily) || !(TY[sz] || []).includes(`${lh}/${w}`)) off.add(`${el.tagName.toLowerCase()}.${el.className} ${c.fontFamily.split(',')[0]} ${sz}/${c.lineHeight}/${w}`)
  }
  out.type = { textElements: tn, offRole: [...off].slice(0, 20) }
  // focus ring: every interactive kind, in every theme; ring colour against the base it sits on
  const KINDS = ['.ds-btn[data-variant="primary"]', '.ds-btn[data-variant="secondary"]', '.ds-btn[data-variant="danger"]', '.ds-input', '.ds-switch', '.ds-check > input', '.ds-radio > input', '.ds-row', '.ds-tab', '.ds-tabpanel', '.ds-window-ctl', '.ds-taskitem', '.ds-ask .ds-btn[data-choice="deny"]', '[aria-haspopup="menu"]']
  out.focus = {}
  for (const [p] of THEMES) for (const [h, host] of [['', ''], [' compact', ' [data-density="compact"]'], [' neutral', ' [data-surface="neutral"]']]) {
    out.focus[p + h] = [...KINDS, '.gal-canvas'].map(sel => {
      const el = $$(`#panel-${p}${host} ${sel}`).find(x => !x.closest('[inert]') && !x.disabled && x.getClientRects().length)
      if (!el) return `${sel}: none`
      el.focus({ focusVisible: true, preventScroll: true })
      const c = getComputedStyle(el), ring = rgb(c.outlineColor)
      return `${sel}: fv=${el.matches(':focus-visible')} ${c.outlineStyle} ${c.outlineWidth} off ${c.outlineOffset} ring:base ${ratio(ring, baseOf(el))}`
    })
  }
  document.activeElement.blur()
  // S0 D2: rendered sizes in the compact strips; S0 D3: inside a neutral host every colour is achromatic except place colours
  out.compact = Object.fromEntries(THEMES.map(([p]) => [p, ['.ds-btn', '.ds-input', '.ds-switch', '.ds-check', '.ds-radio', '.ds-tab', '.ds-row'].map(sel => {
    const r = $(`#panel-${p} [data-density="compact"] ${sel}`).getBoundingClientRect(); return `${sel} ${r.width.toFixed(0)}x${r.height.toFixed(1)}`
  })]))
  const tinted = new Set(); let nn = 0
  for (const el of $$('[data-surface="neutral"], [data-surface="neutral"] *')) {
    if (el.closest('.ds-place, .ds-node') || !el.getClientRects().length) continue
    const c = getComputedStyle(el); nn++
    for (const prop of ['color', 'backgroundColor', 'borderTopColor', 'outlineColor', 'fill', 'stroke']) {
      const v = rgb(c[prop]); if (v && c[prop] !== 'none' && Math.max(...v) - Math.min(...v) > 2) tinted.add(`${el.tagName.toLowerCase()}.${el.className.baseVal ?? el.className} ${prop} ${c[prop]}`)
    }
  }
  out.neutral = { elements: nn, tinted: [...tinted].slice(0, 20) }
  // keyboard handlers (synthetic keydown reaches these handlers only; native Tab/Space/radio arrows are exercised by kbd.mjs)
  const key = (el, k) => el.dispatchEvent(new KeyboardEvent('keydown', { key: k, bubbles: true, cancelable: true }))
  const tab1 = $('#panel-dark [role="tab"]'); tab1.focus(); key(tab1, 'ArrowRight')
  out.tabsArrow = document.activeElement.id + ' selected=' + document.activeElement.getAttribute('aria-selected') + ' panelShown=' + !$('#dark-tp-2').hidden
  key(document.activeElement, 'Home')
  const row = $('#panel-dark [role="option"][aria-selected="true"]'); row.focus(); key(row, 'ArrowDown')
  out.listArrow = document.activeElement.textContent.slice(0, 30) + ' selected=' + document.activeElement.getAttribute('aria-selected')
  key(document.activeElement, 'Home')
  const mbtn = $('#dark-menu-btn'); mbtn.focus(); key(mbtn, 'ArrowDown')
  const first = document.activeElement.textContent.trim().slice(0, 12); key(document.activeElement, 'ArrowDown')
  const second = document.activeElement.textContent.trim().slice(0, 12); key(document.activeElement, 'Escape')
  out.menu = `ArrowDown->${first} ArrowDown->${second} Escape->hidden=${$('#dark-menu').hidden} focus=${document.activeElement.id}`
  mbtn.click()
  $('#panel-dark [data-open-ask]').click()
  out.askOpen = `open=${askDlg.open} active=${document.activeElement.dataset.choice} theme=${askDlg.dataset.theme} kern=${$('.ds-ask', askDlg).dataset.theme}`
  askDlg.close('deny'); await new Promise(r => setTimeout(r, 50))
  out.askClosed = `open=${askDlg.open} focusBack=${document.activeElement.matches('[data-open-ask]')} toast="${$('#panel-dark .ds-toast-body').textContent}"`
  $('#panel-dark [data-open-delete]').click()
  out.deleteOpen = `open=${delDlg.open} active=${document.activeElement.textContent.trim()}`
  delDlg.close('cancel'); await new Promise(r => setTimeout(r, 50)); document.activeElement.blur()
  // reduced motion
  const bar = $('#panel-dark .ds-progress[data-state="indeterminate"] .ds-progress-bar'), halo = $('#panel-dark .ds-node[data-state="ask"]')
  const anim = () => `bar=${getComputedStyle(bar).animationName}/${getComputedStyle(bar).width} halo=${getComputedStyle(halo, '::before').animationName} dur=${rcs.getPropertyValue('--motion-duration-base').trim()}`
  const was = root.dataset.motion
  delete root.dataset.motion; const a0 = anim()
  root.dataset.motion = 'reduce'; const a1 = anim()
  if (was) root.dataset.motion = was; else delete root.dataset.motion
  out.motion = { normal: a0, reduce: a1, media: matchMedia('(prefers-reduced-motion: reduce)').matches }
  out.scale = (() => { const r = $('#scale-host .ds-btn').getBoundingClientRect(); return `zoom=${getComputedStyle($('#scale-host')).zoom} btnHeight=${r.height}` })()
  out.errorsAfter = window.__errs.length
  $('#out').textContent = JSON.stringify(out, null, 1)
  document.title += ' [measured]'
}
if (q.get('measure')) measure()
