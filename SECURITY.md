# Security Policy

Cadence is offline-first: the scheduling engine, the model, and the desktop
application work without network access. The release artefact links no runtime
network capability by design (see `docs/research/update-check-stub.md` for the
deliberate "reads the version, no network call" decision). Still, bugs happen —
especially in a project that parses files and does arithmetic on dates.

## Supported versions

Only the latest released version gets security fixes. Security updates ship as
a patch or minor release on the newest tag; older tags are not patched.

| Version | Supported |
| --- | --- |
| `0.1.x` (latest release) | Yes |
| Older tags | No |

## Reporting a vulnerability

**Do not open a public issue for a security problem.**

Use GitHub's private reporting: **Security → Report a vulnerability** on the
repository, or contact the maintainers at the address in `README.md`.

Include:

- Which version or commit is affected, and which OS.
- What happened, and what should have happened.
- Steps to reproduce, and any input file needed.
- Whether the problem needs a specific calendar, file, or command.

You will get an acknowledgement within seven days, an assessment within thirty
days, and credit in the release notes when you want it. Disclosure after a fix
ships is fine; coordinated disclosure is preferred but not required.

## What counts as a security problem for Cadence

- Parsing a crafted project file that crashes, hangs, or executes code
  (deserialisation, date parsing, archive handling).
- Any path where the app writes outside its own project directory, or reads
  files it was not asked to open.
- Any update-check or crash-report path that sends more data than documented,
  or sends it to an undocumented destination.
- A dependency used in a way that exposes a known vulnerability.

Rendering mistakes, scheduling disagreements with other tools, and missing
features are ordinary bugs — please report them with the normal issue templates.

## Handling of dependency vulnerabilities

Cadence keeps its dependency graph small (`cargo tree` is inspected at release
time). Licence and advisory status of dependencies is checked in CI with
`cargo deny` during the security workflow. A fix for an advisory in a
transitive dependency ships in the next patch release.

## Threat model

A short, honest model, so researchers know what Cadence actually trusts.

**What Cadence does.** Reads a project file the user chooses (SQLite, MSPDI XML,
MPX), computes a schedule in process, and writes files the user chooses
(project, exports, Markdown/PDF report). It draws a desktop window. It makes no
network request.

**What it trusts.**

- The user's own project files — but a *crafted* file must not be able to do
  more than crash, so parsing is treated as untrusted input.
- The local filesystem paths the user selects.
- Third-party libraries (`egui`/`eframe`, `rusqlite`/SQLite, `time`, `serde`)
  for memory safety; a memory-safety bug in those is in scope for a Cadence
  advisory if Cadence can be made to trigger it.

**What it does not trust, and therefore defends against.**

- Malicious or corrupt project files (import is a parser boundary).
- The network: there is no network code in the application, so an unexpected
  network call is itself a vulnerability.
- Passively-collected user data: no telemetry exists, so any transmission is a
  vulnerability, not a feature.

**How it could be misused.** Straightforwardly: it is a local tool that reads
and writes files the user names. The dangerous classes are therefore arbitrary
file write through a malicious project file, path traversal in import/export,
and denial of service through a pathological schedule (unbounded loops on a
crafted dependency graph).

**Out of threat model.** Anyone with the ability to run arbitrary code as the
user, or to modify the installed binary, already has the user's privileges.

## Incident response

Short, and written down so that a bad day has a plan:

1. **Triage.** The maintainer reads the report in the private advisory, judges
   severity (critical: remote/offline code execution or silent data
   exfiltration; high: arbitrary file write, or a crash-on-open that is cheaply
   triggered; medium: denial of service, information disclosure; low: anything
   that needs a hostile local actor), and replies within seven days.
2. **Mitigate.** Reproduce, then fix on a private branch. If the report concerns
   a dependency, the fix is a dependency bump plus, if needed, a workaround.
3. **Disclose.** Ship the fix in a patch release, publish a GitHub security
   advisory with credit, and note it in [`CHANGELOG.md`](CHANGELOG.md). If the
   issue affects users of an older tag, say so in the advisory.
4. **Tell people.** The release notes and advisory are the notification channel;
   there is no mailing list. If a downstream fork or consumer is affected, they
   are contacted directly where identifiable.
5. **Learn.** Any incident gets recorded in [`DECISIONS.md`](DECISIONS.md) or the
   changelog with the reasoning, so the same class of bug is cheaper to catch
   next time.

## Out of scope for a coordinated report

- Anything requiring physical access to the reporter's machine.
- Vulnerabilities in the host desktop environment or the toolchain itself —
  report those upstream, but tell us if Cadence's behaviour amplifies them.
- Theories without a reproduction path, unless they describe a class of bug we
  already know about.
