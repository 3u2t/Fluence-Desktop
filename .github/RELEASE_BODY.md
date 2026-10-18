<!-- Release notes convention: this file ALWAYS contains the notes the next
     pushed tag publishes. release.yml reads it verbatim into `releaseBody`
     (steps:notes), so there is exactly one copy — edit here, never in the
     workflow. Never add secrets, internal hostnames, or model URLs not
     already pinned by digest in src-tauri/moonshine-v2-server/VENDOR.json.

     REPLACE THIS ENTIRE FILE (comment included) when cutting a release. -->

## A release about not typing into the wrong window

This release is mostly about **where your text ends up**. Speaking while
focussed on one app and pasting into another is the kind of bug you only
notice after it has happened.

### Paste can no longer land in the wrong window

When you start a recording, Fluence now remembers which window you were in.
Right before it pastes, it checks that window is still the one in front, and
if you switched apps mid-dictation it **stops and tells you** instead of
typing into whatever happened to come to the front.

The transcript is never lost. You get **Retry** and **Copy** so you can send
it where you meant.

### A blocked paste now says so

Previously Fluence would report "Inserted" even when Windows refused the
keystrokes — which happens when the target app is running elevated, or when
focus is shifting. Partial or fully blocked input is now reported as the
failure it is, with the specific reason on the overlay.

### Your clipboard is no longer silently wiped

Pasting has to take over the clipboard, and Fluence could only put text back.
If you had copied an **image or files**, those were destroyed with no warning.
Now the overlay says **"Inserted (clipboard replaced)"** so you know it
happened.

### Offline Moonshine transcription works

Local Moonshine models failed to start on many machines and, when they did,
the app could sit in a "processing" loop for a minute with no explanation.
Fixed at the source: the sidecar now authenticates itself over a private
per-launch token, and an outdated sidecar build is reported immediately
instead of hanging.

### Per-window permissions on internal actions

Text injection, keystroke simulation and window controls are now restricted to
the app surface that legitimately needs them, rather than being callable from
any open panel.

### Also in this release

- Release pipeline: third-party actions are pinned to commit hashes, the Rust
  toolchain is pinned, and pull requests now run the full test suite instead of
  only the release doing it.
- Version numbers are now checked for consistency across every manifest, so
  they cannot silently drift apart again.

### For the technical minded

The wire format used to sync your data is unchanged and still compatible with
previous releases. Nothing about your history, dictionary, snippets, or agents
needs re-syncing or re-importing.

### Known limitations

- The installer is **not yet code-signed**, so Windows SmartScreen may warn
  first time. This is unchanged from previous releases.
- Agent mode executes its actions without asking for confirmation, so a
  misheard command can act on the focused document.
- Audio copied to the clipboard as an image or file list is still replaced by
  a paste, and cannot be restored afterwards.
