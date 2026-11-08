# System-service rules

Read the root instructions and security model. Every service requires explicit responsibilities, inputs, authority, quotas, and recovery behaviour.

Version IPC schemas and always validate boundaries and provenance. No implicit global access. Examine confused-deputy risks and delegation.

Apply revision 4 of `docs/LOCAL-PRIVACY-CONTRACT.md` and `docs/implementation/ENCRYPTED-OBJECT-STORAGE.md`. Personal NAS synchronisation uses an explicitly paired NAS owned by the user or friends/family. LAN is preferred; internet access requires explicit peer/destination permissions and actual transport authentication.

Users may also explicitly select an external storage provider as an untrusted backend for client-side encrypted opaque ciphertext objects only. Providers receive no plaintext, sensitive manifest content, or Baryon OS content keys and are never a silent fallback or mandatory OS relay.

User keys remain within bounded client/Key Service boundaries. NAS administrators may delete their own stored ciphertext and manage quotas without guest consent but have no right to read content. Revocation also blocks old sessions, upload commits, and resumed synchronisation.

The **Security Floor** remains in force when privacy choices are expanded: internet or document access never grants unrelated system, key, or other-app authority. The AI inference service has no network permission. The model/update downloader receives no personal data or general NAS credentials. Network authority is limited per service and destination, not by a global OS ban; only previously granted permissions allow network access. Logs are local and exclude secrets and personal content by default.

Use maintained cryptographic libraries; do not invent primitives or protocols. Entropy, key lifecycle, rollback protection, and failure modes belong in the integration design.

Test that a crash remains contained and recovery does not silently broaden privileges.
