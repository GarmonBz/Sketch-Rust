# Appraise-It Pro integration plan

## Current workflow

Keep the editable drawing in a Sketchy workfile or SKX. Export a PNG sketch
image or a PDF with calculations using the File menu. Attach the output where
the Appraise-It Pro report supports it, then review and enter the area totals
in the report. Use the licensed Apex application for existing AXF files. If
an older Sketch Rust preview changed your report's sketch button, ask SFREP
or Apex support to restore the original provider before upgrading.

## Proposed desktop connection

1. Use [SFREP's published COM API](https://api.sfrep.com/comdocs/intro.html)
   from the standalone Windows app. Resolve the installed COM server at
   runtime and keep the installer independent of both vendors' binaries.
   Use the documented dynamic report interface on supported UAD 3.6 builds.
2. Ask SFREP to confirm the supported way to attach sketch images and write
   area totals for legacy and UAD 3.6 reports, and to confirm usage terms.
   Its public API lists [sketch and area data types](https://api.sfrep.com/net/Sfrep.AppraiseIt.Client.Integration.Sketch.html).
   A documented third-party launch and sketch-transfer workflow still needs
   confirmation. Start with an explicit **Send to Appraise-It Pro** action in
   Sketch Rust. Add launch from the report only if SFREP documents a provider
   registration mechanism.
3. Let the user select an open report and preview the destination, page images
   and area totals before writing. Use Sketch Rust's existing rendering and
   calculations. Recheck the report identity before a short transaction, close
   the connection on success or failure, and preserve the local Sketchy or SKX
   file. Keep the connector opt-in and limit it to documented SFREP interfaces.
4. If the COM API does not cover the required fields or attachments, prototype
   SFREP's [documented RPTI interchange](https://api.sfrep.com/rpti/intro.html)
   on disposable reports. Verify its import and merge behavior before using it
   with an existing report. Continue offering manual image and PDF exports for
   report types or fields without a verified transfer path.
5. Verify against licensed copies of Appraise-It Pro on disposable legacy and
   UAD 3.6 reports. Reopen the reports and verify images, page order, units,
   totals, and preservation of unrelated data. Exercise cancellation, a
   closed or switched report, a missing COM server, and repeated transfer.

## Older releases

Existing release assets, cached installers, and historical Git references are
separate from this source change. Take down affected public downloads and
review public tags, branches, pull request refs, and caches for older source.
Coordinate any history rewrite with repository maintainers and GitHub Support.
Rebuild replacement packages from the updated tree.
