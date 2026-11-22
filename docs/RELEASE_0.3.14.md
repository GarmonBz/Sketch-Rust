## What's new

- Fixed Appraise-It Pro COM integration failing with `Appraise-It Pro returned an unexpected value for ReportPath.` when `ReportPath` (or other COM text fields) returns `VT_EMPTY` or `VT_NULL` from .NET COM interop.
- Checked the `InitializeForProcess` boolean return value when reconnecting to a reviewed Appraise-It Pro session.
- Wired **Auto-Fill from Report** and **Draw the sketch for this report** to connect to the open Appraise-It Pro report, populate `File No.` when blank, and prime the **Send to report** session.
- Updated Command Palette Appraise-It actions (`Check Appraise-It report`, `Preview sketch for Appraise-It`, `Send sketch to Appraise-It`) to open the User Guide integration panel and retain check results.

## Windows

Run **SketchRust-0.3.14-x64.msi** to install or upgrade Sketch Rust for the current Windows user. A portable ZIP and SHA-256 checksums are also provided.
