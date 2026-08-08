# Security Policy

## Supported versions

SpazeFinder is developed by a single maintainer. Only the latest release receives
security fixes.

| Version | Supported |
|---|---|
| Latest release | Yes |
| Older releases | No |

## Reporting a vulnerability

Please report privately. Do not open a public issue for an unfixed vulnerability.

Preferred: GitHub private vulnerability reporting —
<https://github.com/Leuename/SpazeFinder/security/advisories/new>

Alternative: email millaveemmanuel15@gmail.com with `SpazeFinder security` in the
subject.

## What to include

- SpazeFinder version and Windows version.
- What the issue allows an attacker to do.
- Steps to reproduce, using synthetic files and folders.
- Whether elevation (Administrator) is required to trigger it.
- Any proof-of-concept, as a minimal test case.

**Do not include real personal files, confidential filenames, credentials, access
tokens, API keys, or full filesystem dumps.** Redact usernames and home-directory
paths from screenshots and logs. A reproduction under `C:\test\` is more useful
than one under your Documents folder.

## Response

- Acknowledgement: within 7 days.
- Assessment and plan: within 30 days.
- Fix released, then a public advisory crediting the reporter unless anonymity is
  requested.

This is a volunteer project with no paid security team and no bug bounty.

## Disclosure

Please allow a fix to be released before publishing details. If a report is not
acknowledged within 30 days, public disclosure is reasonable.

## Scope

In scope:

- Privilege escalation or unintended use of the app's Administrator rights.
- Path handling flaws that cause an operation to affect an unintended file.
- Command injection or unsafe process launching (`open`, `reveal`).
- Memory-safety issues in the Rust scanner.
- Any code path that transmits filesystem information off the device — official
  builds should have none.

Out of scope:

- Behavior of third-party forks and repackaged builds.
- Issues that require an attacker to already have Administrator access.
- Missing code signing on unofficial or self-built binaries.
- The fact that release builds request elevation — this is documented, intended
  behavior. See [PRIVACY.md](PRIVACY.md) and [DISCLAIMER.md](DISCLAIMER.md).
