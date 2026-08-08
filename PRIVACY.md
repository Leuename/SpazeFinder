# Privacy Notice

**Applies to:** official SpazeFinder source code and releases published at
<https://github.com/Leuename/SpazeFinder>.

**Last updated:** 2026-08-09

SpazeFinder is a local-first Windows desktop utility. It scans a drive, shows what
is using the space, and performs filesystem actions you ask it to perform. It runs
entirely on your device.

## Summary

SpazeFinder processes filesystem information locally on your device and does not
transmit that information to project-operated servers.

- No user account.
- No login.
- No advertising.
- No analytics.
- No telemetry.
- No crash reporting.
- No cloud storage.
- No remote scan history.
- No device fingerprinting.

## Information processed locally

To calculate and display disk usage, SpazeFinder reads:

- Drive letters, total size, and free space of fixed drives.
- Directory entries (folder and file names).
- File sizes and directory metadata.
- Whether an entry is a file, a folder, or a symbolic link.

SpazeFinder does **not** open or read file contents. The scanner uses directory
listings and file-size metadata only. It does not parse documents, index text, or
inspect media.

Symbolic links are skipped during scanning to avoid cycles and double-counting.

Filenames and paths can themselves contain personal information (for example
`C:\Users\Example\Documents\Medical\Lab-Result.pdf`). SpazeFinder therefore does
not claim to avoid personal data — it claims that this information stays on your
device.

## Information stored on your device

| What | Where | Lifetime |
|---|---|---|
| Scan results (the folder tree and sizes) | Application memory only | Discarded when the app closes |
| Theme preference (light/dark) | WebView local storage, key `theme` | Until you clear it or uninstall |

SpazeFinder writes no scan history file, no database, and no log file.

## Network

SpazeFinder makes no network requests. Core functionality — drive listing,
scanning, tree browsing, open, reveal, rename, move, and Recycle Bin delete —
works fully offline. There is no update checker.

## Administrator privileges

Official Windows release builds request Administrator elevation through UAC at
launch, so that system-protected folders are counted instead of being reported as
inaccessible.

Elevation is optional. If you decline the prompt, SpazeFinder keeps running with
your standard account's access and displays a notice that protected system files
and folders are skipped. Nothing about the scan is hidden from you either way.

This affects what SpazeFinder can read on your machine; it does not change what
leaves your machine, which is nothing.

Files you double-click to open are launched through `explorer.exe`, so the opened
program runs without inheriting Administrator rights.

SpazeFinder does not attempt to bypass Windows security controls. Folders it
cannot read are counted as inaccessible.

## Third-party services

The project operates no backend service and uses no third-party analytics,
logging, or crash-reporting service.

If you report a bug on GitHub, information you choose to include in that report is
handled by GitHub under GitHub's own terms and privacy policy. Please remove
personal filenames, usernames, home-directory paths, and credentials from
screenshots and logs before posting.

## Official builds vs forks

SpazeFinder is MIT licensed, so anyone may modify and redistribute it. This notice
applies to the official source code and releases maintained by this project.
Third-party forks, modified builds, and repackaged distributions may behave
differently and are not covered by this notice.

## Changes

If a future version adds network access, telemetry, analytics, crash reporting,
accounts, cloud sync, persistent scan history, file-content inspection, or any
third-party service, this notice will be updated before that version is released.

## Contact

Privacy questions: open an issue at
<https://github.com/Leuename/SpazeFinder/issues>, or email
millaveemmanuel15@gmail.com.
