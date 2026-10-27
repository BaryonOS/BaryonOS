import { createFixtureState, STORAGE_KEY, COLLECTIONS, NAS_IDENTITY } from './fixtures.js';
import { reducer, selectedObject, currentVersion, activeGrants, resumeState } from './state.js';
import { listObjects } from './broker.js';

const app = document.querySelector('#app');
const announcements = document.querySelector('#announcements');
let state;
try { state = resumeState(sessionStorage.getItem(STORAGE_KEY)); } catch { state = createFixtureState(); }
let query = '';
let collection = 'Alles';
let menuOpen = false;
let inspectorOpen = false;
let focusAfterRoute = null;
let saveTimer;
let installTimer;
let summaryTimer;
let storageUnavailable = false;
let previousRoute = '';

const escape = (value) => String(value ?? '').replace(/[&<>"']/g, (char) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[char]);
const paths = {
  material: '<path d="M4 5h6l2 2h8v13H4z"/><path d="M4 11h16"/>',
  tools: '<path d="m14 5 5 5M5 14l5 5M4 20l4-1L20 7a3 3 0 0 0-4-4L4 15z"/>',
  devices: '<rect x="3" y="4" width="18" height="12" rx="2"/><path d="M8 20h8M12 16v4"/>',
  assistant: '<path d="M12 3v3m0 12v3M3 12h3m12 0h3"/><path d="m12 6 2 4 4 2-4 2-2 4-2-4-4-2 4-2z"/>',
  shield: '<path d="M12 3 4 6v6c0 4 4 7 8 9 4-2 8-5 8-9V6z"/><path d="m8 12 3 3 5-6"/>',
  search: '<circle cx="10" cy="10" r="6"/><path d="m15 15 5 5"/>',
  arrow: '<path d="M5 12h14m-6-6 6 6-6 6"/>',
  back: '<path d="M19 12H5m6-6-6 6 6 6"/>',
  plus: '<path d="M12 5v14M5 12h14"/>',
  document: '<path d="M6 3h8l4 4v14H6zM14 3v5h4M9 12h6M9 16h6"/>',
  image: '<rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="8" cy="8" r="1.5"/><path d="m3 17 5-5 4 4 4-6 5 7"/>',
  check: '<path d="m5 12 4 4L19 6"/>',
  close: '<path d="m6 6 12 12M6 18 18 6"/>',
  warning: '<path d="m12 3 10 18H2zM12 9v4m0 4v.1"/>',
  nas: '<rect x="4" y="3" width="16" height="18" rx="2"/><path d="M4 9h16M4 15h16M8 6h.1M8 12h.1M8 18h.1M12 6h5M12 12h5M12 18h5"/>',
  reset: '<path d="M4 10a8 8 0 1 1 1 8M4 4v6h6"/>',
  globe: '<circle cx="12" cy="12" r="9"/><ellipse cx="12" cy="12" rx="4" ry="9"/><path d="M3 12h18"/>',
  menu: '<path d="M4 6h16M4 12h16M4 18h16"/>',
  clock: '<circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/>',
  spinner: '<path d="M12 3a9 9 0 1 1-9 9"/>',
  info: '<circle cx="12" cy="12" r="9"/><path d="M12 11v6m0-10v.1"/>',
};
const icon = (name, className = '') => `<svg class="icon ${className}" viewBox="0 0 24 24" aria-hidden="true">${paths[name] || paths.document}</svg>`;
const button = (text, action, className = '', attrs = '', iconName = '') => `<button type="button" class="${className}" data-action="${action}" ${attrs}>${iconName ? icon(iconName) : ''}${text}</button>`;
const status = (text, name = 'check', className = '') => `<span class="status ${className}">${icon(name)}${escape(text)}</span>`;
const notice = (text, kind = 'info') => `<div class="notice ${kind}">${icon(kind === 'error' || kind === 'warning' ? 'warning' : 'info')}<p>${escape(text)}</p></div>`;

function currentRoute() {
  const route = location.hash.replace(/^#/, '') || (state.onboarding.completed ? '/materiaal' : '/start');
  if (/^\/object\/[a-z0-9-]+$/.test(route) || ['/start', '/materiaal', '/toegang', '/gereedschap', '/apparaten', '/assistent'].includes(route)) return route;
  return '/materiaal';
}

function persist() {
  try { sessionStorage.setItem(STORAGE_KEY, JSON.stringify(state)); } catch { storageUnavailable = true; }
}

function dispatch(event, { redraw = true, focus = null } = {}) {
  state = reducer(state, event);
  persist();
  if (state.message) announcements.textContent = state.message.text;
  if (redraw) render(focus);
}

function navigate(route, focus = 'page-title') {
  menuOpen = false;
  focusAfterRoute = focus;
  if (location.hash === `#${route}`) routeChanged();
  else location.hash = route;
}

function routeChanged() {
  let route = currentRoute();
  if (route.startsWith('/object/')) {
    dispatch({ type: 'openObject', id: route.slice('/object/'.length) }, { redraw: false });
  } else if (route !== '/toegang' && state.selectedObjectId) {
    dispatch({ type: 'closeObject' }, { redraw: false });
  }
  if (previousRoute === '/toegang' && state.grantRequest && route !== '/toegang') {
    dispatch({ type: 'denyGrant' }, { redraw: false });
  }
  const focus = focusAfterRoute || (previousRoute ? 'page-title' : null);
  previousRoute = route;
  focusAfterRoute = null;
  render(focus);
}

function shell(content, route) {
  const nav = [
    ['/materiaal', 'Mijn materiaal', 'material'], ['/gereedschap', 'Gereedschap', 'tools'],
    ['/apparaten', 'Apparaten', 'devices'], ['/assistent', 'Assistent', 'assistant'],
  ];
  const current = route.startsWith('/object/') || route === '/toegang' ? '/materiaal' : route;
  return `<div class="shell ${route === '/start' ? 'onboarding-shell' : ''}">
    <aside class="rail ${menuOpen ? 'menu-open' : ''}" aria-label="Werkruimte">
      <a class="brand" href="#/materiaal" aria-label="Materiaal, naar Mijn materiaal"><svg class="brand-mark" viewBox="0 0 32 32" aria-hidden="true"><path fill="currentColor" d="M3 6h7v20H3zm10 0h7v20h-7zm10 0h6v20h-6z"/><path fill="#EDF0E9" d="m3 16 7-5v6l-7 5zm10-5 7 5v6l-7-5z"/></svg><span>Materiaal</span></a>
      ${button('Menu', 'toggleMenu', 'mobile-menu quiet', `aria-expanded="${menuOpen}" aria-controls="main-nav"`, 'menu')}
      <nav class="nav" id="main-nav" aria-label="Hoofdnavigatie">${nav.map(([path, text, symbol]) => `<a href="#${path}" title="${text}" ${current === path ? 'aria-current="page"' : ''}>${icon(symbol)}<span class="nav-label">${text}</span></a>`).join('')}</nav>
      <div class="rail-foot"><span class="avatar" aria-hidden="true">${escape(state.onboarding.name[0] || 'L')}</span><div class="profile-copy"><p class="meta">${escape(state.onboarding.name)}</p><p class="meta">Op dit apparaat</p></div></div>
    </aside>
    <div class="workspace"><header class="topbar"><p class="simulation-label">Interactiemodel · simulatie</p><div class="top-actions">${button('<span class="reset-label">Demo opnieuw beginnen</span>', 'resetDemo', 'quiet', 'aria-label="Demo opnieuw beginnen" title="Demo opnieuw beginnen"', 'reset')}</div></header>
    <main id="main" tabindex="-1">${state.message ? notice(state.message.text, state.message.kind) : ''}${storageUnavailable ? notice('De browser bewaart deze demo niet tussen paginaweergaven. Deze sessie blijft bruikbaar.', 'warning') : ''}${content}${route !== '/start' ? prototypePanel() : ''}</main></div>
  </div>`;
}

function pageHead(title, description = '', action = '') {
  return `<header class="page-head"><div><h1 id="page-title" tabindex="-1">${escape(title)}</h1>${description ? `<p>${escape(description)}</p>` : ''}</div>${action}</header>`;
}

function startScreen() {
  const step = state.onboarding.step;
  const titles = ['Een eigen plek om te werken.', 'Alles begint lokaal.', 'Jouw materiaal. Jouw plek.'];
  const intro = ['Stel de bediening in. De rest kan later.', 'Een lokaal profiel is genoeg om te beginnen.', 'Bewaar je werk op dit apparaat. Je eigen NAS koppel je wanneer je wilt.'];
  let body;
  if (step === 1) body = `<label class="field">Taal<select id="language" aria-describedby="language-help"><option>Nederlands</option></select></label><p class="meta" id="language-help">Deze eerste demo is beschikbaar in het Nederlands.</p><label class="checkbox"><input id="reduce-motion" type="checkbox" ${state.onboarding.reducedMotion ? 'checked' : ''}>Minder beweging</label><p class="meta">Alle handelingen werken ook met Tab, Shift+Tab en Enter.</p>`;
  if (step === 2) body = `<label class="field">Naam van je voorbeeldprofiel<input id="profile-name" value="${escape(state.onboarding.name)}" maxlength="40" autocomplete="off" aria-describedby="profile-help"></label><p class="meta" id="profile-help">Gebruik een fictieve naam. Geen accountregistratie of wachtwoord nodig.</p><div class="onboarding-facts"><p>${icon('devices')}Je instellingen blijven in deze lokale demo.</p><p>${icon('shield')}Gereedschappen vragen toegang per document.</p></div>`;
  if (step === 3) body = `<div class="onboarding-facts"><p>${icon('devices')}Begin op dit apparaat. Je kunt direct documenten maken en bewerken.</p><p>${icon('nas')}Koppel later je eigen NAS binnen je eigen netwerk.</p><p>${icon('assistant')}De lokale assistent is optioneel. In deze demo is het antwoord gesimuleerd.</p></div>${notice('Dit is een interactieprototype met fictieve gegevens. Er wordt geen OS geïnstalleerd.')}`;
  return `<section class="onboarding"><div class="steps" aria-label="Stap ${step} van 3">${[1, 2, 3].map((number) => `<span class="${number <= step ? 'complete' : ''}"></span>`).join('')}</div>${icon(step === 3 ? 'material' : step === 2 ? 'shield' : 'devices', 'onboarding-icon')}<p class="meta">Stap ${step} van 3</p><h1 id="page-title" tabindex="-1">${titles[step - 1]}</h1><p class="muted">${intro[step - 1]}</p><div class="onboarding-body">${body}</div><div class="onboarding-actions">${step > 1 ? button('Terug', 'onboardingBack', 'quiet', '', 'back') : '<span></span>'}${button(step === 3 ? 'Begin lokaal' : 'Verder', step === 3 ? 'finishOnboarding' : 'onboardingNext', 'primary', '', 'arrow')}</div></section>`;
}

function materialScreen() {
  const result = listObjects(state, { query, collection }).value;
  const last = state.objects.find((object) => object.id === state.lastOpenedId) || state.objects[0];
  return `${pageHead('Mijn materiaal', 'Waar wil je aan werken?', button('Nieuw document', 'newDocument', 'primary', '', 'plus'))}
    <section class="continue" aria-labelledby="continue-title"><div class="continue-copy"><p class="meta">Verder waar je was</p><h2 id="continue-title">${escape(last.title)}</h2><p class="meta">${escape(last.collectionId)} · Versie ${last.revision} · Op dit apparaat</p><a class="button" href="#/object/${escape(last.id)}">Verder werken ${icon('arrow')}</a></div><div class="paper-preview" aria-hidden="true"><article><h3>${escape(last.title)}</h3><p>${escape(currentVersion(last).text.slice(0, 160))}</p></article></div></section>
    <section aria-label="Materiaal en collecties"><div class="material-toolbar"><div class="collections" role="group" aria-label="Collectie">${['Alles', ...COLLECTIONS].map((name) => button(escape(name), 'collection', '', `data-value="${name}" aria-pressed="${collection === name}"`)).join('')}</div><div class="search"><label class="sr-only" for="material-search">Zoek in mijn materiaal</label>${icon('search')}<input id="material-search" type="search" placeholder="Zoek in je materiaal" value="${escape(query)}" autocomplete="off"></div></div>
      <div class="section-heading"><h2>${query ? 'Zoekresultaten' : collection === 'Alles' ? 'Je documenten' : collection}</h2><span class="meta" aria-live="polite">${result.length} ${result.length === 1 ? 'object' : 'objecten'}</span></div>
      ${result.length ? `<ul class="object-list" id="search-results" aria-label="Documenten">${result.map((object) => `<li class="object-row"><span class="object-type ${object.mediaType}" aria-hidden="true">${icon(object.mediaType === 'image' ? 'image' : 'document')}</span><div class="object-info"><a class="object-title result-link" href="#/object/${escape(object.id)}">${escape(object.title)}</a><span class="meta">${object.mediaType === 'image' ? 'Afbeelding' : 'Tekstdocument'} · ${escape(object.collectionId)}</span></div><span class="meta object-location">Op dit apparaat<br>Versie ${object.revision}</span><a class="button quiet icon-button" href="#/object/${escape(object.id)}" aria-label="Open ${escape(object.title)}">${icon('arrow')}</a></li>`).join('')}</ul>` : `<div class="empty"><h2>${query ? 'Niets gevonden' : 'Hier is nog ruimte.'}</h2><p class="muted">${query ? `Geen materiaal met “${escape(query)}”. Probeer een andere titel of collectie.` : 'Deze collectie is nog leeg. Je andere documenten blijven bij Alles staan.'}</p>${button(query ? 'Wis zoekopdracht' : 'Bekijk alle materiaal', 'clearSearch', '')}</div>`}
    </section><p class="inline-note">${icon('devices')}Je materiaal staat op dit apparaat. Gereedschappen krijgen alleen toegang als jij dat toestaat.</p>`;
}

function planDrawing() {
  return `<svg class="plan" viewBox="0 0 400 400" role="img" aria-labelledby="plan-title plan-description"><title id="plan-title">Fictieve plattegrond van de tuin</title><desc id="plan-description">Een terras linksonder, border aan de rechterkant, open ruimte in het midden en schuur linksboven. Geen schaaltekening.</desc><rect x="20" y="20" width="360" height="360" rx="4" fill="#F4F5F1" stroke="#768783"/><rect x="40" y="40" width="110" height="70" fill="#E7EEE7" stroke="#768783"/><rect x="40" y="270" width="160" height="90" fill="#EAE6D9" stroke="#768783"/><path d="M160 320V130H95V110" fill="none" stroke="#DDD7C7" stroke-width="22"/><rect x="280" y="40" width="80" height="320" rx="22" fill="#DCE7DC" stroke="#768783"/><g fill="#0A5E55" opacity=".6"><circle cx="320" cy="82" r="19"/><circle cx="320" cy="142" r="23"/><circle cx="320" cy="207" r="18"/><circle cx="320" cy="268" r="23"/><circle cx="320" cy="326" r="17"/></g><g fill="#162B2B" font-family="system-ui, sans-serif" font-size="15"><text x="63" y="82">Schuur</text><text x="90" y="323">Terras</text><text x="173" y="210">Open ruimte</text></g></svg>`;
}

function nasLabel(object) {
  if (object.nasState === 'local') return 'Alleen op dit apparaat';
  if (state.nas.state !== 'online') return 'Wacht op eigen NAS';
  return 'Wacht op synchronisatie · simulatie';
}

function inspector(object) {
  const grants = activeGrants(state).filter((grant) => grant.objectIds.includes(object.id));
  const available = state.tools.filter((tool) => tool.installed && tool.types.includes(object.mediaType));
  return `<aside class="inspector ${inspectorOpen ? 'is-open' : ''}" id="object-inspector" aria-label="Documentinformatie"><section><h2>Gegevenslocatie</h2><p>${status('Op dit apparaat', 'devices')}</p><p class="meta section-gap">${escape(nasLabel(object))}</p><a class="button quiet" href="#/apparaten">Bekijk apparaten ${icon('arrow')}</a></section><section><h2>Gereedschap & toegang</h2><label class="field" for="tool-choice">Open dit document met</label><select id="tool-choice">${available.map((tool) => `<option value="${tool.id}">${tool.name}</option>`).join('')}</select>${button('Vraag toegang', 'requestChosenTool', 'quiet', 'id="request-tool"', 'shield')}${grants.length ? grants.map((grant) => `<p class="meta">${escape(state.tools.find((tool) => tool.id === grant.toolId).name)} · alleen dit document</p>${button('Trek toegang in', 'revoke', 'quiet', `data-id="${grant.id}"`)}`).join('') : '<p class="meta">Geen actieve toegang.</p>'}${state.lastGrantState === 'denied' ? '<p class="meta">Laatste aanvraag: geweigerd.</p>' : ''}</section><section><h2>Versiegeschiedenis</h2><p class="meta">Herstel bewaart alle eerdere versies.</p><ol class="version-list">${[...object.versions].reverse().map((version) => `<li>${button(`Versie ${version.id}${version.id === object.revision ? ' · huidig' : ''}<small>${escape(version.label)}</small>`, 'previewRevision', '', `data-revision="${version.id}"`)}</li>`).join('')}</ol></section></aside>`;
}

function objectScreen() {
  const object = selectedObject(state);
  if (!object) return `${pageHead('Document niet gevonden')}<p>Dit voorbeeld bestaat niet.</p><a class="button" href="#/materiaal">Terug naar Mijn materiaal</a>`;
  const editing = ['edit', 'saving'].includes(state.mode);
  let content;
  if (state.mode === 'preview') {
    const old = object.versions.find((version) => version.id === state.restoreRevision);
    content = `<div class="notice info">${icon('clock')}<p>Vergelijk versie ${old.id} met de huidige versie. Herstellen maakt versie ${object.revision + 1}; je eerdere versies blijven bewaard.</p></div><div class="comparison"><section><h2>Huidig · versie ${object.revision}</h2><p class="document-text">${escape(currentVersion(object).text)}</p></section><section><h2>Herstelpreview · versie ${old.id}</h2><p class="document-text">${escape(old.text)}</p></section></div><div class="actions preview-actions">${button('Annuleer herstel', 'cancelRestore')}${button('Herstel als nieuwe versie', 'restoreRevision', 'primary', `data-base="${object.revision}"`)}</div>`;
  } else {
    content = `<div class="document-layout"><article class="document" aria-label="Inhoud van ${escape(object.title)}">${object.mediaType === 'image' ? planDrawing() + '<p class="meta">Lokale SVG-fixture · geen schaaltekening. Pixelbewerking is in deze proef niet beschikbaar.</p>' : editing ? `<label class="editor-label" for="document-editor">Notities · alleen dit document${state.mode === 'saving' ? ' · Opslaan…' : ''}</label><textarea id="document-editor" maxlength="20000" spellcheck="false" ${state.mode === 'saving' ? 'readonly' : ''}>${escape(state.draft?.text)}</textarea>` : `<div class="document-text">${escape(currentVersion(object).text) || 'Dit document is nog leeg. Kies Bewerken om te beginnen.'}</div>`}</article>${inspector(object)}</div>`;
  }
  return `<a class="button quiet object-back" href="#/materiaal" data-close-object>${icon('back')}Mijn materiaal</a>${pageHead(object.title, `${object.collectionId} · Versie ${object.revision} · Op dit apparaat`, `<div class="actions">${button('Informatie', 'toggleInspector', 'inspector-toggle', `aria-expanded="${inspectorOpen}" aria-controls="object-inspector"`, 'info')}${object.mediaType === 'text' && state.mode !== 'preview' ? button(state.mode === 'saving' ? 'Opslaan…' : editing ? 'Opslaan' : 'Bewerken', editing ? 'save' : 'requestEdit', 'primary', `id="edit-object" ${state.mode === 'saving' ? 'disabled' : ''}`, editing ? 'check' : 'tools') : ''}</div>`)}${content}`;
}

function accessScreen() {
  if (state.grantRequest) return objectScreen();
  const grants = state.grants.filter((grant) => ['active', 'revoked'].includes(grant.state));
  return `${pageHead('Toegang', 'Je bepaalt welk gereedschap welk materiaal kan gebruiken.')}<p class="inline-note">${icon('shield')}Dit scherm simuleert een vertrouwd systeemoppervlak. De browser is geen OS-beveiligingsgrens.</p>${grants.length ? `<ul class="grants-list">${grants.map((grant) => `<li><div><h2>${escape(state.tools.find((tool) => tool.id === grant.toolId)?.name)}</h2><p>${grant.objectIds.map((id) => escape(state.objects.find((object) => object.id === id)?.title)).join(', ')} · ${grant.operations.includes('write') ? 'lezen en wijzigen' : 'lezen'}</p><p class="meta">${grant.state === 'active' ? 'Tot je dit document sluit' : 'Ingetrokken'}</p></div>${grant.state === 'active' ? button('Trek toegang in', 'revoke', '', `data-id="${grant.id}"`) : status('Ingetrokken', 'close')}</li>`).join('')}</ul>` : '<div class="empty"><h2>Geen actieve toegang</h2><p class="muted">Open een document om een gereedschap tijdelijk toegang te geven.</p></div>'}<a class="button" href="#/materiaal">Naar Mijn materiaal ${icon('arrow')}</a>`;
}

function accessDialog() {
  const request = state.grantRequest;
  if (!request) return '';
  const tool = state.tools.find((entry) => entry.id === request.toolId);
  const object = state.objects.find((entry) => entry.id === request.objectIds[0]);
  return `<dialog id="access-dialog" aria-labelledby="access-title" aria-describedby="access-intro"><div class="dialog-content"><div class="dialog-heading">${icon('shield')}<span>Systeemtoegang · simulatie</span></div><h2 id="access-title">Geef ${escape(tool.name)} toegang?</h2><p id="access-intro">${escape(tool.name)} mag ${escape(object.title)} ${request.operations.includes('write') ? 'lezen en wijzigen' : 'lezen'} tot je dit document sluit.</p><dl class="permission-details"><div><dt>Materiaal</dt><dd>${escape(object.title)} · alleen dit document</dd></div><div><dt>Handeling</dt><dd>${request.operations.includes('write') ? 'Lezen en wijzigen' : 'Lezen'}</dd></div><div><dt>Duur</dt><dd>Tot je dit document sluit</dd></div><div><dt>Netwerk</dt><dd>${request.internet ? 'Internet naar demo.invalid · fictieve bestemming' : 'Geen internet, geen NAS-toegang'}</dd></div></dl>${request.internet ? notice('Dit programma kan de toegestane inhoud naar een internetdienst sturen.', 'warning') : ''}<p class="meta section-gap">Intrekken blokkeert nieuwe handelingen. Eerder gemaakte kopieën worden niet gewist.</p></div><div class="dialog-actions">${button('Annuleer', 'denyGrant', '', 'id="deny-access" autofocus')}${button('Sta tijdelijk toe', 'grantSession', 'primary')}</div></dialog>`;
}

function toolScreen() {
  return `${pageHead('Gereedschap', 'Je materiaal blijft van jou. Kies waarmee je eraan werkt.')}<div class="tool-list">${state.tools.map((tool) => {
    const installing = state.install.state === 'checking' && state.install.toolId === tool.id;
    return `<article class="tool-entry"><div class="tool-logo">${icon(tool.id === 'webvenster' ? 'globe' : tool.id === 'beeldbewerker' ? 'image' : 'tools')}</div><div><h2>${tool.name}</h2><p class="muted">${tool.description}</p><span class="meta">${tool.installed ? 'Beschikbaar op dit apparaat' : state.install.state === 'removed' && state.install.toolId === tool.id ? 'Voorbeeldpakket verwijderd' : 'Lokaal voorbeeldpakket · versie 1.0'} · ${activeGrants(state).filter((grant) => grant.toolId === tool.id).length} actieve rechten</span><details><summary>Pakketdetails en rechten</summary><div class="detail-content"><p>Fictieve uitgever: Materiaal Voorbeelden. De demo voert geen pakketcode uit.</p><p>${tool.id === 'webvenster' ? 'Internettoegang en toegang tot een document worden samen zichtbaar aangevraagd. Er wordt geen internetverbinding gemaakt.' : 'Installatie verleent geen toegang tot documenten, NAS of netwerk. Toegang volgt pas bij een concrete handeling.'}</p>${tool.id === 'beeldbewerker' ? '<p>De controle en installatie zijn gesimuleerd. Afbeeldingsbewerking zelf is nog niet beschikbaar.</p>' : ''}</div></details></div><div class="actions">${tool.removable ? tool.installed ? button('Verwijder voorbeeld', 'removeTool', '', `data-id="${tool.id}"`) : button(installing ? 'Controle…' : 'Installeer voorbeeld', 'install', 'primary', `data-id="${tool.id}" ${installing ? 'disabled' : ''}`, installing ? 'spinner' : 'plus') : button('Open met materiaal', 'openTool', '', `data-id="${tool.id}"`, 'arrow')}</div></article>`;
  }).join('')}</div><p class="inline-note">${icon('shield')}De catalogus bevat alleen lokale fictieve pakketten. Geen download, appcompatibiliteit of echte handtekeningcontrole.</p>`;
}

function devicesScreen() {
  const condition = state.nas.state;
  const descriptions = {
    unpaired: 'Koppel je eigen NAS wanneer je je materiaal ook daar wilt bewaren.',
    verifying: 'Vergelijk deze voorbeeldidentiteit met de code op je eigen NAS. In deze demo is geen apparaat verbonden.',
    online: 'De voorbeeldidentiteit is bevestigd. Alleen de geselecteerde dataset is gekoppeld.',
    offline: 'Thuis-NAS is niet bereikbaar. Je wijzigingen blijven op dit apparaat en wachten op je eigen NAS.',
    networkChanged: 'Het netwerk is veranderd. Synchronisatie is gepauzeerd totdat je de lokale verbinding opnieuw controleert.',
    identityMismatch: 'De identiteit wijkt af. Synchronisatie is gestopt. Vertrouw deze verbinding pas na een nieuwe identiteitscontrole.',
  };
  const labels = { unpaired: 'Niet gekoppeld', verifying: 'Code vergelijken', online: 'Gekoppeld · simulatie', offline: 'Wacht op eigen NAS', networkChanged: 'Netwerk veranderd', identityMismatch: 'Identiteit afwijkend' };
  const problem = ['offline', 'networkChanged', 'identityMismatch'].includes(condition);
  let action = '';
  if (condition === 'unpaired') action = button('Koppel eigen NAS', 'pairNas', 'primary');
  if (condition === 'verifying') action = `<div class="identity-code">${NAS_IDENTITY}</div><label class="checkbox"><input type="checkbox" id="nas-confirm">Ik heb de voorbeeldidentiteit vergeleken</label><div class="actions">${button('Annuleer', 'unpairNas')}${button('Bevestig koppeling', 'confirmNas', 'primary', 'id="confirm-nas" disabled')}</div>`;
  if (condition === 'online') action = `<dl><div><dt>Bevestigde identiteit</dt><dd>${NAS_IDENTITY}</dd></div><div><dt>Geselecteerde dataset</dt><dd>Huis</dd></div><div><dt>Synchronisatie</dt><dd>Geen bestanden verstuurd in deze demo</dd></div></dl><div class="actions">${button('Ontkoppel', 'unpairNas')}</div>`;
  if (problem) action = `<div class="actions">${button('Verbinding opnieuw controleren', 'pairNas', 'primary')}</div>`;
  return `${pageHead('Apparaten', 'Hier staat je materiaal. Jij bepaalt wat je koppelt.')}<div class="device-layout"><section class="device">${icon('devices', 'device-symbol')}<h2>Dit apparaat</h2>${status('Lokaal beschikbaar')}<dl><div><dt>Type</dt><dd>Desktop · voorbeeld</dd></div><div><dt>Materiaal</dt><dd>${state.objects.length} lokale objecten</dd></div><div><dt>Opslag</dt><dd>Fictieve browsersessie</dd></div></dl><div class="actions"><a class="button" href="#/materiaal">Open materiaal ${icon('arrow')}</a></div></section><section class="device">${icon('nas', 'device-symbol')}<h2>Thuis-NAS</h2>${status(labels[condition], problem ? 'warning' : condition === 'online' ? 'check' : 'nas', condition === 'identityMismatch' ? 'error' : problem ? 'warning' : '')}<p class="section-gap">${descriptions[condition]}</p>${action}</section></div><p class="inline-note">${icon('shield')}Alleen je eigen NAS binnen je eigen netwerk. Lokaal werken blijft mogelijk als de verbinding wegvalt.</p>`;
}

function assistantScreen() {
  const ai = state.ai;
  let content;
  if (ai.availability !== 'ready') content = `${notice(ai.availability === 'missingModel' ? 'Er is geen lokaal model beschikbaar. De assistent kan deze taak niet uitvoeren.' : 'Er is onvoldoende lokaal geheugen. De assistent kan deze taak nu niet uitvoeren.', 'warning')}<h2>Je kunt gewoon verder werken.</h2><p class="muted">Je documenten blijven beschikbaar. Er wordt geen externe assistent gestart.</p><div class="actions"><a class="button" href="#/materiaal">Gebruik zonder assistent ${icon('arrow')}</a></div>`;
  else if (ai.state === 'running') content = `<div class="loading">${icon('spinner')}<h2>Voorbeeldsamenvatting voorbereiden…</h2></div><p class="muted section-gap">Alleen de geselecteerde context wordt in deze lokale simulatie gebruikt.</p><div class="actions">${button('Annuleer', 'cancelSummary')}</div>`;
  else content = `<h2>Een frisse blik op je selectie.</h2><p class="muted">Kies het materiaal dat de assistent mag gebruiken. De rest blijft buiten deze handeling.</p><div class="actions">${button('Vat selectie samen', 'startSummary', 'primary', ai.context.length ? '' : 'disabled', 'assistant')}</div>${!ai.context.length ? '<p class="meta section-gap">Selecteer eerst ten minste één document.</p>' : ''}${ai.state === 'cancelled' ? notice('Geannuleerd. Er is geen antwoord opgeslagen.') : ''}${ai.state === 'result' ? `<section class="answer" aria-labelledby="answer-title"><h3 id="answer-title">Voorbeeldantwoord · simulatie</h3><p>${escape(ai.response)}</p></section>` : ''}`;
  return `${pageHead('Lokale assistent', 'Alleen het materiaal dat jij selecteert.')}<div class="assistant-layout"><section><h2>Geselecteerde context</h2><p class="meta section-gap">Maximaal drie documenten voor deze proef.</p><div class="context-list">${state.objects.map((object) => `<label class="checkbox"><input type="checkbox" data-context="${escape(object.id)}" ${ai.context.includes(object.id) ? 'checked' : ''} ${ai.context.length >= 3 && !ai.context.includes(object.id) ? 'disabled' : ''}><span>${escape(object.title)}<br><small>Op dit apparaat</small></span></label>`).join('')}</div><p class="inline-note">${icon('shield')}Geen internetrecht. Geen algemene toegang tot je materiaal.</p></section><section class="assistant-main">${content}</section></div><p class="inline-note">${icon('info')}Deterministische voorbeeldrespons. Er draait geen AI-model; er wordt niets gedownload.</p>`;
}

function prototypePanel() {
  return `<details class="prototype-panel" id="prototype-panel"><summary>Prototypepaneel · test een situatie</summary><p class="meta">Alleen voor deze interactieproef. Deze bediening hoort niet bij de uiteindelijke OS-ervaring. Er wordt geen echt apparaat of model aangestuurd.</p><div class="prototype-grid"><label class="field">Eigen NAS<select id="demo-nas">${[['unpaired', 'Niet gekoppeld'], ['offline', 'NAS offline'], ['networkChanged', 'Netwerk veranderd'], ['identityMismatch', 'Verkeerde identiteit']].map(([value, text]) => `<option value="${value}" ${state.nas.state === value ? 'selected' : ''}>${text}</option>`).join('')}${['online', 'verifying'].includes(state.nas.state) ? `<option value="${state.nas.state}" selected>${state.nas.state === 'online' ? 'Gekoppeld' : 'Identiteit controleren'}</option>` : ''}</select></label><label class="field">Lokale assistent<select id="demo-ai">${[['ready', 'Model aanwezig · simulatie'], ['missingModel', 'Model ontbreekt'], ['lowMemory', 'Onvoldoende geheugen']].map(([value, text]) => `<option value="${value}" ${state.ai.availability === value ? 'selected' : ''}>${text}</option>`).join('')}</select></label><div><label class="checkbox"><input id="demo-package" type="checkbox" ${state.conditions.invalidPackage ? 'checked' : ''}>Ongeldige pakketsignatuur</label><p class="meta">Bij de volgende voorbeeldinstallatie.</p></div></div><a class="button quiet" href="#/start">Bekijk de eerste start ${icon('arrow')}</a><a class="button quiet" href="#/toegang">Bekijk toegang ${icon('shield')}</a></details>`;
}

function render(focusId = null) {
  const oldActive = document.activeElement;
  const oldId = oldActive?.id;
  const selection = typeof oldActive?.selectionStart === 'number' ? [oldActive.selectionStart, oldActive.selectionEnd] : null;
  const panelOpen = document.querySelector('#prototype-panel')?.open;
  const route = currentRoute();
  document.documentElement.classList.toggle('reduce-motion', state.onboarding.reducedMotion);
  let content;
  if (route === '/start') content = startScreen();
  else if (route === '/materiaal') content = materialScreen();
  else if (route.startsWith('/object/')) content = objectScreen();
  else if (route === '/toegang') content = accessScreen();
  else if (route === '/gereedschap') content = toolScreen();
  else if (route === '/apparaten') content = devicesScreen();
  else content = assistantScreen();
  app.innerHTML = shell(content, route) + (route === '/toegang' ? accessDialog() : '');
  document.title = `${document.querySelector('h1')?.textContent || 'Materiaal'} · Materiaal prototype`;
  if (panelOpen && document.querySelector('#prototype-panel')) document.querySelector('#prototype-panel').open = true;
  const editor = document.querySelector('#document-editor');
  if (editor && state.draft) editor.setSelectionRange(state.draft.cursor, state.draft.cursor);
  const dialog = document.querySelector('#access-dialog');
  if (dialog) {
    dialog.showModal();
    dialog.addEventListener('cancel', (event) => { event.preventDefault(); denyAccess(); });
    document.querySelector('#deny-access').focus();
  } else {
    const target = document.getElementById(focusId || oldId);
    target?.focus({ preventScroll: !focusId });
    if (!focusId && selection && target?.setSelectionRange && !['checkbox', 'number'].includes(target.type)) target.setSelectionRange(...selection);
  }
}

function requestAccess(toolId) {
  dispatch({ type: 'requestEdit', toolId }, { redraw: false });
  if (state.grantRequest) navigate('/toegang', null); else render();
}

function denyAccess() {
  const id = state.selectedObjectId;
  dispatch({ type: 'denyGrant' }, { redraw: false });
  navigate(id ? `/object/${id}` : '/materiaal', 'edit-object');
}

function save() {
  if (state.mode !== 'edit') return;
  dispatch({ type: 'beginSave' }, { focus: 'edit-object' });
  clearTimeout(saveTimer);
  // A short, cancellable scheduled state makes the asynchronous contract visible.
  saveTimer = setTimeout(() => dispatch({ type: 'saveDraft' }, { focus: 'edit-object' }), 160);
}

document.addEventListener('click', (event) => {
  if (event.target.closest('.skip-link')) {
    event.preventDefault();
    document.querySelector('#main')?.focus();
    return;
  }
  const element = event.target.closest('[data-action]');
  if (!element || element.disabled) return;
  const action = element.dataset.action;
  switch (action) {
    case 'toggleMenu': menuOpen = !menuOpen; render(); break;
    case 'resetDemo': {
      clearTimeout(saveTimer); clearTimeout(installTimer); clearTimeout(summaryTimer);
      try { sessionStorage.removeItem(STORAGE_KEY); } catch { /* Session remains usable in memory. */ }
      state = createFixtureState(); query = ''; collection = 'Alles'; inspectorOpen = false;
      navigate('/start'); break;
    }
    case 'onboardingNext': dispatch({ type: 'onboardingStep', step: state.onboarding.step + 1 }, { focus: 'page-title' }); break;
    case 'onboardingBack': dispatch({ type: 'onboardingStep', step: state.onboarding.step - 1 }, { focus: 'page-title' }); break;
    case 'finishOnboarding': dispatch({ type: 'finishOnboarding' }, { redraw: false }); navigate('/materiaal'); break;
    case 'collection': collection = element.dataset.value; render('material-search'); break;
    case 'clearSearch': query = ''; collection = 'Alles'; render('material-search'); break;
    case 'newDocument': dispatch({ type: 'newDocument' }, { redraw: false }); navigate(`/object/${state.selectedObjectId}`); break;
    case 'requestEdit': requestAccess('notities'); break;
    case 'requestChosenTool': requestAccess(document.querySelector('#tool-choice').value); break;
    case 'denyGrant': denyAccess(); break;
    case 'grantSession': {
      const id = state.selectedObjectId;
      dispatch({ type: 'grantSession' }, { redraw: false });
      navigate(`/object/${id}`, 'edit-object'); break;
    }
    case 'revoke': dispatch({ type: 'revokeGrant', id: element.dataset.id }, { focus: 'page-title' }); break;
    case 'save': save(); break;
    case 'previewRevision': dispatch({ type: 'previewRestore', revision: Number(element.dataset.revision) }, { focus: 'page-title' }); break;
    case 'cancelRestore': dispatch({ type: 'cancelRestore' }, { focus: 'page-title' }); break;
    case 'restoreRevision': dispatch({ type: 'restoreRevision', baseRevision: Number(element.dataset.base) }, { focus: 'page-title' }); break;
    case 'toggleInspector': inspectorOpen = !inspectorOpen; render(); break;
    case 'install': {
      const id = element.dataset.id;
      dispatch({ type: 'beginInstall', id });
      clearTimeout(installTimer);
      installTimer = setTimeout(() => dispatch({ type: 'installFixture', id }, { focus: 'page-title' }), 180); break;
    }
    case 'removeTool': dispatch({ type: 'removeTool', id: element.dataset.id }, { focus: 'page-title' }); break;
    case 'openTool': {
      dispatch({ type: 'openObject', id: 'obj-001' }, { redraw: false });
      requestAccess(element.dataset.id); break;
    }
    case 'pairNas': dispatch({ type: 'pairNas' }, { focus: 'page-title' }); break;
    case 'unpairNas': dispatch({ type: 'nasCondition', condition: 'unpaired' }, { focus: 'page-title' }); break;
    case 'confirmNas': {
      const confirmed = Boolean(document.querySelector('#nas-confirm')?.checked);
      if (confirmed) dispatch({ type: 'confirmNas', confirmed: true, identity: NAS_IDENTITY }, { focus: 'page-title' });
      break;
    }
    case 'startSummary': {
      dispatch({ type: 'startSummary' });
      if (state.ai.state === 'running') {
        const runId = state.ai.runId;
        clearTimeout(summaryTimer);
        summaryTimer = setTimeout(() => dispatch({ type: 'finishSummary', runId }, { focus: 'page-title' }), 240);
      }
      break;
    }
    case 'cancelSummary': clearTimeout(summaryTimer); dispatch({ type: 'cancelSummary' }, { focus: 'page-title' }); break;
  }
});

document.addEventListener('input', (event) => {
  const element = event.target;
  if (element.id === 'material-search') { query = element.value; render(); }
  if (element.id === 'profile-name') dispatch({ type: 'profileName', name: element.value }, { redraw: false });
  if (element.id === 'document-editor') dispatch({ type: 'updateDraft', text: element.value, cursor: element.selectionStart }, { redraw: false });
});

document.addEventListener('change', (event) => {
  const element = event.target;
  if (element.id === 'reduce-motion') dispatch({ type: 'reducedMotion', value: element.checked });
  if (element.id === 'nas-confirm') document.querySelector('#confirm-nas').disabled = !element.checked;
  if (element.id === 'demo-nas') dispatch({ type: 'nasCondition', condition: element.value });
  if (element.id === 'demo-ai') dispatch({ type: 'aiCondition', condition: element.value });
  if (element.id === 'demo-package') dispatch({ type: 'invalidPackage', value: element.checked });
  if (element.dataset.context) dispatch({ type: 'aiContext', id: element.dataset.context, selected: element.checked });
});

document.addEventListener('keydown', (event) => {
  const modal = document.querySelector('dialog[open]');
  if (modal) {
    // Keep keyboard navigation in the active system-access dialog. Browser
    // focus traversal can otherwise leave a native dialog at its first button.
    if (event.key === 'Tab') {
      const controls = [...modal.querySelectorAll('button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])')]
        .filter((element) => element.getClientRects().length > 0);
      const first = controls[0];
      const last = controls.at(-1);
      if (first && (event.shiftKey ? document.activeElement === first || !modal.contains(document.activeElement) : document.activeElement === last || !modal.contains(document.activeElement))) {
        event.preventDefault();
        (event.shiftKey ? last : first).focus();
      }
    } else if ((event.ctrlKey || event.metaKey) && ['k', 's'].includes(event.key.toLowerCase())) {
      event.preventDefault();
    }
    return;
  }
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'k') {
    event.preventDefault(); navigate('/materiaal', 'material-search'); return;
  }
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 's' && currentRoute().startsWith('/object/')) {
    event.preventDefault(); save(); return;
  }
  if (event.key === 'Escape' && state.mode === 'preview') dispatch({ type: 'cancelRestore' }, { focus: 'page-title' });
  const links = [...document.querySelectorAll('.result-link')];
  if (['ArrowDown', 'ArrowUp'].includes(event.key) && (event.target.id === 'material-search' || event.target.classList.contains('result-link'))) {
    if (!links.length) return;
    event.preventDefault();
    const index = links.indexOf(event.target);
    const next = index < 0 ? (event.key === 'ArrowDown' ? 0 : links.length - 1) : (index + (event.key === 'ArrowDown' ? 1 : -1) + links.length) % links.length;
    links[next].focus();
  }
});

window.addEventListener('hashchange', routeChanged);
document.querySelector('.skip-link').addEventListener('click', event => {
  event.preventDefault();
  document.querySelector('#main')?.focus();
});
routeChanged();
