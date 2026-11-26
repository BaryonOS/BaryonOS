// N03-T2 H2 test peer (HOST EVIDENCE ONLY): one TLS 1.3 connection through
// Node.js's bundled OpenSSL, driven by tests/loopback.rs. TEST-ONLY credentials
// arrive as DER files written by the Rust test; nothing is persisted.
//
// argv: role(server|client) certDer keyDer peerCertDer group alpn servername [port]
// stdout: one JSON line per event ("ready", "secure", "data", "error").
import { readFileSync } from "node:fs";
import tls from "node:tls";

const [role, certPath, keyPath, peerPath, group, alpn, servername, port] = process.argv.slice(2);
const pem = (label, der) =>
  `-----BEGIN ${label}-----\n${der.toString("base64").match(/.{1,64}/g).join("\n")}\n-----END ${label}-----\n`;
const peerDer = readFileSync(peerPath);
const options = {
  cert: pem("CERTIFICATE", readFileSync(certPath)),
  key: pem("PRIVATE KEY", readFileSync(keyPath)),
  ca: [pem("CERTIFICATE", peerDer)],
  minVersion: "TLSv1.3",
  maxVersion: "TLSv1.3",
  ciphersuites: "TLS_AES_256_GCM_SHA384",
  ecdhCurve: group,
  ALPNProtocols: [alpn],
};
const emit = (event) => process.stdout.write(`${JSON.stringify(event)}\n`);
const fail = (error) => {
  emit({ event: "error", code: error.code ?? null, message: String(error.message) });
  process.exitCode = 3;
};
// Exact-certificate pin on top of chain validation to the pinned self-signed leaf.
const secure = (socket) => {
  const peer = socket.getPeerCertificate(false);
  const pinned = Buffer.isBuffer(peer.raw) && peer.raw.equals(peerDer);
  emit({
    event: "secure",
    openssl: process.versions.openssl,
    protocol: socket.getProtocol(),
    cipher: socket.getCipher().standardName,
    alpn: socket.alpnProtocol,
    authorized: socket.authorized,
    pinned,
  });
  if (!socket.authorized || !pinned) {
    socket.destroy();
    process.exitCode = 4;
    return false;
  }
  return true;
};

if (role === "server") {
  const server = tls.createServer({ ...options, requestCert: true, rejectUnauthorized: true }, (socket) => {
    if (!secure(socket)) return server.close();
    socket.once("data", (data) => {
      emit({ event: "data", text: data.toString() });
      socket.end(`openssl:${data}`);
      server.close();
    });
  });
  server.on("tlsClientError", (error) => {
    fail(error);
    server.close();
  });
  server.listen(0, "127.0.0.1", () => emit({ event: "ready", openssl: process.versions.openssl, port: server.address().port }));
} else if (role === "client") {
  const socket = tls.connect({ ...options, host: "127.0.0.1", port: Number(port), servername, rejectUnauthorized: true }, () => {
    if (!secure(socket)) return;
    socket.write("openssl client");
  });
  socket.once("data", (data) => {
    emit({ event: "data", text: data.toString() });
    socket.end();
  });
  socket.on("error", fail);
} else {
  fail(new Error(`unknown role ${role}`));
}
