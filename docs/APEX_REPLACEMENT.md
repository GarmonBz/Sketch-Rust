# Apex desktop compatibility — 0.1.29 development build

The requested native geometry/calculation, draft, code-table URL and multiple-file
startup paths are implemented inside sketch-app.exe. No companion EXE, bridge
service or HTML portal is distributed or started. Multiple native inputs use
independent windows/processes of this same sketch executable.

Host image snapshots now include the floorplan plus summary and breakdown
tables below it. They share the live measurement calculation model and retain
all calculation rows. Explicit print-form requests still use the chosen form.
0.1.12 centers compact calculation boxes with equal margins and corrects the
metafile recording frame at scaled Windows display DPIs. Actual stored images
in Pro legacy and UAD 3.6 reports have been verified, including both pages.
UAD's native Transfer sketches dialog must be completed to commit the images.
0.1.13 matches the native helper's 2615 x 4202 image aspect to prevent its resize
from stretching floorplan geometry. Actual legacy report rendering and UAD image
pixels confirm that a 10 x 10 area remains square. See BUILD_VALIDATION.md.

In integrated mode, closing offers Save to report, Discard changes and Cancel.
Save returns the current editable sketch, measurements and full report images,
then closes after the host confirms receipt. Discard cancels this editing
session and preserves the report's previously saved sketch. Transfer failures
leave the editor open for retry. Alt+F4 uses the same prompt.

## Native documents and calculations

- Independent AXF reader/writer for layouts 1–38, including encrypted payloads,
  names/codes, subject data, attributes, boundaries, free lines/arcs, CAD blocks,
  text and georeferences. The installed Apex reader accepts generated fixtures.
- Metric files are converted to the editor's foot-based world, then saved in
  their original native units. SDK/legacy measurements and georeference positions
  return in the native unit. The editing UI currently displays feet.
- Nested inserts, unequal/reflected X/Y scales, arcs/ellipses and point entities
  convert to editable symbol placements. Drawable native blocks are flattened
  for Apex's renderer; original topology is retained inside the AXF file.
- Native areas use signed multipliers and recursive child-net subtraction,
  independently of area-code sign defaults. Ordinary standalone areas retain
  their previous calculation rules. Newly drawn host-linked areas use native
  calculation rules too. Invalid negative subtraction links are excluded with
  a calculation warning; native Apex does not permit those links.
- Face measurements export real measured native boundaries. Matching private
  history restores editable wall centerlines and wall settings on reopen.
  Changed native boundaries invalidate that history instead of restoring stale
  centerlines. Native angles, dimension references and styled-block opacity
  follow the actual drawing-engine contract.

## Unfinished documents

Open native boundaries stay open and out of closed-area totals. Native saves
preserve repair history, original names/codes/factors/attributes and surviving
wall IDs. Optional history is stored in unused CAD blocks inside the genuine
AXF container, not a disguised SKX document or external bridge. Layouts 1–38
have draft round-trip fixtures. The actual Rust redraw/native-save workflow is
also tested on representative old and current layouts.

Native symbol history uses the same unused-block approach. Active geometry is
ordinary drawable CAD data. Other editors can strip unused blocks; doing so
can remove repair/centerline/original-topology history. Geometry fingerprints
prevent stale private history from overriding changed native geometry.

## Host interfaces

The host transport is embedded and hosted in the sketch PID. Its managed
component is materialized as a private DLL; no companion executable is used.
The implementation covers New/Open, repeated New/Open, native Save/Save As,
Base64/Load, subject/tags, live pages/areas, deletion/zoom, wall metrics,
breakdown, image export, georeferences, callbacks, Save without close and Return.

Native filename launches and -TOTALN/-TOTALS/-WINTOTALN/-WINTOTALS XML/COM entry
paths are included. Existing host authorization is respected. Shared SDK launch
redirection and exact restoration are built into the EXE. Production host
settings are backed up exactly before redirection. The disposable Pro tests
used a user-authorized temporary launch redirection and unique sketch file IDs.

0.1.10 fixes registration on protected shared configurations: the same sketch
EXE requests Windows elevation only for setup/restoration, waits for its exit,
and verifies the resulting configuration. Normal editing stays unelevated.
Visible result dialogs replace previously invisible console-only setup errors.
Cancelling Windows permission leaves setup unverified and reports failure.

0.1.21 exposes the same registration backend in Help > Appraise-It Integration.
Enable integration registers the running Sketch EXE; Restore previous provider
restores the saved launch setting. The panel displays the target and result,
keeps the UI responsive while Windows permission is requested, and prevents
duplicate setup actions while one is pending. A protected upgrade rolls back
its prior registration metadata on failure before the elevated retry.
This UI does not modify Appraise-It Pro's files or validate a fresh PC without
any previous Apex installation; the existing host qualification limits remain.

/areacodetableurl accepts HTTP, HTTPS, file URLs and file paths. Native Apex XML
is flattened into the editor catalogue with nested entries, names/descriptions,
signed factors, colors/fills, calculation categories, wall-height defaults and
smart-label metadata. Loading is bounded to 1 MiB with network timeouts; external
XML entities are disabled. Host catalogues remain document/session data and do
not overwrite the user's reusable code catalogue merely by being loaded.

Multiple AX2–AX7 arguments open independent native sessions. Each window keeps
its own filename and save destination; files are not merged into one document.
Spaces and option values are parsed separately from filenames.

## Verification and remaining qualification

Repeatable tests: tools/test-apex-files.ps1 and tools/test-apex-compat.ps1 -Live.
Reference testing uses installed Apex 7.1.16.0 only as a development validator;
it is not a production drawing-engine dependency. SDK reference: 6.0.188.0.
Reference binaries, inspection files and fixture-server/test EXEs are excluded
from distribution. See BUILD_VALIDATION.md for the current run results.

This remains a development compatibility build, not universal host certification:

- Actual Appraise-It Pro legacy/UAD 3.6 disposable live report transfers now pass
  for the areas and images documented in BUILD_VALIDATION.md. Report disk
  save/reopen and UAD vehicle-storage mapping remain additional qualification.
  TOTAL/WinTOTAL and other hosts still require report-field validation. The older
  host COM programs are absent on this PC; their XML contracts have fixtures.
- Exact Apex print-template duplication and exhaustive CAD font/style corpus
  fidelity have not been established. Full forms use Sketch Rust report layouts.
- Hosts hardcoding different Apex registrations/locations need validated setup
  support beyond the implemented shared SDK configuration route.

Physical DISTO testing and original HTML parent-suite integration remain deferred.
