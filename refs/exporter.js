class Exporter {
  constructor(app) {
    this.app = app;
    this.PPF = window.Geometry.PIXELS_PER_FOOT;
  }

  // --- Shared helpers ---

  escapeXml(value) {
    return String(value == null ? '' : value)
      .replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;').replace(/'/g, '&#39;');
  }

  num(value) {
    return Number.isFinite(Number(value)) ? Number(value) : 0;
  }

  // Screen-oriented transform: world feet (y-up) -> export units (y-down).
  makeTransform(origin) {
    return {
      x: world => (this.num(world.x) - origin.minX) * this.PPF,
      y: world => (origin.maxY - this.num(world.y)) * this.PPF
    };
  }

  // Current page as it appears on screen (filters/visibility applied).
  getVisiblePage() {
    const model = this.app.buildExportModel();
    return model.pages[this.app.currentPage] || model.pages[0] || { lines: [], polygons: [], labels: [], symbols: [] };
  }

  // Highest bow point of a curved wall in world coordinates.
  curveApex(line) {
    const dx = line.end.x - line.start.x;
    const dy = line.end.y - line.start.y;
    const chord = Math.hypot(dx, dy);
    const bow = this.num(line.bowAngle);
    if (chord < 1e-6 || Math.abs(bow) < 0.5) return null;
    const rise = chord / 2 * Math.tan(bow * Math.PI / 720);
    return {
      x: (line.start.x + line.end.x) / 2 + (-dy / chord) * rise,
      y: (line.start.y + line.end.y) / 2 + (dx / chord) * rise
    };
  }

  // Circle through a curved wall's endpoints and apex (world coordinates).
  curveCircle(line) {
    const apex = this.curveApex(line);
    if (!apex) return null;
    const s = line.start, e = line.end;
    const d1 = 2 * (s.x * (e.y - apex.y) + e.x * (apex.y - s.y) + apex.x * (s.y - e.y));
    if (Math.abs(d1) < 1e-9) return null;
    const sq = p => p.x * p.x + p.y * p.y;
    const cx = (sq(s) * (e.y - apex.y) + sq(e) * (apex.y - s.y) + sq(apex) * (s.y - e.y)) / d1;
    const cy = (sq(s) * (apex.x - e.x) + sq(e) * (s.x - apex.x) + sq(apex) * (e.x - s.x)) / d1;
    return { center: { x: cx, y: cy }, radius: Math.hypot(s.x - cx, s.y - cy) };
  }

  flattenedPoints(poly) {
    if (window.Geometry && typeof window.Geometry.flattenRing === 'function') {
      try { return window.Geometry.flattenRing(poly, undefined, 0.02); } catch (_) { /* fall back */ }
    }
    return (poly.points || []).map(point => ({ ...point }));
  }

  // Bounds (world feet) covering everything the export will draw.
  getExportBounds(page) {
    const points = [];
    page.lines.forEach(line => {
      points.push(line.start, line.end);
      const apex = this.curveApex(line);
      if (apex) points.push(apex);
    });
    page.polygons.forEach(poly => points.push(...this.flattenedPoints(poly)));
    page.labels.forEach(label => points.push({ x: this.num(label.x), y: this.num(label.y) }));
    page.symbols.forEach(symbol => points.push({ x: this.num(symbol.x), y: this.num(symbol.y) }));
    page.polygons.forEach(poly => {
      const position = poly.labelPosition;
      if (position) points.push(position);
    });
    const usable = points.filter(p => p && Number.isFinite(this.num(p.x)) && Number.isFinite(this.num(p.y)));
    if (!usable.length) return { minX: 0, maxX: 10, minY: 0, maxY: 10 };
    return {
      minX: Math.min(...usable.map(p => this.num(p.x))),
      maxX: Math.max(...usable.map(p => this.num(p.x))),
      minY: Math.min(...usable.map(p => this.num(p.y))),
      maxY: Math.max(...usable.map(p => this.num(p.y)))
    };
  }

  // Area Line-color overrides, mirroring canvas.buildAreaLineColors.
  buildAreaLineColors(page) {
    const codeMap = new Map((this.app.areaCodes || []).map(code => [code.code, code]));
    const colors = new Map();
    (page.polygons || []).forEach(poly => {
      if (!poly.type || poly.type === 'UND') return;
      const color = poly.lineColor || codeMap.get(poly.type)?.color;
      if (!color) return;
      (poly.edges || []).forEach(edge => {
        if (edge.lineId != null && !colors.has(edge.lineId)) colors.set(edge.lineId, color);
      });
    });
    return { colors, codeMap };
  }

  download(content, fileName, mime) {
    const blob = new Blob([content], { type: mime });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = fileName;
    a.click();
    URL.revokeObjectURL(url);
  }

  // --- SVG ---

  wallColor(line, areaLineColors) {
    const options = this.app.state.options;
    const isExt = line.type === 'exterior';
    return areaLineColors.get(line.id) || line.color || (isExt ? options.exteriorColor : options.interiorColor);
  }

  // Stroke path for a single wall (used for legacy single-line output and for
  // bowed walls, which stay stroked even when solids are enabled).
  strokeWallPath(line, t, areaLineColors, symbols = []) {
    const color = this.escapeXml(this.wallColor(line, areaLineColors));
    const options = this.app.state.options;
    const isExt = line.type === 'exterior';
    const width = this.num(line.lineThickness) || this.num(options[isExt ? 'exteriorWidth' : 'interiorWidth']) || (isExt ? 4 : 2);
    const dash = line.lineStyle === 'dashed' ? ' stroke-dasharray="10 6"' : line.lineStyle === 'dotted' ? ' stroke-dasharray="2 5"' : '';
    const apex = this.curveApex(line);
    const at = f => ({
      x: line.start.x + (line.end.x - line.start.x) * f,
      y: line.start.y + (line.end.y - line.start.y) * f
    });
    const d = apex ? this.curvedWallPath(line, t)
      : window.WallSolids.openingRuns(line, symbols).map(run => {
        const a = at(run.start), b = at(run.end);
        return `M${t.x(a).toFixed(2)},${t.y(a).toFixed(2)} L${t.x(b).toFixed(2)},${t.y(b).toFixed(2)}`;
      }).join(' ');
    if (!d) return '';
    return `<path d="${d}" fill="none" stroke="${color}" stroke-width="${width}"${dash} stroke-linecap="round" stroke-linejoin="round"/>`;
  }

  // Filled wall-solid polygons for straight walls when geometric thickness is
  // enabled. Bowed walls are excluded here (they keep their stroked arc).
  wallSolidsSVG(page, t, areaLineColors) {
    const solids = window.WallSolids.computeWallSolids(page.lines, line => this.app.wallThicknessOf(line), page.symbols);
    const byId = new Map(page.lines.map(line => [line.id, line]));
    return solids.solids.map(solid => {
      const line = byId.get(solid.lineId);
      if (!line) return '';
      const color = this.escapeXml(this.wallColor(line, areaLineColors));
      return solid.pieces.map(piece => {
        const d = piece.map(p => `${t.x(p).toFixed(2)},${t.y(p).toFixed(2)}`).join(' L');
        return `<path d="M${d} Z" fill="${color}"/>`;
      }).join('');
    }).join('');
  }

  // SVG arc path segment for a curved wall (screen units).
  curvedWallPath(line, t) {
    const circle = this.curveCircle(line);
    if (!circle) {
      return `M${t.x(line.start).toFixed(2)},${t.y(line.start).toFixed(2)} L${t.x(line.end).toFixed(2)},${t.y(line.end).toFixed(2)}`;
    }
    const s = { x: t.x(line.start), y: t.y(line.start) };
    const e = { x: t.x(line.end), y: t.y(line.end) };
    const apex = this.curveApex(line);
    const m = { x: t.x(apex), y: t.y(apex) };
    const r = circle.radius * this.PPF;
    // Choose the sweep flag so the arc passes through the apex.
    const c = {
      x: 0, y: 0
    };
    // Candidate centers: offset from chord midpoint along the chord normal.
    const mid = { x: (s.x + e.x) / 2, y: (s.y + e.y) / 2 };
    const chord = Math.hypot(e.x - s.x, e.y - s.y);
    const halfChord = chord / 2;
    const rise = Math.hypot(m.x - mid.x, m.y - mid.y);
    const centerDist = Math.sqrt(Math.max(0, r * r - halfChord * halfChord));
    const n = { x: (m.x - mid.x) / (rise || 1), y: (m.y - mid.y) / (rise || 1) };
    c.x = mid.x - n.x * centerDist;
    c.y = mid.y - n.y * centerDist;
    const a1 = Math.atan2(s.y - c.y, s.x - c.x);
    const aM = Math.atan2(m.y - c.y, m.x - c.x);
    const a2 = Math.atan2(e.y - c.y, e.x - c.x);
    const mod = v => ((v % (2 * Math.PI)) + 2 * Math.PI) % (2 * Math.PI);
    const dM = mod(aM - a1);
    const d2 = mod(a2 - a1);
    // Positive-angle direction passes through the apex when dM <= d2.
    const positive = dM <= d2;
    const sweepAngle = positive ? d2 : 2 * Math.PI - d2;
    const largeArc = sweepAngle > Math.PI ? 1 : 0;
    const sweep = positive ? 1 : 0;
    return `M${s.x.toFixed(2)},${s.y.toFixed(2)} A${r.toFixed(2)},${r.toFixed(2)} 0 ${largeArc},${sweep} ${e.x.toFixed(2)},${e.y.toFixed(2)}`;
  }

  symbolEntities(symbol) {
    const definition = window.SKETCH_SYMBOL_LIBRARY?.symbols?.[symbol.symbolType];
    if (!definition?.geometry?.length) return '';
    const stroke = this.escapeXml(symbol.color || '#111111');
    const polylinePath = (vertices, closed) => {
      if (!vertices?.length) return '';
      let d = `M${this.num(vertices[0].x).toFixed(2)},${(-this.num(vertices[0].y)).toFixed(2)}`;
      const count = closed ? vertices.length : vertices.length - 1;
      for (let i = 0; i < count; i++) {
        const start = vertices[i];
        const end = vertices[(i + 1) % vertices.length];
        const bulge = this.num(start.b);
        const dx = this.num(end.x) - this.num(start.x);
        const dy = this.num(end.y) - this.num(start.y);
        const chord = Math.hypot(dx, dy);
        if (Math.abs(bulge) < 1e-7 || chord < 1e-7) {
          d += ` L${this.num(end.x).toFixed(2)},${(-this.num(end.y)).toFixed(2)}`;
        } else {
          const radius = chord * (1 + bulge * bulge) / (4 * Math.abs(bulge));
          const sweep = Math.abs(4 * Math.atan(bulge));
          d += ` A${radius.toFixed(2)},${radius.toFixed(2)} 0 ${sweep > Math.PI ? 1 : 0},${bulge > 0 ? 0 : 1} ${this.num(end.x).toFixed(2)},${(-this.num(end.y)).toFixed(2)}`;
        }
      }
      return d + (closed ? ' Z' : '');
    };
    const entities = definition.geometry.map(entity => {
      if (entity.entityType === 'LINE') return `<path d="M${this.num(entity.startPoint?.x)},${-this.num(entity.startPoint?.y)} L${this.num(entity.endPoint?.x)},${-this.num(entity.endPoint?.y)}"/>`;
      if (entity.entityType === 'POLYLINE') return `<path d="${polylinePath(entity.vertices, Boolean(entity.closed))}" fill="none"/>`;
      if (entity.entityType === 'ELLIPSE' || entity.entityType === 'ARC') {
        const center = entity.center || { x: 0, y: 0 };
        const axis = entity.axisPoint || center;
        const rx = Math.hypot(this.num(axis.x) - this.num(center.x), this.num(axis.y) - this.num(center.y));
        const ry = rx * Math.abs(this.num(entity.ratio) || 1);
        if (entity.entityType === 'ELLIPSE') return `<ellipse cx="${this.num(center.x)}" cy="${-this.num(center.y)}" rx="${rx}" ry="${ry}" fill="none"/>`;
        const startAngle = this.num(entity.startAngle) * Math.PI / 180;
        const endAngle = this.num(entity.endAngle) * Math.PI / 180;
        const start = { x: this.num(center.x) + rx * Math.cos(startAngle), y: -this.num(center.y) - ry * Math.sin(startAngle) };
        const end = { x: this.num(center.x) + rx * Math.cos(endAngle), y: -this.num(center.y) - ry * Math.sin(endAngle) };
        const sweep = ((this.num(entity.endAngle) - this.num(entity.startAngle)) % 360 + 360) % 360;
        return `<path d="M${start.x},${start.y} A${rx},${ry} 0 ${sweep > 180 ? 1 : 0},0 ${end.x},${end.y}" fill="none"/>`;
      }
      if (entity.entityType === 'TEXT') {
        const insert = entity.insertionPoint || entity.insertPoint || { x: 0, y: 0 };
        return `<text x="${this.num(insert.x)}" y="${-this.num(insert.y)}" font-size="${Math.max(1, this.num(entity.fontSize || entity.FontSize) || 10)}" stroke="none" fill="currentColor">${this.escapeXml(entity.textString || '')}</text>`;
      }
      return '';
    }).join('');
    return { entities, stroke };
  }

  autoPostLines(poly) {
    const s = this.app.state;
    const codeMap = new Map((this.app.areaCodes || []).map(code => [code.code, code]));
    if (!poly.type || poly.type === 'UND') return [];
    const lines = [];
    if (s.autoPost.name) lines.push(poly.name || codeMap.get(poly.type)?.name || poly.type);
    if (s.autoPost.calcs) lines.push(`${Math.round(this.num(poly.net ?? poly.area ?? poly.gross))} ${s.autoPost.suffix || 'sf'}`);
    return lines;
  }

  exportSVG() {
    const page = this.getVisiblePage();
    const bounds = this.getExportBounds(page);
    const margin = 40; // export units
    const width = (bounds.maxX - bounds.minX) * this.PPF + margin * 2;
    const height = (bounds.maxY - bounds.minY) * this.PPF + margin * 2;
    const padded = { minX: bounds.minX - margin / this.PPF, maxX: bounds.maxX + margin / this.PPF, minY: bounds.minY - margin / this.PPF, maxY: bounds.maxY + margin / this.PPF };
    const t = this.makeTransform(padded);
    const options = this.app.state.options;
    const { colors: areaLineColors, codeMap } = this.buildAreaLineColors(page);

    const polygons = page.polygons.filter(poly => poly.points?.length >= 3).map(poly => {
      const definition = codeMap.get(poly.type) || {};
      const outline = this.escapeXml(poly.lineColor || definition.color || '#111111');
      const fillColor = poly.fillColor || definition.fillColor || definition.color || '#ffffff';
      const opacity = poly.fillOpacity ?? definition.fillOpacity ?? 10;
      const style = poly.fillPattern || definition.fillStyle || 'solid';
      let fill = style === 'none' || poly.type === 'UND' ? 'none' : fillColor;
      let fillOpacity = style === 'none' || poly.type === 'UND' ? 0 : Math.max(0, Math.min(100, this.num(opacity))) / 100;
      const d = this.flattenedPoints(poly).map(p => `${t.x(p).toFixed(2)},${t.y(p).toFixed(2)}`).join(' L');
      return `<path d="M${d} Z" fill="${fill}" fill-opacity="${fillOpacity.toFixed(2)}" stroke="${outline}" stroke-width="${Math.max(1, this.num(poly.lineThickness) || 1.5)}" stroke-linejoin="round"/>`;
    }).join('');

    // Walls: filled solid polygons when geometric wall thickness is enabled
    // (bowed walls stay stroked arcs), otherwise legacy single-line strokes.
    const solidsEnabled = Boolean(this.app.wallSolidsEnabled && this.app.wallSolidsEnabled());
    const walls = solidsEnabled
      ? this.wallSolidsSVG(page, t, areaLineColors) +
        page.lines.filter(line => this.curveApex(line)).map(line => this.strokeWallPath(line, t, areaLineColors, page.symbols)).join('')
      : page.lines.map(line => this.strokeWallPath(line, t, areaLineColors, page.symbols)).join('');

    const areaLabels = page.polygons.map(poly => {
      const lines = this.autoPostLines(poly);
      if (!lines.length || poly.points?.length < 3) return '';
      const centroid = this.app.getAreaLabelPosition(poly);
      const anchor = { x: t.x(poly.labelPosition || centroid), y: t.y(poly.labelPosition || centroid) };
      const rotation = -this.num(poly.labelRotation);
      const fontSize = Math.max(9, this.num(poly.labelFontSize) || this.num(options.autoPostFontSize) || 16);
      const family = this.escapeXml(poly.labelFontFamily || options.autoPostFont || 'Arial');
      const color = this.escapeXml(poly.labelColor || options.textColor || '#111111');
      const lineSpacing = fontSize * 1.25;
      const texts = lines.map((text, index) => {
        const y = (index - (lines.length - 1) / 2) * lineSpacing;
        const bold = (poly.labelBold ?? options.autoPostBold) || index === 0;
        const italic = (poly.labelItalic ?? options.autoPostItalic) ? ' font-style="italic"' : '';
        return `<text x="0" y="${y.toFixed(2)}" font-size="${fontSize}"${italic} font-weight="${bold ? 'bold' : 'normal'}">${this.escapeXml(text)}</text>`;
      }).join('');
      return `<g transform="translate(${anchor.x.toFixed(2)} ${anchor.y.toFixed(2)}) rotate(${rotation.toFixed(2)})" fill="${color}" font-family="${family}" text-anchor="middle" dominant-baseline="middle">${texts}</g>`;
    }).join('');

    const labels = page.labels.map(label => {
      const p = { x: t.x(label), y: t.y(label) };
      const size = Math.max(7, this.num(label.fontSize) || 12);
      return `<text x="${p.x.toFixed(2)}" y="${p.y.toFixed(2)}" transform="rotate(${-this.num(label.rotation)} ${p.x.toFixed(2)} ${p.y.toFixed(2)})" text-anchor="middle" dominant-baseline="middle" fill="${this.escapeXml(label.color && String(label.color).toLowerCase() !== '#e2e8f0' ? label.color : '#111111')}" font-family="${this.escapeXml(label.fontFamily || 'Arial')}" font-size="${size}" style="${label.bold ? 'font-weight:bold;' : ''}${label.italic ? 'font-style:italic;' : ''}">${this.escapeXml(label.text || '')}</text>`;
    }).join('');

    const symbols = page.symbols.map(symbol => {
      const scale = this.PPF * (this.num(symbol.scale) || 1) / 12;
      const built = this.symbolEntities(symbol);
      if (!built || !built.entities) return '';
      const p = { x: t.x(symbol), y: t.y(symbol) };
      const strokeWidth = Math.max(1.5 / scale, 0.2);
      return `<g transform="translate(${p.x.toFixed(2)} ${p.y.toFixed(2)}) rotate(${-this.num(symbol.rotation)}) scale(${scale.toFixed(4)})" color="${built.stroke}" stroke="currentColor" stroke-width="${strokeWidth.toFixed(3)}" stroke-linecap="round" stroke-linejoin="round">${built.entities}</g>`;
    }).join('');

    const svg = `<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="${width.toFixed(0)}" height="${height.toFixed(0)}" viewBox="0 0 ${width.toFixed(0)} ${height.toFixed(0)}">
<!-- Generated by Appraisal Sketch 1.1. 1 unit = 1 screen pixel (${this.PPF} units per foot). North: ${this.app.normalizeNorthAngle(this.app.state.northAngle).toFixed(1)} degrees clockwise from up. -->
${polygons}${walls}${areaLabels}${labels}${symbols}
</svg>`;
    this.download(svg, `${this.app.fileName}.svg`, 'image/svg+xml');
    this.app.ui.notify('SVG exported.');
  }

  // --- DXF (R12 ASCII) ---

  dxfPair(code, value) {
    return `  ${code}\n${value}\n`;
  }

  dxfText(value) {
    // DXF text escapes are minimal; strip control characters.
    return String(value == null ? '' : value).replace(/[\x00-\x1f]/g, ' ');
  }

  exportDXF() {
    const page = this.getVisiblePage();
    const bounds = this.getExportBounds(page);
    const { colors: _areaLineColors, codeMap: _codeMap } = this.buildAreaLineColors(page);
    const entities = [];
    const add = entity => entities.push(entity);

    const dxfLine = (layer, a, b) =>
      this.dxfPair(0, 'LINE') + this.dxfPair(8, layer) +
      this.dxfPair(10, a.x.toFixed(4)) + this.dxfPair(20, a.y.toFixed(4)) +
      this.dxfPair(11, b.x.toFixed(4)) + this.dxfPair(21, b.y.toFixed(4));

    const dxfArc = (layer, circle, startAngle, endAngle) =>
      this.dxfPair(0, 'ARC') + this.dxfPair(8, layer) +
      this.dxfPair(10, circle.center.x.toFixed(4)) + this.dxfPair(20, circle.center.y.toFixed(4)) +
      this.dxfPair(40, circle.radius.toFixed(4)) +
      this.dxfPair(50, startAngle.toFixed(4)) + this.dxfPair(51, endAngle.toFixed(4));

    const dxfPolyline = (layer, points) => {
      let out = this.dxfPair(0, 'POLYLINE') + this.dxfPair(8, layer) + this.dxfPair(66, 1) + this.dxfPair(70, 1);
      points.forEach(p => {
        out += this.dxfPair(0, 'VERTEX') + this.dxfPair(8, layer) + this.dxfPair(10, p.x.toFixed(4)) + this.dxfPair(20, p.y.toFixed(4));
      });
      return out + this.dxfPair(0, 'SEQEND');
    };

    const dxfText = (layer, position, height, text, rotation = 0) =>
      this.dxfPair(0, 'TEXT') + this.dxfPair(8, layer) +
      this.dxfPair(10, position.x.toFixed(4)) + this.dxfPair(20, position.y.toFixed(4)) +
      this.dxfPair(40, height.toFixed(2)) + this.dxfPair(1, this.dxfText(text)) +
      (rotation ? this.dxfPair(50, rotation.toFixed(2)) : '');

    // Walls: straight walls as LINE, bowed walls as ARC (world is y-up, so
    // DXF angles are used as-is). With geometric wall thickness enabled the
    // straight walls export as closed solid polygons instead.
    const solidsEnabledDXF = Boolean(this.app.wallSolidsEnabled && this.app.wallSolidsEnabled());
    if (solidsEnabledDXF) {
      const solids = window.WallSolids.computeWallSolids(page.lines, line => this.app.wallThicknessOf(line), page.symbols);
      const byId = new Map(page.lines.map(line => [line.id, line]));
      solids.solids.forEach(solid => {
        const line = byId.get(solid.lineId);
        if (!line) return;
        const layer = line.type === 'exterior' ? 'WALLS-EXTERIOR' : 'WALLS-INTERIOR';
        solid.pieces.forEach(piece => add(dxfPolyline(layer, piece)));
      });
    }
    page.lines.forEach(line => {
      const layer = line.type === 'exterior' ? 'WALLS-EXTERIOR' : 'WALLS-INTERIOR';
      const circle = this.curveCircle(line);
      if (!circle) {
        if (!solidsEnabledDXF) window.WallSolids.openingRuns(line, page.symbols).forEach(run => add(dxfLine(layer,
          { x: line.start.x + (line.end.x - line.start.x) * run.start, y: line.start.y + (line.end.y - line.start.y) * run.start },
          { x: line.start.x + (line.end.x - line.start.x) * run.end, y: line.start.y + (line.end.y - line.start.y) * run.end })));
        return;
      }
      const apex = this.curveApex(line);
      const toDeg = v => ((v * 180 / Math.PI) % 360 + 360) % 360;
      const aS = Math.atan2(line.start.y - circle.center.y, line.start.x - circle.center.x);
      const aE = Math.atan2(line.end.y - circle.center.y, line.end.x - circle.center.x);
      const aM = Math.atan2(apex.y - circle.center.y, apex.x - circle.center.x);
      const mod = v => ((v % (2 * Math.PI)) + 2 * Math.PI) % (2 * Math.PI);
      // DXF arcs run counter-clockwise from 50 to 51; pick the orientation
      // whose CCW travel passes through the apex.
      const ccw = mod(aM - aS) <= mod(aE - aS);
      add(dxfArc(layer, circle, toDeg(ccw ? aS : aE), toDeg(ccw ? aE : aS)));
    });

    // Area polygons as closed polylines (flattened so bowed edges export).
    page.polygons.forEach(poly => {
      if (poly.points?.length < 3) return;
      add(dxfPolyline(poly.type && poly.type !== 'UND' ? `AREA-${poly.type}` : 'AREA-UNDEFINED', this.flattenedPoints(poly)));
    });

    // Auto-posted area labels and free text labels.
    page.polygons.forEach(poly => {
      const lines = this.autoPostLines(poly);
      if (!lines.length || poly.points?.length < 3) return;
      const centroid = this.app.getAreaLabelPosition(poly);
      const anchor = poly.labelPosition || centroid;
      const height = Math.max(0.5, (this.num(poly.labelFontSize) || this.num(this.app.state.options.autoPostFontSize) || 16) / this.PPF);
      lines.forEach((text, index) => {
        const y = anchor.y + ((lines.length - 1) / 2 - index) * height * 1.25;
        add(dxfText('AREA-TEXT', { x: anchor.x, y }, height, text, this.num(poly.labelRotation)));
      });
    });
    page.labels.forEach(label => {
      const height = Math.max(0.5, (this.num(label.fontSize) || 12) / this.PPF);
      add(dxfText('TEXT', label, height, label.text || '', this.num(label.rotation)));
    });

    const dxf =
      this.dxfPair(0, 'SECTION') + this.dxfPair(2, 'HEADER') +
      this.dxfPair(9, '$ACADVER') + this.dxfPair(1, 'AC1009') +
      this.dxfPair(9, '$EXTMIN') + this.dxfPair(10, bounds.minX.toFixed(4)) + this.dxfPair(20, bounds.minY.toFixed(4)) +
      this.dxfPair(9, '$EXTMAX') + this.dxfPair(10, bounds.maxX.toFixed(4)) + this.dxfPair(20, bounds.maxY.toFixed(4)) +
      this.dxfPair(0, 'ENDSEC') +
      this.dxfPair(0, 'SECTION') + this.dxfPair(2, 'ENTITIES') +
      entities.join('') +
      this.dxfPair(0, 'ENDSEC') +
      this.dxfPair(0, 'EOF');

    this.download(dxf, `${this.app.fileName}.dxf`, 'application/dxf');
    this.app.ui.notify('DXF exported.');
  }
}

window.Exporter = Exporter;

;
/**
 * Sketch - UI Manager
 * Handles DOM interactions, Ribbon tabs, Panel states, and Status Bar.
 */

