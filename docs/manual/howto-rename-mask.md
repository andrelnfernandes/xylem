# Rename many files with a counter

This renames the nine photos of `photos/2025/trip` from `DSC_01.jpg` …
`DSC_09.jpg` to `trip_001.jpg` … `trip_009.jpg`.

## Steps

1. In the tree, put the cursor on `trip` and press Enter to go to its files.
2. Press Ctrl+T. Every file in the list is tagged.
3. Press Ctrl+R. The prompt asks for the mask and opens with `*.*`.
4. Press Ctrl+U to clear the prompt, type `trip_<001>.*` and press Enter.

![The four steps: the files of trip, Ctrl+T, Ctrl+R, and the mask trip_<001>.* giving trip_001.jpg to trip_009.jpg](../screenshots/howto-rename-mask.gif)

The footer shows `Renamed 9 of 9 files`. The files are numbered in the order
of the list, so sort it first (Alt+S) if the order matters.

## The mask

| Part | Meaning |
|---|---|
| `trip_` | text copied to every name |
| `<001>` | a counter with three digits: 001, 002, … |
| `.*` | the extension of each file, unchanged |

Step 4 clears the prompt because typing adds to the `*.*` that is already
there: `*.*trip_<001>.*` would name the first file `DSC_01.DSC_01trip_001.jpg`.

## Other masks

| Mask | `DSC_01.jpg` becomes |
|---|---|
| `*.jpeg` | `DSC_01.jpeg` |
| `2025_*.*` | `2025_DSC_01.jpg` |
| `s/DSC/trip/` | `trip_01.jpg` |
| `*.<upper>` | `DSC_01.JPG` |

[Rename](rename.md) has all the forms.

## When it does not do what you expect

- **Nothing is renamed.** The mask would give two files the same name. With
  no counter and no `*`, every file gets the same name.
- **Only some files were renamed.** Only the tagged files are renamed. A
  Filespec hides files from the list, and Ctrl+T tags only what the list
  shows.
- **Ctrl+R renames one file.** The key was `R` alone, which renames the file
  under the cursor.

See also: [tags](tagging.md), [the Ctrl menu](commands-ctrl.md).
