// Release integrity: the four version declarations that ship together must
// always agree. They drifted once before (web/package.json / Cargo.toml at
// 1.25.1 while tauri.conf.json said 1.27.1), which is exactly the class of
// silent inconsistency this test exists to fail loudly on.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');

function parseJson(rel) {
  return JSON.parse(readFileSync(join(root, rel), 'utf8'));
}

function cargoVersion() {
  const toml = readFileSync(join(root, 'src-tauri', 'Cargo.toml'), 'utf8');
  const match = toml.match(/(^|\n)\s*version\s*=\s*"([^"]+)"/);
  assert.ok(match, 'Cargo.toml must declare a package version');
  return match[2];
}

function lockVersion() {
  const lock = readFileSync(join(root, 'src-tauri', 'Cargo.lock'), 'utf8');
  const match = lock.match(/name = "fluence-windows"\r?\nversion = "([^"]+)"/);
  assert.ok(match, 'Cargo.lock must contain the fluence-windows package');
  return match[1];
}

test('all four version declarations agree', () => {
  const versions = {
    'src-tauri/Cargo.toml': cargoVersion(),
    'src-tauri/Cargo.lock': lockVersion(),
    'root package.json': parseJson('package.json').version,
    'web/package.json': parseJson('web/package.json').version,
    'src-tauri/tauri.conf.json': parseJson('src-tauri/tauri.conf.json').version,
  };
  const unique = new Set(Object.values(versions));
  assert.ok(
    unique.size === 1,
    `all release surfaces must report the same version, got: ${JSON.stringify(versions)}`,
  );
  const v = Object.values(versions)[0];
  assert.match(v, /^\d+\.\d+\.\d+$/, `version must be semver-ish, got ${v}`);
});
