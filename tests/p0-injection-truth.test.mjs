// P0 source invariants: truthful SendInput reporting + foreground target
// validation. These tests parse the Rust clipboard module and the overlay
// script as text, asserting the ordering and sentinels fixed in the P0
// package (paste-into-wrong-window and silent partial delivery).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const clip = readFileSync(join(root, 'src-tauri', 'src', 'clipboard.rs'), 'utf8');
const overlay = readFileSync(join(root, 'src', 'js', 'overlay.js'), 'utf8');

function bodyOf(src, signature) {
  const start = src.indexOf(signature);
  assert.notEqual(start, -1, `expected ${signature}`);
  return src.slice(start);
}

test('inject_text validates the captured foreground target BEFORE touching the clipboard', () => {
  const body = bodyOf(clip, 'pub async fn inject_text(');
  const validate = body.indexOf('foreground_window_matches(target)');
  const lock = body.indexOf('CLIPBOARD_INJECTION_LOCK.lock()');
  assert.notEqual(validate, -1, 'inject_text must consult foreground_window_matches');
  assert.notEqual(lock, -1);
  assert.ok(validate < lock, 'target validation must happen before the clipboard lock');
  assert.ok(body.includes('TARGET_WINDOW_CHANGED:'), 'target-mismatch sentinel required');
});

test('inject_text reports partial paste delivery truthfully', () => {
  const body = bodyOf(clip, 'pub async fn inject_text(');
  assert.ok(body.includes('PASTE_FAILED:'), 'partial-delivery sentinel required');
  assert.ok(
    body.includes('ctrlv_sent == 4'),
    'paste_ok must be derived from the actual delivered key-event count',
  );
});

test('execute_keyboard_action reports partial delivery and rejects unknown actions', () => {
  const body = bodyOf(clip, 'pub async fn execute_keyboard_action(');
  assert.ok(body.includes('KEYBOARD_ACTION_FAILED:'));
  assert.ok(body.includes('Unknown keyboard action'));
  assert.ok(body.includes('delete_chars requires a charCount'));
  const validate = body.indexOf('foreground_window_matches(target)');
  const sleeps = body.indexOf('sleep(Duration::from_millis(80))');
  assert.notEqual(validate, -1);
  assert.ok(validate < sleeps, 'target validation must precede the first keystroke delay');
});

test('capture_target_window returns the raw HWND and never dereferences it', () => {
  const body = bodyOf(clip, 'pub fn capture_target_window(');
  assert.ok(body.includes('Some(foreground.0 as usize as u64)'));
  assert.ok(
    !body.includes('GetWindowText') && !body.includes('GetWindowTextLength'),
    'HWND must not be dereferenced to text: raw compare only',
  );
});

test('overlay captures a paste target for every recording session', () => {
  assert.ok(overlay.includes("invoke('capture_target_window')"));
  // Both hotkey-start paths record the target alongside the app-icon fetch.
  assert.ok(
    overlay.includes('captureTargetForSession(sessionId)'),
    'hotkey-start listeners must capture the target',
  );
  // Superseded takes are intentionally never injected (foreground belongs to
  // the new session), so their finished transcripts are saved but no paste
  // target is recorded for them. The orphan branch must never inject.
  const orphan = overlay.indexOf('Superseded take:');
  assert.notEqual(orphan, -1);
  const widget = overlay.slice(orphan, overlay.indexOf('}', overlay.indexOf('return;', orphan)));
  assert.ok(!widget.includes('attemptPaste'), 'superseded takes must not inject');
});

test('overlay treats TARGET_WINDOW_CHANGED as recoverable, not fatal', () => {
  assert.ok(overlay.includes('TARGET_WINDOW_CHANGED'));
  assert.ok(overlay.includes('Return to target window'));
  assert.ok(overlay.includes('Insert failed'));
  assert.ok(overlay.includes('pendingInjection'), 'failed injections must be retained for retry');
  assert.ok(overlay.includes('retryPendingInjection'));
  assert.ok(overlay.includes('Copied - paste with Ctrl+V'));
});

test('overlay keeps transcripts of superseded sessions unless the user discarded them', () => {
  const body = bodyOf(overlay, 'async function runSttFlow');
  assert.ok(
    body.includes('save_history_entry') && body.includes('discardedSessions'),
    'orphaned STT results must still be saved to history unless discarded',
  );
});

test('agent actions run without a confirmation prompt', () => {
  // Agent mode executes insert/rewrite/delete/select-all/submit directly.
  assert.ok(!overlay.includes('AGENT_DANGEROUS_ACTIONS'));
  assert.ok(!overlay.includes('requestActionConfirmation'));
  // Paste recovery (Retry / Copy) is independent of any prompt and remains.
  assert.ok(overlay.includes('handlePasteFailure'));
});

// The overlay window is created with "focus": false and "focusable": false
// (WS_EX_NOACTIVATE comes from focusable:false, tauri.conf.json): clicks on
// it never move the OS foreground window, so
// there is nothing to "restore" before injecting, and hiding the overlay
// first would only hide the recovery UI. Injections run against the
// still-foreground target, whose identity Inject already fail-closed-validates.
test('overlay never hides itself to hand back foreground before injecting', () => {
  assert.ok(
    !overlay.includes('prepareForegroundForInjection'),
    'focus-transfer helper must stay removed',
  );
  assert.ok(
    !overlay.includes('reshowOverlayForSession'),
    'reshow helper was only needed by the removed hide dance',
  );
  const execute = bodyOf(overlay, 'async function attemptKeyboardAction');
  const recovery = overlay.indexOf('function setupPasteRecovery');
  const windowAroundInjection = overlay.slice(
    execute,
    recovery > 0 ? recovery : overlay.length,
  );
  assert.ok(
    !windowAroundInjection.includes("invoke('hide_overlay')"),
    'injection/retry paths must not hide the overlay',
  );
});
