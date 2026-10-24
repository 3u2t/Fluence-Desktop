<!--
     Release notes convention: this file ALWAYS contains the notes the next
     pushed tag publishes. release.yml reads it verbatim, so there is exactly
     one copy, edit here, never in the workflow. Replace this ENTIRE file
     (this comment included) when cutting a release. Keep the language plain:
     write for the person using Fluence, not for the person building it. -->

## Your dictionary just got smarter, and a lot cleaner

### Corrections now have to repeat before they show up

When Fluence types something odd, like "chief White" when you said a
name, it quietly remembers the fix. From now on, that fix only appears in
your Suggestions list if it happens **again**. One-time slips never clutter
the list; they just keep counting in the background.

### Junk suggestions are stopped before they start

Half-written fixes, random typos, and "corrections" that replace a word with
basically the same word are no longer saved at all. On top of that, the app
now cleans out old junk entries when it starts, in our testing this cleared
over 700 useless rows in one go, so the list you see is actually worth
looking at.

### Nothing is added without asking you first

When you accept a suggestion, you now get a short confirmation that says
exactly what will happen: the pair is added to your dictionary, and your
future dictations of that misheard word are fixed automatically. Nothing
gets added silently anymore.

### The "Learning signals" list is gone

The in-progress list we showed last release turned out to be confusing on
screen. It has been removed. The learning behind it still runs, you simply
never see anything that isn't ready to be useful.

### Also in this release

- The confirmation message you see when accepting a suggestion is now
  written in one place, so it says exactly the same thing on every screen.
- More automated tests were added so the fixes above stay fixed.

### For the technical minded

Nothing about your data changed. History, dictionary, snippets and agents
all work exactly as before, you can upgrade and go back without any
import, export, or re-sync. Recording, transcription and typing behaviour
are untouched.

### Known limitations

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
