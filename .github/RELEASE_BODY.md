<!-- Release notes convention: this file ALWAYS contains the notes the next
     pushed tag publishes. release.yml reads it verbatim into `releaseBody`
     (steps:notes), so there is exactly one copy — edit here, never in the
     workflow. Never add secrets, internal hostnames, or model URLs not
     already pinned by digest in src-tauri/moonshine-v2-server/VENDOR.json.

     REPLACE THIS ENTIRE FILE (comment included) when cutting a release. -->

## Offline Moonshine works, and it tells you when it can't

### Local Moonshine transcription starts reliably

Local Moonshine models often failed to start, and when they did the app could
sit in a "processing" loop for a minute with no explanation. Several causes,
all fixed:

- The sidecar now identifies itself with a private per-launch token over a
  local handshake, so the app only talks to the copy it actually started.
- The app waits properly for the model to load instead of giving up after
  three seconds and killing a sidecar that was still starting.
- If the sidecar is out of date or the port is held by something else, it says
  so in seconds rather than hanging silently.
- A sidecar left behind by a previous run no longer blocks a new one.

### Offline transcription no longer runs unbounded

Very long takes are now rejected with a clear message instead of growing
memory without limit. The recording limit is ten minutes.

### A blocked paste now says so

Fluence would previously report "Inserted" even when Windows refused the
keystrokes, which happens when the target app is running elevated or focus is
shifting. Partial or blocked input is now reported as the failure it is, with
the reason on screen, and the transcript stays recoverable — you can **Retry**
or **Copy** it instead of losing it.

### Your clipboard is no longer silently wiped

Pasting takes over the clipboard, and Fluence can only put text back. If you
had copied an **image or files**, they were destroyed with no warning. The
overlay now says **"Inserted (clipboard replaced)"** so you know it happened.

### Agent failures are visible

If an agent action fails — including a copy — the overlay now reports what went
wrong instead of showing success.

### Also in this release

- Release pipeline: third-party actions are pinned to commit hashes, the Rust
  toolchain is pinned, and pull requests now run the full test suite.
- Version numbers are checked across every manifest so they cannot drift apart
  again.
- Internal commands are restricted to the app surface that legitimately needs
  them.

### For the technical minded

Nothing about your data format changed. History, dictionary, snippets and
agents are all compatible with previous releases — no re-sync or re-import, and
you can upgrade and downgrade freely. Audio processing is untouched.

### Known limitations

- The installer is **not code-signed**, so Windows SmartScreen may warn the
  first time you run it. Choose "More info" → "Run anyway". This is unchanged
  from previous releases.
- Agent mode carries out its actions without asking first, so a misheard
  command can act on the window you are in.
- Text goes wherever your cursor is when transcription finishes — that is the
  intended behaviour, and it means switching apps while dictating pastes into
  whichever app you end up in.
- An image or file list on your clipboard is replaced by a paste and cannot be
  restored afterwards. You are told when this happens.
