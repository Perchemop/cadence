# Research — crash report submission

Status: draft, not human-approved.

## Behaviour wanted

The local crash log (already written by the panic hook) becomes an *optional*
GitHub Issue: the user previews what would be sent, edits or cancels. No
privileged token is embedded in the binary.

## Acceptance criteria

1. On the next launch after a crash, the app offers "report the last crash"
   and shows the exact text.
2. The user can edit the notes, and can decline; declining writes nothing to
   the network.
3. With no token configured, the fallback opens a pre-filled
   "New issue" URL in the system browser.
4. With a user-supplied token (stored in the OS keychain or an environment
   variable, never in the project file), the report is created as a GitHub
   Issue labelled `crash`.
5. The privacy note in the README describes exactly what is included.

## Implementation direction (original notes)

- Create an issue unauthenticated-in-repo via `POST /repos/{owner}/{repo}/issues`
  with a user token that has repo scope; the API documents required fields
  (`title`, `body`) and returns the issue URL:
  https://docs.github.com/en/rest/issues/issues?apiVersion=2022-11-28
- A user token is required because the repository's issues are open but the
  app must never embed a privileged bot token (guide §7.1). The pre-filled URL
  fallback works with no token at all:
  https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/creating-an-issue
- The crash entry already contains version, OS, panic message and a backtrace;
  the preview is that text minus any line the user strikes.

## Sources

- REST API — create an issue: https://docs.github.com/en/rest/issues/issues?apiVersion=2022-11-28
- Creating an issue via URL: https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/creating-an-issue
