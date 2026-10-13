// S7-rem tests: sign-in/out failure toasts must never render raw provider
// strings. Extracts the REAL `describeSignInError` from src/js/settings.js and
// asserts the mapping — same harness as credential-target-parity.test.mjs.
// Run: node --test tests/signin-toast-sanitization.test.mjs
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { extractFunction, evalFunctions } from './extract-js-function.mjs';

const ROOT = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
const settingsPath = path.join(ROOT, 'src/js/settings.js');
const { describeSignInError } = evalFunctions(
  [extractFunction(settingsPath, 'describeSignInError')],
  '{ describeSignInError }',
);

const M1 = 'Google did not return a refresh token. If you have authorized Fluence before, revoke its access at myaccount.google.com/permissions and try signing in again.';

// The actionable no-refresh-token message is user-facing by design and must
// survive verbatim — including its own remedy URL.
test('M1 actionable message survives verbatim, URL intact', () => {
  assert.equal(describeSignInError('Error: ' + M1), M1);
  assert.equal(describeSignInError(M1, 'Sign-out'), M1);
});

// reqwest-style errors embed the request URL. It must never reach a toast.
test('request URL errors are mapped, URL never rendered', () => {
  const raw =
    'account lookup failed: error sending request for url (https://oauth2.googleapis.com/token): connection closed';
  const out = describeSignInError(raw);
  assert.doesNotMatch(out, /https?:\/\//);
  assert.doesNotMatch(out, /oauth2\.googleapis/);
  assert.match(out, /Google account/);
});

// Token-endpoint failures can carry provider bodies.
test('provider JSON bodies are mapped, never rendered', () => {
  const raw = 'token endpoint HTTP 503: {"error": {"code": 503, "message": "backend_error_x9"}}';
  const out = describeSignInError(raw);
  assert.doesNotMatch(out, /backend_error_x9/);
  assert.doesNotMatch(out, /[{}]/);
});

// Markup is cut the same way.
test('HTML error pages are mapped', () => {
  const out = describeSignInError('<html><head><title>503 Service Unavailable</title></head></html>');
  assert.doesNotMatch(out, /<html/);
});

// Short plain internal errors stay visible with the action prefix.
test('short plain errors keep the action label', () => {
  assert.equal(describeSignInError('sign-in did not produce an access token'), 'Sign-in failed: sign-in did not produce an access token');
  assert.equal(describeSignInError('vault locked', 'Sign-out'), 'Sign-out failed: vault locked');
});

// Long diagnostics are dumps, not messages.
test('over-long diagnostics collapse to the generic fallback', () => {
  const out = describeSignInError('x'.repeat(500));
  assert.equal(out, 'Sign-in failed. Please try again.');
});

// Empty and odd inputs never crash and never render raw.
test('empty and non-string inputs are safe', () => {
  assert.equal(describeSignInError(''), 'Sign-in failed. Please try again.');
  assert.equal(describeSignInError(null), 'Sign-in failed. Please try again.');
  assert.equal(describeSignInError(undefined, 'Sign-out'), 'Sign-out failed. Please try again.');
});

// The default action is Sign-in when the caller passes nothing.
test('action defaults to Sign-in', () => {
  assert.match(describeSignInError('some unknown short failure'), /^Sign-in failed/);
});
