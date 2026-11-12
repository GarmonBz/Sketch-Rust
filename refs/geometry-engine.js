class GeometryEngine {
  constructor() {
    this.PIXELS_PER_FOOT = 20; // Default scale
    // A full 360-degree arc cannot join two distinct endpoints. Keeping one
    // degree of headroom permits stable major arcs without the singularity.
    this.MAX_BOW_ANGLE = 359;
  }

  clampBowAngle(value) {
    const angle = Number(value) || 0;
    return Math.max(-this.MAX_BOW_ANGLE, Math.min(this.MAX_BOW_ANGLE, angle));
  }

  validateBowAngle(value) {
    const angle = Number(value) || 0;
    if (Math.abs(angle) > this.MAX_BOW_ANGLE) {
      throw new RangeError(`bowAngle must be between -${this.MAX_BOW_ANGLE} and ${this.MAX_BOW_ANGLE} degrees`);
    }
    return angle;
  }

  // Calculate distance between two points (in logical/world feet)
  distance(p1, p2) {
    return Math.hypot(p2.x - p1.x, p2.y - p1.y);
  }

  // Convert degrees to radians
  degToRad(deg) {
    return deg * (Math.PI / 180);
  }

  // Calculate square footage. With no edge metadata this remains the familiar
  // shoelace calculation; callers that provide edges receive the exact
  // circular-segment contribution as well.
  calculateArea(points, edges) {
    return Math.abs(this.calculateSignedCurvedArea(points, edges));
  }

  calculateSignedCurvedArea(points, edges) {
    let area = this.calculateSignedArea(points);
    if (!edges || !edges.length) return area;
    for (let i = 0; i < points.length; i++) {
      area += this.circularSegmentArea(
        points[i], points[(i + 1) % points.length], Number(edges[i] && edges[i].bowAngle) || 0
      );
    }
    return area;
  }

  // Signed area also identifies winding. Positive is counter-clockwise in
  // world coordinates; negative is clockwise.
  calculateSignedArea(points) {
    if (points.length < 3) return 0;
    let area = 0;
    for (let i = 0; i < points.length; i++) {
      const next = points[(i + 1) % points.length];
      area += points[i].x * next.y - next.x * points[i].y;
    }
    return area / 2.0;
  }

  // Calculate perimeter of a polygon, using arc length for bowed edges.
  calculatePerimeter(points, edges) {
    if (points.length < 2) return 0;
    let perim = 0;
    for (let i = 0; i < points.length; i++) {
      const next = (i + 1) % points.length;
      const bow = Number(edges && edges[i] && edges[i].bowAngle) || 0;
      perim += this.arcLength(points[i], points[next], bow);
    }
    return perim;
  }

  circularSegmentArea(p1, p2, bowAngle) {
    if (!bowAngle) return 0;
    this.validateBowAngle(bowAngle);
    const theta = Math.abs(bowAngle) * Math.PI / 180;
    const chord = this.distance(p1, p2);
    if (theta < 1e-9 || chord < 1e-9) return 0;
    const radius = (chord / 2) / Math.sin(theta / 2);
    const segment = (radius * radius / 2) * (theta - Math.sin(theta));
    return -Math.sign(bowAngle) * segment;
  }

  arcLength(p1, p2, bowAngle) {
    const chord = this.distance(p1, p2);
    if (!bowAngle || chord < 1e-9) return chord;
    this.validateBowAngle(bowAngle);
    const theta = Math.abs(bowAngle) * Math.PI / 180;
    if (theta < 1e-9) return chord;
    return ((chord / 2) / Math.sin(theta / 2)) * theta;
  }

  // Screen coords → Logical/World coords (feet)
  toLogical(screenPt, pan, zoom) {
    return {
      x: (screenPt.x - pan.x) / (this.PIXELS_PER_FOOT * zoom),
      y: (pan.y - screenPt.y) / (this.PIXELS_PER_FOOT * zoom)
    };
  }

  // Alias for clarity
  toWorld(screenPt, pan, zoom) {
    return this.toLogical(screenPt, pan, zoom);
  }

  // Logical/World coords → Screen coords
  toScreen(logicalPt, pan, zoom) {
    return {
      x: pan.x + logicalPt.x * this.PIXELS_PER_FOOT * zoom,
      y: pan.y - logicalPt.y * this.PIXELS_PER_FOOT * zoom
    };
  }

  // Snap a world point to the nearest grid intersection
  snapToGrid(worldPt, gridSize) {
    return {
      x: Math.round(worldPt.x / gridSize) * gridSize,
      y: Math.round(worldPt.y / gridSize) * gridSize
    };
  }

  // Find the nearest existing endpoint to a world point, within a threshold.
  // An optional skip predicate lets callers exclude points, e.g. vertices of
  // areas that are already closed.
  findNearestEndpoint(worldPt, allLines, threshold, skip) {
    let nearest = null;
    let nearestDist = threshold;

    allLines.forEach(line => {
      [line.start, line.end].forEach(pt => {
        if (skip && skip(pt)) return;
        const d = this.distance(worldPt, pt);
        if (d < nearestDist) {
          nearestDist = d;
          nearest = { ...pt };
        }
      });
    });

    return nearest;
  }

  // Snapping logic: finds nearest endpoint in a given direction
  findAlignment(currentPos, allLines, direction) {
    let nearestDist = Infinity;
    let snapPos = null;

    allLines.forEach(line => {
      [line.start, line.end].forEach(pt => {
        if (direction === 0 || direction === 180) {
          // Vertical movement, look for X alignment
          if (Math.abs(pt.x - currentPos.x) < 0.1) {
            let d = Math.abs(pt.y - currentPos.y);
            if (d < nearestDist && d > 0.1) {
              nearestDist = d;
              snapPos = { x: currentPos.x, y: pt.y };
            }
          }
        } else {
          // Horizontal movement, look for Y alignment
          if (Math.abs(pt.y - currentPos.y) < 0.1) {
            let d = Math.abs(pt.x - currentPos.x);
            if (d < nearestDist && d > 0.1) {
              nearestDist = d;
              snapPos = { x: pt.x, y: currentPos.y };
            }
          }
        }
      });
    });
    
    return snapPos;
  }

  // CTRL+SHIFT+Arrow: Find alignment with the closest line that intersects the cursor's path
  findIntersectingAlignment(currentPos, allLines, direction) {
    let nearestDist = Infinity;
    let snapPos = null;

    allLines.forEach(line => {
      if (direction === 0 || direction === 180) {
        // Moving vertically — find horizontal lines that cross our X
        const minX = Math.min(line.start.x, line.end.x);
        const maxX = Math.max(line.start.x, line.end.x);
        if (currentPos.x >= minX - 0.1 && currentPos.x <= maxX + 0.1) {
          // This line crosses our vertical path
          const midY = (line.start.y + line.end.y) / 2;
          const d = Math.abs(midY - currentPos.y);
          if (d < nearestDist && d > 0.1) {
            // Check direction
            if ((direction === 0 && midY > currentPos.y) || 
                (direction === 180 && midY < currentPos.y)) {
              nearestDist = d;
              snapPos = { x: currentPos.x, y: midY };
            }
          }
        }
      } else {
        // Moving horizontally — find vertical lines that cross our Y
        const minY = Math.min(line.start.y, line.end.y);
        const maxY = Math.max(line.start.y, line.end.y);
        if (currentPos.y >= minY - 0.1 && currentPos.y <= maxY + 0.1) {
          const midX = (line.start.x + line.end.x) / 2;
          const d = Math.abs(midX - currentPos.x);
          if (d < nearestDist && d > 0.1) {
            if ((direction === 90 && midX > currentPos.x) || 
                (direction === 270 && midX < currentPos.x)) {
              nearestDist = d;
              snapPos = { x: midX, y: currentPos.y };
            }
          }
        }
      }
    });

    return snapPos;
  }

  // Distance from a point to a line segment
  distanceToSegment(pt, segA, segB) {
    const proj = this.projectPointToSegment(pt, segA, segB);
    return this.distance(pt, proj);
  }

  projectPointToSegment(pt, segA, segB) {
    const dx = segB.x - segA.x;
    const dy = segB.y - segA.y;
    const lenSq = dx * dx + dy * dy;
    if (lenSq === 0) return { ...segA };

    let t = ((pt.x - segA.x) * dx + (pt.y - segA.y) * dy) / lenSq;
    t = Math.max(0, Math.min(1, t));
    return { x: segA.x + t * dx, y: segA.y + t * dy };
  }

  // Constrain a point to fixed angular increments about an anchor while
  // keeping its distance, so a held modifier can lock a run to 45s.
  constrainToAngleStep(anchor, pt, stepDegrees = 45) {
    const step = Math.abs(Number(stepDegrees)) || 45;
    const dx = pt.x - anchor.x;
    const dy = pt.y - anchor.y;
    const length = Math.hypot(dx, dy);
    if (length < 1e-9) return { x: anchor.x, y: anchor.y };
    const snapped = this.degToRad(Math.round((Math.atan2(dy, dx) * 180 / Math.PI) / step) * step);
    return { x: anchor.x + Math.cos(snapped) * length, y: anchor.y + Math.sin(snapped) * length };
  }

  // Direction in degrees clockwise from straight up, matching the angle the
  // drawing HUD reports.
  bearingDegrees(from, to) {
    const mathAngle = Math.atan2(to.y - from.y, to.x - from.x) * 180 / Math.PI;
    return (90 - mathAngle + 360) % 360;
  }

  // Included angle at a corner, 0-180 degrees. Returns null when either leg
  // has no length to take a direction from.
  cornerAngle(previous, vertex, next) {
    const a = { x: previous.x - vertex.x, y: previous.y - vertex.y };
    const b = { x: next.x - vertex.x, y: next.y - vertex.y };
    const lengthA = Math.hypot(a.x, a.y);
    const lengthB = Math.hypot(b.x, b.y);
    if (lengthA < 1e-9 || lengthB < 1e-9) return null;
    const cosine = Math.max(-1, Math.min(1, (a.x * b.x + a.y * b.y) / (lengthA * lengthB)));
    return Math.acos(cosine) * 180 / Math.PI;
  }

  // Per-leg and cumulative metrics for an open or closed polyline. A closed
  // run of three or more points also reports the area it encloses.
  polylineMetrics(points, closed = false) {
    const pts = Array.isArray(points) ? points : [];
    const ring = closed && pts.length > 2;
    const legs = ring ? pts.length : Math.max(0, pts.length - 1);
    const segments = [];
    let total = 0;
    for (let i = 0; i < legs; i++) {
      const start = pts[i];
      const end = pts[(i + 1) % pts.length];
      const length = this.distance(start, end);
      total += length;
      segments.push({
        start, end, length, cumulative: total,
        dx: end.x - start.x,
        dy: end.y - start.y,
        bearing: this.bearingDegrees(start, end)
      });
    }
    return { segments, total, closed: ring, area: ring ? Math.abs(this.calculateSignedArea(pts)) : 0 };
  }

  // Point-in-polygon test (ray casting)
  pointInPolygon(pt, polygon) {
    let inside = false;
    for (let i = 0, j = polygon.length - 1; i < polygon.length; j = i++) {
      const xi = polygon[i].x, yi = polygon[i].y;
      const xj = polygon[j].x, yj = polygon[j].y;
      const intersect = ((yi > pt.y) !== (yj > pt.y)) &&
        (pt.x < (xj - xi) * (pt.y - yi) / (yj - yi) + xi);
      if (intersect) inside = !inside;
    }
    return inside;
  }

  pointOnBoundary(pt, polygon, tolerance = 1e-4) {
    if (!polygon || polygon.length < 2) return false;
    for (let i = 0; i < polygon.length; i++) {
      if (this.distanceToSegment(pt, polygon[i], polygon[(i + 1) % polygon.length]) <= tolerance) return true;
    }
    return false;
  }

  orientation(a, b, c, tolerance = 1e-9) {
    const value = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
    return Math.abs(value) <= tolerance ? 0 : Math.sign(value);
  }

  // Endpoint touches and collinear overlap are deliberately not "proper"
  // crossings; shared walls are valid containment boundaries.
  segmentsProperlyCross(a, b, c, d, tolerance = 1e-9) {
    const o1 = this.orientation(a, b, c, tolerance);
    const o2 = this.orientation(a, b, d, tolerance);
    const o3 = this.orientation(c, d, a, tolerance);
    const o4 = this.orientation(c, d, b, tolerance);
    return o1 !== 0 && o2 !== 0 && o3 !== 0 && o4 !== 0 && o1 !== o2 && o3 !== o4;
  }

  /**
   * Replace bowed edges by short chords whose maximum sagitta is bounded.
   * Sampling follows CanvasEngine.drawArc's world-space equivalent exactly.
   */
  flattenRing(pointsOrPolygon, edgesArg, maxSagitta = 0.01) {
    const points = Array.isArray(pointsOrPolygon) ? pointsOrPolygon : (pointsOrPolygon && pointsOrPolygon.points) || [];
    const edges = Array.isArray(pointsOrPolygon) ? (edgesArg || []) : (pointsOrPolygon && pointsOrPolygon.edges) || [];
    if (points.length < 2) return points.map(point => ({ ...point }));
    const flattened = [];
    for (let i = 0; i < points.length; i++) {
      const p1 = points[i];
      const p2 = points[(i + 1) % points.length];
      const bow = Number(edges[i] && edges[i].bowAngle) || 0;
      flattened.push({ ...p1 });
      if (Math.abs(bow) < 1e-9) continue;
      this.validateBowAngle(bow);
      const chord = this.distance(p1, p2);
      if (chord < 1e-9) continue;
      const theta = Math.abs(bow) * Math.PI / 180;
      const radius = (chord / 2) / Math.sin(theta / 2);
      const safeSagitta = Math.max(1e-6, Math.min(maxSagitta, radius));
      const maxStep = 2 * Math.acos(Math.max(-1, Math.min(1, 1 - safeSagitta / radius)));
      const segments = Math.max(1, Math.ceil(theta / Math.max(maxStep, 1e-6)));
      const dx = p2.x - p1.x;
      const dy = p2.y - p1.y;
      const left = { x: -dy / chord, y: dx / chord };
      const half = theta / 2;
      const centerDistance = radius * Math.cos(half);
      const sign = Math.sign(bow);
      const center = {
        x: (p1.x + p2.x) / 2 - left.x * centerDistance * sign,
        y: (p1.y + p2.y) / 2 - left.y * centerDistance * sign
      };
      const startAngle = Math.atan2(p1.y - center.y, p1.x - center.x);
      // Screen-space drawArc sweeps by bow; converting back to world reverses
      // angular orientation because screen Y is inverted.
      const sweep = -bow * Math.PI / 180;
      for (let step = 1; step < segments; step++) {
        const angle = startAngle + sweep * (step / segments);
        flattened.push({
          x: center.x + radius * Math.cos(angle),
          y: center.y + radius * Math.sin(angle)
        });
      }
    }
    return flattened;
  }

  polygonContains(outer, inner, tolerance = 1e-4, maxSagitta = 0.01) {
    const outerRing = this.flattenRing(outer, undefined, maxSagitta);
    const innerRing = this.flattenRing(inner, undefined, maxSagitta);
    if (outerRing.length < 3 || innerRing.length < 3) return false;

    for (const point of innerRing) {
      if (!this.pointInPolygon(point, outerRing) && !this.pointOnBoundary(point, outerRing, tolerance)) return false;
    }
    for (let i = 0; i < innerRing.length; i++) {
      const a = innerRing[i], b = innerRing[(i + 1) % innerRing.length];
      for (let j = 0; j < outerRing.length; j++) {
        const c = outerRing[j], d = outerRing[(j + 1) % outerRing.length];
        if (this.segmentsProperlyCross(a, b, c, d, tolerance * tolerance)) return false;
      }
    }
    for (let i = 0; i < innerRing.length; i++) {
      const a = innerRing[i], b = innerRing[(i + 1) % innerRing.length];
      const midpoint = { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 };
      if (this.pointInPolygon(midpoint, outerRing) && !this.pointOnBoundary(midpoint, outerRing, tolerance)) return true;
    }
    return false;
  }

  // Check if a point is inside an axis-aligned rectangle
  pointInRect(pt, min, max) {
    return pt.x >= min.x && pt.x <= max.x && pt.y >= min.y && pt.y <= max.y;
  }

  /**
   * Compute the maximal collinear wall run containing the seed wall, plus the
   * reference points that split it into dimension spans. Bowed walls are
   * treated as chords (v1 limitation). Returns null when the seed is missing
   * or degenerate.
   * Result: { lineIds, start, end, dir, normal, refs, length } where refs are
   * distances along dir from `start` (ascending, endpoints included).
   */
  computeDimensionChain(lines, symbols, seedLineId, tolerance = 0.25, options = {}) {
    const seed = (lines || []).find(line => line.id === seedLineId);
    if (!seed) return null;
    const chord = this.distance(seed.start, seed.end);
    if (chord < 1e-6) return null;
    const dir = { x: (seed.end.x - seed.start.x) / chord, y: (seed.end.y - seed.start.y) / chord };
    const normal = { x: -dir.y, y: dir.x };
    const origin = seed.start;
    // Project a point onto the seed axis: returns { t, off } where t is the
    // along-axis distance and off the perpendicular deviation.
    const project = point => {
      const dx = point.x - origin.x;
      const dy = point.y - origin.y;
      return { t: dx * dir.x + dy * dir.y, off: dx * normal.x + dy * normal.y };
    };
    const parallelTolerance = Math.sin(3 * Math.PI / 180); // 3 degrees
    // Candidate walls: straight, parallel to the axis and lying on it.
    const candidates = (lines || []).filter(line => {
      if (Math.abs(Number(line.bowAngle) || 0) >= 0.5) return false;
      const len = this.distance(line.start, line.end);
      if (len < 1e-6) return false;
      const cross = Math.abs(((line.end.x - line.start.x) / len) * dir.y - ((line.end.y - line.start.y) / len) * dir.x);
      if (cross > parallelTolerance) return false;
      return Math.abs(project(line.start).off) <= tolerance && Math.abs(project(line.end).off) <= tolerance;
    });
    // Connected component of candidates sharing endpoints with the seed.
    const endpointsMatch = (a, b) => this.distance(a, b) <= tolerance;
    const component = new Set([seed.id]);
    let grew = true;
    while (grew) {
      grew = false;
      candidates.forEach(line => {
        if (component.has(line.id)) return;
        const touches = candidates.some(other => {
          if (!component.has(other.id)) return false;
          return endpointsMatch(line.start, other.start) || endpointsMatch(line.start, other.end) ||
            endpointsMatch(line.end, other.start) || endpointsMatch(line.end, other.end);
        });
        if (touches) { component.add(line.id); grew = true; }
      });
    }
    const runLines = candidates.filter(line => component.has(line.id));
    let minT = Infinity, maxT = -Infinity;
    runLines.forEach(line => {
      [line.start, line.end].forEach(point => {
        const { t } = project(point);
        minT = Math.min(minT, t);
        maxT = Math.max(maxT, t);
      });
    });
    if (!(maxT - minT > 1e-6)) return null;
    // Reference parameters: run endpoints, T-junction projections from other
    // walls, and hosted-opening gap edges on the run walls.
    const refs = new Set([minT, maxT]);
    const addRef = t => {
      if (t <= minT + 1e-4 || t >= maxT - 1e-4) return;
      for (const existing of refs) {
        if (Math.abs(existing - t) <= tolerance / 2) return;
      }
      refs.add(t);
    };
    if (options.splitWallsAtJunctions) {
      (lines || []).forEach(line => {
        if (component.has(line.id)) return;
        [line.start, line.end].forEach(point => {
          const { t, off } = project(point);
          if (Math.abs(off) <= tolerance) addRef(t);
        });
      });
    }
    const lineById = new Map((lines || []).map(l => [l.id, l]));
    (symbols || []).forEach(symbol => {
      const gapWidth = Number(symbol.wallGapWidth) || 0;
      if (!symbol.wallId || gapWidth <= 0 || !component.has(symbol.wallId)) return;
      const { t, off } = project(symbol);
      if (Math.abs(off) > tolerance * 2 + gapWidth / 2) return;
      const library = typeof window !== 'undefined' && window.SKETCH_SYMBOL_LIBRARY && window.SKETCH_SYMBOL_LIBRARY.symbols;
      const snap = symbol.wallSnap || (library && library[symbol.symbolType] && library[symbol.symbolType].wallSnap);
      if (snap === 'SIDE') {
        // Hinge-anchored opening: refs at the hinge and at the far edge,
        // which lies along the wall direction unless the swing is flipped.
        const wall = lineById.get(symbol.wallId);
        const dirSign = (wall ? (Math.sign((wall.end.x - wall.start.x) * dir.x + (wall.end.y - wall.start.y) * dir.y) || 1) : 1) *
          (symbol.flipH ? -1 : 1);
        addRef(t);
        addRef(t + dirSign * (symbol.wallSideFlip ? -gapWidth : gapWidth));
      } else {
        addRef(t - gapWidth / 2);
        addRef(t + gapWidth / 2);
      }
    });
    const sorted = [...refs].sort((a, b) => a - b);
    return {
      lineIds: runLines.map(line => line.id),
      start: { x: origin.x + dir.x * minT, y: origin.y + dir.y * minT },
      end: { x: origin.x + dir.x * maxT, y: origin.y + dir.y * maxT },
      dir,
      normal,
      refs: sorted.map(t => t - minT),
      length: maxT - minT
    };
  }

  // Area-weighted centroid of a simple polygon. Falls back to the vertex
  // average for degenerate (near-zero-area) rings.
  polygonCentroid(points) {
    if (!Array.isArray(points) || !points.length) return { x: 0, y: 0 };
    if (points.length < 3) return { x: points[0].x, y: points[0].y };
    let area = 0, cx = 0, cy = 0;
    for (let i = 0; i < points.length; i++) {
      const p0 = points[i];
      const p1 = points[(i + 1) % points.length];
      const cross = p0.x * p1.y - p1.x * p0.y;
      area += cross;
      cx += (p0.x + p1.x) * cross;
      cy += (p0.y + p1.y) * cross;
    }
    area /= 2;
    if (Math.abs(area) < 1e-9) {
      const n = points.length;
      return {
        x: points.reduce((sum, p) => sum + p.x, 0) / n,
        y: points.reduce((sum, p) => sum + p.y, 0) / n
      };
    }
    return { x: cx / (6 * area), y: cy / (6 * area) };
  }

  // Planar face traversal: finds enclosed regions bounded by the given wall
  // segments. Walls are split at junction points (other walls' endpoints that
  // land mid-segment) so T-junctions become graph nodes; bowed walls are
  // treated as their chord (v1 limitation). Returns bounded faces
  // (counter-clockwise, positive signed area) above minArea as
  // { points, area, edges } where each edge carries the source wall's lineId.
  // The unbounded outer face is traced clockwise and is discarded by the
  // positive-area filter.
  findEnclosedFaces(lines, minArea = 1, tolerance = 0.02) {
    const source = lines || [];
    const n = source.length;
    // Split every wall into sub-segments at junction points so the graph is a
    // proper planar subdivision. Handles both T-junctions (another wall's
    // endpoint landing mid-segment) and X-junctions (two walls crossing).
    const chords = new Array(n);
    const dirs = new Array(n);
    const stops = source.map(() => []);
    source.forEach((line, i) => {
      const chord = this.distance(line.start, line.end);
      chords[i] = chord;
      dirs[i] = chord > 1e-6
        ? { x: (line.end.x - line.start.x) / chord, y: (line.end.y - line.start.y) / chord }
        : null;
      stops[i].push(0, chord);
    });
    for (let i = 0; i < n; i++) {
      if (!dirs[i]) continue;
      for (let j = 0; j < n; j++) {
        if (i === j || !dirs[j]) continue;
        // T-junction: endpoints of line j lying on line i.
        [source[j].start, source[j].end].forEach(point => {
          const t = (point.x - source[i].start.x) * dirs[i].x + (point.y - source[i].start.y) * dirs[i].y;
          const off = Math.abs((point.x - source[i].start.x) * -dirs[i].y + (point.y - source[i].start.y) * dirs[i].x);
          if (off <= tolerance && t > tolerance && t < chords[i] - tolerance) stops[i].push(t);
        });
      }
    }
    // X-junctions: proper crossings between pairs of walls.
    for (let i = 0; i < n; i++) {
      for (let j = i + 1; j < n; j++) {
        if (!dirs[i] || !dirs[j]) continue;
        const a = source[i].start, c = source[j].start;
        const r = { x: source[i].end.x - a.x, y: source[i].end.y - a.y };
        const sV = { x: source[j].end.x - c.x, y: source[j].end.y - c.y };
        const denom = r.x * sV.y - r.y * sV.x;
        if (Math.abs(denom) < 1e-9) continue; // parallel
        const t = ((c.x - a.x) * sV.y - (c.y - a.y) * sV.x) / denom;
        const u = ((c.x - a.x) * r.y - (c.y - a.y) * r.x) / denom;
        if (t > 1e-4 && t < 1 - 1e-4 && u > 1e-4 && u < 1 - 1e-4) {
          stops[i].push(t * chords[i]);
          stops[j].push(u * chords[j]);
        }
      }
    }
    const pieces = [];
    source.forEach((line, i) => {
      if (!dirs[i]) return;
      const sorted = stops[i].sort((a, b) => a - b);
      // Merge stops that are effectively the same junction point.
      const merged = [];
      sorted.forEach(value => {
        if (!merged.length || value - merged[merged.length - 1] > tolerance) merged.push(value);
      });
      for (let k = 0; k < merged.length - 1; k++) {
        if (merged[k + 1] - merged[k] < tolerance) continue;
        pieces.push({
          start: { x: line.start.x + dirs[i].x * merged[k], y: line.start.y + dirs[i].y * merged[k] },
          end: { x: line.start.x + dirs[i].x * merged[k + 1], y: line.start.y + dirs[i].y * merged[k + 1] },
          lineId: line.id
        });
      }
    });
    const nodes = [];
    const nodeOf = point => {
      for (let i = 0; i < nodes.length; i++) {
        if (Math.hypot(nodes[i].x - point.x, nodes[i].y - point.y) <= tolerance) return i;
      }
      nodes.push({ x: point.x, y: point.y });
      return nodes.length - 1;
    };
    const halfEdges = [];
    const seenPairs = new Set();
    pieces.forEach(piece => {
      const a = nodeOf(piece.start);
      const b = nodeOf(piece.end);
      if (a === b) return; // degenerate zero-length piece
      const pairKey = Math.min(a, b) + ',' + Math.max(a, b);
      if (seenPairs.has(pairKey)) return; // skip fully overlapping pieces
      seenPairs.add(pairKey);
      halfEdges.push({ from: a, to: b, lineId: piece.lineId });
      halfEdges.push({ from: b, to: a, lineId: piece.lineId });
    });
    if (!halfEdges.length) return [];
    const outgoing = nodes.map(() => []);
    halfEdges.forEach((he, idx) => {
      he.angle = Math.atan2(nodes[he.to].y - nodes[he.from].y, nodes[he.to].x - nodes[he.from].x);
      outgoing[he.from].push(idx);
    });
    outgoing.forEach(list => list.sort((i, j) => halfEdges[i].angle - halfEdges[j].angle));
    const edgeIndex = new Map();
    halfEdges.forEach((he, idx) => edgeIndex.set(he.from + ',' + he.to, idx));
    const visited = new Array(halfEdges.length).fill(false);
    const faces = [];
    for (let startIdx = 0; startIdx < halfEdges.length; startIdx++) {
      if (visited[startIdx]) continue;
      const face = [];
      let cur = startIdx;
      let guard = 0;
      while (!visited[cur] && guard++ <= halfEdges.length) {
        visited[cur] = true;
        face.push(cur);
        const e = halfEdges[cur];
        const revIdx = edgeIndex.get(e.to + ',' + e.from);
        const list = outgoing[e.to];
        const pos = list.indexOf(revIdx);
        cur = list[(pos - 1 + list.length) % list.length];
      }
      if (cur !== startIdx) continue; // malformed cycle (defensive)
      const points = face.map(idx => ({ x: nodes[halfEdges[idx].from].x, y: nodes[halfEdges[idx].from].y }));
      const area = this.calculateSignedArea(points);
      if (area > minArea) {
        faces.push({
          points,
          area,
          edges: face.map(idx => ({ lineId: halfEdges[idx].lineId, bowAngle: 0 }))
        });
      }
    }
    return faces;
  }

  // Where a dimension label the appraiser dragged should draw. The pin is
  // stored with the wall midpoint it was placed against (dimensionAnchor),
  // so the label keeps its offset and follows the wall through any later
  // move, stretch, rotation or paste instead of staying behind.
  pinnedDimensionPoint(line) {
    const pin = line && line.dimensionPosition;
    if (!pin || !Number.isFinite(Number(pin.x)) || !Number.isFinite(Number(pin.y))) return null;
    const mid = { x: (line.start.x + line.end.x) / 2, y: (line.start.y + line.end.y) / 2 };
    const anchor = line.dimensionAnchor && Number.isFinite(Number(line.dimensionAnchor.x)) ? line.dimensionAnchor : mid;
    return { x: Number(pin.x) + mid.x - Number(anchor.x), y: Number(pin.y) + mid.y - Number(anchor.y) };
  }

  // Polar drawing snap for a free-angle wall: the bearing
  // from the anchor rounds to a whole angle step and the length to a whole
  // length step, so an angled wall reads 12.0' at 45 degrees instead of
  // landing on the nearest grid node (12.04' at 41.6 degrees). Returns null
  // when the pointer is too close to the anchor to define a length.
  polarSnap(anchor, pt, lengthStep = 1, angleStepDegrees = 5) {
    const step = Number(lengthStep) > 0 ? Number(lengthStep) : 1;
    const angleStep = Number(angleStepDegrees) > 0 ? Number(angleStepDegrees) : 5;
    const dx = pt.x - anchor.x;
    const dy = pt.y - anchor.y;
    const length = Math.round(Math.hypot(dx, dy) / step) * step;
    if (length < 1e-9) return null;
    const angle = this.degToRad(Math.round((Math.atan2(dy, dx) * 180 / Math.PI) / angleStep) * angleStep);
    // Trig residue (cos 90 = 6e-17) would split a shared corner into two keys.
    const clean = value => { const rounded = Number((Math.round(value / 1e-4) * 1e-4).toFixed(4)); return rounded === 0 ? 0 : rounded; };
    return { x: clean(anchor.x + Math.cos(angle) * length), y: clean(anchor.y + Math.sin(angle) * length) };
  }

  // Default anchor for an area's name/size label. The area centroid is used
  // whenever it sits comfortably inside the outline (every rectangle and most
  // houses); an L, U or courtyard outline whose centroid falls outside or on
  // a wall uses its most interior point instead, so the label never lands
  // outside the area it describes. Every renderer must use this one rule.
  areaLabelAnchor(points) {
    const ring = (points || []).map(point => ({ x: Number(point.x) || 0, y: Number(point.y) || 0 }));
    if (ring.length < 3) return this.polygonCentroid(ring);
    // Rendering and hit-testing ask for this on every pointer move.
    const key = ring.map(point => `${point.x},${point.y}`).join(';');
    if (!this.labelAnchorCache) this.labelAnchorCache = new Map();
    const cached = this.labelAnchorCache.get(key);
    if (cached) return { ...cached };
    const centroid = this.polygonCentroid(ring);
    const pole = this.poleOfInaccessibility(ring);
    let anchor = centroid;
    if (pole && pole.distance > 1e-9) {
      const centroidClearance = this.pointInPolygon(centroid, ring) ? this.boundaryDistance(centroid, ring) : 0;
      if (centroidClearance < pole.distance * 0.5) anchor = { x: pole.x, y: pole.y };
    }
    if (this.labelAnchorCache.size > 500) this.labelAnchorCache.clear();
    this.labelAnchorCache.set(key, { x: anchor.x, y: anchor.y });
    return { x: anchor.x, y: anchor.y };
  }

  boundaryDistance(pt, ring) {
    let best = Infinity;
    for (let i = 0; i < ring.length; i++) {
      best = Math.min(best, this.distanceToSegment(pt, ring[i], ring[(i + 1) % ring.length]));
    }
    return best;
  }

  // Point inside the ring farthest from its boundary (cell refinement after
  // Mapbox polylabel). Returns { x, y, distance } in ring units.
  poleOfInaccessibility(ring, precision = 0.05) {
    const xs = ring.map(point => point.x);
    const ys = ring.map(point => point.y);
    const minX = Math.min(...xs), maxX = Math.max(...xs);
    const minY = Math.min(...ys), maxY = Math.max(...ys);
    const size = Math.min(maxX - minX, maxY - minY);
    if (!(size > 0)) return null;
    const signedDistance = (x, y) => {
      const pt = { x, y };
      const distance = this.boundaryDistance(pt, ring);
      return this.pointInPolygon(pt, ring) ? distance : -distance;
    };
    const cell = (x, y, half) => {
      const d = signedDistance(x, y);
      return { x, y, half, d, max: d + half * Math.SQRT2 };
    };
    const queue = [];
    const half = size / 2;
    for (let x = minX; x < maxX; x += size) {
      for (let y = minY; y < maxY; y += size) queue.push(cell(x + half, y + half, half));
    }
    const centroid = this.polygonCentroid(ring);
    let best = cell(centroid.x, centroid.y, 0);
    const boxCenter = cell((minX + maxX) / 2, (minY + maxY) / 2, 0);
    if (boxCenter.d > best.d) best = boxCenter;
    let guard = 0;
    while (queue.length && guard++ < 20000) {
      let index = 0;
      for (let i = 1; i < queue.length; i++) if (queue[i].max > queue[index].max) index = i;
      const current = queue.splice(index, 1)[0];
      if (current.d > best.d) best = current;
      if (current.max - best.d <= precision) continue;
      const h = current.half / 2;
      queue.push(cell(current.x - h, current.y - h, h), cell(current.x + h, current.y - h, h),
        cell(current.x - h, current.y + h, h), cell(current.x + h, current.y + h, h));
    }
    return { x: best.x, y: best.y, distance: best.d };
  }
}

const Geometry = new GeometryEngine();
if (typeof window !== 'undefined') window.Geometry = Geometry;
if (typeof module !== 'undefined' && module.exports) module.exports = { GeometryEngine, Geometry };

;
/**
 * Sketch area-code catalogue.
 *
 * The application keeps a mutable copy on `app.areaCodes`; this module only
 * owns defaults, validation, and JSON interchange helpers.
 */
(function (root, factory) {
  const api = factory();
  if (typeof module !== 'undefined' && module.exports) module.exports = api;
  if (root) {
    root.AREA_CODES = api.cloneAreaCodes(api.DEFAULT_AREA_CODES);
    root.AreaCodes = api;
    if (root.window && root.window !== root) {
      root.window.AREA_CODES = root.AREA_CODES;
      root.window.AreaCodes = api;
    }
  }
})(typeof globalThis !== 'undefined' ? globalThis : this, function () {
  const DEFAULT_AREA_CODES = [
    { code: 'GLA1', name: 'First Floor', description: 'Gross Living Area', group: 'GLA', multiplier: 1, positive: true, includeInGLA: true, color: '#2B6CB0', fill: 'rgba(43,108,176,.10)' },
    { code: 'GLA2', name: 'Second Floor', description: 'Gross Living Area', group: 'GLA', multiplier: 1, positive: true, includeInGLA: true, color: '#2C5282', fill: 'rgba(44,82,130,.10)' },
    { code: 'GLA3', name: 'Third Floor', description: 'Gross Living Area', group: 'GLA', multiplier: 1, positive: true, includeInGLA: true, color: '#3182CE', fill: 'rgba(49,130,206,.10)' },
    { code: 'GLA4', name: 'Fourth Floor', description: 'Gross Living Area', group: 'GLA', multiplier: 1, positive: true, includeInGLA: true, color: '#4299E1', fill: 'rgba(66,153,225,.10)' },
    { code: 'GBA1', name: 'First Floor', description: 'Gross Building Area', group: 'GBA', multiplier: 1, positive: true, includeInGLA: false, color: '#024F39', fill: 'rgba(2,79,57,.10)' },
    { code: 'GBA2', name: 'Second Floor', description: 'Gross Building Area', group: 'GBA', multiplier: 1, positive: true, includeInGLA: false, color: '#065F40', fill: 'rgba(6,95,64,.10)' },
    { code: 'GBA3', name: 'Third Floor', description: 'Gross Building Area', group: 'GBA', multiplier: 1, positive: true, includeInGLA: false, color: '#017142', fill: 'rgba(1,113,66,.10)' },
    { code: 'GBA4', name: 'Fourth Floor', description: 'Gross Building Area', group: 'GBA', multiplier: 1, positive: true, includeInGLA: false, color: '#238247', fill: 'rgba(35,130,71,.10)' },
    { code: 'GBA5', name: 'Fifth Floor', description: 'Gross Building Area', group: 'GBA', multiplier: 1, positive: true, includeInGLA: false, color: '#42924D', fill: 'rgba(66,146,77,.10)' },
    { code: 'GBA6', name: 'Sixth Floor', description: 'Gross Building Area', group: 'GBA', multiplier: 1, positive: true, includeInGLA: false, color: '#5DA254', fill: 'rgba(93,162,84,.10)' },
    { code: 'GBA7', name: 'Seventh Floor', description: 'Gross Building Area', group: 'GBA', multiplier: 1, positive: true, includeInGLA: false, color: '#77B25B', fill: 'rgba(119,178,91,.10)' },
    { code: 'GBA8', name: 'Eighth Floor', description: 'Gross Building Area', group: 'GBA', multiplier: 1, positive: true, includeInGLA: false, color: '#92C263', fill: 'rgba(146,194,99,.10)' },
    { code: 'GBA9', name: 'Ninth Floor', description: 'Gross Building Area', group: 'GBA', multiplier: 1, positive: true, includeInGLA: false, color: '#ADD26B', fill: 'rgba(173,210,107,.10)' },
    { code: 'GBA10', name: 'Tenth Floor', description: 'Gross Building Area', group: 'GBA', multiplier: 1, positive: true, includeInGLA: false, color: '#C9E274', fill: 'rgba(201,226,116,.10)' },
    { code: 'GAR', name: 'Garage', description: 'Garage', group: 'GAR', multiplier: 1, positive: true, includeInGLA: false, color: '#805AD5', fill: 'rgba(128,90,213,.10)' },
    { code: 'GAR2', name: 'Carport', description: 'Carport', group: 'GAR', multiplier: 1, positive: true, includeInGLA: false, color: '#9F7AEA', fill: 'rgba(159,122,234,.10)' },
    { code: 'BSMT', name: 'Finished BSMT', description: 'Basement', group: 'BSMT', multiplier: 1, positive: true, includeInGLA: false, color: '#4A5568', fill: 'rgba(74,85,104,.10)' },
    { code: 'BSMT2', name: 'Unfinished BSMT', description: 'Basement', group: 'BSMT', multiplier: 1, positive: true, includeInGLA: false, color: '#718096', fill: 'rgba(113,128,150,.10)' },
    { code: 'OFP', name: 'Open Frame Porch', description: 'Porch/Patio', group: 'P/P', multiplier: 1, positive: true, includeInGLA: false, color: '#B7791F', fill: 'rgba(183,121,31,.10)' },
    { code: 'SFP', name: 'Screen Frame Porch', description: 'Porch/Patio', group: 'P/P', multiplier: 1, positive: true, includeInGLA: false, color: '#D69E2E', fill: 'rgba(214,158,46,.10)' },
    { code: 'EFP', name: 'Enclosed Frame Porch', description: 'Porch/Patio', group: 'P/P', multiplier: 1, positive: true, includeInGLA: false, color: '#ECC94B', fill: 'rgba(236,201,75,.10)' },
    { code: 'PAT', name: 'Patio', description: 'Porch/Patio', group: 'P/P', multiplier: 1, positive: true, includeInGLA: false, color: '#DD6B20', fill: 'rgba(221,107,32,.10)' },
    { code: 'PORCH', name: 'Porch', description: 'Porch/Patio', group: 'P/P', multiplier: 1, positive: true, includeInGLA: false, color: '#D69E2E', fill: 'rgba(214,158,46,.10)' },
    { code: 'DK', name: 'Wood Deck', description: 'Deck', group: 'P/P', multiplier: 1, positive: true, includeInGLA: false, color: '#C05621', fill: 'rgba(192,86,33,.10)' },
    { code: 'BAL', name: 'Balcony', description: 'Porch/Patio', group: 'P/P', multiplier: 1, positive: true, includeInGLA: false, color: '#B7791F', fill: 'rgba(183,121,31,.10)' },
    { code: 'POOL', name: 'Pool', description: 'Pool', group: 'OTH', multiplier: 1, positive: true, includeInGLA: false, color: '#00A3C4', fill: 'rgba(0,163,196,.10)' },
    { code: 'SHED', name: 'Shed', description: 'Other', group: 'OTH', multiplier: 1, positive: true, includeInGLA: false, color: '#A0AEC0', fill: 'rgba(160,174,192,.10)' },
    { code: 'LAND', name: 'Land', description: 'Land', group: 'LAND', multiplier: 1, positive: true, includeInGLA: false, color: '#68D391', fill: 'rgba(104,211,145,.10)' },
    { code: 'SITE', name: 'Subject Site', description: 'Subject Site', group: 'SITE', multiplier: 1, positive: true, includeInGLA: false, color: '#4FD1C5', fill: 'rgba(79,209,197,.10)' },
    { code: 'OTH', name: 'Storage', description: 'Other', group: 'OTH', multiplier: 1, positive: true, includeInGLA: false, color: '#A0AEC0', fill: 'rgba(160,174,192,.10)' },
    { code: 'NCA', name: 'Non-Calculated Area', description: 'Non-Calculated Area', group: 'NCA', multiplier: 0, positive: true, includeInGLA: false, color: '#718096', fill: 'rgba(113,128,150,.08)' },
    { code: 'UND', name: 'Undefined Area', description: 'Undefined Area', group: 'UND', multiplier: 0, positive: true, includeInGLA: false, color: '#E53E3E', fill: 'rgba(229,62,62,.08)' }
  ];

  function cloneAreaCodes(codes) {
    return JSON.parse(JSON.stringify(codes || []));
  }

  /* Below-grade rollup for the suite: BSMT is finished basement area and
     BSMT2 unfinished. Both are excluded from GLA but both belong on the
     1004's basement line, so total is the sum and finished is BSMT alone.
     Shared by AreaCalc's recalculate event and the __exportBasement hook so
     the two can never disagree about what the sketch says. */
  function basementRollupFromTotals(totalsByCode) {
    const codes = totalsByCode || {};
    const finished = Math.max(0, Math.round(Number(codes.BSMT) || 0));
    const unfinished = Math.max(0, Math.round(Number(codes.BSMT2) || 0));
    return { total: finished + unfinished, finished, unfinished };
  }

  function normalizeAreaCode(entry) {
    const code = String(entry && entry.code || '').trim().toUpperCase();
    const multiplier = Number(entry && entry.multiplier);
    return {
      code,
      name: String(entry && entry.name || code).trim() || code,
      description: String(entry && entry.description || entry && entry.name || code).trim() || code,
      group: String(entry && entry.group || 'OTH').trim().toUpperCase() || 'OTH',
      multiplier: Number.isFinite(multiplier) ? multiplier : 1,
      positive: entry && entry.positive !== false,
      includeInGLA: Boolean(entry && entry.includeInGLA),
      color: String(entry && entry.color || '#718096'),
      fill: String(entry && entry.fill || 'rgba(113,128,150,.10)'),
      fillColor: String(entry && entry.fillColor || entry && entry.color || '#718096'),
      fillStyle: ['solid', 'none', 'diagonal', 'crosshatch', 'dots'].includes(entry && entry.fillStyle) ? entry.fillStyle : 'solid',
      fillOpacity: Math.max(0, Math.min(100, Number((entry && entry.fillOpacity) ?? 10)))
    };
  }

  function validateAreaCodes(input) {
    if (!Array.isArray(input)) throw new Error('Area codes must be a JSON array.');
    const seen = new Set();
    const result = input.map((entry, index) => {
      const normalized = normalizeAreaCode(entry);
      if (!/^[A-Z0-9][A-Z0-9/_-]{0,15}$/.test(normalized.code)) {
        throw new Error(`Area code ${index + 1} has an invalid code.`);
      }
      if (seen.has(normalized.code)) throw new Error(`Duplicate area code: ${normalized.code}`);
      if (!Number.isFinite(normalized.multiplier) || normalized.multiplier < 0) {
        throw new Error(`Area code ${normalized.code} has an invalid multiplier.`);
      }
      seen.add(normalized.code);
      return normalized;
    });
    if (!result.length) throw new Error('At least one area code is required.');
    return result;
  }

  function fromJSON(text) {
    return validateAreaCodes(JSON.parse(text));
  }

  function toJSON(codes) {
    return JSON.stringify(validateAreaCodes(codes), null, 2);
  }

  function asMap(codes) {
    return new Map((codes || []).map(entry => [entry.code, entry]));
  }

  return { DEFAULT_AREA_CODES, cloneAreaCodes, normalizeAreaCode, validateAreaCodes, fromJSON, toJSON, asMap };
});

;
/**
 * Area breakdown and export engine.
 * Geometry and formatting methods are headless; only show() touches the DOM.
 */
(function (root, factory) {
  const api = factory();
  if (typeof module !== 'undefined' && module.exports) module.exports = api;
  if (root) {
    root.AreaBreakdown = api.AreaBreakdown;
    if (root.window && root.window !== root) root.window.AreaBreakdown = api.AreaBreakdown;
  }
})(typeof globalThis !== 'undefined' ? globalThis : this, function () {
  const EPS = 1e-6;
  const MAX_BOW_ANGLE = 359;

  