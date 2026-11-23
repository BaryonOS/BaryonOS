// Station 1, Aanmelden: lock screen, profile choice, session-service surface (Kern, opposite
// theme), Herstelcode route, session start, Eerste start and the system screen.
// Secret rule: password and recovery inputs are never read, stored, logged or emitted;
// submit only clears them. The outcome comes from the Simulatie panel.
import { T, DATA, el, icon, motionMs, bus } from '../contract.js'
import { wakes } from './keys.js'

export function initSession(ctx) {
  const J = ctx.J
  const S = J.session
  const host = document.getElementById('jr-session')
  const desk = () => document.getElementById('jr-desk')
  let attempts = 5
  let user = DATA.user.id
  let held = null

  const mark = (cls = '') => el('span', { class: `jr-mark ${cls}`, 'aria-hidden': 'true' },
    el('img', { src: '../brand/mark-compact.svg', alt: '', class: 'jr-mark-colour' }),
    el('img', { src: '../brand/mark-mono-ink.svg', alt: '', class: 'jr-mark-mono' }))
  const btn = (label, onclick, variant = 'secondary', extra = {}) => el('button', { class: 'ds-btn', 'data-variant': variant, type: 'button', on: { click: onclick }, ...extra }, label)
  // Kern material, opposite theme (MF1); the band names the trusted owner of the surface.
  const kern = (labelId, ...body) => el('section', { class: 'jr-kern', 'data-kern': '', 'data-theme': ctx.kern(), 'aria-labelledby': labelId, 'data-sim-scope': '' },
    el('div', { class: 'jr-kern-band' }, icon('lock', { size: 18 }), el('span', {}, S.band), ctx.feas('feasible')), ...body)
  const field = (id, label, extra) => el('div', { class: 'ds-field' },
    el('label', { class: 'ds-field-label', for: id }, label),
    el('input', { class: 'ds-input', id, autocomplete: 'off', spellcheck: 'false', ...extra }))

  let first = true // no focus move on page load: the band and Rondleiding stay in view
  function mount(phase, node, focus) {
    ctx.phase = phase
    host.hidden = false
    host.replaceChildren(node)
    const d = desk(); d.inert = true; d.hidden = true // desktop state is kept, only covered
    if (first) { first = false; return }
    ;(focus ?? node.querySelector('[data-initial-focus]') ?? node.querySelector('button, input'))?.focus()
  }

  function lockScreen() {
    const pw = el('div', { class: 'jr-lock-tools' },
      el('div', { class: 'ds-menu-anchor' },
        btn(T.lock.accessibility, (e) => pop(e.currentTarget), 'quiet', { 'aria-expanded': 'false', 'aria-controls': 'jr-lock-a11y' }),
        el('div', { class: 'ds-menu jr-lock-pop', id: 'jr-lock-a11y', role: 'group', 'aria-label': T.lock.accessibility, hidden: true },
          el('fieldset', { class: 'ds-fieldset' }, el('legend', {}, J.a11y.axes.theme[0]), el('div', { class: 'ds-seg' },
            J.a11y.axes.theme[1].map(([v, l]) => el('label', { class: 'ds-radio' }, el('input', { type: 'radio', name: 'jr-lock-theme', value: v, checked: (ctx.axes.theme ?? '') === v, on: { change: () => ctx.setAxis('theme', v) } }), l)))),
          el('fieldset', { class: 'ds-fieldset' }, el('legend', {}, J.a11y.axes.motion[0]), el('div', { class: 'ds-seg' },
            J.a11y.axes.motion[1].map(([v, l]) => el('label', { class: 'ds-radio' }, el('input', { type: 'radio', name: 'jr-lock-motion', value: v, checked: (ctx.axes.motion ?? '') === v, on: { change: () => ctx.setAxis('motion', v) } }), l)))))),
      el('div', { class: 'ds-menu-anchor' },
        btn(T.lock.power, (e) => pop(e.currentTarget), 'quiet', { 'aria-expanded': 'false', 'aria-controls': 'jr-lock-power' }),
        el('div', { class: 'ds-menu jr-lock-pop', id: 'jr-lock-power', role: 'group', 'aria-label': S.power, hidden: true },
          btn(S.restart, () => starting(false, true)), btn(S.shutdown, () => ctx.live(S.shutdownNote)),
          el('div', { class: 'jr-later', 'aria-disabled': 'true', tabindex: '0' }, T.menu.restartRecovery, el('span', { class: 'ds-menu-why' }, J.tour.laterWhy)))))
    return el('section', { class: 'jr-lock', 'aria-label': J.stations.aanmelden.title },
      el('button', { class: 'jr-wake', type: 'button', 'data-initial-focus': '', on: { click: chooser } },
        mark(), el('span', { class: 'jr-clock' }, DATA.now.clock), el('span', { class: 'jr-date' }, DATA.now.date), el('span', { class: 'jr-hint' }, T.lock.hint)),
      pw)
  }
  function pop(b) {
    const p = document.getElementById(b.getAttribute('aria-controls'))
    const open = p.hidden
    for (const q of host.querySelectorAll('.jr-lock-pop')) q.hidden = true
    for (const q of host.querySelectorAll('[aria-controls^="jr-lock-"]')) q.setAttribute('aria-expanded', 'false')
    p.hidden = !open; b.setAttribute('aria-expanded', String(open))
    if (open) p.querySelector('input:checked, button, [tabindex]')?.focus()
  }
  host.addEventListener('keydown', (e) => {
    const p = e.target.closest?.('.jr-lock-pop')
    if (e.key === 'Escape' && p) { e.stopPropagation(); p.hidden = true; const b = host.querySelector(`[aria-controls="${p.id}"]`); b.setAttribute('aria-expanded', 'false'); b.focus() }
  })

  function show() { attempts = 5; mount('locked', lockScreen()) }

  // Any key (not Tab, modifiers, F1, F6, repeats) wakes the sign-in surface; the key is consumed.
  addEventListener('keydown', (e) => {
    if (ctx.phase !== 'locked' || host.hidden || e.target.closest?.('.jr-band, .jr-panel, .jr-lock-tools')) return
    if (wakes(e) && e.key !== 'Escape') { e.preventDefault(); chooser() }
  })

  // the note follows the Simulatie panel, so the page never says "every input works" beside a refusal
  // and never beside a profile that is not set up: Sem and Andere gebruiker get their own explanation (repair 2)
  const simNote = () => (user === 'other' ? J.r2.otherLater : user !== DATA.user.id ? J.r2.onlyNoor : ctx.sim.signin === 'incorrect' ? J.more.simIncorrect : T.lock.simulatedNote)
  bus.on('jr:sim', (g) => { const n = document.getElementById('jr-signin-sim'); if (g === 'signin' && n) n.textContent = simNote() })

  function chooser() {
    const users = [...DATA.users, { id: 'other', name: T.lock.other, initials: '+' }]
    const name = () => DATA.users.find((u) => u.id === user)?.name ?? T.lock.other
    const secret = el('input', { class: 'ds-input', id: 'jr-secret', type: 'password', autocomplete: 'off', spellcheck: 'false', 'aria-describedby': 'jr-secret-note jr-secret-err', 'data-initial-focus': '' })
    const label = el('label', { class: 'ds-field-label', for: 'jr-secret' }, T.lock.pinLabel(name()))
    const err = el('p', { class: 'ds-field-error', id: 'jr-secret-err', hidden: true })
    const submit = (e) => {
      e?.preventDefault()
      secret.value = ''
      if (user !== DATA.user.id) { err.hidden = false; err.replaceChildren(icon('warn', { size: 16 }), user === 'other' ? J.r2.otherLater : J.r2.semRefused(name())); if (user !== 'other') secret.focus(); return }
      if (ctx.sim.signin === 'incorrect') {
        attempts = Math.max(1, attempts - 1)
        secret.setAttribute('aria-invalid', 'true')
        err.hidden = false
        err.replaceChildren(el('span', { class: 'ds-status-marker', 'aria-hidden': 'true' }), icon('warn', { size: 16 }), `${T.lock.wrong} ${S.attempts(attempts)}`)
        secret.focus()
        return
      }
      starting()
    }
    held = submit
    // Andere gebruiker has no account-name step in this slice: the field is hidden and the note says why (no PIN for an unknown account)
    const field = el('div', { class: 'ds-field', hidden: user === 'other' }, label, secret, el('p', { class: 'ds-field-hint', id: 'jr-secret-note' }, T.lock.secretNote))
    const form = el('form', { class: 'jr-kern-body', on: { submit } },
      el('h2', { id: 'jr-signin-h', class: 'jr-kern-title' }, T.lock.signIn),
      field, err,
      el('p', { class: 'jr-note', 'data-sim-fn': '' }, el('span', { id: 'jr-signin-sim' }, simNote()), ' ', ctx.feas('simulated')),
      el('div', { class: 'jr-row' },
        el('button', { class: 'ds-btn', 'data-variant': 'primary', 'data-size': 'large', type: 'submit' }, T.lock.signIn),
        btn(S.forgot, recovery, 'quiet')))
    const list = el('fieldset', { class: 'jr-users' }, el('legend', {}, T.lock.choose),
      users.map((u) => el('label', { class: 'jr-user' },
        el('input', { type: 'radio', name: 'jr-user', checked: u.id === user, on: { change: () => { user = u.id; label.textContent = T.lock.pinLabel(name()); field.hidden = user === 'other'; const n = document.getElementById('jr-signin-sim'); if (n) n.textContent = simNote(); err.hidden = true; secret.removeAttribute('aria-invalid') } } }),
        el('span', { class: 'jr-avatar', 'aria-hidden': 'true' }, u.initials),
        el('span', {}, el('span', { class: 'jr-user-name' }, u.name), u.id === DATA.user.id ? el('span', { class: 'jr-user-sub' }, S.lastUser) : null))))
    mount('chooser', el('div', { class: 'jr-signin' }, list, kern('jr-signin-h', form),
      el('div', { class: 'jr-row jr-signin-back' }, btn(S.back, show, 'quiet'))))
  }

  function recovery() {
    const code = el('input', { class: 'ds-input', id: 'jr-code', autocomplete: 'off', spellcheck: 'false', inputmode: 'text', 'aria-describedby': 'jr-code-h', 'data-initial-focus': '' })
    const next = (e) => { e.preventDefault(); code.value = ''; newSecretScreen() }
    mount('recovery', el('div', { class: 'jr-signin jr-single' }, kern('jr-rec-h', el('form', { class: 'jr-kern-body', on: { submit: next } },
      el('h2', { id: 'jr-rec-h', class: 'jr-kern-title' }, S.recoveryTitle),
      el('p', {}, S.recoveryText),
      el('div', { class: 'ds-field' }, el('label', { class: 'ds-field-label', for: 'jr-code' }, S.recoveryLabel), code, el('p', { class: 'ds-field-hint', id: 'jr-code-h' }, S.recoveryHint)),
      el('div', { class: 'jr-row' }, el('button', { class: 'ds-btn', 'data-variant': 'primary', type: 'submit' }, S.continue), btn(S.back, chooser))))))
  }

  function newSecretScreen() {
    const newSecret = el('input', { class: 'ds-input', id: 'jr-new', type: 'password', autocomplete: 'off', spellcheck: 'false', 'data-initial-focus': '' })
    const done = (e) => { e.preventDefault(); newSecret.value = ''; starting() }
    mount('newsecret', el('div', { class: 'jr-signin jr-single' }, kern('jr-new-h', el('form', { class: 'jr-kern-body', on: { submit: done } },
      el('h2', { id: 'jr-new-h', class: 'jr-kern-title' }, S.newSecretTitle),
      el('div', { class: 'ds-field' }, el('label', { class: 'ds-field-label', for: 'jr-new' }, S.newSecretLabel), newSecret),
      el('p', { class: 'jr-note', 'data-sim-fn': '' }, T.lock.simulatedNote, ' ', ctx.feas('simulated')),
      el('div', { class: 'jr-row' }, el('button', { class: 'ds-btn', 'data-variant': 'primary', type: 'submit' }, S.newSecretDone))))))
  }

  // Session start: the Triade orbits converge once (boot token); reduced motion shows the resting mark.
  function starting(hold = false, toLock = false) {
    const node = el('section', { class: 'jr-starting', 'aria-labelledby': 'jr-start-h', 'aria-busy': 'true' },
      mark('jr-converge'),
      el('h2', { id: 'jr-start-h', class: 'jr-start-title', tabindex: '-1' }, toLock ? S.restart : T.lock.signingIn),
      el('div', { class: 'ds-progress', 'data-state': 'indeterminate', role: 'progressbar', 'aria-labelledby': 'jr-start-h' }, el('span', { class: 'ds-progress-bar' })),
      el('p', { class: 'jr-note' }, S.starting))
    mount('starting', node, node.querySelector('h2'))
    ctx.live(T.lock.signingIn)
    if (!hold) setTimeout(() => (toLock ? show() : ctx.signedInDone()), Math.max(motionMs(1400), 600))
  }

  function oobe(step) {
    const names = S.oobeSteps
    let ok = step !== 3
    const nextBtn = el('button', { class: 'ds-btn', 'data-variant': 'primary', type: 'button', disabled: !ok, on: { click: () => (step === 5 ? starting() : oobe(step + 1)) } }, step === 5 ? S.oobeFinish : S.continue)
    const back = () => (step === 1 ? cancelOobe() : oobe(step - 1))
    const oobeSecret = el('input', { class: 'ds-input', id: 'jr-oobe-s', type: 'password', autocomplete: 'off', spellcheck: 'false' })
    const body = {
      1: [el('p', {}, S.oobeLang)],
      2: [field('jr-oobe-n', S.oobeName, { value: DATA.user.full }), el('div', { class: 'ds-field' }, el('label', { class: 'ds-field-label', for: 'jr-oobe-s' }, S.oobeSecret), oobeSecret, el('p', { class: 'ds-field-hint' }, T.lock.secretNote))],
      3: [el('p', {}, S.oobeCode), el('p', { class: 'jr-code' }, S.oobeCodeSample),
        el('label', { class: 'ds-check' }, el('input', { type: 'checkbox', on: { change: (e) => { nextBtn.disabled = !e.target.checked } } }), S.oobeKept)],
      4: [el('fieldset', { class: 'ds-fieldset' }, el('legend', {}, names[3]), el('label', { class: 'ds-radio' }, el('input', { type: 'radio', name: 'jr-oobe-st', checked: true }), S.oobeStorageLocal)), el('p', {}, S.oobeStorageLater)],
      5: [el('p', {}, S.oobePrivacy1), el('p', {}, S.oobePrivacy2)],
    }[step]
    const node = el('div', { class: 'jr-signin jr-single jr-oobe' }, kern('jr-oobe-h', el('div', { class: 'jr-kern-body', on: { keydown: (e) => { if (e.key === 'Escape') { e.stopPropagation(); back() } } } },
      el('p', { class: 'jr-step' }, S.step(step), ' ', ctx.feas('simulated')),
      el('ol', { class: 'jr-steps' }, names.map((n, i) => el('li', { 'aria-current': i + 1 === step ? 'step' : null }, n))),
      el('h2', { id: 'jr-oobe-h', class: 'jr-kern-title', tabindex: '-1' }, `${S.oobeTitle}: ${names[step - 1]}`),
      ...body,
      el('div', { class: 'jr-row' }, nextBtn, btn(S.back, back)))))
    node.dataset.simFn = ''
    mount('oobe', node, node.querySelector('h2'))
  }
  function cancelOobe() { show(); ctx.live(S.oobeCancelled) }

  // System screen (Ctrl+Alt+End): full Kern surface; Escape = Annuleren.
  function system() {
    const back = () => { hide(); ctx.phase = 'desktop'; ctx.shell.focusFallback() }
    const signOut = async () => {
      if (ctx.shell.dirty()) { const v = await ctx.confirm(S.signOutAsk); if (v === 'cancel') return; if (v === 'save') ctx.shell.saveAll(); else ctx.shell.discardAll() }
      ctx.signOut()
    }
    const node = el('div', { class: 'jr-system', on: { keydown: (e) => { if (e.key === 'Escape') { e.stopPropagation(); back() } } } }, kern('jr-sys-h', el('div', { class: 'jr-kern-body' },
      el('h2', { id: 'jr-sys-h', class: 'jr-kern-title' }, S.systemTitle),
      el('div', { class: 'jr-sys-list' },
        btn(S.lock, () => ctx.go('aanmelden'), 'secondary', { 'data-initial-focus': '' }),
        btn(S.switchUser, () => { ctx.phase = 'locked'; chooser() }),
        btn(S.signOut, signOut),
        el('div', { class: 'jr-later', 'aria-disabled': 'true', tabindex: '0', 'data-sim-fn': '' }, S.taskManager, ' ', ctx.feas('feasible')),
        btn(S.cancel, back)))))
    mount('system', node)
  }

  function lock() { show(); ctx.live(S.locked) }
  function hide() { host.hidden = true; host.replaceChildren(); const d = desk(); d.inert = false; d.hidden = false }

  return { show, lock, hide, chooser, recovery, starting, oobe, system, submitForTest: () => held?.() }
}
