## Reactive amp bars and voice-following startup

### Your waveform responds from the first moment

Recording now starts visibly smooth: the waveform tracks your voice within the first fraction of a second, ramps in cleanly, and no longer stays flat or twitches while the noise floor is still being learned.

### Amp bars light up across the whole row

In Amp Bars style, the bars near the left and right edges of the overlay now react just like the centre ones, so the entire row dances with your voice instead of only the middle.

### What stays the same

Your audio detection, dictation shortcuts, language models, history, snippets, settings, and sync data are completely unaffected. Quiet rooms still produce a calm, low waveform, and background noise or the hotkey chime does not inflate the meter. The changes in this release are visual only.

### Known limitations

- The installer is **not code-signed**, so Windows SmartScreen may warn the first time you run it. Choose "More info" → "Run anyway". This is unchanged from previous releases.
- Agent mode carries out its actions without asking first, so a misheard command can act on the window you are in.
- Text goes wherever your cursor is when transcription finishes; switching apps while dictating pastes into whichever app you end up in.
- An image or file list on your clipboard is replaced by a paste and cannot be restored afterwards. You are told when this happens.
