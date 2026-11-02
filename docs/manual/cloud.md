# Cloud storage (rclone)

Xylem has no Google Drive or S3 client of its own, but it does not need one:
[rclone](https://rclone.org) mounts them (and some 70 other services:
OneDrive, Dropbox, Backblaze B2, Cloudflare R2, MinIO...) as a folder, and
Xylem works on that folder like on any other. Everything works there:
logging, Branch, tags, the viewer, Copy and Move in both directions, Rename,
Makedir, Delete.

## Why rclone

We chose to work with rclone rather than talk to Google Drive and Amazon S3
directly. Code of our own for each service would be one more place for
mistakes, and Google Drive would also bring Google's policies along: an app
that reads all of someone's files needs Google's verification and a yearly
security assessment, or else shows an "unverified app" warning and is limited
to a few users. For S3, the official library is very large and would add a
lot of complexity, which Xylem, by design, avoids whenever it can. rclone
already does all this well, for many services at once.

## Setting up (once)

Install rclone (`apt install rclone`, `dnf install rclone`, `brew install
rclone`, or from [rclone.org](https://rclone.org/downloads/)) and create a
remote with `rclone config`. You give it a name, which is what goes before
the colon below.

- **Google Drive:** storage type `drive`; the defaults are fine. rclone opens
  the browser for you to sign in with Google and allow it. On a machine
  without a browser (over SSH), answer *no* to the browser question: rclone
  shows a command to run with `rclone authorize` on a machine that has one.
- **Amazon S3:** storage type `s3`, provider `AWS`, then either the access key
  and secret, or *env_auth* to use what the AWS tools already use
  (`~/.aws/credentials`, environment variables, the instance's role). Other
  S3 services (R2, MinIO, Wasabi...) are other providers in the same list.

`rclone lsd gdrive:` (or `rclone lsd s3:`) lists the top folders (the
buckets, in S3): if it works, the remote is ready.

## Mounting and opening

Linux, with a remote called `gdrive`:

```bash
mkdir -p ~/gdrive
rclone mount gdrive: ~/gdrive --vfs-cache-mode writes --daemon
xy ~/gdrive
```

For S3, the same with `s3:` (every bucket) or `s3:my-bucket` (one of them).
`fusermount -u ~/gdrive` unmounts.

- `--vfs-cache-mode writes` keeps a local copy of files being written, which
  editors (`E`) and some programs need.
- `--dir-cache-time 1h` keeps directory listings longer than the default 5
  minutes: logging again (F3) is faster, but changes made elsewhere take
  longer to show.

On **macOS**, `rclone mount` needs [macFUSE](https://osxfuse.github.io);
`rclone nfsmount gdrive: ~/gdrive` needs nothing extra. On **Windows**, it
needs [WinFsp](https://winfsp.dev) and mounts on a drive letter:
`rclone mount gdrive: X:`, then `xy X:\`.

To have the folder at hand, save it as a [bookmark](bookmarks.md) (Alt+B).

On Linux, Xylem notices the folders mounted by rclone: the DISK panel shows
the service (Google Drive, Amazon S3, OneDrive...) and, below it, the remote
(`rclone gdrive:`). Info (Alt+I) and Avail (`A`) show them too. The service
comes from rclone's configuration file (`~/.config/rclone/rclone.conf`, or
`$RCLONE_CONFIG`); when it is encrypted, the panel just says rclone.

## What is different there

- **Everything goes over the network.** Each directory read is a request, and
  a service like Google Drive refuses them when they are too many. So, on a
  mounted folder, `*`, Branch (`B`), the Usage view and Compare on a branch ask
  rclone for the whole branch in a single listing, which is far quicker, unless
  it is all read already; Esc leaves the tree as it was. If rclone fails or shows
  other folders than the mount, they read folder by folder. Only `*` expands the
  tree.
- **The screen does not wait.** The tree shows at once and fills in as folders
  arrive, also when you open one (→ or `+`) or move the cursor to one; the file
  window says `reading...` meanwhile. F3 and going to a typed path still wait for
  their one read.
- **Changes made elsewhere** (in the browser, on the phone) show up only after
  F3, and after the directory cache expires (`--dir-cache-time`).
- **Delete:** answer `P` (permanently) rather than `T`: on Linux, the trash
  of a mounted folder is a `.Trash-` folder (with your user number) at its
  top, so the file would stay in the remote, taking space. Google Drive
  keeps what rclone deletes in the Drive's own trash anyway; on S3 a deleted
  file is gone, unless the bucket has versioning.
- **Permissions and owners** are not kept: the mount shows the same ones for
  every file, and Attributes cannot change them. Dates are kept.
- **Google Docs, Sheets and Slides** are not files: rclone shows them exported
  (`.docx`, `.xlsx`, `.pptx`), with size 0. `--drive-skip-gdocs` hides them.
- **S3 has no real directories**, only names with `/` in them: an empty
  directory may vanish once unmounted, and renaming or moving a directory
  copies and deletes each file in it.

## When the service refuses: error 403

Google Drive limits how many requests it takes per minute. On a Drive crowded
with small files and folders (backups of projects, `node_modules`, editor
extensions), **403 errors are common**: `Quota exceeded`, `rateLimitExceeded`.
rclone waits and tries again, so everything slows down, and in the end it may
give up. Any file manager that reads folder by folder through the mount meets
the same limit.

What Xylem does: `*` shows how many requests were refused and, after 100, asks
whether to keep trying or stop; stopping leaves the tree as it was. If rclone
gives up, Xylem says so and does not read folder by folder on its own, because
that would only be refused more.

What helps, starting with the first:

- **Log by folders.** Instead of `*` at the root, use `*` on smaller folders, or
  open them with → or `+`. It is often the better way on a crowded Drive.
- **Your own `client_id`.** Unless you made one, rclone uses a client shared with
  its other users, and so is the quota. Making your own is free:
  [rclone's guide](https://rclone.org/drive/#making-your-own-client-id) explains
  it. Afterwards, authorize again with `rclone config`. The limits of your
  account remain, but other people's use no longer counts against you.
- **Keep listings longer:** `--dir-cache-time 1h` on `rclone mount`.
- **Remove what can be recreated.** Fewer folders mean fewer requests, for any
  program.

## Finding what fills your Google Drive

When the Drive keeps asking for a bigger storage plan, Xylem can show where
the space goes before you pay for more. Mount the Drive, go to its root, log
it all with `*` (on a large Drive it takes a while) and choose Alt+K →
**Percent** ([Directory sizes](info.md)): each folder shows how much of the
one above it takes, with a bar against the largest of its siblings. Follow
the biggest bars down to the old backups, videos and duplicates.

The [Usage view](usage.md) (Alt+W, at the Drive's root) goes further: the
files and folders together, largest first, so a single huge file shows as
well as a full folder. And Branch (`B`) lists the files of a whole folder:
[sorted](sort.md) by size, largest first (Alt+S, then `O` for descending and
`S` for Size), the biggest come on top.

A few things count toward the storage plan without showing in the mount:

- **The Drive's trash** takes space until it is emptied (or 30 days pass).
  `rclone about gdrive:` shows the total, used and trashed; mounting with
  `--drive-trashed-only` shows only what is in the trash.
- **Gmail and Google Photos** share the same storage, and only the Drive's
  files are in the mount. If `rclone about` shows much more used than the
  mount adds up to, the rest is probably there.
- **Google Docs, Sheets and Slides** show with size 0, but newer ones count
  toward the storage.
- Files **shared with you** do not count: they are in their owner's storage.
