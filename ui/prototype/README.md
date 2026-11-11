# Materiaal · interaction prototype P02

This is a local HTML/CSS/JavaScript prototype with synthetic data. It runs in the browser on the development host, not yet on the own kernel. It performs no real NAS synchronisation, package installation, cryptographic verification or AI inference.

## Starting

From the project root:

```powershell
python -m http.server 8765 --bind 127.0.0.1 --directory ui/prototype
```

Open [Materiaal locally](http://127.0.0.1:8765/). Stop the server with `Ctrl+C`. If the port is occupied, another free loopback port can be chosen. Open the modules via this local server; `file://` is not a supported run path.

The first start consists of three steps. After that the routes `#/materiaal`, `#/object/obj-001`, `#/toegang`, `#/gereedschap`, `#/apparaten` and `#/assistent` are reachable. `#/start` remains available via the prototype panel.

## What works in the simulation

- Search, collections, empty results, a new document and opening existing material.
- Search updates the results without rebuilding the rest of the screen. Draft and profile input is batched to `sessionStorage` at most 180 ms from the first not-yet-saved input, also during sustained typing; later input does not postpone that timer. Loss of focus, a next state transition and leaving the page write the latest state immediately. This is not durable OS storage; a blocked browser may run timers later.
- Requesting, refusing, granting and revoking temporary access. Closing or leaving the document ends the active rights. Refreshing the page preserves fictitious content, but does not resume active rights.
- Editing the garden plan and saving it as a new version. Version restore first shows a comparison and then adds a new version. Existing versions are preserved.
- A local editing draft and cursor position when closing/resuming the same document. The prototype keeps one unfinished editing draft at a time.
- Example installation with valid/invalid signature, removal without removing material and zero rights at installation.
- Explicit warning for Webvenster (Web window) with read access and a fictitious internet destination. No connection is made.
- NAS pairing via a confirmed example identity. After the one-time internet choice the mock automatically switches between home network and internet as soon as the same confirmed NAS is reachable there. There is no second consent request or connect click. Missing reachability/identity evidence, missing consent and a deviating identity pause the connection; local saving keeps working.
- Three storage routes in the same environment: **Mijn NAS** (My NAS), **Ruimte ontvangen** (Receive space) and **Ruimte aanbieden** (Offer space). Internet access is permitted separately per confirmed NAS; there is no network traffic in the browser demo.
- Received space requires identity confirmation and acknowledgement that the administrator can delete their NAS copy. The example invitation is single-use and expires after ten minutes.
- The administrator allocates a fictitious capacity of 100 GiB, sees only guest name/quota/usage/status and can revoke access and wipe their NAS copy. Only the administrator confirms this action; there is no consent request to the guest and no decryption key needed.
- NAS administration requires a separately activated **lokale voorbeeldbeheerrol** (local example administrator role). A data pairing grants no administration rights; activating, inviting and deleting via the internet route are refused. Remote administration is not yet available. A network transition ends the active local administrator role and an open delete confirmation.
- After deletion, old upload responses remain invalid. A new invitation gets a different space identity. Local documents are preserved.
- A fixed example summary of exclusively selected context. Missing model, insufficient memory and cancellation have their own states.

The **Prototypepaneel** (Prototype panel) at the bottom of every work screen activates error scenarios. It is explicitly test control and not part of the intended OS product.

## Operation

- `Tab` / `Shift+Tab`: move through controls. `Enter` activates a focused button/link.
- `Ctrl+K` (or `Cmd+K`): to the visible search field, if the browser passes the key through.
- `Ctrl+S` (or `Cmd+S`): save an active edit. The **Opslaan** (Save) button remains available.
- Arrow up/down: move through search results from the search field or a result.
- `Escape`: cancel an access request or close the restore preview. The local draft is preserved.
- Below 1200 px, **Informatie** (Information) opens the object inspector. Below 768 px, **Menu** opens the main navigation.

Native `dialog` excludes the background from keyboard operation and starts on **Annuleer** (Cancel). After closing, focus returns to the document action; after granting, that action is called **Opslaan** (Save). The narrow view uses the same dialog over the full screen.

## Architecture and boundaries

`fixtures.js` contains exclusively fictitious data. `state.js` manages transitions as a pure reducer; `broker.js` re-checks mock operations for object, action, revocation, lifetime and revision. `app.js` handles the routes and semantic operation. `tokens.css` follows the established product colours and `components.css` the responsive component rules.

`nas-sharing.js` manages the additional NAS product states. The codes, invitations, quotas, uploads and deletions are fictitious. No cryptographic keys are generated or stored. Actual encryption in the separate host prototype under `services/nas/` does not make this browser prototype a security boundary. Pairing, trusted input, identity verification and transport must be integrated into the native client/services.

`observeNasRoute` receives exclusively synthetic route observations: reachability, verified peer identity and a trusted home network are simulated by the prototype panel. An observation grants no pairing, internet consent or administration right. `networkChanged` remains the separate scenario without confirmed route evidence; only the new positive observation can make the known NAS automatically reachable again. On the internet or with a paused connection the local administrator role lapses, even if the data later resumes automatically at home.

On refresh, new internet/received-NAS/administration confirmations lapse in this browser trial. This is a temporary mock limitation, not a product goal: durable secure key management and use on multiple devices must later be integrated into the native services. Only bounded administration history is retained; deleted guest records stay deleted. Browser storage can be modified by the browser user and is never evidence of OS authorisation.

The mock broker is not a real security boundary. In a future native port the kernel/services must enforce the capabilities. The trusted visual dialog can be imitated in a browser; that later requires a compositor/input boundary. Installation is simulated and says nothing about compatibility or cryptographic authenticity.

Only `sessionStorage['nieuw-os-prototype-v1']` is used. **Demo opnieuw beginnen** (Restart demo) clears only that key. On an invalid stored structure a clean fixture state is loaded. Tool definitions are rebuilt from the fixed fixtures on resume; stored data may not change tool rights or markup fields.

The app has no network API, external assets, fonts, framework or dependencies. The CSP blocks connections, forms, plugins, inline script and external resources. The app's own initial HTML/CSS/JS requests to the local server remain necessary. This restriction applies to the prototype code and proves nothing about browser extensions or the later OS network layer.

## Executed local checks

On 27 October 2026 with Node **v24.16.0**, from the project root:

```powershell
node --test ui/prototype/tests/state.test.mjs
node --check ui/prototype/app.js
```

Result: **22 tests passed, 0 failed**, both commands exit status 0. The tests cover, among other things, scope/revocation, expired rights, modified stored data, installation without rights, NAS error cases with local storage, missing AI capacity, context selection, cancellation with stale callbacks, preservation of version history and stale-revision rejection. The network check is a source and CSP check; actual browser network observation is a separate acceptance check.

The base tokens actually used have been checked with the sRGB luminance formula for text contrast (at least 4.5:1) and control border contrast (at least 3:1). This covers fixed colour pairs and is not a complete accessibility statement.

The main agent handles browser acceptance and screenshots at 1440×900, 1024×768 and 390×844, plus reflow at 320 CSS px. The executed browser checks and screenshots are recorded in the integration report. Screen reader use, a real user trial and native OS accessibility have not been demonstrated by this.

The targeted P02V follow-up check on 27 October 2026 restores skip-link navigation, focus retention on Menu/Informatie, search by document type and the prescribed 288px inspector. The current Node suite has **23 passing tests**. New evidence, source hashes, screenshots and remaining accessibility dependencies are in [verification/P02V-2026-10-27.md](verification/P02V-2026-10-27.md). The earlier reports are retained as historical executions.

## Visual alignment with the chosen specification

The existing text specification `docs/implementation/PRODUCT-SPEC.md` is the design source; no new concept selection or raster concept has been added. The implementation uses the prescribed mineral background, green primary controls, system typography, material list, 224/72 px navigation rail and full narrow dialog. The main task starts with documents; tools are subordinate.

During browser review a discovered overlap between the search icon and input text was fixed by explicitly bounding the input padding. Further visual findings belong to the integration check. The local SVG floor plan is a fictitious document from the specification, not an external illustration or hardware evidence.

## Recovery and next step

Stopping the server changes no system files. **Demo opnieuw beginnen** (Restart demo) restores the fictitious initial state and removes the fictitious edits of this demo. The ZIP source and files outside `ui/prototype/` are not modified by the prototype.

Next product step: controlled usability and screen reader trials, then replacement of the mock interfaces by actually enforced kernel/service contracts. This prototype gives no conformance or security guarantee.

## Executed browser check

On27 October2026 the core tasks, modal focus loop/cancellation and four viewport widths were actually checked in the browser. Network logging saw seven initial loopback assets and then zero additional requests during the checked flows; console without warnings/errors. See ../../docs/verification/browser-report.json and screenshots/. The final check used127.0.0.1:8776 after empty responses on8765. Real200% browser zoom, screen reader and human user trial are still open; reflow at320 CSS px has been demonstrated separately.

## NAS follow-up check — 27 October 2026

The browser evidence above is historical and does not cover the new NAS screens. On Node v24.22.1 the combined current state suite was run:

```sh
node --test --test-isolation=none ui/prototype/tests/*.test.mjs
```

Result: **39 tests passed, 0 failed**, exit 0: the 24 existing tests plus 15 NAS tests for identity, single-use/expired invitations, peer-bound internet choice, automatic route switching with existing consent, blocking without reachability/identity evidence, quota, separate local administration without a remote admin route, administrator deletion without guest consent, old upload responses, administration data without document content, revocation and resumption. `node --check` passes for app/state/broker/nas-sharing/fixtures. This is host/mock evidence; no network, encryption or native OS evidence.

An attempt to check the new screens via the Browser plugin was blocked because the required original browser context is missing. Render checking, modal interaction and small viewports of this change are therefore still open; earlier screenshots are not new evidence.

The intended user route: Apparaten (Devices) → Mijn NAS (My NAS) → confirm identity → optionally **Gebruik mijn NAS ook buitenshuis** (Also use my NAS away from home). After that the prototype panel demonstrates, with **Verbindingswissel** (Connection switch), the automatic transition home/on the road, without new consent. For received space: Apparaten → Ruimte ontvangen → example invitation → confirm identity and deletion risk → optionally permit internet. For offering: first pair own NAS → Ruimte aanbieden → activate offer → quota/name → invitation. **Intrekken en wissen** (Revoke and wipe) opens an administrator dialog with Annuleer (Cancel) as first focus. The prototype panel can then demonstrate old connections, revocation and fictitious space usage.
