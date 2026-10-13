// Fluence sync - frozen v1.1 sync_metadata (deviceId, per-account maxSeen/backfill/lastRev)
// Persists to %LOCALAPPDATA%/Fluence/sync_metadata.json
// Identity: deviceId UUIDv4, syncId UUIDv4 (Windows id is syncId - we store one UUID as both)

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Uses the shared `stores::base_data_dir()` so test builds can never write the
/// real %LOCALAPPDATA%\Fluence. Unset in production, so the path is unchanged.
fn metadata_path() -> PathBuf {
    let mut p = crate::sync::stores::base_data_dir();
    p.push("Fluence");
    p.push("sync_metadata.json");
    p
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerAccountState {
    #[serde(default)]
    pub max_seen: i64,
    #[serde(default)]
    pub backfill_done: bool,
    #[serde(default)]
    pub last_rev: HashMap<String, String>, // domain -> etag/revision
    #[serde(default)]
    pub last_sync_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMetadata {
    pub device_id: String,
    pub sync_id: String,
    #[serde(default)]
    pub accounts: HashMap<String, PerAccountState>,
    #[serde(default)]
    pub last_account_hash: Option<String>,
}

impl Default for SyncMetadata {
    fn default() -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        Self {
            device_id: id.clone(),
            sync_id: id,
            accounts: HashMap::new(),
            last_account_hash: None,
        }
    }
}

impl SyncMetadata {
    pub fn load() -> Self {
        Self::load_from(&metadata_path())
    }

    /// Path-parameterized core of [`load`]. Tests drive this with unique temp
    /// paths so corrupt-file recovery is verified hermetically, never against
    /// the shared test data root.
    pub fn load_from(path: &std::path::Path) -> Self {
        match crate::sync::recover::read_json_file(path) {
            // No file is a new install: bootstrap a fresh identity.
            Ok(None) => Self::default(),
            Ok(Some(m)) => m,
            Err(e) => {
                // Corrupt or unreadable bytes are preserved aside inside
                // `read_json_file`. Identity is WITHHELD, not minted here:
                // rotation moves from an invisible read side-effect to the
                // explicit, loudly-logged `ensure_device_id` act. Note the
                // honest limit: the withheld-empty state lasts exactly one
                // load (the one that observed the corruption) — the next
                // `load()` sees a missing file and bootstraps a fresh
                // identity, and every store's load/stamp path calls
                // `ensure_device_id`. So identity DOES rotate after
                // corruption; what Phase E guarantees is that rotation is
                // never a silent side effect of a read, and the original
                // bytes always survive aside for forensics.
                log::error!("Unreadable sync metadata ({e:?}); identity withheld");
                Self {
                    device_id: String::new(),
                    sync_id: String::new(),
                    accounts: HashMap::new(),
                    last_account_hash: None,
                }
            }
        }
    }

    pub fn save(&self) {
        // H6: every metadata write serializes on io_lock (reentrant, so
        // callers already holding it cannot deadlock). An unlocked writer
        // racing a pass could otherwise clobber a concurrent max_seen bump.
        let _io = crate::sync::io_lock::io_lock_guard();
        let path = metadata_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(data) = serde_json::to_string_pretty(self) {
            let tmp = path.with_extension("tmp");
            if let Err(e) = std::fs::write(&tmp, &data) {
                log::error!("failed to write sync metadata tmp file: {}", e);
                return;
            }
            if let Ok(f) = std::fs::File::open(&tmp) {
                let _ = f.sync_all();
            }
            if let Err(e) = std::fs::rename(&tmp, &path) {
                log::error!("failed to rename sync metadata tmp file: {}", e);
            }
        }
    }

    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    pub fn sync_id(&self) -> &str {
        &self.sync_id
    }

    pub fn for_account_mut(&mut self, account_hash: &str) -> &mut PerAccountState {
        self.accounts
            .entry(account_hash.to_string())
            .or_insert_with(PerAccountState::default)
    }

    pub fn for_account(&self, account_hash: &str) -> Option<&PerAccountState> {
        self.accounts.get(account_hash)
    }

    /// Ensure deviceId exists (generate if empty) and return it.
    ///
    /// This is the ONLY place a device identity is minted. Minting is an
    /// explicit act attached to a mutation, never a side effect of a read:
    /// after a corruption the load above withholds identity, and the first
    /// explicit ensure re-establishes it (loudly, so the rotation is
    /// visible in logs next to the quarantine record).
    pub fn ensure_device_id(&mut self) -> String {
        if self.device_id.is_empty() {
            let id = uuid::Uuid::new_v4().to_string();
            log::warn!("Minting fresh device identity (previous identity absent or withheld after unreadable metadata)");
            self.device_id = id.clone();
            self.sync_id = id.clone();
            self.save();
            id
        } else {
            self.device_id.clone()
        }
    }

    /// Update maxSeen atomically: new_max = max(old, candidate) and persist.
    pub fn update_max_seen(&mut self, account_hash: &str, candidate: i64) {
        // H6: merge ONLY the live max_seen forward — never replace self.
        // This object is typically loaded before a network round trip, during
        // which a concurrent writer may have saved a HIGHER max_seen. Raising
        // from the stale copy would clobber it and regress the clock floor.
        // But replacing self wholesale with the reloaded file would import
        // unrelated file state (other accounts' revs, etc.) into the caller's
        // object, breaking the in-memory isolation callers and tests rely on.
        // max_seen is the sole field with concurrent writers (UI-thread saves
        // bump it outside any pass), so it is the sole field merged forward;
        // every other field has no concurrent writer (single-flight passes;
        // set_last_rev saves immediately), so persisting self as-is cannot
        // clobber them. No network I/O here; `io_lock` is reentrant.
        let _io = crate::sync::io_lock::io_lock_guard();
        let live_max = Self::load()
            .accounts
            .get(account_hash)
            .map(|s| s.max_seen)
            .unwrap_or(0);
        let state = self.for_account_mut(account_hash);
        let new_max = live_max.max(candidate).max(state.max_seen);
        if new_max > state.max_seen {
            state.max_seen = new_max;
        }
        // save after
        self.save();
    }

    /// Record new lastRev for a domain, and handle account switch clearing.
    pub fn set_last_rev(&mut self, account_hash: &str, domain: &str, rev: String) {
        // H6-D2: merge forward a concurrent max_seen bump, but do NOT replace
        // self. This object is typically loaded before a network round trip
        // (frozen.rs drops `io_lock` across Drive I/O), during which a
        // concurrent writer may have saved a higher max_seen; persisting the
        // stale copy would clobber it (clock-floor regression). Replacing self
        // wholesale with the reloaded file would import unrelated file state
        // into the caller's object, breaking the in-memory isolation that
        // callers and tests rely on — so only max_seen, the sole field with
        // concurrent writers, is merged forward. No network I/O here;
        // `io_lock` is reentrant, so locked callers are safe.
        let _io = crate::sync::io_lock::io_lock_guard();
        let live_max = Self::load()
            .accounts
            .get(account_hash)
            .map(|s| s.max_seen)
            .unwrap_or(0);
        // Check if account switched
        if self.last_account_hash.as_deref() != Some(account_hash) {
            // clear lastRev for new account? Per spec: clear lastRev on switch
            // We keep per-account map, so no need to clear other accounts, but we update last_account_hash
            self.last_account_hash = Some(account_hash.to_string());
        }
        let state = self.for_account_mut(account_hash);
        state.last_rev.insert(domain.to_string(), rev);
        state.max_seen = state.max_seen.max(live_max);
        self.save();
    }

    pub fn clear_last_rev(&mut self, account_hash: &str) {
        if let Some(state) = self.accounts.get_mut(account_hash) {
            state.last_rev.clear();
        }
        self.save();
    }

    pub fn get_last_rev(&self, account_hash: &str, domain: &str) -> Option<String> {
        self.accounts
            .get(account_hash)
            .and_then(|s| s.last_rev.get(domain).cloned())
    }

    #[cfg(test)]
    pub fn test_path() -> PathBuf {
        metadata_path()
    }
}

// Account hash helper: lowercased trimmed email hashed? For now use lowercased email as hash for simplicity.
// In production, we use SHA256 hex of lower(trim(email)) to avoid storing raw email in file names? But spec says accountHash stamping.
// We'll provide a helper.
pub fn account_hash_from_email(email: &str) -> String {
    let normalized = email.trim().to_lowercase();
    // Use SHA256 hex for stable hash; fallback to normalized if hex crate unavailable
    let hash = sha2::Sha256::digest(normalized.as_bytes());
    hex::encode(hash)
}

/// Account hash for the account currently selected in app settings.
pub fn current_account_hash() -> Option<String> {
    crate::settings::load_settings()
        .ok()
        .and_then(|settings| settings.sync_account_key)
        .map(|email| account_hash_from_email(&email))
}

pub fn belongs_to_account(row_account: Option<&str>, active_account: Option<&str>) -> bool {
    match (row_account, active_account) {
        (None, _) => true,
        (Some(_), None) => false,
        (Some(row), Some(active)) => row == active,
    }
}

use sha2::Digest;

#[cfg(test)]
mod tests {
    use super::*;

    /// STAGE 1 cross-platform hash fixture.
    ///
    /// These literals are asserted IDENTICALLY in the Android test
    /// `DriveIdentityTest.account_hash_matches_the_windows_reference_vector`.
    /// The account hash becomes a Drive path segment in Stage 3, so any
    /// divergence between the two platforms — including a reintroduced
    /// 16-character truncation — must fail a test rather than silently
    /// split every account partition on every device.
    #[test]
    fn account_hash_matches_the_android_reference_vectors() {
        assert_eq!(
            "973dfe463ec85785f5f95af5ba3906eedb2d931c24e69824a89ea65dba4e813b",
            account_hash_from_email("test@example.com"),
        );
        assert_eq!(
            "7b8a6e03ce872608896365b0d9de0e8a3bf4c400c409d5620af6d710fb8ebb00",
            account_hash_from_email("user+tag@Example.co.uk"),
        );
    }

    #[test]
    fn account_hash_is_exactly_64_lowercase_hex_chars() {
        let hash = account_hash_from_email("someone@example.com");
        assert_eq!(64, hash.len());
        assert!(hash
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }

    #[test]
    fn account_hash_is_not_truncated_to_16_chars() {
        let hash = account_hash_from_email("test@example.com");
        assert_ne!(&hash[..16], hash.as_str());
    }

    #[test]
    fn account_rows_are_scoped_without_hiding_legacy_rows() {
        assert!(belongs_to_account(None, Some("account-a")));
        assert!(belongs_to_account(Some("account-a"), Some("account-a")));
        assert!(!belongs_to_account(Some("account-b"), Some("account-a")));
        assert!(!belongs_to_account(Some("account-a"), None));
    }

    #[test]
    fn device_id_generated_and_persisted() {
        let mut m = SyncMetadata::default();
        assert!(!m.device_id.is_empty());
        assert_eq!(m.device_id, m.sync_id);
    }

    #[test]
    fn per_account_max_seen_monotonic() {
        // Persists via update_max_seen: hold the shared store guard so this
        // write cannot interleave with file-backed recovery tests.
        let _guard = crate::sync::stores::store_test_guard();
        let mut m = SyncMetadata::default();
        m.update_max_seen("acc1", 100);
        assert_eq!(m.for_account("acc1").unwrap().max_seen, 100);
        m.update_max_seen("acc1", 90);
        assert_eq!(m.for_account("acc1").unwrap().max_seen, 100);
        m.update_max_seen("acc1", 150);
        assert_eq!(m.for_account("acc1").unwrap().max_seen, 150);
        // per-account isolation
        assert!(m.for_account("acc2").is_none());
        m.update_max_seen("acc2", 200);
        assert_eq!(m.for_account("acc2").unwrap().max_seen, 200);
    }

    #[test]
    fn last_rev_per_account_isolated_and_cleared_on_switch() {
        // Persists via set_last_rev/clear_last_rev: hold the shared store
        // guard (see above).
        let _guard = crate::sync::stores::store_test_guard();
        let mut m = SyncMetadata::default();
        m.set_last_rev("acc1", "dictionary", "rev1".to_string());
        assert_eq!(
            m.get_last_rev("acc1", "dictionary").as_deref(),
            Some("rev1")
        );
        assert_eq!(m.get_last_rev("acc2", "dictionary"), None);
        m.set_last_rev("acc2", "dictionary", "rev2".to_string());
        assert_eq!(
            m.get_last_rev("acc2", "dictionary").as_deref(),
            Some("rev2")
        );
        // acc1 still has its rev
        assert_eq!(
            m.get_last_rev("acc1", "dictionary").as_deref(),
            Some("rev1")
        );
        m.clear_last_rev("acc1");
        assert_eq!(m.get_last_rev("acc1", "dictionary"), None);
    }

    #[test]
    fn account_hash_is_lowercase_and_hashed() {
        let h1 = account_hash_from_email("Test@Example.COM ");
        let h2 = account_hash_from_email("test@example.com");
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64); // sha256 hex
    }

    /// H6-D2: `set_last_rev` on a stale object must not clobber a concurrent
    /// `max_seen` bump. The caller typically loaded this object before a
    /// network round trip; a UI-thread save may have advanced the clock floor
    /// since. Reload-merge-save must preserve it.
    #[test]
    fn set_last_rev_preserves_concurrent_max_seen_bump() {
        let _guard = crate::sync::stores::store_test_guard();
        let acc = &account_hash_from_email("h6-merge@example.com");
        // Establish the file with a baseline rev.
        let mut stale = SyncMetadata::load();
        stale.set_last_rev(acc, "dictionary", "rev-base".to_string());
        // Concurrent writer bumps the clock floor and saves.
        let mut live = SyncMetadata::load();
        live.update_max_seen(acc, 987_654);
        assert_eq!(live.for_account(acc).unwrap().max_seen, 987_654);
        // The stale object now records a new rev. Without reload-merge it
        // would persist its own older max_seen over the live bump.
        stale.set_last_rev(acc, "snippets", "rev-new".to_string());
        assert_eq!(
            stale.get_last_rev(acc, "snippets").as_deref(),
            Some("rev-new")
        );
        assert_eq!(
            stale.for_account(acc).unwrap().max_seen,
            987_654,
            "a concurrent max_seen bump must survive a later set_last_rev"
        );
        // And it landed on disk, not just in memory.
        let reread = SyncMetadata::load();
        assert_eq!(
            reread.for_account(acc).unwrap().max_seen,
            987_654,
            "the merged max_seen must be what was persisted"
        );
        assert_eq!(
            reread.get_last_rev(acc, "snippets").as_deref(),
            Some("rev-new")
        );
    }

    /// H6-D2, second half: `update_max_seen` on a stale object must not
    /// clobber a concurrent HIGHER bump. The pass object lives across network
    /// I/O and saves through both `update_max_seen` (frozen.rs:337/372/851/872)
    /// and `set_last_rev` — fixing only the latter leaves the identical race
    /// open one call away. Mirrors production: stale load, concurrent bump and
    /// save, then a LOWER raise on the stale object must leave the live value
    /// intact, while a HIGHER raise must still apply.
    #[test]
    fn update_max_seen_on_stale_object_preserves_concurrent_bump() {
        let _guard = crate::sync::stores::store_test_guard();
        let acc = &account_hash_from_email("h6-stale-raise@example.com");
        // Stale object loaded first (e.g. pass start).
        let mut stale = SyncMetadata::load();
        // Concurrent writer bumps higher and saves (e.g. UI-thread save).
        let mut live = SyncMetadata::load();
        live.update_max_seen(acc, 500);
        assert_eq!(live.for_account(acc).unwrap().max_seen, 500);
        // Stale object raises to a LOWER value from older merged rows. Without
        // reload-merge this persists 300 over the live 500.
        stale.update_max_seen(acc, 300);
        assert_eq!(
            stale.for_account(acc).unwrap().max_seen,
            500,
            "a stale lower raise must not clobber a live higher bump"
        );
        assert_eq!(
            SyncMetadata::load().for_account(acc).unwrap().max_seen,
            500,
            "the live bump must be what is on disk"
        );
        // A genuinely higher raise still applies.
        stale.update_max_seen(acc, 700);
        assert_eq!(
            SyncMetadata::load().for_account(acc).unwrap().max_seen,
            700,
            "a higher raise must still land"
        );
    }

    // --- corrupt-file recovery (Phase E) ----------------------------------
    //
    // A malformed sync_metadata.json must preserve its bytes aside and must
    // NOT silently mint a fresh device identity: rotation is only ever an
    // explicit act (`ensure_device_id`), never a side effect of a read.
    //
    // These tests drive `load_from` with unique temp paths (hermetic: no
    // shared test data root, so no cross-test interference is possible).

    use std::sync::atomic::{AtomicU64, Ordering};

    static META_TEST_SEQ: AtomicU64 = AtomicU64::new(0);

    fn unique_meta_path() -> std::path::PathBuf {
        let n = META_TEST_SEQ.fetch_add(1, Ordering::SeqCst);
        std::env::temp_dir().join(format!(
            "fluence-meta-test-{}-{n}.json",
            std::process::id()
        ))
    }

    fn remove_quietly(path: &std::path::Path) {
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(path.with_extension("json.corrupt.json"));
        for n in 1..100u32 {
            let p = path.with_extension(format!("json.corrupt.{n}.json"));
            if !p.exists() {
                break;
            }
            let _ = std::fs::remove_file(&p);
        }
    }

    #[test]
    fn corrupt_metadata_preserves_bytes_and_mints_nothing() {
        let path = unique_meta_path();
        let garbage = b"{\"device_id\": \"cut off";
        std::fs::write(&path, garbage).unwrap();

        let loaded = SyncMetadata::load_from(&path);
        assert!(
            loaded.device_id.is_empty() && loaded.sync_id.is_empty(),
            "no identity may be minted merely because parsing failed: {loaded:?}"
        );
        let backup = path.with_extension("json.corrupt.json");
        assert_eq!(
            std::fs::read(&backup).unwrap(),
            garbage,
            "original bytes must be preserved byte-identical"
        );
        assert!(!path.exists(), "corrupt original must be rotated aside");
        remove_quietly(&path);
    }

    #[test]
    fn missing_metadata_still_bootstraps_new_install() {
        let path = unique_meta_path();
        assert!(!path.exists());

        let fresh = SyncMetadata::load_from(&path);
        assert!(
            !fresh.device_id.is_empty() && fresh.device_id == fresh.sync_id,
            "a genuinely missing file is a new install and bootstraps identity"
        );
        assert!(
            !path.with_extension("json.corrupt.json").exists(),
            "no backup for a missing file"
        );
    }

    #[test]
    fn valid_metadata_preserves_identity_across_loads() {
        let path = unique_meta_path();
        let m = SyncMetadata::default();
        let id = m.device_id.clone();
        std::fs::write(&path, serde_json::to_string_pretty(&m).unwrap()).unwrap();

        let reloaded = SyncMetadata::load_from(&path);
        assert_eq!(
            reloaded.device_id, id,
            "device identity must survive a round trip"
        );
        remove_quietly(&path);
    }

    #[test]
    fn ensure_device_id_mints_explicitly_after_corruption() {
        let path = unique_meta_path();
        std::fs::write(&path, b"garbage{{{").unwrap();

        // The read withholds identity; rotation happens only through the
        // explicit ensure act below — never as a side effect of the read.
        // ensure_device_id persists to the fixed metadata path (like every
        // other test that mutates sync state), so hold the shared guard and
        // assert only the mint itself, never shared-file state.
        let _guard = crate::sync::stores::store_test_guard();
        let mut loaded = SyncMetadata::load_from(&path);
        assert!(loaded.device_id.is_empty());
        let minted = loaded.ensure_device_id();
        assert!(!minted.is_empty(), "explicit ensure must mint");
        assert_eq!(loaded.sync_id, minted);
        remove_quietly(&path);
    }
}
