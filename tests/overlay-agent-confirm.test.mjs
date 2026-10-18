// Agent-action confirmation + injection responsiveness invariants.
// These parse src/js/overlay.js (and tauri.conf.json) to pin the behavior
// fixed in the hardening pass:
//   - dangerous agent actions (delete/select-all/submit) cannot run without
//     an explicit Confirm click (cancel/timeout never executes them);
//   - non-destructive actions (insert/rewrite/copy) stay one-click and are
//     not members of the dangerous set;
//   - the overlay never hides itself around injection (the window is
//     focus:false/WS_EX_NOACTIVATE, so no focus hand-back is needed);
//   - confirmed actions run against the validated target and surface a
//     visible success state, and failures stay recoverable.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const overlay = readFileSync(join(root, 'src', 'js', 'overlay.js'), 'utf8');
const tauriConf = readFileSync(join(root, 'src-tauri', 'tauri.conf.json'), 'utf8');

test('the overlay window cannot take OS focus (WS_EX_NOACTIVATE by config)', () => {
  const from = tauriConf.indexOf('"label": "overlay"');
  assert.notEqual(from, -1, 'overlay window definition must exist');
  const windowJson = tauriConf.slice(from, tauriConf.indexOf('}', from));
  assert.match(windowJson, /"focus"\s*:\s*false/, 'overlay must be focus:false');
  assert.match(
    windowJson,
    /"focusable"\s*:\s*false/,
    'overlay must be focusable:false (this is what applies WS_EX_NOACTIVATE)',
  );
});

test('dangerous agent actions are pinned to delete/select-all/submit only', () => {
  const setLine = overlay.match(
    /AGENT_DANGEROUS_ACTIONS\s*=\s*new Set\(\[([^\]]*)\]\)/,
  );
  assert.notEqual(setLine, null, 'dangerous set must exist');
  const members = setLine[1].split(',').map((s) => s.trim().replaceAll("'", ''));
  assert.deepEqual(members.sort(), ['delete_chars', 'select_all', 'submit']);
});

test('destructive agent actions gate on an explicit confirmation resolution', () => {
  const flow = overlay.slice(overlay.indexOf('function requestActionConfirmation'));
  assert.match(flow, /resolve\(value\)/, 'confirmation must resolve with the user answer');
  assert.match(flow, /actionConfirmTimer\s*=\s*setTimeout/, 'prompt must auto-timeout');
  assert.match(flow, /resolve\(false\)/, 'timeout/cancel resolves to NOT confirmed');
  const wire = overlay.slice(overlay.indexOf('AGENT_DANGEROUS_ACTIONS.has(action.action)'));
  assert.match(wire, /await requestActionConfirmation\(action\)/);
  assert.match(wire, /if \(!confirmed\)/, 'unconfirmed path must not fall through to execution');
});

test('cancel/timeout marks the session cancelled and returns before any keystroke', () => {
  const gate = overlay.indexOf('AGENT_DANGEROUS_ACTIONS.has(action.action)');
  assert.notEqual(gate, -1, 'confirm gate must exist in the agent flow');
  const cancelledIdx = overlay.indexOf("'Cancelled'", gate);
  assert.notEqual(cancelledIdx, -1, 'cancel must be user-visible');
  const firstDispatch = overlay.indexOf('attemptKeyboardAction', gate);
  assert.ok(firstDispatch > cancelledIdx, 'cancellation precedes execution');
  const between = overlay.slice(cancelledIdx, firstDispatch);
  assert.match(between, /return;/, 'cancel path must return before execution');
  assert.match(between, /fadeAndHide/, 'cancel dismisses the overlay promptly');
});

test('confirmed dispatch window: all four actions, visible success, recoverable failure', () => {
  const start = overlay.indexOf('let executed = false');
  assert.notEqual(start, -1, 'dispatch block must exist');
  const end = overlay.indexOf("invoke('save_history_entry'", start);
  assert.notEqual(end, -1, 'agent flow must end by saving history');
  const dispatch = overlay.slice(start, end);
  assert.match(dispatch, /await attemptPaste\(action\.content/, 'insert/rewrite dispatch');
  assert.match(dispatch, /attemptKeyboardAction\('delete_chars'/);
  assert.match(dispatch, /attemptKeyboardAction\('select_all'/);
  assert.match(dispatch, /attemptKeyboardAction\('submit'/);
  assert.ok(!dispatch.includes('requestActionConfirmation'), 'dispatch must not re-confirm');
  assert.ok(!dispatch.includes('hide_overlay'), 'dispatch must not hide the overlay');
  assert.ok(!dispatch.includes('prepareForeground'), 'no staged focus hand-back');
  assert.match(dispatch, /setState\('success'\)/, 'success state must be surfaced');
  assert.match(
    dispatch,
    /setStatusMessage\(executed \? 'Executed' : 'Done'\)/,
    'success must be visible in status',
  );
  assert.match(dispatch, /handlePasteFailure\(pending, err/, 'failures must enter recovery');
  assert.match(dispatch, /kind: 'text'/, 'text recoverable as paste retry');
  assert.match(dispatch, /kind: 'keyboard'/, 'keyboard recoverable as keyboard retry');
});

test('copy agent action is one-click and never touches the confirm gate', () => {
  const copyBranch = overlay.indexOf("action.action === 'copy'");
  assert.notEqual(copyBranch, -1);
  const gate = overlay.indexOf('AGENT_DANGEROUS_ACTIONS.has(action.action)');
  assert.ok(copyBranch < gate || copyBranch > gate, 'copy branch exists in the agent flow');
  const copyWindowEnd = overlay.indexOf('fadeAndHide', copyBranch);
  const copyWindow = overlay.slice(copyBranch, copyWindowEnd > 0 ? copyWindowEnd : copyBranch + 500);
  assert.match(copyWindow, /invoke\('copy_text'/);
  assert.ok(!copyWindow.includes('requestActionConfirmation'));
});

test('non-destructive actions are never members of the dangerous set', () => {
  for (const action of ['insert', 'rewrite', 'copy']) {
    const setLine = overlay.match(/AGENT_DANGEROUS_ACTIONS\s*=\s*new Set\(\[([^\]]*)\]\)/);
    assert.ok(!setLine[1].includes(action), `${action} is not a destructive action`);
  }
});

test('retry re-runs the exact pending action against the validated target', () => {
  const retry = overlay.slice(overlay.indexOf('async function retryPendingInjection'));
  // Retry is a deliberate second click, so it carries allowClipboardLoss:
  // it is the user's consent to replace an image/file clipboard.
  assert.match(retry, /attemptPaste\(pending\.text, pending\.monitorAutoLearn, true\)/);
  assert.match(retry, /attemptKeyboardAction\(pending\.action, pending\.charCount\)/);
  assert.match(retry, /handlePasteFailure\(pending, err/, 'a failed retry re-enters recovery');
  const args = overlay.slice(overlay.indexOf('function injectionArgsWithTarget'));
  assert.match(args, /targetWindow/, 'injection args must carry the captured target');
});
