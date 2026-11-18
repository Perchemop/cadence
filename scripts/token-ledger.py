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

PRIVACY CONSTRAINTS (binding on this script):
  * It reads ONLY numeric usage columns plus path/timestamp metadata from the
    ``session_v2`` and ``project`` tables.
  * It NEVER reads message or part tables, message JSON blobs, session titles
    or any other content that could carry conversation text. It could not
    print personal content even by accident, because it never touches it.
  * It opens the database read-only (falling back to a read-only temporary
    copy if the live store is locked). It performs no network access.

ATTRIBUTION: a session belongs to this repository when its own directory, or
the worktree of its project, is the repository checkout or a sibling worktree
named ``<repo>-*`` (e.g. ``cadence-wt-tokens``). Sessions launched from the
home directory are not attributable to any repository by this store and are
therefore EXCLUDED — the ledger is a measured floor, not a ceiling. This
limitation is recorded in ``docs/tokens.md`` and DECISIONS.md (D34).

BUCKETING: git tags (semver ``vX.Y.Z``) with their tag dates split the
timeline into release cycles ``(previous tag, this tag]``; sessions that
started before the first tag fall into the first release's cycle. A session
is bucketed by its start time.

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
# "releases" array is additive and safe — but these three must never be
# removed or renamed.
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


def fetch_sessions(con: sqlite3.Connection) -> list[dict]:
    """Numeric aggregates + attribution metadata only — never content."""
    rows = con.execute(
        """
        SELECT s.id, s.directory, s.project_id, s.time_created,
               s.tokens_input, s.tokens_output, s.tokens_reasoning,
               s.tokens_cache_read, s.tokens_cache_write
        FROM session_v2 s
        """
    ).fetchall()
    worktrees = dict(
        con.execute("SELECT id, worktree FROM project").fetchall()
    )
    sessions = []
    for sid, directory, project_id, created, *toks in rows:
        sessions.append(
            {
                "id": sid,
                "directory": directory or "",
                "project_worktree": worktrees.get(project_id) or "",
                "created_ms": created,
                "tokens": sum(t or 0 for t in toks),
            }
        )
    return sessions


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


def bucket_by_release(
    sessions: list[dict], tags: list[tuple[str, datetime]]
) -> list[tuple[str, str, int]]:
    """Return [(version, date, tokens_this)] per release, oldest first.

    Cycle for release i is (tag[i-1].date, tag[i].date]; sessions that started
    before the first tag fall into the first release's cycle. Sessions are
    bucketed by their start time. Dates are the tag dates in the maintainer's
    local timezone, matching the dates the repository's own CHANGELOG uses.
    """
    buckets: list[tuple[str, str, int]] = [
        (name, created.astimezone().date().isoformat(), 0)
        for name, created in tags
    ]
    for s in sessions:
        started = datetime.fromtimestamp(s["created_ms"] / 1000, tz=timezone.utc)
        idx = 0
        for i, (_name, tag_time) in enumerate(tags):
            if started <= tag_time:
                idx = i
                break
        else:
            # Started after the newest tag: belongs to the in-development
            # cycle, which has no release row until it is tagged. It is still
            # counted in the total below.
            continue
        buckets[idx] = (buckets[idx][0], buckets[idx][1], buckets[idx][2] + s["tokens"])
    return buckets


def write_ledger_md(path: Path, rows: list[tuple[str, str, int, int]]) -> None:
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
        "  columns and path/timestamp metadata; it never reads message content.",
        "- **What counts:** the sum of the store's per-session token columns",
        "  (input, output, reasoning, cache read, cache write) for sessions",
        "  attributed to this repository by path (the checkout and its",
        "  `cadence-*` worktrees), bucketed by release-tag date.",
        "- **Known limitation (conservative bias):** sessions launched from the",
        "  home directory — including orchestrator sessions and their",
        "  subagents — are not attributable to a repository by this store and",
        "  are excluded. This ledger is therefore a measured **floor**, not a",
        "  ceiling.",
        "- **Regenerate:** run `python3 scripts/token-ledger.py` before tagging",
        "  a release; it rewrites this file and `docs/tokens.json` together.",
        "",
        "| Version | Date | Tokens this release | Tokens cumulative | Notes |",
        "|---------|------|--------------------:|------------------:|-------|",
    ]
    for version, date, this, cumulative in rows:
        lines.append(f"| v{version} | {date} | {this:,} | {cumulative:,} | |")
    lines.append("")
    path.write_text("\n".join(lines), encoding="utf-8")


def write_ledger_json(
    path: Path,
    rows: list[tuple[str, str, int, int]],
    total: int,
    latest: tuple[str, str, int, int],
) -> None:
    doc = {
        "_comment": (
            "Measured token ledger (guide §24; D34). Aggregated from the local "
            "OpenCode usage records by scripts/token-ledger.py; validated for "
            "internal consistency by CI. total_tokens_since_start / "
            "release_tokens / release_version are read by the app's "
            "build-time release-notes reader and must keep their names; "
            "release_* describe the latest tagged release's cycle."
        ),
        "current_version": latest[0],
        "total_tokens_since_start": total,
        "release_tokens": latest[2],
        "release_version": latest[0],
        "releases": [
            {
                "version": version,
                "date": date,
                "tokens_this": this,
                "tokens_cumulative": cumulative,
            }
            for version, date, this, cumulative in rows
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
        sessions = fetch_sessions(con)
    finally:
        con.close()

    mine = [s for s in sessions if path_matches_repo(s["directory"], repo)
            or path_matches_repo(s["project_worktree"], repo)]
    tags = git_tags(repo)
    if not tags:
        print("error: no semver tags found in the repository", file=sys.stderr)
        return 1

    per_release = bucket_by_release(mine, tags)
    rows: list[tuple[str, str, int, int]] = []
    cumulative = 0
    for version, date, tokens in per_release:
        cumulative += tokens
        rows.append((version[1:], date, tokens, cumulative))  # drop the tag's "v"
    # Sessions started after the newest tag are counted in the total (they
    # are real, measured usage of the in-development cycle) but have no
    # release row until that cycle is tagged.
    after_latest = sum(
        s["tokens"]
        for s in mine
        if datetime.fromtimestamp(s["created_ms"] / 1000, tz=timezone.utc) > tags[-1][1]
    )
    total = cumulative + after_latest

    print(f"store sessions: {len(sessions)}, attributed to {repo.name}: {len(mine)}")
    for version, date, tokens, cum in rows:
        print(f"  v{version}  {date}  +{tokens:,}  = {cum:,}")
    if after_latest:
        print(f"  (in-development cycle since v{tags[-1][0][1:]}: {after_latest:,})")
    print(f"total measured: {total:,}")

    if args.dry_run:
        return 0

    docs = checkout / "docs"
    if not docs.is_dir():
        print(f"error: {docs} not found (run inside the repository)", file=sys.stderr)
        return 1
    json_path = checkout / LEDGER_JSON
    md_path = checkout / LEDGER_MD
    write_ledger_json(
        json_path, rows, total, (rows[-1][0], rows[-1][1], rows[-1][2], rows[-1][3])
    )
    write_ledger_md(md_path, rows)
    print(f"wrote {md_path} and {json_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
