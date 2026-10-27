import { createCipheriv, createDecipheriv, createSecretKey, randomBytes } from 'node:crypto';
import { fail, requireId, requireRevision, requireKeys, decode64 } from './errors.mjs';

export const MAX_NAME_BYTES = 512;
export const MAX_CONTENT_BYTES = 16_384;
export const MAX_ENVELOPE_BYTES = 140_000;
const MAX_PLAINTEXT_BYTES = 104_000;
const MAX_ENCRYPTIONS_PER_KEY = 8_192;
const ENVELOPE_KEYS = ['v', 'algorithm', 'namespace', 'objectId', 'revision', 'nonce', 'tag', 'ciphertext'];
const keys = new WeakMap();

function binding(value) {
  requireId(value.namespace, true); requireId(value.objectId); requireRevision(value.revision);
  return Buffer.from(JSON.stringify(['nieuw-os/nas-object', 1, 'aes-256-gcm',
    value.namespace, value.objectId, value.revision]), 'utf8');
}
function validateDocument(value) {
  requireKeys(value, ['name', 'content']);
  if (typeof value.name !== 'string' || Buffer.byteLength(value.name) > MAX_NAME_BYTES ||
      typeof value.content !== 'string' || Buffer.byteLength(value.content) > MAX_CONTENT_BYTES) fail('DOCUMENT_TOO_LARGE');
}

// Syntax validation only. The NAS cannot authenticate ciphertext without the client key.
export function parseEnvelope(bytes) {
  if (!Buffer.isBuffer(bytes) || bytes.length < 1 || bytes.length > MAX_ENVELOPE_BYTES) fail('INVALID_ENVELOPE');
  let value;
  try { value = JSON.parse(bytes.toString('utf8')); } catch { fail('INVALID_ENVELOPE'); }
  requireKeys(value, ENVELOPE_KEYS);
  if (value.v !== 1 || value.algorithm !== 'aes-256-gcm') fail('INVALID_ENVELOPE');
  binding(value);
  decode64(value.nonce, 12, 12); decode64(value.tag, 16, 16);
  decode64(value.ciphertext, MAX_PLAINTEXT_BYTES);
  return value;
}

class DocumentReader {
  #namespace;
  constructor(namespace, key) {
    this.#namespace = requireId(namespace, true);
    if (!Buffer.isBuffer(key) || key.length !== 32) fail('INVALID_KEY');
    keys.set(this, createSecretKey(key));
  }
  get namespace() { return this.#namespace; }
  // Explicit caller-owned recovery export; never pass it to the NAS or a logger.
  exportRecoveryKey() { return keys.get(this).export(); }
  decrypt(expected, bytes) {
    try {
      requireId(expected.objectId); requireRevision(expected.revision);
      const envelope = parseEnvelope(bytes);
      if (envelope.namespace !== this.#namespace || envelope.objectId !== expected.objectId ||
          envelope.revision !== expected.revision) fail('AUTHENTICATION_FAILED');
      const decipher = createDecipheriv('aes-256-gcm', keys.get(this), decode64(envelope.nonce, 12, 12), { authTagLength: 16 });
      decipher.setAAD(binding(envelope));
      decipher.setAuthTag(decode64(envelope.tag, 16, 16));
      // No partial plaintext is exposed before final() verifies the authentication tag.
      const plaintext = Buffer.concat([decipher.update(decode64(envelope.ciphertext, MAX_PLAINTEXT_BYTES)), decipher.final()]);
      try {
        const document = JSON.parse(plaintext.toString('utf8'));
        validateDocument(document);
        return document;
      } finally { plaintext.fill(0); }
    } catch { fail('AUTHENTICATION_FAILED'); }
  }
}

class DocumentCipher extends DocumentReader {
  #usedNonces = new Set();
  constructor(namespace) {
    const key = randomBytes(32);
    super(namespace, key);
    key.fill(0);
  }
  encrypt({ objectId, revision, name, content }) {
    const document = { name, content }; validateDocument(document);
    const header = { v: 1, algorithm: 'aes-256-gcm', namespace: this.namespace, objectId, revision };
    const associatedData = binding(header);
    if (this.#usedNonces.size >= MAX_ENCRYPTIONS_PER_KEY) fail('KEY_ROTATION_REQUIRED');
    let nonce;
    for (let attempt = 0; attempt < 8; attempt++) {
      const candidate = randomBytes(12);
      if (!this.#usedNonces.has(candidate.toString('hex'))) { nonce = candidate; break; }
    }
    if (!nonce) fail('ENTROPY_FAILURE');
    // Burn the nonce even if a later operation fails. No restored key can encrypt.
    this.#usedNonces.add(nonce.toString('hex'));
    const plaintext = Buffer.from(JSON.stringify(document), 'utf8');
    try {
      const cipher = createCipheriv('aes-256-gcm', keys.get(this), nonce, { authTagLength: 16 });
      cipher.setAAD(associatedData);
      const result = { ciphertext: Buffer.concat([cipher.update(plaintext), cipher.final()]), tag: cipher.getAuthTag() };
      const bytes = Buffer.from(JSON.stringify({ ...header, nonce: nonce.toString('base64'),
        tag: result.tag.toString('base64'), ciphertext: result.ciphertext.toString('base64') }));
      if (bytes.length > MAX_ENVELOPE_BYTES) fail('DOCUMENT_TOO_LARGE');
      return bytes;
    } finally { plaintext.fill(0); }
  }
}

export const createDocumentCipher = (namespace) => new DocumentCipher(namespace);
// Recovery is read-only: no reset of a nonce ledger while reusing an old AES key.
export const restoreDocumentReader = (namespace, key) => {
  const reader = new DocumentReader(namespace, key);
  return Object.freeze({ namespace: reader.namespace, decrypt: reader.decrypt.bind(reader) });
};
