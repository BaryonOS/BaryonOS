// Disposable classical-authentication fixture. Never prints/commits private keys.
import { mkdtemp, readFile, writeFile, chmod, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { certificatePin } from '../transport.mjs';

const execute = promisify(execFile);
export async function createTlsFixture() {
  const directory = await mkdtemp(join(tmpdir(), 'nieuw-os-n02-tls-'));
  await chmod(directory, 0o700);
  const buffers = [];
  const command = async (args) => {
    try { await execute('openssl', args, { cwd: directory, timeout: 10_000, maxBuffer: 16_384 }); }
    catch { throw new Error('TLS_FIXTURE_GENERATION_FAILED'); }
  };
  try {
    await command(['genpkey', '-algorithm', 'ED25519', '-out', 'ca.key']);
    await command(['req', '-new', '-x509', '-key', 'ca.key', '-out', 'ca.crt', '-subj', '/CN=Nieuw OS disposable CA', '-days', '1',
      '-addext', 'basicConstraints=critical,CA:TRUE', '-addext', 'keyUsage=critical,keyCertSign,cRLSign']);
    const ca = await readFile(join(directory, 'ca.crt')); buffers.push(ca);
    async function issue(name, role, serial) {
      await command(['genpkey', '-algorithm', 'ED25519', '-out', `${name}.key`]);
      await command(['req', '-new', '-key', `${name}.key`, '-out', `${name}.csr`, '-subj', `/CN=${name}`]);
      const extensions = ['basicConstraints=critical,CA:FALSE', 'keyUsage=critical,digitalSignature',
        `extendedKeyUsage=${role}`, ...(role === 'serverAuth' ? ['subjectAltName=DNS:nas.loopback.invalid'] : [])].join('\n');
      await writeFile(join(directory, `${name}.ext`), `${extensions}\n`, { mode: 0o600 });
      await command(['x509', '-req', '-in', `${name}.csr`, '-CA', 'ca.crt', '-CAkey', 'ca.key',
        '-set_serial', String(serial), '-days', '1', '-extfile', `${name}.ext`, '-out', `${name}.crt`]);
      const key = await readFile(join(directory, `${name}.key`));
      const cert = await readFile(join(directory, `${name}.crt`)); buffers.push(key, cert);
      return { key, cert, ca };
    }
    const server = await issue('nas', 'serverAuth', 1);
    const client = await issue('client', 'clientAuth', 2);
    const stranger = await issue('stranger', 'clientAuth', 3);
    const otherServer = await issue('other-nas', 'serverAuth', 4);
    // Remove temporary key files after fixture creation. This is not evidence
    // of physical secure erasure; only synthetic, disposable keys are used.
    await rm(directory, { recursive: true, force: true });
    let closed = false;
    return Object.freeze({ server, client, stranger, otherServer, serverName: 'nas.loopback.invalid',
      serverPin: certificatePin(server.cert), clientPin: certificatePin(client.cert),
      otherServerPin: certificatePin(otherServer.cert),
      async close() { if (closed) return; closed = true; for (const buffer of buffers) buffer.fill(0);
        await rm(directory, { recursive: true, force: true }); },
    });
  } catch (error) {
    for (const buffer of buffers) buffer.fill(0);
    await rm(directory, { recursive: true, force: true }); throw error;
  }
}
