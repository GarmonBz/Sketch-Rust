// P8 ref extract: detect-regions — lines 17520-17560 of 00) sketch standalone.html

17520:     this.defineAreaTarget(target, code, name);
17521:   }
17522: 
17523:   // The area a define click means: the tightest region the exterior walls
17524:   // enclose around the point. An exterior divider drawn across an
17525:   // area makes each side definable on its own. Read-only: returns the
17526:   // existing area to use ({ poly }) or the wall face to materialize
17527:   // ({ face }); the hover preview and the click both resolve through here.
17528:   resolveRegionAt(worldPt) {
17529:     const visible = this.state.polygons.filter(poly => this.isEntityVisible(poly) &&
17530:       window.Geometry.pointInPolygon(worldPt, window.Geometry.flattenRing(poly)));
17531:     const byArea = poly => Math.abs(poly.area ?? poly.gross ?? 0);
17532:     const smallest = visible.slice().sort((a, b) => byArea(a) - byArea(b))[0] || null;
17533:     const exterior = this.state.lines.filter(line => line.type === 'exterior' && this.isEntityVisible(line) &&
17534:       window.Geometry.distance(line.start, line.end) > 1e-6);
17535:     // Pointer hover asks on every move; reuse faces until a wall changes.
17536:     const key = exterior.map(line => `${line.id}:${line.start.x},${line.start.y},${line.end.x},${line.end.y}`).join('|');
17537:     if (!this.regionFaceCache || this.regionFaceCache.key !== key) {
17538:       this.regionFaceCache = { key, faces: window.Geometry.findEnclosedFaces(exterior, 1) };
17539:     }
17540:     const face = this.regionFaceCache.faces
17541:       .filter(item => window.Geometry.pointInPolygon(worldPt, item.points))
17542:       .sort((a, b) => a.area - b.area)[0];
17543:     if (!face) return { poly: smallest, face: null };
17544:     const same = visible.find(poly => this.polygonRingsEqual(poly.points, face.points));
17545:     if (same) return { poly: same, face: null };
17546:     if (smallest && byArea(smallest) <= face.area + 1e-6) return { poly: smallest, face: null };
17547:     if (smallest && this.isEntityLocked(smallest)) return { poly: smallest, face: null };
17548:     return { poly: null, face };
17549:   }
17550: 
17551:   // Give edge `index` of `poly` a wall of its own running exactly along that
17552:   // edge, styled like the wall it borrowed. Used where an area must not share
17553:   // or partially reuse another area's wall: moving the porch then moves only
17554:   // the porch, never the house wall it was drawn against.
17555:   ownBoundaryLine(poly, index) {
17556:     const edge = poly.edges[index];
17557:     const source = this.state.lines.find(line => line.id === edge?.lineId);
17558:     const start = poly.points[index];
17559:     const end = poly.points[(index + 1) % poly.points.length];
17560:     const line = {
