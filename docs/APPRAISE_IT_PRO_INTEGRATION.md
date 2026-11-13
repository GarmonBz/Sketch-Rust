# Appraise-It Pro integration

## Transfer sketch images on Windows

1. Open a saved, editable UAD 3.6 report with an **Additional Sketch(es)**
   field in Appraise-It Pro. Open or draw the sketch in Sketch Rust.
2. Choose **Help > Appraise-It Reports > Check open report** in Sketch Rust.
   Select the session in SFREP's picker and verify the report path and form.
3. Choose **Preview sketch**. Review the nonblank pages and select **Send
   sketch images**.
4. In Appraise-It Pro, open **Sketch > Additional Sketch(es)** and review the
   images. Save the report in Appraise-It Pro.

Each send submits one PNG per nonblank page in sketch page order. Appraise-It Pro stores the images as report exhibits.
Existing exhibits remain in the list. Keep the editable drawing in a Sketchy
or SKX file and review and enter any area totals in Appraise-It Pro.

The connector uses Appraise-It Pro 3.5 or later and its COM registration for the
matching Windows architecture. It rechecks the destination in the selected
process immediately before writing. It calls SFREP's
[dynamic report data store](https://api.sfrep.com/com/Sfrep.AppraiseIt.Client.Integration.Com.DataStore.DataStoreIntegrationCom.html)
and [image-list API](https://api.sfrep.com/com/Sfrep.AppraiseIt.Client.Integration.Com.DataStore.Fields.DataStoreImageListFieldIntegrationCom.html)
to append images to `SketchImageList`, the subject-level field identified in
Appraise-It Pro's Field Details panel. The installed Appraise-It Pro application
supplies the COM server.

## Verification on Appraise-It Pro 3.8.8

A two-page Sketchy 0.2 drawing was transferred to a disposable copy of SFREP's
UAD 3.6 single-family URAR sample. After saving, closing, and reopening the
report, Additional Sketch(es) contained the original exhibit, page 1, and
page 2 in that order. The original exhibit's checksum matched the baseline,
and both added images matched their pre-save checksums. A comparison of 37
numeric area-related fields in the MISMO 3.6 exports found no differences.

## Other report types and older previews

Use **File > Export as PNG** or **File > Export as PDF** for a manual attachment
when the report has no Additional Sketch(es) field. Ask SFREP or Apex support to
restore the original provider if an older Sketch Rust preview changed the report's
sketch button. Open existing AXF workfiles in the licensed Apex application.
