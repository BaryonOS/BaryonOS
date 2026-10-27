import { NasStore } from '../store.mjs';
import { createDocumentCipher } from '../crypto.mjs';

// Local child-process fixture only. Secrets travel over test IPC, never stdout/logs.
const { store, adminCapability } = NasStore.createSynthetic();
const offer = store.createInvitation(adminCapability);
const paired = store.acceptInvitation({ ...offer, confirmedPeerIdentity: offer.peerIdentity });
const session = store.openSession(paired);
const cipher = createDocumentCipher(offer.namespace);
const objectId = 'a'.repeat(32);
const bytes = cipher.encrypt({ objectId, revision: 1, name: 'synthetic', content: 'synthetic' });
store.commitUpload(session, store.stageUpload(session, bytes));
if (process.argv[2] === 'revoke') store.revokeNamespace(adminCapability, offer.namespace);
if (process.argv[2] === 'evict') store.evictNamespace(adminCapability, offer.namespace);
process.send({ directory: store.directory, adminCapability, paired, session, objectId });
setInterval(() => {}, 1_000); // Parent kills this process after the durable-operation acknowledgement.
