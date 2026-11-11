# N02 — real hybrid TLS probe on loopback

`transport.mjs` adds an executable, bounded network probe alongside the N01 storage modules. The OS's own kernel does not yet have a network stack as a result. The goal is to test the transition from synthetic transport facts to an actually negotiated, authenticated TLS connection with the existing rights model.

## Running

From the project root, with the tested Node 24.22.1/OpenSSL 3.5.9 runtime and the `openssl` CLI for temporary classical test certificates:

```sh
node services/nas/transport-demo.mjs
node --test --test-isolation=none services/nas/tests/transport.test.mjs
node --test --test-isolation=none services/nas/tests/*.test.mjs ui/prototype/tests/*.test.mjs
```

The demo and transport tests open only `127.0.0.1` on automatically assigned temporary ports. A sandbox may require explicit loopback execution permission for this. An EPERM run is not an executed transport test. The original N01 demo continues to work without sockets.

Test identities are generated per run in a dedicated temp directory with directory mode 0700. The fixture removes this directory after reading; the CA private key is not passed to the adapter. `close()` overwrites the managed Buffer copies. This is no secure-erase guarantee or evidence that all copies in OpenSSL/memory disappear. No certificates, private keys, session tickets or document content in Git/logs.

## Fixed cryptographic probe profile

| Property | Enforced value |
|---|---|
| Listen/connect address | only `127.0.0.1`; no external host argument |
| Protocol | TLS 1.3 as both minimum and maximum |
| Key exchange | only `X25519MLKEM768`, RFC 10024 |
| TLS record cipher | `TLS_AES_256_GCM_SHA384` |
| Application protocol | ALPN `nieuw-os-nas-host/2` |
| Authentication | mutual CA validation, exact SHA-256 certificate pin; server hostname check |
| Certificate keys | classical Ed25519; no PQ authentication claim |
| Resumption | no session cache/resumption; full handshake, no 0-RTT |
| Application content | bounded N01 ciphertext envelope; only the client decrypts |

After the handshake the client checks the actually reported group, TLS version, cipher, ALPN, peer and `isSessionReused`. If the runtime cannot report a matching group, the connection fails. The server API does not report this group via `getEphemeralKeyInfo`; server evidence therefore distinguishes between the configured group and the negotiation observed by the client.

No home-made hybrid KDF, extra PSK combiner or custom TLS record encryption. Node/OpenSSL executes the standardized protocol. A wrapper around TLS handles only project policy, framing, limits and error handling. The current standard and the rationale are in [the security blueprint](../../docs/implementation/VERIFIABLE-SECURITY-AND-PQC.md).

## API and authorities

`createLoopbackNasServer({credentials, allowedClientPin, namespace})` returns a port number, minimal `evidence()` and `close()`. The endpoint checks mTLS, the client pin and the envelope namespace. It echoes ciphertext from RAM. This is not a NAS storage service, not a durable ACK and not a new administration interface.

`connectLoopbackNas({port, credentials, expectedPeerPin, serverName, engine, app, networkScope, localNetworkId})` returns only `sendCiphertext(bytes)`, `evidence()` and `close()`. The socket and opaque transport fact remain internal. The trusted host adapter holds `engine.authority`; an untrusted app must not receive it.

Before socket creation the adapter checks `authorizeNasConnection`. This preflight confirms only permission to attempt, not peer identity. Only after real TLS validation is a transport fact created, bound to the app, the current peer pairing and, for internet, the revocation generation of that right. Before every ciphertext send and before delivering the reply, `engine.check` is executed again. An error or closed connection revokes the fact. After revoking and re-permitting, a previously authenticated fact remains invalid; a new connection is required. Internet revocation leaves the permitted LAN route in place. Bytes already sent cannot be recalled. An idle socket is not actively closed by a policy notification and lives at most for the fixed connection lifetime. This is not a native process or socket capability boundary.

`networkScope` and `localNetworkId` are explicit harness assumptions. Even a test labelled `internet` physically runs over loopback. The tests prove the peer-bound permission path, not WAN reachability, interface identification, firewall or NAT traversal. Trusted network observation remains a native integration task.

Pins here are hashes of the full certificate. Certificate renewal therefore changes the pin; this fixture choice is not a finished consumer design for identity/rotation. That product flow must be able to run securely and automatically within pre-granted authorities.

## Limits and recovery

At most four simultaneous TCP connections, also before TLS authentication; eight frames per connection; at most 140.000 bytes per frame. A frame has a four-byte length header. The parser reserves one bounded payload buffer only after the length check. The frame deadline starts at the first byte and is not extended by slowly trickling input.

The probe limits are 1.500 ms for the handshake, 750 ms for a frame/reply and 5.000 ms for the full connection. These are deterministic loopback probe values, not internet performance requirements or consumer limits. The endpoint stops on bad framing, invalid namespace, quota overrun or deadline. No retry with a weaker profile, other peer or vendor relay.

## Evidence and open work

The tests use real TLS clients, including clients that bypass the normal adapter to check the server boundary. Among other things they check real downgrade failures, wrong identities/certificates, missing permission before socket creation, revocation, old facts, framing, quotas and slow input. An offered TLS 1.3 session ticket must again lead to a full hybrid handshake. Test counts, source hashes, commands and review are in [verification/pqc-transport](../../docs/verification/pqc-transport/).

Still open: native network/crypto port, production identity/certificate management, post-quantum authentication, real multi-device pairing, usable persistent client keys, background sync, store-transport integration, OS isolation and an independent human audit. The existing admin revoke/evict behaviour is tested by N01; the ciphertext echo is not connected to it as a server. M1–M6a, qsim, the boot chain and physical equipment are not modified by this probe.
