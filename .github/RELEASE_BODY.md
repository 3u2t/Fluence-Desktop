<!--
     Release notes convention: this file ALWAYS contains the notes the next
     pushed tag publishes. release.yml reads it verbatim, so there is exactly
     one copy, edit here, never in the workflow. Replace this ENTIRE file
     (this comment included) when cutting a release. Keep the language plain:
     write for the person using Fluence, not for the person building it. -->

## Choose the color of Fluence

### Six accent presets, from Fluence Original to pure monochrome

Open Settings, go to General and pick the new Appearance tab. There you
find six accent colors: Fluence Original, Orange, Olive Green, Yellow,
Rose and Monochrome. Pick one and the accent color follows you across the
settings window: primary buttons, toggles, selection highlights, the update
button and the charts on your dashboard all pick it up.

### Reads well in dark and light

Every preset arrives in two shades. When the app looks dark you get the
brighter shade, when you switch to light mode the app swaps to a darker one
so buttons and text stay easy to read. The switch happens automatically and
you can flip the look at any time with Ctrl+Shift+L.

### What stays the same on purpose

The app's surfaces keep their calm dark styling. The fluence logo, the
floating recording overlay and the first-run wizard look exactly like
before, no accent in the list touches them. Picking Fluence Original gives
you precisely the color behavior of the previous release, so if you want
nothing to change, leave it there.

### Also in this release

- In light mode the outline shown when you move through controls with the
  keyboard now matches your chosen accent instead of always being teal.
- The charts on the dashboard follow your accent. With Fluence
  Original they keep the same purple to cyan range as before.

### For the technical minded

Nothing about your data changed. History, dictionary, snippets and agents
work exactly as before, and recording, transcription and typing behaviour
are untouched. Accent choice lives on the device in the settings window,
it is not part of sync yet. The accent layer is built only from token
references with safe fallbacks, so windows that do not know about it keep
rendering the previous values.

### Known limitations

- The accent applies to the settings window. The overlay and the first-run
  wizard keep their own look on purpose.
- The installer is **not code-signed**, so Windows SmartScreen may warn the
  first time you run it. Choose "More info" → "Run anyway". This is unchanged
  from previous releases.
- Agent mode carries out its actions without asking first, so a misheard
  command can act on the window you are in.
- Text goes wherever your cursor is when transcription finishes, that is the
  intended behaviour, and it means switching apps while dictating pastes into
  whichever app you end up in.
- An image or file list on your clipboard is replaced by a paste and cannot be
  restored afterwards. You are told when this happens.
