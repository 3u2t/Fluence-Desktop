// IPC gating invariants: every gated `#[tauri::command]` must call
// require_caller with the intended per-window labels, close to the fn body.
// Guards against future commands silently shipping ungated.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const src = (p) => readFileSync(join(root, 'src-tauri', 'src', p), 'utf8');

const O = 'crate::acl::OVERLAY_WINDOW';
const M = 'crate::acl::MAIN_WINDOW';
const W = 'crate::acl::WIZARD_WINDOW';

const table = [
  ['clipboard.rs', 'inject_text', [O]],
  ['clipboard.rs', 'copy_text', [O, M]],
  ['clipboard.rs', 'execute_keyboard_action', [O]],
  ['clipboard.rs', 'grab_active_selection', [O]],
  ['clipboard.rs', 'capture_target_window', [O]],
  ['workflow.rs', 'stop_and_transcribe_recording', [O]],
  ['workflow.rs', 'finish_transcription_flow', [O]],
  ['workflow.rs', 'retry_transcription_flow', [O]],
  ['app_icon.rs', 'get_foreground_app_icon', [O]],
  ['overlay.rs', 'show_overlay', [O]],
  ['overlay.rs', 'hide_overlay', [O]],
  ['overlay.rs', 'set_overlay_style', [O]],
  ['overlay.rs', 'show_main_window', [M, W]],
  ['overlay.rs', 'hide_main_window', [M]],
  ['overlay.rs', 'minimize_main_window', [M]],
  ['overlay.rs', 'toggle_maximize_main_window', [M]],
  ['overlay.rs', 'show_wizard_window', [M, W]],
  ['overlay.rs', 'minimize_wizard', [W]],
  ['overlay.rs', 'close_wizard', [W]],
  ['offline_downloader.rs', 'get_offline_model_status', [M]],
  ['offline_downloader.rs', 'get_moonshine_v2_small_model_status', [M]],
  ['offline_downloader.rs', 'get_moonshine_v2_medium_model_status', [M]],
];

const files = new Set(table.map(([f]) => f));

function gateFor(file, fn) {
  const srcText = src(file);
  const start = srcText.indexOf(`fn ${fn}(`);
  assert.notEqual(start, -1, `${file}: ${fn} not found`);
  const region = srcText.slice(start, start + 900);
  const gateIdx = region.indexOf('crate::acl::require_caller(&window');
  assert.notEqual(gateIdx, -1, `${file}: ${fn} has no require_caller gate`);
  const rest = region.slice(gateIdx);
  const end = rest.indexOf(';');
  return rest.slice(0, end);
}

test('each gated command calls require_caller with exactly the intended labels', () => {
  for (const [file, fn, labels] of table) {
    const gate = gateFor(file, fn);
    for (const label of labels) {
      assert.ok(
        gate.includes(label),
        `${file}: ${fn} must allow ${label}; gate was: ${gate}`,
      );
    }
    const other = [O, M, W].filter((x) => !labels.includes(x));
    for (const label of other) {
      assert.ok(
        !gate.includes(label),
        `${file}: ${fn} must NOT allow ${label}; gate was: ${gate}`,
      );
    }
  }
});

test('no accidental empty allow-lists', () => {
  for (const file of files) {
    const text = src(file);
    assert.ok(
      text.includes('crate::acl::require_caller(&window'),
      `${file} should contain gated commands`,
    );
    assert.ok(!text.includes('&[]'), `${file}: empty allow-list found`);
  }
});

test('acl.rs no longer advertises an unfixed open list', () => {
  const acl = src('acl.rs');
  assert.ok(!acl.includes('NOT FIXED'), 'stale NOT-FIXED section should be refreshed');
  assert.ok(acl.includes('Deliberately open'), 'open commands must be documented');
  assert.ok(!acl.includes('1.25.1'), 'caller table version stamp must be current');
});
