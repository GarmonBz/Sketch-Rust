> Superseded scope: the user now requires a complete, host-agnostic Apex replacement,
> with all compatibility code inside sketch-app.exe. See APEX_REPLACEMENT.md.
> Protocol impersonation is expressly authorized; the earlier restriction below
> is historical and does not apply to the active task.

# Appraise-It Pro native desktop integration

Requested 2026-10-29. Target: the installed Windows Appraise-It Pro application,
with an Apex 7-style launch/edit/return workflow. Both older fixed-form reports
and dynamic UAD 3.6 reports are required. No HTML integration portal.

## Verified on this PC

- Appraise-It Pro integration assembly: version 3.8.8.0.
- Installation: C:/Program Files/SFREP/Appraise-It Pro.
- Apex Sketch v7 is also installed.
- Pro's native Tools > Sketch group contains Apex, Sketch-It and Sketch32.
  Settings > Advanced has no exposed third-party executable registration field.
- The public desktop APIs support selecting/attaching to a Pro process.
- Read-only reflection of the installed .NET integration DLL confirmed
  TransferSketchData, IsDynamicReport, ReportPath, transaction, attachment and
  numeric/image field methods. The probe did not instantiate a connection or
  modify a report. Evidence: target/appraise-it-pro-api-probe.json.
- IntegrationClient.TransferSketchData(SketchAndAreaDataClient) and the COM
  counterpart support dynamic/UAD 3.6 reports only. This is not a legacy return
  path and must not be presented as one.
- IntegrationClient also documents report transactions, field/image access, and
  attachment storage, which are candidates for legacy mapping and editable SKX
  storage once the exact host contract is confirmed.

## Required user workflow

1. Open a report in the Windows Appraise-It Pro application.
2. Choose Sketch Rust from Pro; launch the native executable with an unambiguous
   report/session identity. Existing Apex integration must remain available.
3. Read subject information and reopen this integration's prior editable sketch
   for that report, or start a blank sketch. Multiple report windows must never
   share sketch state accidentally.
4. Draw/edit and choose Return to Appraise-It Pro.
5. Review and transfer sketch pages and area calculations to the originating
   report, including GLA, below-grade finished/unfinished area, garage and other
   classified areas as supported by the report. Retain editable sketch data.
6. Launch again from the same report and edit the same sketch. Updates must
   replace this integration's previous output without duplicating pages or
   changing unrelated report fields.

## Proposed implementation boundary

The Rust application owns drawing, area calculation, SKX persistence and page
rendering. A native desktop adapter uses the supported COM/.NET interface.
The choice between direct COM and a small managed adapter should follow the
confirmed launch/registration SDK; it is not an implementation commitment yet.
The adapter must end transactions and release its connection after all failures.
No transaction should remain open while the user draws.

For UAD 3.6, map native sketch/area/level data to the documented transfer DTOs.
Confirm how the host associates each sketch with a property, structure, unit or
outbuilding; do not infer that list order is a stable identifier.
For legacy reports, obtain the supported sketch exchange contract or verified
field/addendum mappings for each supported form. Do not send legacy data to
TransferSketchData. Attachment APIs may hold versioned SKX state after their
behavior is validated on both report generations.

## Unresolved vendor contract

A supported public method for registering a new native sketch executable in
Pro's Tools ribbon has not been found in the public API documentation or the
inspected native settings. The Tools ribbon observation does not prove that
partner registration is unavailable; SFREP must confirm the supported mechanism.
No developer/partner registration documentation is available from the user.

Before an Apex-equivalent integration can be implemented and accepted, obtain:

- Desktop sketch-provider registration/launcher SDK or supported configuration.
- Launch arguments and report/session identity/lifetime rules.
- Legacy sketch, image and area return contract, including accepted formats,
  classifications, repeated addenda and update/reopen behavior.
- UAD 3.6 structure/unit association rules, classifications, units, rounding and
  supported sketch image formats.
- Editable sketch storage/retrieval guidance and minimum supported Pro versions.

Do not replace Apex executables, edit SFREP program binaries, impersonate Apex,
or claim that generic desktop field writes establish full sketch integration.

## Acceptance gate

Use disposable copies of vendor sample reports, never a production appraisal.
Verify older 1004, at least one other supported fixed form, and UAD 3.6
single-family plus a structure/unit case. For each: launch from Pro, return
numeric areas and all sketch pages, save/reopen the Pro report, relaunch and
edit, return again without duplication. Exercise Cancel, failed transfer, report
close/switch, two Pro processes, unsaved/renamed reports and read-only reports.
Compare the same fixture's numeric output with Apex 7 and inspect addenda.
Only after these checks pass may the build claim Apex-style integration.

## Current status

Discovery/specification only. No connector has been added to the application,
no report has been modified, and the working 0.1.7 build remains unchanged.
A draft request for SFREP is in SFREP_DESKTOP_INTEGRATION_REQUEST.txt; it has not
been sent.

## Primary vendor references

- https://api.sfrep.com/
- https://api.sfrep.com/netdocs/intro.html
- https://api.sfrep.com/comdocs/intro.html
- https://api.sfrep.com/net/Sfrep.AppraiseIt.Client.Integration.IntegrationClient.html
- https://api.sfrep.com/net/Sfrep.AppraiseIt.Client.Integration.Sketch.SketchAndAreaDataClient.html
- https://api.sfrep.com/net/Sfrep.AppraiseIt.Client.Integration.Sketch.SketchDataClient.html
