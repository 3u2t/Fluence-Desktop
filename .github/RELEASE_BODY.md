## Smarter dictionary suggestions — and less noise

### Suggestions now earn their way onto the list

Every correction Fluence notices while you dictate has always been recorded,
but until now even a single odd sighting could appear as a suggestion.
Suggestions are now shown only when they repeat:

- A correction is saved the first time it is noticed, but it only reaches the
  Suggestions list once it is seen again — one-off slips stay invisible and
  just keep counting quietly.
- Broken pairs are never recorded at all: random typos, near-identical words,
  and empty or malformed corrections are rejected at the moment of capture.
- On startup, the app cleans out accumulated junk rows from earlier versions
  so the list you see is genuinely relevant.

### Accepting a suggestion now asks first

Pressing accept opens a confirmation telling you exactly what will happen:
the pair is added to your dictionary, and future dictations that hear that
mishearing are rewritten automatically. Nothing is added silently anymore.

### The "Learning signals" list is gone

The in-progress list of sub-threshold corrections that shipped in the
previous release turned out to be noise on screen. It is removed; the
learning logic behind it still runs, and you simply no longer see rows that
are not actionable yet.

### Also in this release

- The accept-confirmation wording is defined in one place per interface, so
  the message cannot drift apart between settings screens.
- New tests pin the threshold behaviour so suggestions cannot silently
  change buckets again.

### For the technical minded

Nothing about your data format changed. History, dictionary, snippets and
agents are all compatible with previous releases — no re-sync or re-import,
and you can upgrade and downgrade freely. Audio processing, transcription
and text injection are untouched.

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
