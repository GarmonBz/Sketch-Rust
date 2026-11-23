# Appraise-It Pro integration

## Transfer sketch images on Windows

1. Open a saved, editable report in Appraise-It Pro. Open or draw the sketch in
   Sketch Rust.
2. Choose **Help > Appraise-It Reports > Check open report** in Sketch Rust.
   Select the session in SFREP's picker and verify the report path and form.
3. Choose **Preview sketch** and review the nonblank pages. Select **Send
   sketch images**, or select the available area option and review the proposed
   values before sending.
4. In Appraise-It Pro, review **Additional Sketch(es)** for UAD 3.6 reports
   or the new **Sketch** addenda for legacy reports. Review any updated area
   fields and save the report there.

Each send submits one image per nonblank page in sketch page order and preserves
existing sketches. Keep the editable drawing in a Sketchie or SKX file.

The connector uses Appraise-It Pro 3.5 or later and its COM registration for the
matching Windows architecture. It rechecks the destination in the selected
process immediately before writing. For UAD 3.6, it calls SFREP's
[dynamic report data store](https://api.sfrep.com/com/Sfrep.AppraiseIt.Client.Integration.Com.DataStore.DataStoreIntegrationCom.html)
and [image-list API](https://api.sfrep.com/com/Sfrep.AppraiseIt.Client.Integration.Com.DataStore.Fields.DataStoreImageListFieldIntegrationCom.html)
to append images to `SketchImageList`, the subject-level field identified in
Appraise-It Pro's Field Details panel. For legacy reports that support repeatable
**Sketch** addenda, it uses SFREP's
[static-form COM API](https://api.sfrep.com/com/Sfrep.AppraiseIt.Client.Integration.Com.IntegrationCom.html)
to add a new `Sketch` form per page and write its `SketchImage` field. If a send
stops partway through, review the Sketch addenda and remove any incomplete
transfer before retrying. The installed Appraise-It Pro application supplies
the COM server.

## Optional area totals

**Include area totals** defaults to off and is available on an editable UAD 3.6
URAR with one dwelling, one living unit, and no ADUs. Sketch Rust sums the
calculated net areas across all pages, then rounds each total to whole square
feet. Enter area totals directly in Appraise-It Pro for legacy reports.

The transfer fills all six URAR area-breakdown fields (UAD 3.6 URAR Appendix
F-1), in form order:

| Sketch Rust area codes | Appraise-It Pro living-unit field |
| --- | --- |
| `GLA1`–`GLA10` | `FinishedAreaAboveGrade` (10.003) |
| `GLAN` | `FinishedAreaAboveGradeNonstandard` (10.004) |
| `GLAU` | `UnfinishedAreaAboveGrade` (10.005) |
| `BSMT` | `FinishedAreaBelowGrade` (10.006) |
| `BSMTN` | `FinishedAreaBelowGradeNonstandard` (10.007) |
| `BSMT2` | `UnfinishedAreaBelowGrade` (10.008) |

Classify outlines before transfer. `GLAN`, `GLAU` and `BSMTN` are finished or
unfinished area that F-1 reports outside the standard finished total; they are
separate codes (not part of the `GLA`/`BSMT` totals) precisely so the area
breakdown can report them on their own lines. Unclassified Sketchie areas,
gross building area, garage, amenities, and other categories have no F-1 field
and require manual entry. The preview shows each positive rounded total and the
report value it will replace. Enter zeroes and clear report fields directly in
Appraise-It Pro. Sketch Rust writes the fields through SFREP's
[living-unit field API](https://api.sfrep.com/com/Sfrep.AppraiseIt.Client.Integration.Com.DataStore.FieldsIndexIntegrationCom.html),
reading each one back and restoring the previous values if the whole transfer
fails.

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

On a disposable copy of SFREP's legacy 1004 sample, both added pages rendered
in order. After reopening the autosaved report, all three images matched
their pre-save checksums. A separate COM test on a disposable legacy 1073
condo report also accepted both Sketch addenda in order.

## Other report types and older previews

Use **File > Export as PNG** or **File > Export as PDF** for a manual attachment
when the report lacks a supported sketch destination. Ask SFREP or Apex support to
restore the original provider if an older Sketch Rust preview changed the report's
sketch button. Open existing AXF workfiles in the licensed Apex application.
