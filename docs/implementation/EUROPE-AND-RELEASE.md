# European requirements and publication

Researched: **26 October 2026**. Status: substantiated design and release criteria; no implementation, certificate or legal declaration of conformity. These agreements allow the first QEMU boot to proceed. Reassess legal applicability once audience, distribution, legal entity and business model become concrete.

There is no single overarching “European OS standard” that signs off privacy, cybersecurity, accessibility and interoperability together. Treat these as separate requirements with their own evidence. The local privacy boundary is also stricter than a general requirement to process personal data lawfully: an explicitly selected external storage provider may serve built-in OS services only as an untrusted backend for opaque, client-side encrypted objects.

## Applicability: which situation changes the obligations?

| Subject | Conditional applicability | Consequence for this project |
|---|---|---|
| GDPR | Roles follow actual processing: whoever determines purposes and essential means is the controller; whoever processes on that party's behalf may be a processor. Supplying software alone does not automatically make someone a processor. Lacking data access does not automatically exclude responsibility. [1–2] | Record who decides and processes for every data flow. OS design, possible support data and repository accounts are different activities. “Everything local” is not a general GDPR exemption. |
| Personal use | The household exemption concerns solely personal/household processing without a professional or commercial connection. [1] | Do not extend that exemption to the OS provider, an employer, school or business NAS user. |
| Cyber Resilience Act: manufacturer | FOSS developed/provided outside commercial activity falls outside ordinary product obligations. Open source or a free download alone does not demonstrate that exemption: the actual distribution and business model matter. [3] | Before a distribution release, record who places what on the EU market, under which name and with what commercial relationship. OS products may fall under the important-products regime; determine the concrete classification and conformity route in that case. |
| CRA: open-source steward | A legal entity that sustainably and systematically supports specific FOSS intended for commercial activities and ensures its viability may be a steward. A separate regime applies, including security policy and vulnerability cooperation. [3] | A future foundation is neither automatically exempt nor automatically a manufacturer. Document its actual role. |
| Accessibility | The European Accessibility Act includes general-purpose consumer computers and their operating systems placed on the market after **28 June 2025**. National transposition, product, role and possible exemptions determine concrete obligations. [6–7] | Include accessibility from the first interaction prototype. An internal boot experiment is not evidence of an accessible consumer product. |
| Standards and procurement | EN 301 549 includes ICT requirements, including software and interoperability with assistive technology. WCAG alone does not cover a complete native OS. Harmonisation applies to a particular version and legal basis; the Web Accessibility Directive concerns public-sector websites/apps. [8–9] | Use EN 301 549 V3.2.1 chapter 11 as a documented design baseline; WCAG 2.2 AA as an additional objective for web prototypes. At release, check the designated standard versions and specific procurement requirements then applicable. Open formats are a product choice, not a general certificate. |

**CRA dates:** in force since 10 December 2024; the main product obligations apply from **11 December 2027**. Manufacturers have been required to report actively exploited vulnerabilities and serious security incidents since **11 September 2026**. Stewards' reporting duty begins on **11 December 2027**. [4–5] An applicable manufacturer must be prepared for 24-hour and 72-hour reports, among other requirements; this is not an automatic OS telemetry service.

## Technical acceptance criteria

These are **project-specific requirements**, not a list of legally prescribed implementations quoted verbatim. Build them as each relevant subsystem emerges; claim them only after testing.

| ID | To build and demonstrably verify |
|---|---|
| EU-01 | Data-flow register with dataset, purpose, location, retention period, authority and deletion path. Device storage, end-to-end encrypted storage on an explicitly paired personal NAS or friends'/family's NAS, or opaque client-side encrypted objects at an explicitly selected external storage provider. NAS internet access requires peer-bound consent and authenticated transport. The NAS administrator may revoke/delete storage but receives no content key; the external provider receives no content key or plaintext. No mandatory vendor account; a selected provider may require its own account. No silent relay/fallback, key escrow or external OS telemetry. Execute all applicable LP tests from LOCAL-PRIVACY-CONTRACT.md revision 4, including LP-26–LP-31, once the relevant features exist; this is not a legal conformity claim. |
| EU-02 | AI inference without network authority; model downloader without personal context. Missing model, memory error and LAN loss never activate cloud fallback. Test helper processes and local brokers too. |
| EU-03 | App installation grants no general authority. Test files, NAS datasets, LAN, internet, camera and microphone separately; test revocation and bypasses through intermediary services. Make combined read and internet rights visible. |
| EU-04 | Export documents, settings and selected AI history in documented formats without an account. Also delete indices, caches and embeddings. Separately show what remains in NAS recovery points; do not claim to retrieve and erase external copies already made. |
| EU-05 | Test onboarding, login, permissions, installation and recovery fully with a keyboard; design visible focus, scalable text, contrast, reduced motion and a semantic accessibility tree for screen readers. Retain scenarios and results per build. |
| EU-06 | Generate a machine-readable SBOM per release with direct/transitive components, versions, hashes and licences; pin build tools. Document reproducibility, source provenance and exceptions. Check firmware, fonts and models too. |
| EU-07 | Signed update chain, key rotation/revocation, protection against unauthorised downgrade and recovery after interrupted updates. Test incorrect signature, old package, corruption and power failure. Maintain an offline update path. |
| EU-08 | On actual distribution, publish supported versions, support period, security contact and disclosure process. Assign triage, correction, advisories and conditional statutory reporting to responsible people. An SBOM and issue tracker do not prove that all vulnerabilities are known. |

Use synthetic data in CI and Codex. Release evidence includes version, configuration, tests performed and remaining limitations. This matrix is not exhaustive for later sectors such as healthcare, vehicles or a specific AI application.

<a id="confirmed-direction-and-licence-review"></a>
<a id="bevestigde-richting-en-licentiecontrole"></a>

## Confirmed direction and licence

**Decision of 8 November 2026:** public development is the confirmed project direction.

**Application on 9 November 2026:** MPL-2.0 is applied to the project-owned software explicitly included in [MPL-2.0-FILES.txt](../../MPL-2.0-FILES.txt). The official text is in [LICENSE](../../LICENSE); [LICENSING.md](../../LICENSING.md) describes the scope, and [THIRD-PARTY.md](../../THIRD-PARTY.md) refers to third-party terms.

Documentation, historical evidence files, third-party assets and project marks are not placed under MPL-2.0 solely by the root licence. An applied source licence is not release approval, a complete SBOM, a privacy guarantee or a product safety claim.

**MPL-2.0** uses file-level copyleft: distributed modifications to covered files remain subject to the relevant MPL provisions, while separate, uncovered files may have other compatible terms. On distribution, the official licence terms and notices of the components actually included remain authoritative.

## Primary sources

Accessed on 26 October 2026; links refer to official texts/explanations. The project criteria above are independent derivations.

1. European Commission. (n.d.). *Data protection explained*. https://commission.europa.eu/law/law-topic/data-protection/data-protection-explained_en
2. European Data Protection Board. (2022). *Guidelines 07/2020 on the concepts of controller and processor in the GDPR* (version 2.1). https://www.edpb.europa.eu/system/files/2023-10/EDPB_guidelines_202007_controllerprocessor_final_en.pdf
3. European Commission. (2026, October 15). *Cyber Resilience Act — Open source*. https://digital-strategy.ec.europa.eu/en/policies/cra-open-source
4. European Commission. (2026, September 7). *Cyber Resilience Act*. https://digital-strategy.ec.europa.eu/en/policies/cyber-resilience-act
5. European Commission. (2026, September 11). *Cyber Resilience Act — Reporting obligations*. https://digital-strategy.ec.europa.eu/en/policies/cra-reporting
6. European Commission. (n.d.). *European Accessibility Act (EAA)*. https://commission.europa.eu/strategy-and-policy/policies/justice-and-fundamental-rights/disability/european-accessibility-act-eaa_en
7. European Parliament & Council. (2019). *Directive (EU) 2019/882*, article 2. https://eur-lex.europa.eu/eli/dir/2019/882/oj
8. ETSI. (2021). *EN 301 549 V3.2.1: Accessibility requirements for ICT products and services*. https://www.etsi.org/deliver/etsi_en/301500_301599/301549/03.02.01_60/en_301549v030201p.pdf
9. European Commission. (2025, May 5). *Web Accessibility Directive — Standards and harmonisation*. https://digital-strategy.ec.europa.eu/en/policies/web-accessibility-directive-standards-and-harmonisation
10. Mozilla. (n.d.). *Mozilla Public License, version 2.0*, §§ 1.10, 3.1–3.4. https://www.mozilla.org/en-US/MPL/2.0/

