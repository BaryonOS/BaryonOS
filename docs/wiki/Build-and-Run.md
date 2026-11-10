# Build and Run

Start with the [canonical development guide](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/DEVELOPMENT.md). It contains the supported host routes, pinned tools, firmware pairing, build commands and VM test profiles. Keeping commands there avoids a second set of instructions drifting out of date.

1. Read [current project state](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/PROJECT-STATE.md) to choose the intended experiment; historical milestone examples are not necessarily the latest session path.
2. Follow the matching host section in [DEVELOPMENT](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/DEVELOPMENT.md) to obtain the tools and run its preflight checks. A preflight checks prerequisites, not OS correctness.
3. Build and run the selected profile using that guide. Use isolated disposable VM images, never a host disk; this boundary is defined in [the project instructions](https://github.com/simonvanieperen/BaryonOS/blob/main/AGENTS.md).
4. Compare the real output with the profile's documented gate and evidence requirements. Use [Testing and Evidence](https://github.com/simonvanieperen/BaryonOS/wiki/Testing-and-Evidence) to interpret the result.

Missing tools or firmware are setup problems, not passing test results. This path is for experiments in QEMU; physical installation and supported hardware must be established separately under [the release requirements](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/FULL-TEST-RELEASE.md).

---

[Home](https://github.com/simonvanieperen/BaryonOS/wiki/Home) · [Canonical project status](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/PROJECT-STATE.md)
