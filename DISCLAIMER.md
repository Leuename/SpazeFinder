# Disclaimer and Responsible Use

## Authorized use only

Use SpazeFinder only on devices, drives, filesystems, and data that you own or are
authorized to inspect or modify.

SpazeFinder does not attempt to bypass Windows security or permission controls. If
a folder is not readable, it is reported as inaccessible rather than opened by
other means.

## Filesystem risk

SpazeFinder can move, rename, and delete files. It performs those operations only
at your direction, on the item you selected.

You are responsible for:

- Reviewing which file or folder is selected before acting on it.
- Understanding that a folder action affects everything inside that folder.
- Maintaining backups of important data.

Deleting a folder sends the whole folder — and everything under it — to the
Recycle Bin.

## Recycle Bin

Delete sends items to the Windows Recycle Bin. SpazeFinder contains no permanent
delete.

Files in the Recycle Bin are not guaranteed to be recoverable. The Recycle Bin can
be emptied, can be size-limited, and does not apply to every drive or filesystem
configuration. Do not rely on it as a backup.

## Administrator privileges

Official Windows release builds request Administrator elevation at launch so that
protected system folders are included in the scan. Running with Administrator
rights means filesystem actions taken in the app are not blocked by the
protections that would normally stop a standard user. Review destructive actions
carefully.

Files you open from SpazeFinder are launched through `explorer.exe` so they do not
inherit Administrator rights.

## Accuracy

Reported sizes are calculated from filesystem metadata at scan time. They may
differ from other tools or from Windows itself due to inaccessible folders,
compression, sparse files, hard links, allocated-vs-actual size, and changes made
after the scan. Symbolic links are skipped and not counted.

The scan is a snapshot. It does not update itself when the disk changes outside
the app.

## Warranty

SpazeFinder is provided under the MIT License, **without warranty of any kind**,
to the fullest extent permitted by applicable law. See [LICENSE](LICENSE) for the
full warranty and liability disclaimer.

## Official releases

Official SpazeFinder releases are distributed through
<https://github.com/Leuename/SpazeFinder> and its GitHub Releases page. Modified
or third-party distributions may contain changes that were not reviewed or
approved by this project, and this disclaimer does not extend to them.
