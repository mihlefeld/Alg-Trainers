# Rocket + Yew migration progress

Reference used: upstream `dev` at `8df81345766fb73cd14aba338d785d68be27514d`.

## Completed in the current pass

### Selection behavior

- Restored the complete preset toolbar (`+`, `-`, `Del`, `Set`, and `Use`).
- Added preset persistence for selection cases and selected algsets.
- Added algset selectors for trainers with multiple algsets, including BLD trainers.
- Added hidden-selected feedback while filtering.
- Enabled fuzzy text matching for names, setups, case IDs, and algorithms.
- Added fuzzy matching for algset and group labels.
- Restored the graphical cube-diagram pattern builder and color swatches.
- Added diagram matching across every supported U-angle orientation.
- Added consistent hiding and counts for diagram-filtered and disabled cases.
- Restored case-probability display controlled by the existing setting.
- Preserved compatibility with the existing array-only selection format.
- Synchronized cases, presets, and algset flags through the existing revision payload; no database-table migration was required.

### Training behavior

- Corrected recap counts and kept recap mode active until its current case is completed.
- Restored `B` (bookmark), `R` (remove/re-add the last case), and `Shift+H` (last-case hint) shortcuts.
- Restored touch press/hold/release timer controls.
- Added stopped-result color feedback.
- Added basic session statistics (`Times` and distinct `Cases`), with case information available on each recent result.
- Added grouped per-case statistics, histograms, detailed time lists, and an
  exponential-moving-average trend line.
- Added hint previous/next navigation and custom-algorithm save/reset/download/upload controls.
- Restored any-key timer stopping, Escape-to-abort, keyboard hint navigation, and touch cancellation.
- Restored `?select`, `?train`, and `?recap` direct-link and browser-history behavior, including case selection after asynchronous storage loading.
- Translated BLD scrambles with the configured corner/edge letter scheme.
- Added BLD letter-pair-word JSON import and word substitution.
- Restored Square-1 scramble slice normalization and pre/post move application.
- Corrected prior-case hint behavior.

### Styling and usability

- Reduced the desktop trainer selection panel to a compact centered layout close to the reference.
- Restored the compact preset toolbar and improved mobile wrapping.
- Added a responsive grouped grid for algorithm sheets.
- Added a page-dimming modal backdrop for settings, credits, hotkeys, and hints.
- Removed horizontal overflow at the tested 390px mobile width.
- Improved touch targets and practice-page spacing.
- Restored centered, backdrop-click-dismissible dialogs and Material Symbols rendering.
- Restored the floating start-practice control.

## Verified

- Rocket, Yew, and the production Docker image build successfully.
- Presets survive a browser reload and restore the expected selected cases.
- Browser regression probes verified:
  - recap count remains `Recap 1 Cases` after the first of two solves;
  - `B`, `R`, and `Shift+H` update the expected state;
  - BLD outputs translated letter pairs;
  - Square-1 output is normalized;
  - mobile timer hold turns green, release starts timing, and touch stops/records the result;
  - no horizontal overflow at 390x844.

## Remaining differences

### Settings and custom data

- Custom cube-color schemes, including global/puzzle/trainer assignments and scheme management, are not implemented.
- Backup import/export is consolidated into one trainer-data backup; the reference's separate custom-algorithm and letter-pair export workflows are not fully represented.
- Cube-color inputs are still raw text values rather than scheme selectors.

### Visual polish

- Home cards remain slightly more compact than the upstream reference.
- The practice footer and icon spacing still need final alignment across unusual viewport heights.
- Algorithm-sheet card proportions and typography are close but not pixel-identical to the reference.