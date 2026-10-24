# Governance

Cadence is currently a small project with one human owner. This document states
who decides what, how decisions are made and recorded, and how someone can grow
into a bigger role — so that "no" is a policy, not a mood, and so the project
can continue if the current maintainer steps back.

## Roles

| Role | What it means here |
|---|---|
| **User** | Anyone who runs Cadence. Reports matter; a bug report is a contribution. |
| **Contributor** | Anyone who adds value: an issue with a reproduction, a triage, a doc fix, a merged patch. No commit access implied. |
| **Maintainer** | Feels responsible for the project's direction. Not necessarily a coder — documentation, triage and community work count. Holds commit/admin access. |

Automation (the build agents) is **not** a role in this model. Agents act on
behalf of the maintainer with scoped credentials; every agent change arrives as
a pull request with CI evidence, exactly like a human's.

## Model: BDFL, with a written trail

Cadence uses the **BDFL** model (one maintainer has the final say), which fits a
single-author project. The maintainer is bound by the recorded process:

- Decisions that affect the product, the licence, or the process are written in
  [`DECISIONS.md`](DECISIONS.md) with the date and the reasoning.
- The [vision](VISION.md) and the roadmap act as the project's constitution:
  debates are re-anchored on "does this match the stated goals?" rather than on
  personal preference.
- Anything the guide or documentation leaves open is listed as an open
  question in [`VISION.md`](VISION.md) or
  [`docs/assumptions-and-questions.md`](docs/assumptions-and-questions.md),
  not decided silently.

If the project gains more than one regular maintainer, the model should move to
consensus among maintainers, and this file must be updated at that point.

## How decisions are made

1. **Consensus seeking over voting.** Discuss until serious concerns have been
   heard; proceed when only minor concerns remain. Voting produces winners and
   losers and is reserved as a tiebreaker.
2. **Public by default.** Design discussion, decisions and review notes happen
   in public issues and pull requests. Two exceptions have a private channel and
   nothing else does: **security reports** ([`SECURITY.md`](SECURITY.md)) and
   **sensitive code-of-conduct reports** ([`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md)).
   If something is decided privately, the reasoning is published afterwards.
3. **Tiebreaker.** The maintainer decides when consensus cannot be reached.
   That is the BDFL part, and it is stated here *before* it is needed.
4. **Tests and CI are the arbiter of "done".** A change is not accepted because
   someone argues for it, but because the CI result and the acceptance criteria
   agree. See the Definition of Done in the build process and
   [`docs/development.md`](docs/development.md).

## Commit access and the default branch

- Commit access is granted sparingly: to the maintainer, and to a contributor
  who has demonstrated sustained, careful work and wants it.
- The default branch (`main`) is protected: changes land through a pull request
  with passing required status checks — the multi-platform build and package
  checks plus the security and lockfile checks listed in `.ci-required.txt` —
  the branch must be up to date with `main`, and force
  pushes and branch deletion are disabled. Nobody relies on direct pushes; the
  build tooling opens a pull request for every change.
- **Review approval is not required while there is one maintainer**, because
  GitHub does not allow self-approval and a rule nobody can satisfy is not a
  control. The moment a second maintainer exists, one approving review should be
  required. The maintainer account is exempt from the check requirement
  (`enforce_admins`), so a broken pipeline cannot lock the project out; that
  exemption is the only bypass, and it is exercised only to fix the pipeline
  itself.
- An agent's credentials are treated as a privileged account: scoped to the
  minimum needed, never a personal long-lived token, and covered by the same
  review rules.

## Resolving conflict

- The maintainer acts as a moderator, not a combatant; kindness is the bar.
- Re-anchor disagreements on the vision and roadmap, not on the person.
- Keep conversations on action; close threads that are not converging, saying
  why.
- A single loud voice is not automatically the community; silent users count
  too.
- Code-of-conduct violations follow [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md).

## Commercial participation

If a company ever pays someone to work on Cadence, that is normal and welcome.
Paid contributors get **no special treatment**: contributions are judged on
their merits and discussions stay about the work, not the funder. Any such
relationship must be disclosed in the pull request or the issue where it is
relevant. There is no company involvement today.

## Money, legal entity and funding

No money flows through this project today, and no legal entity exists. If that
changes, the lightest option comes first (GitHub Sponsors), and a fiscal sponsor
would be considered before standing up an entity. Until then, this section is
deliberately empty of commitments.

## How to become a maintainer

There is no hidden path, and this is written down so the maintainers do not look
like a clique:

1. Contribute something real — a merged fix, a test, documentation, triage —
   and do it more than once.
2. Show judgement about the project's scope: propose things that fit the vision,
   and say no to things that do not.
3. Ask. If you want the role, say so; there is no penalty for asking.

Areas that can be owned independently, and would be welcome: **accessibility**,
**documentation**, **interchange formats**, **security triage**. Self-organise
into those areas rather than being assigned.

## Metrics this project tracks

Recorded deliberately, because untracked numbers are motivation without a goal:

| Tracked | Why |
|---|---|
| Release artefact downloads per version | Whether anyone actually uses the builds |
| Unique cloners | Discovery versus usage |
| Open vs. newly opened issues and PRs | Whether the queue is growing or draining |
| Time to first response on issues | The one responsiveness number that predicts contributor retention |
| Stars | Cheap discovery signal, interpreted as noise-with-trend |

Deliberately **not** tracked: page views and referral analytics (no analytics
are embedded anywhere, and none will be added to a privacy-first tool). Popularity
is not a goal — correctness and usability are. There is no telemetry in the
application itself and there never will be.

## Roles for security

Even with one maintainer, the roles are named so that they can be handed over:

- **Triage:** the maintainer reads reports and assigns severity.
- **Dependency review:** the maintainer reviews the automated dependency PRs
  (Dependabot) and the security workflow result.
- **Patch approval and release:** the maintainer approves and tags.

## Sustainability and boundaries

Cadence gets **a few hours a week** — a hobby-scale amount of time, stated so
that expectations are honest. A reply to an issue or pull request can be
expected **within seven days**; a release happens when there is something worth
releasing, not on a calendar. The full expectations are repeated in
[`CONTRIBUTING.md`](CONTRIBUTING.md) and the [README](README.md) so that they can
be pointed at later.

It is explicitly fine for the maintainer to pause. If work stops permanently,
the licence means anyone may continue from a fork.
