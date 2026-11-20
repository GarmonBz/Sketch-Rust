# Opening `.axf` sketches

Sketch Rust opens legacy Apex-IV-era `.axf` sketch files (the ones that start
with the `APX4WIN` object-stream signature once unpacked). The file is
**converted, never written back**: it opens as a new untitled sketch, and the
first Save asks for a `.sketchie` name. The original `.axf` is only read.

Open from File > Open (the filter lists `.axf`), by dropping a file on the
window, or on the command line (`sketch-app.exe plan.axf`).

## What is imported

| In the file | In Sketch Rust |
| --- | --- |
| Area table entry: code, name, factor | One closed area per boundary, with that code, name and factor. Codes not in the built-in list are added to the area-type table. |
| Boundary vertices, including arc (bulge) edges | One exterior wall per edge; arcs become bowed walls, so the area includes the curved segments. |
| Free text labels (room names and the like) | Text labels at the same positions, rotation, italic and underline. |
| Single sheet, feet | One page, feet. |

The importer cross-checks itself: every boundary is re-measured (arcs
included) and compared with the area the file itself recorded for it. A
mismatch of more than 0.5 % is reported in the toast shown after opening.

## What is not imported (yet)

- Door, window and fixture block inserts. The toast says how many were left
  out; they are not drawn.
- Dimension strings. Sketch Rust draws its own dimensions from the walls.
- The property / subject header, report fields and the other data the file
  carries.
- Open (unclosed) polylines that are not part of an area boundary.
- Metric files. The importer recognizes them from their dimension strings and
  refuses to open them rather than mis-scale a drawing.
- Encrypted or otherwise unreadable files. Sketch Rust does not decrypt
  anything; a file that does not unpack as a plain deflate stream with the
  `APX4WIN` signature is reported as unreadable.

## File layout (as far as it is understood)

Everything below is little-endian. The `.axf` is one **raw deflate** stream
(RFC 1951, no zlib or gzip header). Unpacked, it is a flat object stream that
starts with `07 "APX4WIN"`.

- `str06`: a text field: tag `06`, a length byte, then that many bytes
  (NUL padded, Windows-1252).
- **Area table entry**: `str06 code`, `str06 description`, `01`, `f64
  factor`, `f64 area`, `f64 perimeter`, padding, `ff ff 1d 02 79 00 01 01
  00`, `str06 name`. `area` and `perimeter` already include arc segments.
- **Boundary vertex**: `01 30 00`, a length byte and the line-type name
  (`CONTINUOUS`), then `f64 x`, `f64 y`, `f64 0.5`, `f64 w`, `f64 w`, `f64
  bulge`. A run of consecutive vertex records is one boundary; the sample
  boundaries run clockwise. The bulge belongs to the edge leaving that
  vertex and is the DXF-style `tan(angle / 4)` (negative bows to the left of
  travel). Sketch Rust's bow angle is `-4 * atan(bulge)` in degrees.
- **Text**: a kind byte (`01` label, `02` dimension), `str06 text`, four zero
  bytes, `f64 x`, `f64 y`, then a Win16 `LOGFONT` (height, width, escapement
  in tenths of a degree, orientation, weight, italic, underline, strike-out,
  charset, …, face name).
- **Block insert** (doors, fixtures): `01 30` then the block name, either
  directly (`ff len name`) or after a line-type name.

Boundaries are paired with area-table entries in file order. The decoder lives
in `crates/sketch-core/src/axf.rs`; the Open wiring is `open_axf_path` in
`crates/sketch-app/src/main.rs`.

## Tests

`cargo test -p sketch-core axf` runs the decoder and mapping tests against a
small synthetic stream built in the test itself (no real client data is
committed). To check a real file against the numbers the file records, set
`SKETCH_AXF_SAMPLE` to its path and run the same command with `--nocapture`;
it prints each area's computed and recorded area and perimeter and fails on
any mismatch.
