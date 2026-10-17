<!-- Release notes convention: this file ALWAYS contains the notes the next
     pushed tag publishes. release.yml reads it verbatim into `releaseBody`
     (steps:notes), so there is exactly one copy — edit here, never in the
     workflow. Never add secrets, internal hostnames, or model URLs not
     already pinned by digest in src-tauri/moonshine-v2-server/VENDOR.json. -->

## Sync reliability release

A deep audit of the cloud sync engine turned up several bugs that
could quietly lose your data or leave sync silently broken. All of
them are fixed here.

> **Why this is v1.27.1:** the `v1.27.0` tag was cut from the same
> code but never produced a download. Its release pipeline stopped
> at its own test gate before anything was built or published, and a
> regression test for these very fixes was the step that failed.
> Nothing was ever released under `v1.27.0`, so this is the first
> public build of the sync reliability work.

### Data safety

- **A corrupt snippet file can no longer delete your other snippets.**
  If one cloud file failed to parse, it could be treated as valid and
  the readable copy beside it removed. That is fixed.
- **Turning a setting off now sticks.** Changing any other setting
  could silently switch sync back on, and it only revealed itself
  after a restart. Fixed.
- **Your custom Agents and Styles are protected.** A damaged store
  used to read as "you have none", and your next edit overwrote the
  real file. Writes now refuse to touch an account store they could
  not read.
- **Edits made while sync was running are no longer discarded.** A
  setting you changed mid-sync could be overwritten with no trace.
  It is now deferred and carried to the next sync instead.
- **Saving is genuinely durable now.** Every store flush was writing
  through a read-only handle, so the flush silently did nothing. Ten
  call sites fixed, and eight files that could be corrupted by two
  concurrent writers now write through uniquely named temporary files.

### Cloud sync correctness

- **Sync stops immediately if the signed-in account does not match
  the account on file**, instead of quietly switching you to a
  different account in the background.
- **Duplicate cloud files are now merged instead of ignored.**
- **Rate limits are respected.** If Google asks us to wait, we wait,
  instead of retrying every minute against an exhausted quota.
- **Sync no longer gets stuck** on a slow or unresponsive connection.

### For the technical minded

The sync wire format is unchanged and still compatible with previous
releases - same envelope version, same folders, same conflict
resolution rules. All fixes are in how this client reads, merges and
writes those files, so you can upgrade and downgrade freely and your
data stays readable either way.
