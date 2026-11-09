# Privacy and Local-first

Local-first describes the intended ownership and trust model, not a ban on internet use. The [privacy contract](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/LOCAL-PRIVACY-CONTRACT.md) requires local core tasks and permits explicitly paired personal NAS storage and explicitly chosen external storage for client-side encrypted opaque objects.

The design separates storage administration from reading content. A NAS administrator can reclaim hosted space and remove hosted copies without receiving users' content keys or permission to erase local copies. External storage providers are untrusted backends, not key holders or silent fallback destinations. See [NAS sharing and internet](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/implementation/NAS-SHARING-AND-INTERNET.md) and [encrypted object storage](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/implementation/ENCRYPTED-OBJECT-STORAGE.md) for the binding boundaries.

Encryption does not guarantee availability, backup or anonymity: size, timing and connection metadata may remain visible. System AI inference is required to remain on the device, without cloud or NAS inference fallback. These are requirements with explicit implementation limits, recorded in [the privacy contract](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/LOCAL-PRIVACY-CONTRACT.md) and [current project state](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/PROJECT-STATE.md).

---

[Home](https://github.com/simonvanieperen/BaryonOS/wiki/Home) · [Canonical project status](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/PROJECT-STATE.md)
