# macOS app icon

High-fidelity Icon Composer setup modeled on InspectRadar
(`appraisal_flow/inspect/mobile/ios/Inspect/InspectRadar.icon`):

- True vector SVG layers (potrace from the brand floor-plan mark)
- Gradient fill + Liquid Glass translucency / shadow in `SketchRust.icon/icon.json`
- Classic `.icns` for `cargo-bundle` via `tools/build-app-icon.sh`

## Files

| Path | Role |
|------|------|
| `SketchRust.icon/` | Icon Composer document (open in Icon Composer to tune glass) |
| `layers/1-floorplan.svg` | Primary white floor-plan vector |
| `layers/2-floorplan-inner.svg` | Inset layer for glass depth |
| `layers/master-1024.png` | Flattened preview / `.icns` source |
| `AppIcon.icns` | Finder / Dock / `cargo-bundle` |

Hand-authored architectural experiments (`1-outer-walls.svg` …) remain in
`layers/` as design references; Composer uses the potrace layers.

## Regenerate `.icns`

```bash
./tools/build-app-icon.sh assets/macos/layers/master-1024.png
```

## Tune in Icon Composer

1. Open `assets/macos/SketchRust.icon` in **Icon Composer** (Xcode › Open Developer Tool, or Spotlight).
2. Confirm Default / Dark appearances, specular, and translucency (InspectRadar uses translucency `0.3`, shadow `0.5`).
3. Export flattened PNGs for marketing if needed; keep regenerating `AppIcon.icns` from `master-1024.png` until Composer export is wired into the script.
