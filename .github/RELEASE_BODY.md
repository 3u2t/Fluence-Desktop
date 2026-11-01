## A window without its picture frame, and a clipboard that keeps your words

### A unified window at last

The Fluence window no longer reads as "a browser window with the app running inside it". The hairline border around the window and the separate title strip with its separator bar are gone. The window title and the three window buttons now sit directly on the app: over the sidebar the title bar takes the sidebar's color, and over the main content it continues with the content's color, in both dark and light themes. The rounded corners are truly transparent now - you see your desktop through them, so the small dark spots that appeared at the window corners are fixed.

### Your dictated words stay on the clipboard

A new toggle in General settings (on by default): after each dictation, your dictated text and agent results stay on the clipboard even if the paste fails, so you can simply press Ctrl+V to paste them again anywhere. Turn the toggle off if you prefer to keep whatever you had copied earlier instead. This preference is stored on this device only and is not part of sync.

### What stays the same

Your audio detection, dictation shortcuts, language models, history, snippets, agents, styles, settings, and sync data are completely unaffected. The onboarding wizard, the floating overlay, and all commands behave exactly as before.

### Known limitations

- The installer is **not code-signed**, so Windows SmartScreen may warn the first time you run it. Choose "More info" → "Run anyway". This is unchanged from previous releases.
- Agent mode carries out its actions without asking first, so a misheard command can act on the window you are in.
- Text goes wherever your cursor is when transcription finishes; switching apps while dictating pastes into whichever app you end up in.
- With the clipboard toggle on, the app no longer brings back what you had copied before dictating, so your earlier copied content is not recoverable after a dictation. Turn the toggle off to restore that behavior.
- An image or file list on your clipboard is replaced by a paste and cannot be restored afterwards. You are told when this happens.
- During route changes in the settings window, scrolling content can briefly slide under the floating title bar for a moment, as in any modern window with an integrated title bar.
