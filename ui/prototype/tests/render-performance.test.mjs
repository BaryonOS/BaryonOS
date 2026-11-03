import test from 'node:test';
import assert from 'node:assert/strict';

test('search updates only results and input persistence is coalesced and flushed', async () => {
  const handlers = new Map();
  const storage = new Map();
  let appWrites = 0;
  let storageWrites = 0;
  let focusCalls = 0;
  let appMarkup = '';
  const resultElement = { innerHTML: '' };
  const searchInput = { id: 'material-search', focus() { focusCalls += 1; } };
  const appElement = {
    set innerHTML(markup) { appWrites += 1; appMarkup = markup; },
    get innerHTML() { return appMarkup; },
  };
  const announcementElement = { textContent: '' };
  const titleElement = { textContent: 'Mijn materiaal' };
  const pageTitle = { focus() { focusCalls += 1; } };
  const document = {
    activeElement: searchInput,
    documentElement: { classList: { toggle() {} } },
    addEventListener(type, handler) { handlers.set(`document:${type}`, handler); },
    getElementById(id) { return id === 'page-title' ? pageTitle : searchInput; },
    querySelector(selector) {
      if (selector === '#app') return appElement;
      if (selector === '#announcements') return announcementElement;
      if (selector === '#material-results') return resultElement;
      if (selector === '#prototype-panel' || selector === '#document-editor' || selector === '#access-dialog, #nas-removal-dialog') return null;
      if (selector === 'h1') return titleElement;
      return null;
    },
  };
  const window = { addEventListener(type, handler) { handlers.set(`window:${type}`, handler); } };
  const location = { hash: '#/materiaal' };
  const sessionStorage = {
    getItem(key) { return storage.get(key) ?? null; },
    setItem(key, value) { storage.set(key, value); storageWrites += 1; },
    removeItem(key) { storage.delete(key); },
  };
  const globals = new Map();
  for (const [name, value] of Object.entries({ document, window, location, sessionStorage })) {
    globals.set(name, Object.getOwnPropertyDescriptor(globalThis, name));
    Object.defineProperty(globalThis, name, { configurable: true, writable: true, value });
  }

  try {
    await import(`../app.js?render-performance-test=${Date.now()}`);
    const rendersBeforeSearch = appWrites;
    const focusesBeforeSearch = focusCalls;
    handlers.get('document:input')({ target: { id: 'material-search', value: 'TUIN' } });
    assert.equal(appWrites, rendersBeforeSearch, 'search must not replace the application shell');
    assert.equal(focusCalls, focusesBeforeSearch, 'search must keep focus in the existing input');
    assert.match(resultElement.innerHTML, /Tuinplan/);
    assert.doesNotMatch(resultElement.innerHTML, /Boodschappen/);
    assert.equal(storageWrites, 0, 'search does not persist unrelated state');

    location.hash = '#/start';
    handlers.get('window:hashchange')();
    const input = handlers.get('document:input');
    for (const value of ['S', 'Si', 'Simon']) input({ target: { id: 'profile-name', value } });
    assert.equal(storageWrites, 0, 'typing batches synchronous storage writes');
    await new Promise((resolve) => setTimeout(resolve, 220));
    assert.equal(storageWrites, 1);
    assert.equal(JSON.parse(storage.get('nieuw-os-prototype-v1')).onboarding.name, 'Simon');

    input({ target: { id: 'profile-name', value: 'Simon V' } });
    handlers.get('document:focusout')({ target: { id: 'profile-name' } });
    assert.equal(storageWrites, 2, 'leaving the field flushes the latest state');
    input({ target: { id: 'profile-name', value: 'Simon V.' } });
    handlers.get('window:pagehide')();
    assert.equal(storageWrites, 3, 'pagehide flushes pending input');
    await new Promise((resolve) => setTimeout(resolve, 220));
    assert.equal(storageWrites, 3, 'flushing cancels the pending write');
    assert.equal(JSON.parse(storage.get('nieuw-os-prototype-v1')).onboarding.name, 'Simon V.');
  } finally {
    for (const [name, descriptor] of globals) {
      if (descriptor) Object.defineProperty(globalThis, name, descriptor);
      else delete globalThis[name];
    }
  }
});
