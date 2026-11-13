# Appraise-It Pro integration

## Transfer sketch images on Windows

1. Open a saved, editable UAD 3.6 report with an **Additional Sketch(es)**
   field in Appraise-It Pro. Open or draw the sketch in Sketch Rust.
2. Choose **Help > Appraise-It Reports > Check open report** in Sketch Rust.
   Select the session in SFREP's picker and verify the report path and form.
3. Choose **Preview sketch** and review the nonblank pages. Select **Send
   sketch images**, or turn on **Include area totals**, review the replacements,
   and select **Send sketch and area totals**.
4. In Appraise-It Pro, review **Sketch > Additional Sketch(es)** and any
   updated living-unit area fields. Save the report there.

Each send submits one PNG per nonblank page in sketch page order. Appraise-It Pro stores the images as report exhibits.
Existing exhibits remain in the list. Keep the editable drawing in a Sketchy
or SKX file.

The connector uses Appraise-It Pro 3.5 or later and its COM registration for the
matching Windows architecture. It rechecks the destination in the selected
process immediately before writing. It calls SFREP's
[dynamic report data store](https://api.sfrep.com/com/Sfrep.AppraiseIt.Client.Integration.Com.DataStore.DataStoreIntegrationCom.html)
and [image-list API](https://api.sfrep.com/com/Sfrep.AppraiseIt.Client.Integration.Com.DataStore.Fields.DataStoreImageListFieldIntegrationCom.html)
to append images to `SketchImageList`, the subject-level field identified in
Appraise-It Pro's Field Details panel. The installed Appraise-It Pro application
supplies the COM server.

## Optional area totals

The option defaults to off. It is available on an editable UAD 3.6 URAR with
one dwelling, one living unit, and no ADUs. Sketch Rust sums the calculated
net areas across all pages, then rounds each total to whole square feet.

| Sketch Rust area codes | Appraise-It Pro living-unit field |
| --- | --- |
| `GLA1`–`GLA10` | Standard finished above grade |
| `BSMT` | Finished below grade |
| `BSMT2` | Unfinished below grade |

Classify outlines before transfer. In particular, use GLA codes for standard
finished area. Unclassified Sketchy areas, gross building area, nonstandard
area, garage, amenities, and other categories require manual entry. The
preview shows each positive rounded total and the report value it will
replace. Enter zeroes and clear report fields directly in Appraise-It Pro.
Sketch Rust writes the three supported fields through SFREP's
[living-unit field API](https://api.sfrep.com/com/Sfrep.AppraiseIt.Client.Integration.Com.DataStore.FieldsIndexIntegrationCom.html).

## Verification on Appraise-It Pro 3.8.8

A two-page Sketchy 0.2 drawing was transferred to a disposable copy of SFREP's
UAD 3.6 single-family URAR sample. After saving, closing, and reopening the
report, Additional Sketch(es) contained the original exhibit, page 1, and
page 2 in that order. The original exhibit's checksum matched the baseline,
and both added images matched their pre-save checksums. A comparison of 37
numeric area-related fields in the MISMO 3.6 exports found no differences.

The optional area transfer was also exercised with a classified two-page sketch
on a disposable single-family URAR. After saving and reopening, the report
retained both pages in order and the original exhibit. The living unit showed
668 sq ft of standard finished area above grade, 180 sq ft of finished area
below grade, and 70 sq ft of unfinished area below grade.

## Other report types and older previews

Use **File > Export as PNG** or **File > Export as PDF** for a manual attachment
when the report has no Additional Sketch(es) field. Ask SFREP or Apex support to
restore the original provider if an older Sketch Rust preview changed the report's
sketch button. Open existing AXF workfiles in the licensed Apex application.
