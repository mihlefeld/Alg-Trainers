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
- Added account controls and last-successful-sync status to trainer settings.
- Added hint previous/next navigation and custom-algorithm save/reset/download/upload controls.
- Reworked hints and algorithm sheets around stable per-algorithm IDs with
  reorderable secondary-color items, user-alg editing/deletion, and built-in hiding.
- Styled the compact hint algorithm editor with secondary colors and removed
  the legacy reset/import/export controls.
- Restored any-key timer stopping, Escape-to-abort, keyboard hint navigation, and touch cancellation.
- Restored `?select`, `?train`, and `?recap` direct-link and browser-history behavior, including case selection after asynchronous storage loading.
- Translated BLD scrambles with the configured corner/edge letter scheme.
- Added BLD letter-pair-word JSON import and word substitution.
- Restored Square-1 scramble slice normalization and pre/post move application.
- Corrected prior-case hint behavior.

### Styling and usability

- Added named cube-color schemes, built-in color-blind presets, and global,
  puzzle, and trainer assignments with legacy color migration.
- Applied the active scheme to trainer and algorithm-sheet diagrams and added
  contrasting color previews to theme and cube-color inputs.
- Reduced the desktop trainer selection panel to a compact centered layout close to the reference.
- Restored the compact preset toolbar and improved mobile wrapping.
- Added a responsive grouped grid for algorithm sheets.
- Added a page-dimming modal backdrop for settings, credits, hotkeys, and hints.
- Removed horizontal overflow at the tested 390px mobile width.
- Improved touch targets and practice-page spacing.
- Restored centered, backdrop-click-dismissible dialogs and Material Symbols rendering.
- Restored the floating start-practice control.
- Fixed background-click dismissal for every dialog and hint window.
- Corrected selection colors, search toggle states, group spacing, and first-use algset headings.
- Matched selection width to each trainer's `maxAlgsPerRow` metadata, stored it in the trainers table, and exposed it through the catalog API.
- Restored white hotkey keycaps, consistent button sizing, and bookmark/close/delete/undo icon rendering.
- Limited stopped-timer red feedback to the 0.1-second debounce and positioned hint navigation arrows around the case name.
- Matched home-card width and spacing to the upstream layout and restored the algorithm-sheet grid's 18em card sizing.
- Kept practice controls and the footer visible by sizing the timer area to the dynamic viewport and letting its statistics scroll; added footer icon spacing.
- Grouped settings disclosures into exclusive accordions, added bordered per-puzzle scheme disclosures, and standardized flex gaps and case-group header widths.

## Verified

- Rocket, Yew, and the production Docker image build successfully.
- Backend catalog tests pass, the Yew frontend builds for `wasm32-unknown-unknown`, and all 44 trainer catalogs define `maxAlgsPerRow`.
- Database ownership is enforced for result conflicts and sync cleanup, with
  foreign keys and indexes for the frequent user-history and catalog lookups.
- Sync writes are bounded, validated, statement-limited, and committed atomically
  in batches; private responses are marked `no-store`.
- Public API routes are rate-limited, catalog detail loading uses a fixed number
  of bulk queries, and catalog responses are cacheable.
- Production startup rejects missing/placeholder secrets, OAuth calls have
  explicit timeouts, and session cookies are configurable as `Secure`.
- Algset buttons now distinguish disabled, partial, and fully selected states,
  including the disabled-to-empty re-enable transition.
- Presets survive a browser reload and restore the expected selected cases.
- Browser regression probes verified:
  - legacy localStorage and IndexedDB v1 cube colors migrate correctly;
  - scheme assignments survive reload and update trainer and algorithm-sheet
    diagrams, and color inputs meet 4.5:1 contrast for tested opaque colors;
  - recap count remains `Recap 1 Cases` after the first of two solves;
  - `B`, `R`, and `Shift+H` update the expected state;
  - BLD outputs translated letter pairs;
  - Square-1 output is normalized;
  - mobile timer hold turns green, release starts timing, and touch stops/records the result;
  - no horizontal overflow at 390x844.

## Remaining differences

### Settings and custom data

- Backup import/export is consolidated into one trainer-data backup; the reference's separate custom-algorithm and letter-pair export workflows are not fully represented.
