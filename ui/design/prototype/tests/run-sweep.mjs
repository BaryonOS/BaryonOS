// Headless runner for the journey sweep (E10). Not a node --test file (the name does not end in .test.mjs), so the test suite
// never starts a browser. Usage, with the :8791 server serving ui/design:
//   node ui/design/prototype/tests/run-sweep.mjs [OUT.json] [query, e.g. only=320]
// Env: CHROME (default /opt/google/chrome/chrome), SWEEP_URL (default http://127.0.0.1:8791/prototype/sweep.html).
// Mechanism: Chrome --headless=new over the DevTools protocol (Node >= 22 WebSocket), with Emulation.setFocusEmulationEnabled,
// because the sweep stops with NO_FOCUS when the document has no focus. Exit 0 only when the sweep reports zero failures.
import { spawn } from 'node:child_process'
import { mkdtempSync, writeFileSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'

const OUT = process.argv[2] ?? 'sweep-out.json'
const Q = process.argv[3] ?? ''
const URL_ = process.env.SWEEP_URL ?? 'http://127.0.0.1:8791/prototype/sweep.html'
const port = 9400 + Math.floor(Math.random() * 400)
const sleep = (ms) => new Promise((r) => setTimeout(r, ms))
const prof = mkdtempSync(join(tmpdir(), 'jr-sweep-'))
const ch = spawn(process.env.CHROME ?? '/opt/google/chrome/chrome', ['--headless=new', `--remote-debugging-port=${port}`, `--user-data-dir=${prof}`, '--hide-scrollbars', '--no-first-run', '--no-default-browser-check', 'about:blank'], { stdio: 'ignore' })
let code = 2
try {
  let ok = false
  for (let i = 0; i < 60 && !ok; i++) { await sleep(250); try { ok = (await fetch(`http://127.0.0.1:${port}/json/version`)).ok } catch {} }
  if (!ok) throw new Error('chrome did not start')
  const t = await (await fetch(`http://127.0.0.1:${port}/json/new?about:blank`, { method: 'PUT' })).json()
  const ws = new WebSocket(t.webSocketDebuggerUrl)
  await new Promise((res, rej) => { ws.onopen = res; ws.onerror = rej })
  let id = 0
  const pend = new Map()
  ws.onmessage = (m) => { const d = JSON.parse(m.data); if (d.id && pend.has(d.id)) { const [res, rej] = pend.get(d.id); pend.delete(d.id); d.error ? rej(new Error(JSON.stringify(d.error))) : res(d.result) } }
  const send = (method, params = {}) => new Promise((res, rej) => { const i = ++id; pend.set(i, [res, rej]); ws.send(JSON.stringify({ id: i, method, params })) })
  const evalJs = async (expr) => (await send('Runtime.evaluate', { expression: expr, returnByValue: true })).result.value
  await send('Page.enable'); await send('Runtime.enable')
  await send('Emulation.setFocusEmulationEnabled', { enabled: true })
  await send('Emulation.setDeviceMetricsOverride', { width: 1440, height: 1000, deviceScaleFactor: 1, mobile: false })
  await send('Page.navigate', { url: `${URL_}?${Q}&t=${Date.now()}` })
  let st = 'running'
  for (let i = 0; i < 900 && /^running|^$/.test(st ?? ''); i++) { await sleep(1000); st = await evalJs("document.getElementById('sw-state')?.textContent ?? ''") }
  const out = await evalJs("document.getElementById('out').textContent")
  writeFileSync(OUT, out)
  console.log(`state: ${st}`)
  const r = JSON.parse(out)
  console.log(JSON.stringify({ cells: r.cells, failures: r.failures?.length, summary: r.summary, warnings: r.warnings, warningsByKind: r.warningsByKind }))
  code = r.failures && r.failures.length === 0 ? 0 : 1
  ws.close()
} catch (e) {
  console.error(String(e && e.stack || e))
} finally {
  ch.kill('SIGKILL')
  await sleep(300)
  try { rmSync(prof, { recursive: true, force: true }) } catch {}
}
process.exit(code)
