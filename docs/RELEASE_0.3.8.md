## What's new

- Fixed UI hangs when drawing or nudging exterior walls on multi-room floorplans with open wall chains: `heal_exterior_wall_joins` now prunes degree-`< 2` open chains from the cycle-search graph and uses reachability-guided cycle walking instead of exponential backtracking.
- Fixed T-junction wall welds splitting the welding wall instead of the host wall.
- Fixed frame stalls while dragging walls, selections, and labels: `area_label_anchor` now fast-paths rooms whose centroid already clears the boundary, reuses computed label anchors when placing size blocks, and skips recomputing unselected label anchors during drags.
- Fixed periodic UI pauses on large `.sketchie` files (such as Fallingwater): unsaved-change checks now skip serializing the immutable `.sketchie` backing payload, hash area-code definitions directly without JSON round-tripping, and defer hashing while an interactive drag, pan, fence, or resize gesture is active.
- Fixed unnecessary full-canvas redraws when hovering sidebar rows, menus, or dragging a selection fence, and clamped dotted and hatched area fills to the visible canvas viewport.

## Windows

Run **SketchRust-0.3.8-x64.msi** to install or upgrade Sketch Rust for the current Windows user. A portable ZIP and SHA-256 checksums are also provided.
