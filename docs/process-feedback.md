# Process feedback

Append-only log of agent friction with the guide and addendum (addendum
Section N). Entries are advisory until the client promotes a change into the
local guide/addendum; nothing here waives a non-negotiable.

---

```text
PROCESS FEEDBACK
Date / session: 2026-11-02
What I was trying to do: land three code slices in one day (progress-driven
  rescheduling, per-assignment resource units, the 0.10.0 release), each
  verified only through GitHub Actions because cargo is banned locally.
Block or friction: the first `cargo fmt --check --all` failure in each slice
  cost a full CI round trip, because formatting is checked inside the
  platform "Verify" job that also builds and tests — roughly 10 minutes per
  cycle including the CodeQL job, for a change that rustfmt describes in
  seconds. Three slices hit this, so the same mistake cost ~30 minutes of
  wall time.
Why the current guide/addendum caused it (or did not cover it): the guide
  (Part I §4, addendum K.6) requires fmt/clippy/test/build in CI and bans
  local toolchain runs, but says nothing about job ordering, so the cheapest
  check sits behind the most expensive ones.
Suggested change (one concrete edit): in the repository's CI, split
  `cargo fmt --check --all` (and optionally `cargo clippy`) into a separate
  job that runs first on Linux with `needs: []`, so the other platform jobs
  do not start until formatting passes; and have a failing fmt step upload
  the `cargo fmt --diff` output as an artefact so the fix can be applied by
  hand without another round trip.
Severity: slows-work
Workaround I used (if any): parsed the rustfmt diff out of the failing job
  log and applied it by hand, then re-ran CI.
Confirm: did NOT violate clean-room / v0.1.0 gate / no-secrets /
  CI-before-client-try / no meta-guide push
```
