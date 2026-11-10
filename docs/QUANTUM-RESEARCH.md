# Quantum as an explicit research and architecture track

> Revision 5 update: initial implementation choices are fixed in [implementation/QUANTUM-SPEC.md](implementation/QUANTUM-SPEC.md). Use that document for direct implementation; this file preserves the ambition and background for subsequent phases.
Revision 4, 26 October 2026. The original ambition encompasses both quantum resilience and the ability to use quantum computers. This track starts during design and is not relegated to an optional future mention. It has not yet been implemented.

## Three separate goals
1. **Protection against quantum attacks.** Design the crypto inventory, algorithm replacement, key rotation, update verification and communication for post-quantum cryptography. This runs on ordinary hardware. The existing security note remains authoritative for protocols, entropy and implementation checks.
2. **Executing quantum programs.** Design a local, isolated execution service through which apps can submit a quantum task, initially to a local simulator and later to suitable personally controlled quantum hardware.
3. **An independent OS on a hybrid quantum computer.** Investigate how the project's own kernel and services form the classical control layer of a system with classical processors and QPUs, including scheduling, device management, user permissions and development tools.

IBM describes current systems with classical runtime servers and quantum processors. QIR investigates interoperable representations of hybrid programs; OpenQASM describes classical and quantum constructs. These are reference points, not imposed limits on the eventual product (IBM, n.d.; QIR Alliance, n.d.; OpenQASM Contributors, n.d.).

## Research phases
| Stage | Research/result | Acceptance evidence |
|---|---|---|
| Q0 — from M0 | Quantum threats, cryptographic chain, execution models and interface choices in one concise ADR | Distinguish facts, hypotheses and experiments; no implicit cloud dependency |
| Q1 — local prototype phase | Small classical simulator with a bounded quantum-job interface | Reference circuits, output distributions within predefined tolerances, cancellation, quotas and invalid input tested |
| Q2 — local QPU contract | Backend interface for capabilities, jobs, results, errors and resource management | Simulator/test backend passes the same contract tests; no hardware compatibility claim yet |
| Q3 — hybrid system port | Independent OS on the classical host/controller with one specific local QPU platform | Documented physical setup, drivers, timing/controller boundaries and an actually executed quantum task |
| Q4 — further research | Alternative execution models, hybrid scheduling and potentially quantum functions within system services | Bounded experiment with comparison; no general promise of acceleration or feasibility |

Q0 is part of the initial assignment alongside technical boot and product concepts. Q1 may initially run on the development host to investigate an interface; label it as a host prototype. Porting the same runtime to the independent OS is a separate result. Q2 can be designed and tested without a QPU; Q3 requires actual access to suitable personally controlled hardware.

## Concrete design questions
- Which task representation is appropriate: a bounded OpenQASM subset, a QIR profile or another verifiable format? Investigate and pin a version; do not introduce an unnecessary new language for the first experiment.
- How are capabilities, qubit requirements, circuits, measurement repetitions, cancellation and results described?
- Which tasks are classical, which are quantum, and how do data and authority move between them?
- Which timing-critical functions belong in specialised controller electronics? An ordinary OS scheduler receives no unproven real-time guarantee.
- What reset/initialisation guarantee exists between users and jobs? Investigate isolation and potentially limited exclusive allocation; do not assume safe sharing.
- What measurable benefits exist for a concrete workload? Simulation results and theoretical quantum advantages do not prove a faster desktop OS.

## Security and privacy retain the same contract
Only personally controlled, locally managed hardware, connected directly or within the explicitly permitted personal LAN; no external quantum cloud or relay. The NAS contract concerns storage, not general authority to outsource computation. AI inference remains on the device itself. Any quantum-AI workload must meet that same local-device contract and receives no implicit exception.

The quantum service receives only the necessary job data, device access and bounded resources. No access to all documents, NAS storage or the internet. Handle parser errors and hardware outages. Untrusted drivers/firmware remain part of the trust boundary under investigation.

## Evidence status and research scope
A cryptographic post-quantum algorithm does not automatically secure the OS kernel. A simulator executing a circuit does not prove physical QPU support or quantum acceleration. An OS on the classical host of a quantum computer is a genuine architectural objective; do not describe it as all kernel code running on qubits.

Investigate more advanced forms explicitly when a concrete execution model exists. Do not reject ideas solely because existing operating systems lack them; do not claim them on the basis of a name or diagram either. The device can remain fully usable without a QPU; the quantum layer is an optional system module.

## Sources (APA 7)
IBM. (n.d.). *IBM Quantum hardware and roadmap*. Retrieved October 26, 2026, from https://www.ibm.com/quantum/hardware

National Institute of Standards and Technology. (2024a). *Module-lattice-based key-encapsulation mechanism standard* (FIPS 203). https://doi.org/10.6028/NIST.FIPS.203

National Institute of Standards and Technology. (2024b). *Module-lattice-based digital signature standard* (FIPS 204). https://doi.org/10.6028/NIST.FIPS.204

OpenQASM Contributors. (n.d.). *OpenQASM live specification*. Retrieved October 26, 2026, from https://openqasm.com/

QIR Alliance. (n.d.). *QIR Alliance*. Retrieved October 26, 2026, from https://www.qir-alliance.org/

This roadmap is an independent design proposal; the sources do not prove that this future OS already supports hardware.
