import test from 'node:test';
import assert from 'node:assert/strict';

test('search preserves its shell and input persistence is bounded and flushed', async (t) => {
  const handlers = new Map();
  const storage = new Map();
  let appWrites = 0;
  let storageWrites = 0;
  let focusCalls = 0;
  let appMarkup = '';
  const resultElement = { innerHTML: '' };
  const editorElement = { id: 'document-editor', value: '', selectionStart: 0,
    setSelectionRange(start) { this.selectionStart = start; } };
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
      if (selector === '#material-results') return appMarkup.includes('id="material-results"') ? resultElement : null;
      if (selector === '#document-editor') return appMarkup.includes('id="document-editor"') ? editorElement : null;
      if (selector === '#prototype-panel' || selector === '#access-dialog, #nas-removal-dialog') return null;
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
    t.mock.timers.enable({ apis: ['setTimeout'] });
    await import(`../app.js?render-performance-test=${Date.now()}`);
    assert.match(appMarkup, /id="material-results"/, 'results must exist in the rendered application markup');
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
    t.mock.timers.tick(220);
    assert.equal(storageWrites, 1);
    assert.equal(JSON.parse(storage.get('nieuw-os-prototype-v1')).onboarding.name, 'Simon');

    input({ target: { id: 'profile-name', value: 'Simon V' } });
    handlers.get('document:focusout')({ target: { id: 'profile-name' } });
    assert.equal(storageWrites, 2, 'leaving the field flushes the latest state');
    input({ target: { id: 'profile-name', value: 'Simon V.' } });
    handlers.get('window:pagehide')();
    assert.equal(storageWrites, 3, 'pagehide flushes pending input');
    t.mock.timers.tick(220);
    assert.equal(storageWrites, 3, 'flushing cancels the pending write');
    assert.equal(JSON.parse(storage.get('nieuw-os-prototype-v1')).onboarding.name, 'Simon V.');

    for (let index = 0; index < 6; index += 1) {
      input({ target: { id: 'profile-name', value: `Invoer ${index}` } });
      t.mock.timers.tick(60);
      if (index === 2 || index === 5) {
        assert.equal(storageWrites, 3 + (index + 1) / 3, 'continuous typing must persist within each 180ms interval');
        assert.equal(JSON.parse(storage.get('nieuw-os-prototype-v1')).onboarding.name, `Invoer ${index}`);
      }
    }
    input({ target: { id: 'profile-name', value: 'Laatste invoer' } });
    t.mock.timers.tick(180);
    assert.equal(JSON.parse(storage.get('nieuw-os-prototype-v1')).onboarding.name, 'Laatste invoer', 'the trailing input is persisted too');

    location.hash = '#/object/obj-001';
    handlers.get('window:hashchange')();
    const click = (action) => handlers.get('document:click')({ target: {
      closest(selector) { return selector === '[data-action]' ? { dataset: { action } } : null; },
    } });
    click('requestEdit');
    handlers.get('window:hashchange')();
    click('grantSession');
    handlers.get('window:hashchange')();
    assert.match(appMarkup, /id="document-editor"/, 'the editor must be reached through the actual grant route');
    for (const [event, text, cursor] of [['document:focusout', 'Eerste concept', 7], ['window:pagehide', 'Laatste concept', 4]]) {
      const before = storageWrites;
      editorElement.value = text;
      editorElement.selectionStart = cursor;
      input({ target: editorElement });
      assert.equal(storageWrites, before, 'editor input is deferred');
      handlers.get(event)({ target: editorElement });
      assert.equal(storageWrites, before + 1, `${event} flushes the current draft`);
      const saved = JSON.parse(storage.get('nieuw-os-prototype-v1')).draft;
      assert.equal(saved.text, text);
      assert.equal(saved.cursor, cursor);
      t.mock.timers.tick(220);
      assert.equal(storageWrites, before + 1, `${event} cancels the queued draft write`);
    }
  } finally {
    t.mock.timers.reset();
    for (const [name, descriptor] of globals) {
      if (descriptor) Object.defineProperty(globalThis, name, descriptor);
      else delete globalThis[name];
    }
  }
});
