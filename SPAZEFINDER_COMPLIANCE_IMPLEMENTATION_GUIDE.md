# SpazeFinder Open-Source Legal, Privacy, Security, and Release Compliance Guide

## Purpose

This document defines the implementation and release requirements for **SpazeFinder**, a free and open-source desktop application distributed under the **MIT License**.

The goal is to reduce legal, privacy, security, and distribution risk before publishing SpazeFinder for public use, especially in the Philippines.

SpazeFinder is currently designed as a **local-first desktop utility**. It scans a user's local filesystem, displays storage usage, and allows user-directed filesystem actions such as opening, revealing, renaming, moving, and sending files to Trash or the Recycle Bin.

The compliance strategy should preserve that local-first architecture.

This document is an engineering and release checklist. It is not a substitute for formal legal advice.

---

# 1. Core Compliance Principle

SpazeFinder should remain:

- Free to use.
- Open source.
- MIT licensed.
- Local-first.
- Offline-capable.
- Free of mandatory accounts.
- Free of advertising.
- Free of telemetry by default.
- Free of analytics by default.
- Free of cloud scan storage.
- Free of automatic uploads of filesystem information.

The simplest and strongest privacy model is:

> The developer does not receive the user's scanned filesystem information.

Avoid creating unnecessary compliance obligations by introducing server-side collection where the application does not need it.

---

# 2. Required Repository Files

Before public release, the repository should contain the following files:

```text
SpazeFinder/
├── LICENSE
├── README.md
├── PRIVACY.md
├── DISCLAIMER.md
├── SECURITY.md
├── THIRD_PARTY_NOTICES.md
├── PRODUCT.md
├── src/
└── src-tauri/
```

Optional but useful later:

```text
CONTRIBUTING.md
CHANGELOG.md
docs/
```

The required files are described below.

---

# 3. MIT License

Create:

```text
/LICENSE
```

Use the standard MIT License without adding custom restrictions.

Recommended copyright line:

```text
Copyright (c) 2026 Emmanuel Millave
```

The MIT License allows:

- Use.
- Copying.
- Modification.
- Distribution.
- Commercial use.
- Private use.
- Sublicensing.

It also includes important warranty and liability language.

Do not modify the MIT License to prohibit commercial use, closed-source forks, or resale. Doing so would no longer be the standard MIT License.

The README should include:

```markdown
## License

Copyright © 2026 Emmanuel Millave.

SpazeFinder is free and open-source software licensed under the MIT License.
See [LICENSE](LICENSE) for details.
```

---

# 4. Privacy Notice

Create:

```text
/PRIVACY.md
```

The privacy notice must describe what the official SpazeFinder build actually does.

Do not copy a generic SaaS privacy policy.

## 4.1 Information Processed Locally

The notice should explain that SpazeFinder may process the following locally:

- File names.
- Folder names.
- Filesystem paths.
- File sizes.
- Directory structure.
- Drive or volume information.
- Disk usage statistics.

If SpazeFinder does not inspect file contents, state that clearly.

Example:

> SpazeFinder analyzes filesystem metadata required to calculate and display disk usage. The application does not intentionally read file contents as part of normal storage scanning.

---

## 4.2 No Developer-Side Collection

The notice should state:

> Official SpazeFinder builds do not transmit filenames, folder names, filesystem paths, file contents, scan results, or disk metadata to the developer or to third-party servers.

Also state:

- No user account is required.
- No advertising is included.
- No analytics is included.
- No telemetry is included.
- No cloud storage is included.
- No remote scan history is maintained.

Do not write:

> SpazeFinder never processes personal data.

That is too broad because filenames and paths may contain personal information.

Prefer:

> SpazeFinder processes filesystem information locally on the user's device and does not transmit that information to project-operated servers.

---

## 4.3 Local Preferences

If SpazeFinder stores preferences such as theme selection using local storage, disclose that.

Example:

> SpazeFinder may store local application preferences, such as theme settings, on the user's device.

---

## 4.4 Session Data

If the scan tree remains only in memory during the running session, document it.

Example:

> Scan results are maintained locally during the application session and are not uploaded to the developer.

If this changes in the future, update the privacy notice.

---

## 4.5 Official Builds vs Forks

Because the project is MIT licensed, third parties may modify and redistribute it.

Add:

> This privacy notice applies to official SpazeFinder source code and releases maintained by this project. Third-party forks, modified builds, or repackaged distributions may behave differently.

Do not claim that all software called "SpazeFinder" is guaranteed to preserve the same privacy behavior.

---

# 5. Privacy-by-Design Engineering Rules

The following rules should be treated as project requirements.

## 5.1 No Telemetry by Default

Official builds must not automatically transmit:

- Filenames.
- Folder names.
- Paths.
- File contents.
- Disk identifiers.
- Scan results.
- Usernames.
- Home-directory paths.
- Machine identifiers.
- Hardware fingerprints.
- IP-associated analytics information.

Do not add the following without a new privacy review:

- Sentry.
- PostHog.
- Google Analytics.
- Mixpanel.
- Remote logging.
- Cloud crash reporting.
- Usage tracking.
- Session replay.
- Advertising SDKs.

---

## 5.2 No Mandatory Internet Dependency

Core functionality should work offline.

The following should work without Internet access:

- Drive selection.
- Disk scanning.
- Tree rendering.
- File opening.
- Reveal in Finder or Explorer.
- Rename.
- Move.
- Trash or Recycle Bin actions.

If an update checker is added in the future, the core application must still function offline.

---

## 5.3 Do Not Read File Contents Unless Needed

The storage scanner should use filesystem metadata when possible.

Prefer:

```text
path
filename
file size
directory metadata
```

Avoid:

```text
opening file contents
parsing documents
reading media content
indexing file text
```

unless a future feature explicitly requires it.

Any future file-content inspection should trigger a privacy review.

---

## 5.4 Do Not Persist Scan History by Default

Avoid creating persistent storage such as:

```text
scan-history.json
scan-history.db
previous-scans.sqlite
```

unless a future feature clearly requires it.

If saved scans are added later:

- Make them optional.
- Store them locally.
- Allow users to delete them.
- Document retention behavior.
- Consider encryption where appropriate.
- Update `PRIVACY.md`.

---

# 6. User Authorization and Responsible Use

Create:

```text
/DISCLAIMER.md
```

The document should explain that users may only use SpazeFinder on systems they are allowed to access.

Recommended statement:

> Use SpazeFinder only on devices, drives, filesystems, and data that you own or are authorized to inspect or modify.

This reduces ambiguity around unauthorized use.

The application itself should not attempt to bypass operating-system security controls.

If a folder is inaccessible, report it as inaccessible.

Do not implement permission-bypass behavior.

---

# 7. Filesystem Safety Disclaimer

SpazeFinder performs actions that can affect user files.

The disclaimer should explain that users are responsible for reviewing filesystem actions.

Recommended language:

> SpazeFinder performs filesystem operations only at the user's direction. Users are responsible for reviewing files before moving, renaming, or deleting them and for maintaining appropriate backups of important data.

Also state:

> Files moved to Trash or the Recycle Bin may not always be recoverable.

Refer users to the MIT License for the formal warranty and liability disclaimer.

Do not claim that recovery is guaranteed.

---

# 8. Destructive Action Safety

The application should retain safe behavior around destructive operations.

## Required behavior

### Delete

Deletion should:

- Require user confirmation.
- Use Trash on macOS.
- Use Recycle Bin on Windows.
- Avoid permanent deletion by default.

### Rename

Rename should:

- Validate the new filename.
- Avoid silent overwrite.
- Reject invalid destinations where appropriate.

### Move

Move should:

- Avoid silently overwriting existing files.
- Surface errors clearly.
- Require explicit destination selection.

### Permanent Delete

Do not include permanent deletion in the default public release unless there is a strong product need.

If added later:

- Clearly distinguish it from Trash.
- Require explicit confirmation.
- Warn that recovery may be impossible.

---

# 9. Permissions and Least Privilege

SpazeFinder should follow the principle of least privilege.

## Windows

Do not automatically require Administrator privileges unless technically necessary.

Preferred behavior:

```text
Launch normally
    ↓
Scan accessible locations
    ↓
If protected locations are inaccessible
    ↓
Explain limitation
    ↓
Allow the user to choose whether to elevate
```

If the current release always runs elevated, document that behavior clearly and consider changing it in a future release.

---

## macOS

Do not tell users that Full Disk Access is mandatory unless it truly is.

Preferred behavior:

```text
Run normally
    ↓
Scan accessible locations
    ↓
Show inaccessible folders
    ↓
Explain that protected locations may require additional macOS permissions
```

Never attempt to bypass macOS privacy controls.

---

# 10. Security Policy

Create:

```text
/SECURITY.md
```

The file should explain how security vulnerabilities should be reported.

Include:

- Supported versions.
- Preferred reporting method.
- Information reporters should include.
- Responsible disclosure expectations.
- A request not to publish active vulnerabilities before a fix is available.
- A warning not to attach private filesystem information unnecessarily.

Example:

> When reporting a vulnerability, do not include real personal files, confidential filenames, access tokens, credentials, or full filesystem dumps unless absolutely necessary.

---

# 11. Privacy-Safe Bug Reporting

If GitHub Issues are enabled, add a warning in the issue template.

Recommended text:

> Before uploading screenshots, logs, or error messages, remove personal filenames, usernames, home-directory paths, account information, API keys, and other sensitive information.

Users may unintentionally expose private information through screenshots of SpazeFinder.

---

# 12. Third-Party Dependency Compliance

Create:

```text
/THIRD_PARTY_NOTICES.md
```

SpazeFinder uses third-party libraries. The MIT License for SpazeFinder does not replace their licenses.

Audit:

- Direct dependencies.
- Transitive dependencies.
- Icons.
- Fonts.
- Images.
- Other bundled resources.

For Rust dependencies, inspect:

```bash
cargo tree
cargo metadata
```

Recommended tooling:

```bash
cargo install cargo-deny
cargo deny check licenses
```

Also consider:

```bash
cargo install cargo-audit
cargo audit
```

Do not assume every dependency uses MIT.

Record any required notices or attribution in `THIRD_PARTY_NOTICES.md`.

---

# 13. Dependency Security

Before each release:

```bash
cargo audit
cargo deny check
cargo test
cargo build --release
```

Recommended GitHub features:

- Dependabot.
- Secret scanning where available.
- Vulnerability alerts.
- Branch protection.

Keep `Cargo.lock` committed for reproducible application builds.

---

# 14. Asset Licensing

Audit all bundled assets.

Examples:

```text
src-tauri/icons/
fonts/
logos/
images/
sounds/
```

Each asset must be one of the following:

- Created by the project owner.
- Public domain.
- Properly licensed for redistribution.
- Covered by an MIT-compatible or otherwise acceptable license.

"Free to download" does not automatically mean "free to redistribute."

Document third-party assets in `THIRD_PARTY_NOTICES.md`.

---

# 15. README Legal and Privacy Section

Add a visible section near the bottom of `README.md`.

Recommended structure:

```markdown
## Privacy

SpazeFinder processes filesystem information locally on your device.
Official releases do not include telemetry, analytics, advertising,
or cloud-based scan storage.

See [PRIVACY.md](PRIVACY.md) for details.

## Responsible Use

Use SpazeFinder only on devices and filesystems that you own or are
authorized to access.

Review files carefully before moving, renaming, or deleting them.
Maintain backups of important data.

See [DISCLAIMER.md](DISCLAIMER.md).

## Security

Security vulnerabilities should be reported according to
[SECURITY.md](SECURITY.md).

## License

Copyright © 2026 Emmanuel Millave.

SpazeFinder is free and open-source software licensed under the MIT License.
See [LICENSE](LICENSE) for details.
```

---

# 16. Do Not Add Unnecessary Consent Screens

For the current local-only architecture, do not add:

- Cookie banners.
- GDPR-style consent popups.
- Mandatory privacy checkboxes.
- Terms acceptance screens.
- Data deletion portals.
- Account consent workflows.
- Age verification.

These mechanisms solve problems SpazeFinder currently does not have.

The privacy notice should inform users accurately instead.

---

# 17. Philippine Data Privacy Considerations

SpazeFinder may encounter filenames or paths that contain personal information.

Examples:

```text
/Users/example/Documents/Resume.pdf
/Users/example/Medical/Lab-Result.pdf
/Users/example/Taxes/
```

The preferred compliance strategy is to keep that processing local.

The official application should not transmit this information to the developer.

If the project later introduces server-side processing, reassess:

- Data Privacy Act obligations.
- Personal Information Controller or Processor status.
- NPC registration requirements.
- Security obligations.
- Retention rules.
- Breach notification requirements.
- Data subject rights.
- Third-party processors.

Do not automatically register SpazeFinder with the NPC merely because it is open source.

Registration obligations depend on whether the developer or organization actually operates a covered personal-data processing system and meets applicable legal criteria.

---

# 18. Future Features That Require a New Privacy Review

The following changes must trigger a new privacy and legal review before release:

- User accounts.
- Login or authentication.
- Cloud synchronization.
- Remote scan storage.
- Online scan history.
- Telemetry.
- Analytics.
- Advertising.
- Remote logs.
- Automatic crash uploads.
- AI APIs.
- Uploading files for analysis.
- Uploading filenames or paths.
- Licensing servers.
- Device fingerprinting.
- Persistent machine identifiers.
- Online profiles.
- Automatic remote support diagnostics.
- File-content inspection.
- Plugins that transmit scan information.
- Automatic update systems that collect identifiable usage information.

Any PR introducing one of these should update:

```text
PRIVACY.md
README.md
DISCLAIMER.md
SECURITY.md
THIRD_PARTY_NOTICES.md
```

where applicable.

---

# 19. Crash Reporting Rule

Do not automatically upload crash reports containing filesystem paths.

A raw error may expose:

```text
C:\Users\Example\Documents\Medical\Record.pdf
```

or:

```text
/Users/example/Documents/Private/
```

If crash reporting is added later:

- Make it opt-in where practical.
- Redact usernames.
- Redact home-directory paths.
- Redact filenames where possible.
- Disclose the service provider.
- Update the privacy notice.
- Review third-party data processing.

---

# 20. No Device Fingerprinting

Do not collect or transmit:

- MAC address.
- Hardware serial number.
- Disk serial number.
- Machine UUID.
- Device fingerprint.
- Advertising identifier.

There is currently no legitimate reason for SpazeFinder to require these.

---

# 21. Open-Source Fork Disclaimer

MIT allows others to modify and redistribute the software.

The project should clearly distinguish official builds from third-party builds.

Recommended statement:

> Official SpazeFinder releases are distributed through the project's official GitHub repository and release channels. Modified or third-party distributions may include changes not reviewed or approved by the project maintainer.

Do not accept responsibility for third-party forks as though they were official releases.

---

# 22. Official Release Channels

The README should identify:

- Official GitHub repository.
- Official GitHub Releases page.

Do not encourage users to download binaries from random mirrors.

---

# 23. Release Integrity

Recommended release practice:

```text
Git tag
   ↓
GitHub Actions
   ↓
Build
   ↓
Tests
   ↓
License/security checks
   ↓
Release artifact
```

Use semantic versions such as:

```text
v1.0.0
v1.0.1
v1.1.0
```

---

# 24. Release Checksums

For distributed binaries, publish SHA-256 hashes.

Example:

```text
spazefinder-v1.0.0-windows-x64.exe
SHA256: ...

spazefinder-v1.0.0-macos-arm64.dmg
SHA256: ...
```

This allows users to verify artifact integrity.

---

# 25. Windows Code Signing

When practical, sign Windows release binaries with Authenticode.

This is not required merely to use the MIT License, but it improves trust and helps users verify the publisher.

---

# 26. macOS Signing and Notarization

For public macOS distribution, use:

- Developer ID signing.
- Apple notarization.

This reduces warnings and provides users with stronger assurance that the binary came from the expected publisher.

---

# 27. Secret Management

SpazeFinder currently should not require secrets.

Avoid committing:

```text
.env
*.pem
*.key
credentials.json
tokens
API keys
```

If future features require secrets:

- Do not hardcode them.
- Do not commit them.
- Use environment variables or OS-secure storage.
- Update `.gitignore`.
- Enable secret scanning where available.

---

# 28. Contributor Rules

If outside contributions are accepted, create:

```text
/CONTRIBUTING.md
```

Include:

> Do not introduce telemetry, tracking, advertising, remote scan uploads, cloud storage, or external processing of scan-derived information without prior project discussion and privacy review.

Also include:

> Do not commit real user data, screenshots containing private filenames, API keys, secrets, or confidential filesystem information.

A Contributor License Agreement is not necessary for a small MIT project unless there is a specific governance reason to require one.

---

# 29. Synthetic Test Data

Tests should use artificial data.

Good:

```text
big.bin
small.txt
example/
sub/
```

Avoid:

```text
real-user-passport.pdf
actual-tax-record.pdf
real-company-contract.docx
```

Do not commit real personal information to test fixtures.

---

# 30. Do Not Claim Government Certification

Do not write:

```text
NPC Approved
DPA Certified
100% Legally Compliant
Government Certified
```

unless there is formal legal or regulatory basis.

Prefer:

> SpazeFinder is designed around local processing, data minimization, and user-controlled filesystem operations.

---

# 31. Do Not Promise Zero Legal Risk

Neither MIT nor a disclaimer can guarantee that nobody will ever file a lawsuit.

The purpose of these measures is to:

- Clearly define rights.
- Limit warranties.
- Limit liability where legally permitted.
- Inform users.
- Avoid misleading privacy claims.
- Reduce security risk.
- Respect third-party licenses.
- Demonstrate responsible release practices.

---

# 32. Recommended Final Privacy Notice Position

The project's official position should be:

> SpazeFinder is a local-first disk analysis and filesystem management utility. Official releases process filesystem metadata locally on the user's device. The project does not operate a service that receives or stores scan results, filenames, file contents, filesystem paths, or user accounts.

This statement should remain accurate for every official release.

---

# 33. Recommended Final Disclaimer Position

The project's official position should be:

> SpazeFinder performs filesystem operations only at the user's direction. Users are responsible for reviewing actions and maintaining backups. The software is provided under the MIT License without warranty, to the fullest extent permitted by applicable law.

Also state:

> Use SpazeFinder only on systems and data that you own or are authorized to access.

---

# 34. Release Checklist

Before making the repository and binaries publicly available, verify all of the following.

## Legal

- [ ] Standard MIT `LICENSE` exists.
- [ ] Copyright holder and year are correct.
- [ ] README links to the license.
- [ ] Third-party licenses have been reviewed.
- [ ] Bundled assets are legally redistributable.
- [ ] `THIRD_PARTY_NOTICES.md` is complete where required.

## Privacy

- [ ] `PRIVACY.md` exists.
- [ ] Privacy notice accurately describes the current build.
- [ ] No hidden telemetry exists.
- [ ] No analytics exists unless disclosed.
- [ ] No scan data is uploaded.
- [ ] No user account is required.
- [ ] No cloud scan history is stored.
- [ ] Local preferences are disclosed.
- [ ] Official-build vs fork distinction is documented.

## Filesystem Safety

- [ ] Delete requires confirmation.
- [ ] Delete uses Trash or Recycle Bin.
- [ ] Rename avoids silent overwrite.
- [ ] Move avoids silent overwrite.
- [ ] Errors are shown to the user.
- [ ] Permission failures are handled safely.
- [ ] Symlink recursion does not create cycles.
- [ ] Permanent delete is absent or strongly protected.
- [ ] Users are warned to maintain backups.

## Security

- [ ] `SECURITY.md` exists.
- [ ] `cargo audit` passes or findings are reviewed.
- [ ] `cargo deny check` passes or findings are reviewed.
- [ ] `cargo test` passes.
- [ ] Release build succeeds.
- [ ] Secret scanning has been considered.
- [ ] No API keys or secrets are committed.
- [ ] Vulnerability reporting instructions exist.

## Distribution

- [ ] Official release channel is documented.
- [ ] Release is tied to a Git tag.
- [ ] Release artifacts correspond to source.
- [ ] SHA-256 checksums are published.
- [ ] Windows signing is used when practical.
- [ ] macOS signing/notarization is used when practical.

## Documentation

- [ ] README contains Privacy section.
- [ ] README contains Responsible Use section.
- [ ] README contains Security section.
- [ ] README contains License section.
- [ ] Disclaimer clearly addresses filesystem risk.
- [ ] Users are told to use SpazeFinder only where authorized.

---

# 35. Things SpazeFinder Does Not Need Right Now

Do not add these solely for legal appearance:

- Cookie banner.
- Cookie policy.
- Mandatory consent checkbox.
- User registration.
- Data deletion portal.
- DPO popup.
- GDPR-style onboarding screen.
- Data Processing Agreement.
- Data Sharing Agreement.
- Age verification.
- Identity verification.
- Cloud privacy dashboard.
- NPC registration workflow inside the application.

These are unnecessary for the current local-only architecture unless the project changes materially.

---

# 36. Final Recommended Repository State

```text
SpazeFinder/
│
├── LICENSE
├── README.md
├── PRIVACY.md
├── DISCLAIMER.md
├── SECURITY.md
├── THIRD_PARTY_NOTICES.md
├── PRODUCT.md
│
├── src/
│
└── src-tauri/
```

Optional future additions:

```text
CONTRIBUTING.md
CHANGELOG.md
.github/
docs/
deny.toml
```

---

# 37. Final Release Standard

SpazeFinder is ready for responsible public open-source distribution when the project owner can truthfully verify:

1. The project code is distributed under the standard MIT License.
2. All bundled dependencies and assets are legally redistributable.
3. Official builds do not secretly transmit filesystem information.
4. The privacy notice accurately explains local processing.
5. Users are warned about file-management risks.
6. Destructive actions require appropriate user intent.
7. The application does not bypass operating-system permissions.
8. Security vulnerabilities have a reporting path.
9. Official release artifacts can be traced to the public source.
10. Future network, analytics, cloud, or data-storage features require a new privacy review.

---

# 38. Maintainer Rule for Future Changes

Before merging any future feature, ask:

```text
Does this collect new information?
Does this transmit anything off-device?
Does this persist scan information?
Does this introduce a third-party service?
Does this read file contents?
Does this introduce user accounts?
Does this create a new security or privacy risk?
Does PRIVACY.md need to change?
Does DISCLAIMER.md need to change?
Does THIRD_PARTY_NOTICES.md need to change?
Does the Philippine privacy-law assessment need to be revisited?
```

If any answer is **yes**, perform a new compliance review before release.

---

# 39. Scope

This guide applies to the official SpazeFinder project as currently designed:

- Desktop application.
- Local filesystem scanning.
- Local filesystem management.
- No project-operated cloud backend.
- No user accounts.
- No analytics.
- No telemetry.
- No advertising.
- Free distribution.
- MIT open-source license.

If those assumptions change, this guide must be reviewed and updated.

---

# 40. Legal Note

This document is intended to reduce practical legal and compliance risk for an open-source software release.

It does not guarantee immunity from lawsuits, regulatory complaints, or legal disputes.

For a formal legal opinion or maximum certainty regarding Philippine law, intellectual property, consumer protection, privacy obligations, or liability, consult a qualified Philippine attorney familiar with technology and software licensing.
