#!/usr/bin/env python3
"""Token ledger: aggregate measured model-token usage into docs/tokens.{md,json}.

Data source (owner decision D34): the LOCAL OpenCode session store on this
machine — a SQLite database, by default ``~/.local/share/opencode/opencode.db``
(tilde-expanded at runtime; override with ``--db`` or ``$OPENCODE_DB``).
Per guide §24 the ledger is "the model provider's usage export, aggregated by
release-tag boundary"; this store is the export available here. CI cannot read
a local store, so the division of labour is:

  * this script (run by the maintainer before tagging a release) aggregates
    the real records and rewrites ``docs/tokens.md`` + ``docs/tokens.json``;
  * CI only *validates* that the two committed files are mutually consistent
    (see the "Token ledger consistency" step in ``.github/workflows/ci.yml``).
    CI never computes token numbers.

Measured-only rule (guide §19 rule 3): every number written here comes from the
records actually present in the store. Nothing is estimated, interpolated or
invented; cycles with no attributable sessions get 0, not a guess.

PRIVACY CONSTRAINTS (binding on this script, D34 and D36):
  * It reads ONLY numeric usage columns plus path/timestamp/ID metadata from
    the ``session_v2``, ``project`` and ``worktree`` tables. The session
    ``parent_id`` is an opaque ID used solely to build the attribution graph
    below; it is never interpreted as content.
  * It NEVER reads message or part tables, message JSON blobs, session titles
    or any other content that could carry conversation text. It could not
    print personal content even by accident, because it never touches it.
  * It opens the database read-only (falling back to a read-only temporary
    copy if the live store is locked). It performs no network access.

ATTRIBUTION — two tiers (D36):

  Tier A ("direct") — a session is directly attributable when its own
  directory, the worktree of its project, or a directory the store's
  ``worktree`` table lists for one of the repository's projects, is the
  repository checkout or a sibling worktree named ``<repo>-*`` (e.g.
  ``cadence-wt-tokens``). This is the strict, path-attributable floor; it is
  reported per release in the ``Direct (Tier A)`` column of ``docs/tokens.md``
  and as ``tokens_direct`` in ``docs/tokens.json``.

  Tier A+B ("orchestration-inclusive", the headline) — Tier A plus every
  ANCESTOR session (the orchestrators that spawned direct sessions) and every
  DESCENDANT session (further subagents), computed transitively over the
  store's ``parent_id`` links: seed = the Tier-A set; add all transitive
  ancestors of seed members and all transitive descendants of seed members.
  Release rows and ``total_tokens_since_start`` report Tier A+B.

  Documented caveat (accuracy over precision, accepted in D36): an
  orchestrator session that worked across multiple repositories attributes
  wholly to Cadence under the ancestor rule, because the store records no
  per-repository split inside a single session. Tier A+B therefore leans
  high; the direct Tier-A figure is the strict lower bound. Sessions with no
  Tier-A member anywhere in their parent chain are excluded entirely.

  Sessions launched from the home directory that are neither Tier A nor in
  any Tier-A session's ancestor/descendant closure remain EXCLUDED — for
  those, the ledger is still a measured floor.

BUCKETING: git tags (semver ``vX.Y.Z``) with their tag dates split the
timeline into release cycles ``(previous tag, this tag]``; sessions that
started before the first tag fall into the first release's cycle. A session
is bucketed by its start time. Sessions started after the newest tag belong
to the in-development cycle: they are counted in the totals (they are real,
measured usage) and reported via ``tokens_unreleased`` /
``tokens_direct_unreleased``, but have no release row until that cycle is
tagged — so ``total_tokens_since_start`` equals the last release's
``tokens_cumulative`` plus ``tokens_unreleased`` (the shape CI validates).

Usage:
    python3 scripts/token-ledger.py [--db PATH] [--repo PATH] [--dry-run]

Stdlib only. Python 3.9+.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import sqlite3
import subprocess
import sys
import tempfile
from datetime import datetime, timezone
from pathlib import Path

LEDGER_JSON = "docs/tokens.json"
LEDGER_MD = "docs/tokens.md"

# The JSON keys the application's build-time release-notes reader depends on
# (crates/ui-egui/src/lib.rs). serde ignores unknown fields, so the §24
# "releases" array and the attribution keys below are additive and safe —
# but these three must never be removed or renamed.
LEGACY_KEYS = ("total_tokens_since_start", "release_tokens", "release_version")


def repo_default() -> Path:
    """The repository this script lives in (works from any checkout/worktree)."""
    return Path(__file__).resolve().parent.parent


def canonical_repo(repo: Path) -> Path:
    """The main working tree of the repository containing ``repo``.

    Attribution keys on the canonical repo name, so running from a linked
    worktree (``cadence-wt-*``) still attributes sessions of the real repo
    (``cadence``) and of its sibling worktrees. NOTE: used only to compute
    that name — files are always written into the checkout the script was
    invoked against, never into the main working tree.
    """
    out = subprocess.run(
        ["git", "-C", str(repo), "rev-parse", "--git-common-dir"],
        check=True, capture_output=True, text=True,
    ).stdout.strip()
    common = Path(out)
    if not common.is_absolute():
        common = (repo / common).resolve()
    return common.parent


def db_default() -> Path:
    env = os.environ.get("OPENCODE_DB")
    if env:
        return Path(env).expanduser()
    return Path.home() / ".local" / "share" / "opencode" / "opencode.db"


def open_readonly(db: Path) -> sqlite3.Connection:
    """Open the store read-only; on failure, read through a temp copy.

    A live OpenCode process may hold the store (WAL mode). Reading directly
    usually works; if it does not, copy the database plus its WAL/SHM side
    files to a private temp dir and open the copy read-only. Either way the
    original store is never written.
    """
    db = db.expanduser()
    try:
        con = sqlite3.connect(f"file:{db}?mode=ro", uri=True)
        con.execute("SELECT 1 FROM session_v2 LIMIT 1")
        return con
    except sqlite3.Error:
        pass
    tmp = Path(tempfile.mkdtemp(prefix="token-ledger-"))
    for suffix in ("", "-wal", "-shm"):
        src = Path(str(db) + suffix)
        if src.exists():
            shutil.copy2(src, tmp / ("store" + suffix))
    con = sqlite3.connect(f"file:{tmp / 'store'}?mode=ro", uri=True)
    con.execute("SELECT 1 FROM session_v2 LIMIT 1")
    return con


def fetch_sessions(
    con: sqlite3.Connection,
) -> tuple[list[dict], list[tuple[str, str]]]:
    """Numeric aggregates + attribution metadata only — never content.

    Returns the session rows and the raw ``worktree`` table rows
    (``project_id, directory``) used for Tier-A attribution.
    """
    rows = con.execute(
        """
        SELECT s.id, s.directory, s.project_id, s.parent_id, s.time_created,
               s.tokens_input, s.tokens_output, s.tokens_reasoning,
               s.tokens_cache_read, s.tokens_cache_write
        FROM session_v2 s
        """
    ).fetchall()
    worktrees = dict(
        con.execute("SELECT id, worktree FROM project").fetchall()
    )
    wt_rows = con.execute(
        "SELECT project_id, directory FROM worktree"
    ).fetchall()
    sessions = []
    for sid, directory, project_id, parent_id, created, *toks in rows:
        sessions.append(
            {
                "id": sid,
                "directory": directory or "",
                "project_worktree": worktrees.get(project_id) or "",
                "parent_id": parent_id or "",
                "created_ms": created,
                "tokens": sum(t or 0 for t in toks),
            }
        )
    return sessions, wt_rows


def path_matches_repo(path: str, repo: Path) -> bool:
    """True when a store path is the repo checkout or a ``<repo>-*`` worktree."""
    if not path:
        return False
    p = Path(path).expanduser()
    try:
        p = p.resolve()
    except OSError:
        pass
    if p == repo:
        return True
    base = repo.name
    return p.name == base or p.name.startswith(base + "-")


def repo_worktree_dirs(
    wt_rows: list[tuple[str, str]], projects: dict[str, str], repo: Path
) -> set[str]:
    """Directories the store's ``worktree`` table lists for this repository.

    A worktree row belongs to this repository when its project's own worktree
    matches the repo. These are opencode-managed worktree paths whose names
    need not carry the ``<repo>-*`` prefix (e.g.
    ``~/.local/share/opencode/worktree/<id>/shell-polish``).
    """
    dirs = set()
    for project_id, directory in wt_rows:
        if directory and path_matches_repo(projects.get(project_id, ""), repo):
            try:
                dirs.add(str(Path(directory).expanduser().resolve()))
            except OSError:
                dirs.add(directory)
    return dirs


def is_direct(session: dict, repo: Path, wt_dirs: set[str]) -> bool:
    """Tier A: directly attributable by path."""
    if path_matches_repo(session["directory"], repo):
        return True
    if path_matches_repo(session["project_worktree"], repo):
        return True
    try:
        directory = str(Path(session["directory"]).expanduser().resolve())
    except OSError:
        directory = session["directory"]
    return directory in wt_dirs


def orchestration_closure(
    seed: set[str], by_id: dict[str, dict]
) -> tuple[set[str], set[str]]:
    """Transitive ancestors and descendants of the seed sessions.

    Ancestors: follow ``parent_id`` upward from seed members (the
    orchestrators that spawned them, and their orchestrators in turn).
    Descendants: follow the parent links downward from seed members
    (further subagents). Siblings — other children of an ancestor — are
    neither ancestors nor descendants of a seed member and stay excluded.
    Parent IDs that do not resolve to a stored session are ignored.
    """
    parent_of = {
        sid: s["parent_id"] for sid, s in by_id.items() if s["parent_id"]
    }
    children_of: dict[str, list[str]] = {}
    for sid, s in by_id.items():
        parent = s["parent_id"]
        if parent and parent in by_id:
            children_of.setdefault(parent, []).append(sid)

    ancestors: set[str] = set()
    frontier = list(seed)
    while frontier:
        nxt: list[str] = []
        for sid in frontier:
            parent = parent_of.get(sid)
            if parent and parent in by_id and parent not in ancestors and parent not in seed:
                ancestors.add(parent)
                nxt.append(parent)
        frontier = nxt

    descendants: set[str] = set()
    frontier = list(seed)
    while frontier:
        nxt = []
        for sid in frontier:
            for child in children_of.get(sid, ()):
                if child not in descendants and child not in seed:
                    descendants.add(child)
                    nxt.append(child)
        frontier = nxt
    return ancestors, descendants


def git_tags(repo: Path) -> list[tuple[str, datetime]]:
    """Semver tags with their tag dates, oldest first."""
    out = subprocess.run(
        [
            "git",
            "-C",
            str(repo),
            "tag",
            "--sort=creatordate",
            "--format=%(refname:short)%09%(creatordate:iso8601-strict)",
        ],
        check=True,
        capture_output=True,
        text=True,
    ).stdout
    tags = []
    for line in out.splitlines():
        if not line.strip():
            continue
        name, _, date = line.partition("\t")
        if not re.fullmatch(r"v\d+\.\d+\.\d+", name):
            continue
        tags.append((name, datetime.fromisoformat(date)))
    return tags


def cycle_totals(
    sessions: list[dict], tags: list[tuple[str, datetime]]
) -> tuple[list[list], int]:
    """Split sessions into release cycles; return (per-release, unreleased).

    Cycle for release i is (tag[i-1].date, tag[i].date]; sessions that
    started before the first tag fall into the first release's cycle.
    Sessions are bucketed by their start time; sessions started after the
    newest tag are returned separately as the unreleased in-development
    total. Dates are the tag dates in the maintainer's local timezone,
    matching the dates the repository's own CHANGELOG uses.
    """
    per_release: list[list] = [
        [name, created.astimezone().date().isoformat(), 0]
        for name, created in tags
    ]
    unreleased = 0
    for s in sessions:
        started = datetime.fromtimestamp(s["created_ms"] / 1000, tz=timezone.utc)
        for i, (_name, tag_time) in enumerate(tags):
            if started <= tag_time:
                per_release[i][2] += s["tokens"]
                break
        else:
            unreleased += s["tokens"]
    return per_release, unreleased


def write_ledger_md(
    path: Path,
    rows: list[tuple[str, str, int, int, int]],
    latest_tag: str,
    unreleased: int,
    unreleased_direct: int,
) -> None:
    lines = [
        "# Token usage ledger",
        "",
        "Measured record of model-token consumption for Cadence development",
        "(guide §24). Measured-only rule: every number below comes from the",
        "local OpenCode usage records — never estimated, never invented.",
        "",
        "- **Data source:** the local OpenCode session store (SQLite, typically",
        "  `~/.local/share/opencode/opencode.db`), read by",
        "  `scripts/token-ledger.py`. The script reads only numeric usage",
        "  columns and path/timestamp/ID metadata; it never reads message",
        "  content.",
        "- **Attribution (two tiers, D36).** *Tier A — direct:* sessions whose",
        "  own directory is this repository's checkout, a `cadence-*` sibling",
        "  worktree, or a directory the store's `worktree` table lists for",
        "  this repository — the strict path-attributable floor, shown in the",
        "  Direct (Tier A) column. *Tier A+B — orchestration-inclusive:*",
        "  Tier A plus every ancestor session (the orchestrators that spawned",
        "  direct sessions) and every descendant session (further subagents),",
        "  transitively over the store's parent links. Release rows and",
        "  `total_tokens_since_start` report the Tier A+B headline; the",
        "  Direct (Tier A) column reports the strict subset.",
        "- **Known caveat (conservative over-attribution).** An orchestrator",
        "  session that worked across multiple repositories attributes wholly",
        "  to Cadence under the ancestor rule — the store records no",
        "  per-repository split inside a single session. Tier A+B therefore",
        "  leans high; the direct column is the strict lower bound. Both are",
        "  measured figures; nothing is estimated or interpolated.",
        "- **Privacy.** The script opens the store read-only and touches only",
        "  numeric usage columns and opaque path/ID/timestamp metadata",
        "  (including the parent links used for attribution); message and",
        "  part tables, message JSON and session titles are never read.",
        "- **Regenerate:** run `python3 scripts/token-ledger.py` before",
        "  tagging a release; it rewrites this file and `docs/tokens.json`",
        "  together.",
        "",
        "| Version | Date | Tokens this release | Tokens cumulative | Direct (Tier A) | Notes |",
        "|---------|------|--------------------:|------------------:|----------------:|-------|",
    ]
    for version, date, this, cumulative, direct in rows:
        lines.append(f"| v{version} | {date} | {this:,} | {cumulative:,} | {direct:,} | |")
    lines.append("")
    if unreleased or unreleased_direct:
        lines.extend(
            [
                f"In development since v{latest_tag} (the newest tag): "
                f"{unreleased:,} tokens ({unreleased_direct:,} direct) — real,",
                "measured usage of the unreleased cycle, counted in",
                "`total_tokens_since_start` and `total_tokens_direct` but with",
                "no release row until that cycle is tagged.",
                "",
            ]
        )
    path.write_text("\n".join(lines), encoding="utf-8")


def write_ledger_json(
    path: Path,
    rows: list[tuple[str, str, int, int, int]],
    total: int,
    total_direct: int,
    unreleased: int,
    unreleased_direct: int,
) -> None:
    doc = {
        "_comment": (
            "Measured token ledger (guide §24; D34/D36). Aggregated from the "
            "local OpenCode usage records by scripts/token-ledger.py; "
            "validated for internal consistency by CI. "
            "total_tokens_since_start / release_tokens / release_version are "
            "read by the app's build-time release-notes reader and must keep "
            "their names; release_* describe the latest tagged release's "
            "cycle. Attribution is two-tier (D36): headline figures are the "
            "orchestration-inclusive Tier A+B; tokens_direct / "
            "total_tokens_direct are the strict path-attributable Tier-A "
            "subset. *_unreleased count measured sessions started after the "
            "newest tag, which have no release row yet."
        ),
        "current_version": rows[-1][0],
        "total_tokens_since_start": total,
        "release_tokens": rows[-1][2],
        "release_version": rows[-1][0],
        "total_tokens_direct": total_direct,
        "tokens_unreleased": unreleased,
        "tokens_direct_unreleased": unreleased_direct,
        "releases": [
            {
                "version": version,
                "date": date,
                "tokens_this": this,
                "tokens_cumulative": cumulative,
                "tokens_direct": direct,
            }
            for version, date, this, cumulative, direct in rows
        ],
    }
    path.write_text(
        json.dumps(doc, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--db", type=Path, default=db_default())
    parser.add_argument("--repo", type=Path, default=repo_default())
    parser.add_argument(
        "--dry-run", action="store_true", help="print the result, write nothing"
    )
    args = parser.parse_args()
    checkout = args.repo.expanduser().resolve()  # files are written here
    repo = canonical_repo(checkout)  # canonical name drives attribution only

    con = open_readonly(args.db)
    try:
        sessions, wt_rows = fetch_sessions(con)
        projects = dict(
            con.execute("SELECT id, worktree FROM project").fetchall()
        )
    finally:
        con.close()

    wt_dirs = repo_worktree_dirs(wt_rows, projects, repo)
    by_id = {s["id"]: s for s in sessions}

    seed = {s["id"] for s in sessions if is_direct(s, repo, wt_dirs)}
    ancestors, descendants = orchestration_closure(seed, by_id)
    attributed = seed | ancestors | descendants

    tags = git_tags(repo)
    if not tags:
        print("error: no semver tags found in the repository", file=sys.stderr)
        return 1

    headline = [by_id[sid] for sid in attributed]
    direct = [by_id[sid] for sid in seed]
    headline_per, unreleased = cycle_totals(headline, tags)
    direct_per, unreleased_direct = cycle_totals(direct, tags)

    rows: list[tuple[str, str, int, int, int]] = []
    cumulative = 0
    direct_cumulative = 0
    for (version, date, tokens), (_v, _d, dtokens) in zip(
        headline_per, direct_per
    ):
        cumulative += tokens
        direct_cumulative += dtokens
        rows.append(
            (version[1:], date, tokens, cumulative, dtokens)  # drop the tag's "v"
        )
    # Sessions started after the newest tag are counted in the totals (they
    # are real, measured usage of the in-development cycle) but have no
    # release row until that cycle is tagged; the *_unreleased keys keep the
    # totals consistent with the release rows (the shape CI validates).
    total = cumulative + unreleased
    total_direct = direct_cumulative + unreleased_direct

    excluded = len(sessions) - len(attributed)
    print(f"store sessions: {len(sessions)}")
    print(
        f"  Tier A (direct): {len(seed)} sessions, "
        f"{total_direct:,} tokens"
    )
    print(
        f"  Tier A+B (headline): {len(attributed)} sessions "
        f"(+{len(ancestors)} ancestors, +{len(descendants)} descendants), "
        f"{total:,} tokens"
    )
    print(f"  excluded (no Tier-A member in their parent chain): {excluded}")
    for version, date, tokens, cum, dtokens in rows:
        print(f"  v{version}  {date}  +{tokens:,}  = {cum:,}  (direct {dtokens:,})")
    if unreleased or unreleased_direct:
        print(
            f"  (in-development cycle since v{tags[-1][0][1:]}: "
            f"{unreleased:,} tokens, {unreleased_direct:,} direct)"
        )
    print(f"total measured (Tier A+B): {total:,}  (direct floor: {total_direct:,})")

    if args.dry_run:
        return 0

    docs = checkout / "docs"
    if not docs.is_dir():
        print(f"error: {docs} not found (run inside the repository)", file=sys.stderr)
        return 1
    json_path = checkout / LEDGER_JSON
    md_path = checkout / LEDGER_MD
    write_ledger_json(
        json_path, rows, total, total_direct, unreleased, unreleased_direct
    )
    write_ledger_md(
        md_path, rows, tags[-1][0][1:], unreleased, unreleased_direct
    )
    print(f"wrote {md_path} and {json_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
