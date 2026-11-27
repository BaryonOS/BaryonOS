# Accessibility

## Commitment and scope

Accessibility is a design requirement for Baryon OS, not a separate theme or an
optional mode. The project aims to make its software, documentation, and
community processes usable by people with diverse visual, auditory, motor,
cognitive, and sensory needs.

This statement covers the Baryon OS source repository, project documentation,
browser prototypes, and experimental native operating-system work. GitHub's own
website interface is operated by GitHub and is outside the project's technical
control.

## Current conformance status

[WCAG 2.2](https://www.w3.org/TR/WCAG22/) Level AA informs web and
documentation work where it applies. Neither the complete repository experience
nor the native operating system has undergone a complete formal WCAG 2.2 or EN
301 549 assessment. No conformance claim is made.

Baryon OS is experimental research and development software. It has no
supported production release or supported physical-hardware accessibility
profile. The authoritative implementation status remains
[U04 — Native accessibility](docs/PROGRESS.md#u04--native-accessibility), which
is open.

Current evidence establishes only bounded parts of the intended experience:

- selected native QEMU flows provide keyboard operation, focus information,
  and local semantic roles and actions;
- selected native layouts have automated evidence at 100% and 200% scaling;
- an application-local semantic tree exists as preparation for a future
  system-wide accessibility service; and
- appearance and reduced-motion behaviour have bounded host or native checks.

This evidence does not establish complete accessibility. In particular:

- no working OS-wide screen-reader, text-to-speech, or braille chain has been
  demonstrated;
- no representative human accessibility task study has been accepted;
- keyboard, contrast, motion, scaling, and semantic coverage remains incomplete
  across the required native task set;
- the compact error status has a known clipping defect at 640x480 with 200%
  scaling; and
- the History dialog has a known semantic reading-order defect.

Browser prototypes, host tests, native QEMU runs, physical-hardware tests, and
human evaluation are reported separately because they provide different levels
of evidence.

## Supported and tested environments

Repository documentation is maintained as Markdown and is primarily reviewed
through GitHub. No project-specific browser and assistive-technology
compatibility matrix has yet been completed.

Native evidence is currently limited to the documented QEMU reference profiles
and the exact source revisions linked from the progress register. It must not be
generalised to other virtual machines, physical hardware, or assistive
technologies. Where a barrier prevents access to project information, an
alternative text format or assistance can be requested through the feedback
routes below.

## Reporting an accessibility barrier

Accessibility feedback is welcome even when a complete fix is not yet known.
Use the route that matches the report:

- Open a reproducible barrier through the
  [issue forms](https://github.com/BaryonOS/BaryonOS/issues/new/choose) and
  select the desktop, Materiaal, or accessibility component.
- Use the
  [Product and user experience Discussion](https://github.com/BaryonOS/BaryonOS/discussions/categories/product-and-user-experience)
  for design observations, needs, and proposed improvements.
- Email `github@baryonos.org` if the barrier prevents use of the public forms,
  requires an alternative format, or cannot safely be described in public.
- Follow [SECURITY.md](SECURITY.md) for a security vulnerability; exploitable
  details must not be posted in a public accessibility report.

Where known, include the affected page or component, source revision or build,
environment and assistive configuration, steps to reproduce, expected and
observed behaviour, and sanitised screenshots or logs. Medical details and
other sensitive personal information are not required and should not be
included.

No response or remediation deadline is guaranteed. Public reports may be
triaged and linked to the relevant implementation criterion; private reports
are handled through the selected private route.

## Contribution expectations

Contributions that affect a user-facing workflow should preserve keyboard
operation, visible focus, meaningful structure and labels, scalable layout,
usable contrast, reduced-motion safety, and alternatives for information that
would otherwise depend on colour, sound, or motion alone. New evidence must
state whether it comes from source review, a browser prototype, host tests,
native QEMU, physical hardware, or human evaluation.

The detailed design direction is recorded in
[DESIGN-AMBITION.md](docs/DESIGN-AMBITION.md#addition-of-15-november-2026--accessibility-as-part-of-the-design-identity).

## Assessment and maintenance

This statement was reviewed on 26 November 2026 using the current progress
register, repository documentation, automated evidence summaries, and project
issue routes. It is a self-assessment of documented project status, not an
external accessibility audit or certification.

The project owner maintains this statement. It should be reviewed when the
supported environment, reporting route, conformance evidence, or a material
known barrier changes, and at least annually while the project remains active.
