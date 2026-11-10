# Start here

**Recommended: one local Git project directory in Codex, testing in QEMU first and using GitHub as a practical remote.** Keep the OS design independent of that hosting so a European forge can become the primary location later.

## 1. Project components

- A **project directory/repository** is the source code plus version history. This is the actual project.
- A **Codex project** opens that directory and gives the agent access to files and development tools.
- A **ChatGPT project** can collect ideas, but is optional and does not replace the repository and build environment.
- A **GitHub Project** is an optional task board; it is not needed for the first boot.

Use one canonical directory. Do not copy the project again for every chat. Start with a clean new directory so existing files are not overwritten.

## 2. Initial setup without terminal experience

1. Extract the ZIP file on the local computer. Open the directory that directly contains `AGENTS.md` and `README.md`.
2. Use GitHub Desktop or Git to make this directory a local repository. Commit the starter package.
3. Publish the repository through GitHub Desktop after checking its contents and licence. Starting privately is possible; a private project is not yet a public open-source project. The repository contains an actual licence text and a defined software licensing scope; see LICENSE, LICENSING.md and THIRD-PARTY.md.
4. Open the same local directory in Codex. Select the local computer/workspace, taking care not to select a different cloud environment accidentally.
5. Start in the root directory. Ask Codex to confirm in its first response which project directory and instruction files it uses.
6. Choose the model below and paste PROMPT-START.md.

GitHub is hosting; Git is version control. Working locally with Codex does not require GitHub hosting. The local agent uses the checkout; no GitHub connector is needed for that. For push/pull, Git/GitHub Desktop must be authenticated through the normal sign-in process. Do not put an access token in a prompt, source file or remote URL.

## 3. Optional Git commands

Run this only **in the newly extracted project directory**, if it is not already a Git repository:

```bash
git init -b main
git status --short
git add .
git diff --cached --stat
git commit -m "Add independent OS research and Codex starter"
```

If Git asks for identity, set your own name and a suitable email address. You may use the noreply address supplied by GitHub; do not invent that address.

Then create an **empty** repository on GitHub; do not add a README there as well. Replace both placeholders in the URL below before running the commands: `JOUW-GEBRUIKERSNAAM` is your username and `JOUW-REPOSITORY` is your repository name.

```bash
git remote add origin https://github.com/JOUW-GEBRUIKERSNAAM/JOUW-REPOSITORY.git
git remote -v
git push -u origin main
```

Use the browser/credential-manager sign-in offered by Git. If `origin` already exists, inspect it first rather than blindly running `remote add` again. Pushing to a public repository makes files public: check contents and licence beforehand.

## 4. GitHub or a European alternative?

| Route | Consideration |
|---|---|
| GitHub + local Codex | Best initial route for a beginner because of documented integrations and practical collaboration. It is a US service. |
| Codeberg + local Codex | More suitable if European hosting is a firm requirement from the start; check terms of use and available CI for the project. |
| Self-hosted Forgejo + local Codex | More control over hosting and administration, with responsibility for forge maintenance, backups, updates and security. Leave this for later. |

The same Git workflow can use another remote. A mirror copies Git history, but not automatically all issues, CI secrets, settings and release processes. Therefore keep build scripts, decisions and release specifications in the repository.

Codex is a cloud service/model workflow, even when files are local. Working locally does not mean all processing stays local. The future OS has a stricter, separate product requirement: AI exclusively on the device. Codex/GitHub serve development only here and must not become a runtime requirement or channel for personal OS data. Check the current privacy/data settings of the account being used; this package establishes no specific retention period or confidentiality.

## 5. Model selection

Status as of 26 October 2026, based on OpenAI's model documentation:

- **GPT-6 Astra, High or Extra High if available:** recommended for architecture, memory isolation, concurrent kernel code and difficult security reviews.
- **GPT-6.1 Sol:** practical working model for bounded implementation and iterative development; increase reasoning effort for complex tasks.
- Smaller models only for simple documentation or triage, not as the sole reviewer of security-critical code.

This is a task-oriented recommendation, not an independent benchmark declaring a model the best OS builder. **The options available in the account being used have not been verified.** Use the strongest available model for high-risk decisions; do not force a model ID that the client does not accept. More reasoning time or more agents does not prove correctness.

## 6. Build environment

Research choices are fixed in docs/implementation/START-CODING.md and BOOT-AND-TOOLCHAIN.md. Have Codex start with `python3 tools/preflight.py` and make the selected toolchain available. Use the chosen versions; record and justify any demonstrably necessary deviation. Do not restart general toolchain research. The script installs nothing and checks presence, not complete usability.

On Windows, a Linux development environment through WSL2 is a reasonable option, but not mandatory. Keep checkout, compiler and build commands within one chosen environment. Running the compiler on Windows or Linux does not mean the new OS derives from either.

QEMU can use software emulation when hardware acceleration is absent. The first build must support that. No real disks, USB installers, BIOS changes or phone flashing for the first milestones.

## 7. Codex Cloud only as a development alternative

This section concerns writing public/suitable project code, not cloud functions or AI in the future OS. The OS contract is in docs/LOCAL-PRIVACY-CONTRACT.md.

The current cloud route uses an environment with selected repositories. In web/desktop: choose the cloud environment, connect GitHub when prompted, select only this repository and have the build environment prepared. Check setup results before publishing the environment. “Publish environment” is not the same as making the source code public.

Explicitly test whether QEMU with software emulation, required packages and timeouts works there. Hardware acceleration or access to real devices is not assumed. If cloud booting fails, use it for review/host tests and run the boot check locally. Report the two separately.

## 8. Conversation and task structure

Start with one main chat and at most three concurrent subagents. Give each follow-up chat one technical or product milestone, clear permitted files and an acceptance test. The product choice has already been researched: Materiaal. The first execution batch builds the boot test, interaction prototype and local quantum simulator in parallel, following docs/implementation/FIRST-BATCH.md. Use separate worktrees for independent implementation chats. Merge code only after tests and review pass on the combined version.

AGENTS.md files are instructions, not an automatically launched team or a security boundary. Optional TOML roles must be supported by the client in use. Have the agent check what is loaded; otherwise use the same roles through explicit task prompts. The main chat remains responsible.

## Sources

OpenAI. (n.d.). *Models*. Retrieved 26 October 2026, from https://learn.chatgpt.com/docs/models

OpenAI. (n.d.). *Run coding tasks in the cloud*. Retrieved 26 October 2026, from https://learn.chatgpt.com/docs/cloud

OpenAI. (n.d.). *Worktrees*. Retrieved 26 October 2026, from https://learn.chatgpt.com/docs/environments/git-worktrees

GitHub. (n.d.). *Creating a new repository*. https://docs.github.com/en/repositories/creating-and-managing-repositories/creating-a-new-repository

GitHub. (n.d.). *Cloning a repository*. https://docs.github.com/en/repositories/creating-and-managing-repositories/cloning-a-repository

For primary sources on Codeberg/Forgejo and open source: see docs/PRODUCT-AND-SOVEREIGNTY.md.
