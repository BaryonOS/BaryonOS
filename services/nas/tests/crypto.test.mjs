import test from 'node:test';
import assert from 'node:assert/strict';
import { randomBytes } from 'node:crypto';
import { createDocumentCipher, restoreDocumentReader, parseEnvelope, MAX_CONTENT_BYTES, MAX_ENVELOPE_BYTES } from '../crypto.mjs';

const namespace = 'a'.repeat(64);
const objectId = 'b'.repeat(32);
const expected = { objectId, revision: 1 };
const document = { name: 'Persoonlijke notitie — patiënt.txt', content: 'Synthetische inhoud, géén echte gegevens. 🌱' };
const fresh = () => { const cipher = createDocumentCipher(namespace); return { cipher, bytes: cipher.encrypt({ ...expected, ...document }) }; };
const authenticationError = (error) => error.code === 'AUTHENTICATION_FAILED';

test('AES-256-GCM encrypts name and content and authenticates a real round trip', () => {
  const { cipher, bytes } = fresh();
  assert.deepEqual(cipher.decrypt(expected, bytes), document);
  const envelope = parseEnvelope(bytes);
  assert.equal(Buffer.from(envelope.nonce, 'base64').length, 12);
  assert.equal(Buffer.from(envelope.tag, 'base64').length, 16);
  assert.equal(cipher.exportRecoveryKey().length, 32);
  assert.equal(bytes.includes(Buffer.from(document.name)), false);
  assert.equal(bytes.includes(Buffer.from(document.content)), false);
  assert.equal(JSON.stringify(cipher).includes(cipher.exportRecoveryKey().toString('base64')), false);
});

test('each encryption obtains a distinct cryptographically random nonce and ciphertext', () => {
  const cipher = createDocumentCipher(namespace); const nonces = new Set(); const ciphertexts = new Set();
  for (let revision = 1; revision <= 128; revision++) {
    const envelope = parseEnvelope(cipher.encrypt({ objectId, revision, ...document }));
    nonces.add(envelope.nonce); ciphertexts.add(envelope.ciphertext);
  }
  assert.equal(nonces.size, 128); assert.equal(ciphertexts.size, 128);
});

test('another client key cannot decrypt the same namespace', () => {
  const { bytes } = fresh(); const stranger = createDocumentCipher(namespace);
  assert.throws(() => stranger.decrypt(expected, bytes), authenticationError);
});

for (const field of ['nonce', 'tag', 'ciphertext']) {
  test(`a changed ${field} fails closed without partial plaintext`, () => {
    const { cipher, bytes } = fresh(); const envelope = JSON.parse(bytes);
    const changed = Buffer.from(envelope[field], 'base64'); changed[0] ^= 1;
    envelope[field] = changed.toString('base64');
    assert.throws(() => cipher.decrypt(expected, Buffer.from(JSON.stringify(envelope))), authenticationError);
  });
}

for (const [field, replacement] of [['namespace', 'c'.repeat(64)], ['objectId', 'c'.repeat(32)], ['revision', 2]]) {
  test(`authenticated binding rejects altered ${field}, including a matching requested revision`, () => {
    const { cipher, bytes } = fresh(); const envelope = JSON.parse(bytes); envelope[field] = replacement;
    const matchingRequest = { objectId: envelope.objectId, revision: envelope.revision };
    assert.throws(() => cipher.decrypt(matchingRequest, Buffer.from(JSON.stringify(envelope))), authenticationError);
  });
}

test('caller-held expected revision rejects replay of an older authentic document', () => {
  const { cipher, bytes } = fresh();
  assert.throws(() => cipher.decrypt({ objectId, revision: 2 }, bytes), authenticationError);
});

test('strict envelope rejects extra fields, wrong version, truncated tag and malformed base64', () => {
  const { cipher, bytes } = fresh(); const original = JSON.parse(bytes);
  for (const envelope of [{ ...original, name: document.name }, { ...original, v: 2 },
    { ...original, tag: randomBytes(8).toString('base64') }, { ...original, nonce: original.nonce + '=' }]) {
    assert.throws(() => cipher.decrypt(expected, Buffer.from(JSON.stringify(envelope))), authenticationError);
  }
  assert.throws(() => cipher.decrypt(expected, Buffer.from('{broken')), authenticationError);
  assert.throws(() => parseEnvelope(Buffer.alloc(MAX_ENVELOPE_BYTES + 1)), { code: 'INVALID_ENVELOPE' });
});

test('input limits reject oversize plaintext and unsafe revisions before encryption', () => {
  const cipher = createDocumentCipher(namespace);
  assert.throws(() => cipher.encrypt({ ...expected, name: 'x', content: 'x'.repeat(MAX_CONTENT_BYTES + 1) }), { code: 'DOCUMENT_TOO_LARGE' });
  assert.throws(() => cipher.encrypt({ ...expected, name: 'x'.repeat(513), content: '' }), { code: 'DOCUMENT_TOO_LARGE' });
  assert.throws(() => cipher.encrypt({ ...expected, revision: Number.MAX_SAFE_INTEGER + 1, ...document }), { code: 'INVALID_REVISION' });
  assert.throws(() => cipher.encrypt({ ...expected, objectId: '../../outside', ...document }), { code: 'INVALID_ID' });
});

test('read-only client recovery decrypts without allowing nonce-ledger reset and new writes', () => {
  const { cipher, bytes } = fresh(); const key = cipher.exportRecoveryKey();
  const reader = restoreDocumentReader(namespace, key); key.fill(0);
  assert.deepEqual(reader.decrypt(expected, bytes), document);
  assert.equal(reader.encrypt, undefined);
  assert.equal(reader._encrypt, undefined);
  assert.equal(cipher._encrypt, undefined);
  assert.equal(Object.isFrozen(reader), true);
  assert.throws(() => restoreDocumentReader(namespace, Buffer.alloc(31)), { code: 'INVALID_KEY' });
});
