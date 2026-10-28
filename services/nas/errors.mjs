export class NasError extends Error {
  constructor(code) { super(code); this.name = 'NasError'; this.code = code; }
}
export const fail = (code) => { throw new NasError(code); };
export const ID = /^[a-f0-9]{32}$/;
export const NAMESPACE = /^[a-f0-9]{64}$/;
export const CAPABILITY = /^[A-Za-z0-9_-]{43}$/;
export function requireId(value, namespace = false) {
  if (typeof value !== 'string' || !(namespace ? NAMESPACE : ID).test(value)) fail('INVALID_ID');
  return value;
}
export function requireRevision(value) {
  if (!Number.isSafeInteger(value) || value < 1) fail('INVALID_REVISION');
  return value;
}
export function requireKeys(value, keys) {
  if (!value || typeof value !== 'object' || Array.isArray(value) ||
      Object.keys(value).sort().join(',') !== [...keys].sort().join(',')) fail('INVALID_FORMAT');
}
export function decode64(value, maximum, exact = undefined) {
  if (typeof value !== 'string' || value.length > Math.ceil(maximum / 3) * 4) fail('INVALID_FORMAT');
  const bytes = Buffer.from(value, 'base64');
  if (bytes.length > maximum || (exact !== undefined && bytes.length !== exact) ||
      bytes.toString('base64') !== value) fail('INVALID_FORMAT');
  return bytes;
}
