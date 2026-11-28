# RapidSketch and ACI Sketch files

Sketch Rust opens and writes RapidSketch (`.rs`) and ACI Sketch (`.acisketch`)
files. They are one format under two names.

- **Import**: File > Import > ACI Sketch… or RapidSketch…, File > Open, or drop
  the file on the window. The file opens as a new sketch and is never saved
  back in place.
- **Export**: File > Export > ACI Sketch or RapidSketch writes the current page.

## How far this has been tested

The layout below was worked out from **two real `.acisketch` files**, both
written by version 2.8.0.250.

- Both read with nothing skipped and survive export and re-import with the same
  area totals, and the same numbers of walls, notes and symbols.
- Exported files load in Microsoft's .NET Framework `BinaryFormatter` against
  stand-in classes with the same names and members. That checks the stream and
  the object graph, **not** how RapidSketch or ACI Sketch behaves with them:
  no exported file has been opened in either program.
- No `.rs` file has been tried. The shared format is inferred from the class
  names in the `.acisketch` files, which are all `RapidSketch.*`; the `.rs`
  extension comes from public software listings.
- Other program versions, or features neither file uses, may add objects or
  members. An import names any kind of object it leaves out. The first
  file had no interior walls, so `FreeformLine` was not supported until 0.3.18.

## The format

The file is the .NET Remoting Binary Format (MS-NRBF) — what `BinaryFormatter`
writes — holding a `RapidSketch.Sketch` object. It is neither encrypted nor
compressed.

Sketch Rust reads it with its own parser (`sketch-core/src/nrbf.rs`), which
builds a table of records and creates no .NET objects. Do not load untrusted
sketch files with `BinaryFormatter`: it can be made to run code.

| Object | Holds |
|---|---|
| `Sketch` | `Controls` (areas, interior walls, notes, symbols), `Areas`, `SketchInformation` |
| `FreeformLine` | an interior wall: `Start` and `End`, each a `SnappingHandle` with a `Location`; no arc, pen or dimension |
| `Area` | `Name`, `LivingArea`, `Multiplier`, `FillColor`; `Controls` = size label, walls, name label |
| `Line` | `Start`, `End` (each a `Point` joined by `ConnectedPoint`), `ArcHeight`, `ArcHandle`, `DashStyle`, dimension `Label` |
| `Note`, `Label` | `Lines` of text, centre `X`/`Y`, `Rotation`, `Font`, `Color`, `Visible` |
| `ContainerControl` | a placed symbol: centre `Location`, `Size`, `Rotation`, flips, `Graphic` |
| `Symbols.Symbol` | `Category`, `Name`, and a drawing made of areas and nested containers |

- Coordinates are 7 units to the foot, y down. Angles are degrees clockwise.
- `ArcHeight` is the arc's rise from its chord. The apex (also stored as the
  `ArcHandle` location) is `mid + ArcHeight * (dy, -dx) / chord`.
- A symbol's graphic is stretched to the container's `Size`; its drawn extent
  includes the pen width.
- An area's walls are not stored in outline order.

## What carries across

| | Import | Export |
|---|---|---|
| Walls, arcs, dashes, hidden dimensions | yes | yes |
| Areas, names, multipliers, fills | yes | yes |
| Area label positions and rotation | yes | yes |
| Text notes with font and style | yes | yes |
| Symbols | with their own drawing, as custom symbols | drawing only, without text |
| Subject property fields | yes | yes |
| Interior walls (`FreeformLine`) | as interior walls, without dimensions | straight, solid interior walls; their dimensions are not kept |
| Other walls outside any area | yes | as an open, unlabelled area |
| Background picture | no | no |
| Wall colours, hatching | no | no |
| Dimension placement and font size | regenerated | regenerated |
| Pages | one per file | current page only |

A RapidSketch area has a name and a "living area" switch but no area type. On
import the switch decides whether the type counts as living area, and the name
picks the type within that (`Basement`, `Covered Deck`, `Shed`, `Second Floor`
and so on); other names become `GLA1` or `OTH`. Check the types after import.
