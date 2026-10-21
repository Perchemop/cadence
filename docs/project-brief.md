# Project Brief

## A. Product Identity

1. **Working title / purpose:** Cadence — a free, open-source construction and
   capital-project scheduling application.
2. **Primary user:** Construction project managers and site engineers at
   small-to-mid contractors, *and* owner-side capital-project controls teams
   (PMOs) — both equally.
3. **Problem solved:** Professional CPM scheduling (WBS, dependency logic,
   calendars, baselines, earned value) is locked behind expensive per-seat
   licences. Cadence delivers the core capabilities teams actually need,
   offline, with open file formats, at zero cost.

## B. Inspiration & Scope

4. **Feature inspiration (public behaviour only, clean-room):** Microsoft
   Project and Primavera P6 — inspired only by publicly documented concepts
   (see `docs/inspiration.md`).
5. **Most important capabilities:** hierarchical WBS; full dependency logic
   (FS/SS/FF/SF + lag/lead); CPM with float and critical path; interactive
   Gantt; resources and leveling; multiple calendars; baselines and variance;
   progress/EVM; open import/export.
6. **Out of scope for v1:** multi-user collaboration, cloud sync, mobile
   apps, ERP/cost-accounting integration, macros/scripting (see
   `docs/scope.md`).

## C. Technical Preferences

7. **Language / frameworks:** Pure Rust; egui + eframe + wgpu for the GUI.
8. **Target platforms:** Linux, Windows, macOS desktop. Browser/WASM is a
   post-v1 packaging goal. Multi-OS CI deferred (see `DECISIONS.md`).
9. **Offline:** Yes — fully offline, no telemetry, open formats.
10. **Constraints:** MIT OR Apache-2.0 dual licence; permissive-only
    dependencies listed in `ATTRIBUTION.md`; completely free of proprietary
    code or assets.

## D. Quality & Process

11. **UI polish:** Premium, commercial-grade feel is a requirement.
12. **Local machine:** Resource-constrained. All heavy builds and tests run
    on GitHub Actions; the repository is public.
13. **Design system / accessibility:** Original light and dark themes designed
    from scratch; accessibility (contrast, full keyboard support,
    screen-reader basics) is a requirement, not an afterthought.

## E. Name

14. **Product name:** Cadence — locked, see `docs/product-name.md`.
