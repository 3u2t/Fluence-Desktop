# Release notes template

`release.yml` publishes its `releaseBody` for the tagged version. Keep this
file and the inline block in `.github/workflows/release.yml` semantically in
sync — this markdown is the reviewable copy reviewers read first, without
sifting through the workflow YAML.

Copy the template below and edit it per release. Never add release notes that
contain secrets, internal hostnames, or model download URLs that are not
already pinned by digest in `src-tauri/moonshine-v2-server/VENDOR.json`.

---

```markdown
## <headline for this release>

<2-3 sentence plain-language summary of what changed and why it matters.>

> **Why this is vX.Y.Z:** <one line explaining renamed/rereleased versions,
> if applicable; drop for a clean version jump.>

### Data safety

- **<User-visible data-safety fix, in user terms>**

### <Feature-area section, e.g. Cloud sync correctness>

- **<Fix or change, one bullet each, user orientation>**

### For the technical minded

<One short paragraph: on-disk/wire compatibility, and where a security
reviewer can find the hardening changes (e.g. CLAUDE-AUDIT.md or PR link).>
```
