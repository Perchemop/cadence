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

---

```text
PROCESS FEEDBACK
Date / session: 2026-11-18
What I was trying to do: land the assistant HTTP-backends slice with the
  fmt gate green on the first verification round (no cargo locally).
Block or friction: hand-writing rustfmt-conformant code is unreliable.
  Round 2 failed the fmt gate; round 3 failed clippy on a concat! I had
  added purely to keep a line under 100 columns (fmt and clippy disagree
  about how to express a long string literal). Each round trip costs a
  full multi-platform Verify, roughly 15 minutes. This repeats the
  2026-11-02 entry below from a different angle.
Why the current guide/addendum caused it (or did not cover it): the guide
  bans cargo locally (correctly) but does not distinguish the standalone
  rustfmt binary — a millisecond pure-text transformer over edited files,
  no workspace resolution, no build — from cargo subcommands, so
  hand-formatting looked like the only local option.
Suggested change (one concrete edit): record in the guide's K.11 local-work
  allowance that running the standalone `rustfmt` binary directly (never
  `cargo fmt`) on edited files is permitted text processing; CI's
  `cargo fmt --check --all` remains the authoritative gate.
Severity: slows-work
Workaround I used (if any): ran `rustfmt --edition 2021 <files>` directly;
  output byte-identical to the CI gate's formatter, no cargo invoked.
Confirm: did NOT violate clean-room / v0.1.0 gate / no-secrets /
  CI-before-client-try / no meta-guide push
```
