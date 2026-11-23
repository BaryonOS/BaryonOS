// The page's file-open broker (E11) on the page's permission instance; see fileaccess.js. Prototype only.
// `fileAccess` (with its shell facet) is for the shell and the bootstrap only; app code imports `appAccess`, which can read
// what the user handed over and write after a trusted edit, but can never open, escalate, revoke or transfer anything.
import { permissions, T } from '../contract.js'
import { APP } from '../ask-view.js'
import { createFileAccess, realInput } from './fileaccess.js'
// ?test=1 only: the documented token the sweep hook uses to open a document without a gesture; null on the normal page
export const TEST_OPEN = new URLSearchParams(globalThis.location?.search ?? '').has('test') ? Symbol('test-open') : null
export const isUserGesture = (e) => realInput(e) || (TEST_OPEN !== null && e === TEST_OPEN)
export const fileAccess = createFileAccess(permissions, { isUserGesture })
export const appAccess = (app) => fileAccess.forApp(app)
// B repair 3 (lead decision 4): one app name for every journey surface, from the same APP map the prompt uses, so the
// window title, Orbit, switcher, Menu, Zoeken, Appmachtigingen and the prompt provenance cannot drift apart. Journey only:
// the app id 'browser' is presented as the third-party app Webvenster; the gallery's APP map and output are unchanged.
export const appName = (app) => APP[app] ?? T.apps[app] ?? app
