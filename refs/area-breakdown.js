class AreaBreakdown {
    constructor(config = {}) {
      this.codeTable = config.codeTable || [];
      this.options = {
        dimensionStyle: 'decimal',
        precision: 1,
        areaPrecision: 2,
        minPieceArea: 0.005,
        mergeRectangles: true,
        ...(config.options || {})
      };
    }

    static signedArea(points) {
      let area = 0;
      for (let i = 0; i < points.length; i++) {
        const next = points[(i + 1) % points.length];
        area += points[i].x * next.y - next.x * points[i].y;
      }
      return area / 2;
    }

    static normalize(points, bows) {
      if (AreaBreakdown.signedArea(points) >= 0) return { pts: points.map(p => ({ ...p })), bows: bows.slice() };
      const pts = points.slice().reverse().map(p => ({ ...p }));
      const count = points.length;
      const reversedBows = [];
      for (let i = 0; i < count; i++) reversedBows.push(-(bows[(count - 2 - i + count) % count] || 0));
      return { pts, bows: reversedBows };
    }

    static segmentArea(p1, p2, bowDeg) {
      if (!bowDeg) return { area: 0, radius: 0, angle: 0 };
      if (Math.abs(bowDeg) > MAX_BOW_ANGLE) {
        throw new RangeError(`bowAngle must be between -${MAX_BOW_ANGLE} and ${MAX_BOW_ANGLE} degrees`);
      }
      const theta = Math.abs(bowDeg) * Math.PI / 180;
      const chord = Math.hypot(p2.x - p1.x, p2.y - p1.y);
      if (theta < 1e-9 || chord < EPS) return { area: 0, radius: 0, angle: 0 };
      const radius = (chord / 2) / Math.sin(theta / 2);
      const area = (radius * radius / 2) * (theta - Math.sin(theta));
      return { area: -Math.sign(bowDeg) * area, radius, angle: Math.abs(bowDeg) };
    }

    static edgeLength(p1, p2, bowDeg) {
      const chord = Math.hypot(p2.x - p1.x, p2.y - p1.y);
      if (!bowDeg) return chord;
      if (Math.abs(bowDeg) > MAX_BOW_ANGLE) {
        throw new RangeError(`bowAngle must be between -${MAX_BOW_ANGLE} and ${MAX_BOW_ANGLE} degrees`);
      }
      const theta = Math.abs(bowDeg) * Math.PI / 180;
      if (theta < 1e-9 || chord < EPS) return chord;
      return ((chord / 2) / Math.sin(theta / 2)) * theta;
    }

    static isRectilinear(points) {
      return points.every((point, i) => {
        const next = points[(i + 1) % points.length];
        return Math.abs(next.x - point.x) <= 1e-4 || Math.abs(next.y - point.y) <= 1e-4;
      });
    }

    rectilinearPartition(points) {
      const ys = [...new Set(points.map(point => +point.y.toFixed(6)))].sort((a, b) => a - b);
      let rects = [];
      for (let band = 0; band < ys.length - 1; band++) {
        const y0 = ys[band], y1 = ys[band + 1];
        if (y1 - y0 < EPS) continue;
        const middle = (y0 + y1) / 2;
        const xs = [];
        for (let i = 0; i < points.length; i++) {
          const point = points[i], next = points[(i + 1) % points.length];
          if ((point.y > middle) === (next.y > middle)) continue;
          const t = (middle - point.y) / (next.y - point.y);
          xs.push(point.x + t * (next.x - point.x));
        }
        xs.sort((a, b) => a - b);
        for (let i = 0; i + 1 < xs.length; i += 2) {
          if (xs[i + 1] - xs[i] > EPS) rects.push({ x0: xs[i], x1: xs[i + 1], y0, y1 });
        }
      }
      return this.options.mergeRectangles ? AreaBreakdown.mergeBands(rects) : rects;
    }

    static mergeBands(rects) {
      const byColumn = new Map();
      rects.forEach(rect => {
        const key = `${rect.x0.toFixed(6)}|${rect.x1.toFixed(6)}`;
        if (!byColumn.has(key)) byColumn.set(key, []);
        byColumn.get(key).push(rect);
      });
      const result = [];
      for (const column of byColumn.values()) {
        column.sort((a, b) => a.y0 - b.y0);
        let current = null;
        for (const rect of column) {
          if (current && Math.abs(rect.y0 - current.y1) < EPS) current.y1 = rect.y1;
          else {
            if (current) result.push(current);
            current = { ...rect };
          }
        }
        if (current) result.push(current);
      }
      return result;
    }

    static squareOff(points, maxDiagonals = 4) {
      const diagonals = [];
      for (let i = 0; i < points.length; i++) {
        const next = points[(i + 1) % points.length];
        if (Math.abs(next.x - points[i].x) > 1e-4 && Math.abs(next.y - points[i].y) > 1e-4) diagonals.push(i);
      }
      if (!diagonals.length || diagonals.length > maxDiagonals) return null;
      const squared = [];
      const deducts = [];
      const baseArea = AreaBreakdown.signedArea(points);
      for (let i = 0; i < points.length; i++) {
        const point = points[i], next = points[(i + 1) % points.length];
        squared.push({ ...point });
        if (Math.abs(next.x - point.x) < 1e-4 || Math.abs(next.y - point.y) < 1e-4) continue;
        const candidates = [{ x: next.x, y: point.y }, { x: point.x, y: next.y }];
        let best = null;
        for (const corner of candidates) {
          const trial = points.slice(0, i + 1).concat([corner], points.slice(i + 1));
          const delta = AreaBreakdown.signedArea(trial) - baseArea;
          if (!best || delta > best.delta) best = { corner, delta };
        }
        if (!best || best.delta <= EPS) return null;
        squared.push(best.corner);
        deducts.push({
          shape: 'Triangle',
          base: Math.abs(next.x - point.x),
          height: Math.abs(next.y - point.y),
          area: -Math.abs((next.x - point.x) * (next.y - point.y)) / 2
        });
      }
      return AreaBreakdown.isRectilinear(squared) ? { pts: squared, deducts } : null;
    }

    static triangulate(points) {
      const vertices = points.map(point => ({ ...point }));
      if (AreaBreakdown.signedArea(vertices) < 0) vertices.reverse();
      const triangles = [];
      const cross = (a, b, c) => (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
      const insideTriangle = (p, a, b, c) =>
        cross(a, b, p) >= -EPS && cross(b, c, p) >= -EPS && cross(c, a, p) >= -EPS;
      let guard = 0;
      while (vertices.length > 3 && guard++ < 10000) {
        let clipped = false;
        for (let i = 0; i < vertices.length; i++) {
          const a = vertices[(i - 1 + vertices.length) % vertices.length];
          const b = vertices[i];
          const c = vertices[(i + 1) % vertices.length];
          if (cross(a, b, c) <= EPS) continue;
          if (vertices.some(point => point !== a && point !== b && point !== c && insideTriangle(point, a, b, c))) continue;
          triangles.push([a, b, c]);
          vertices.splice(i, 1);
          clipped = true;
          break;
        }
        if (!clipped) break;
      }
      if (vertices.length === 3) triangles.push([vertices[0], vertices[1], vertices[2]]);
      return triangles;
    }

    static triangleDims(triangle) {
      const area = Math.abs(AreaBreakdown.signedArea(triangle));
      const sides = [[triangle[0], triangle[1]], [triangle[1], triangle[2]], [triangle[2], triangle[0]]];
      let best = null;
      for (const [start, end] of sides) {
        const base = Math.hypot(end.x - start.x, end.y - start.y);
        if (base < EPS) continue;
        const axis = Math.abs(end.x - start.x) < 1e-4 || Math.abs(end.y - start.y) < 1e-4;
        const candidate = { base, height: 2 * area / base, area, axis };
        if (!best || (candidate.axis && !best.axis)) best = candidate;
      }
      return best || { base: 0, height: 0, area };
    }

    computeForPolygon(poly) {
      const bows = poly.points.map((_, i) => Number(poly.edges && poly.edges[i] && poly.edges[i].bowAngle) || 0);
      const normalized = AreaBreakdown.normalize(poly.points, bows);
      const points = normalized.pts;
      const edgeBows = normalized.bows;
      const chordArea = Math.abs(AreaBreakdown.signedArea(points));
      const pieces = [];

      const pushRectangles = ring => {
        for (const rect of this.rectilinearPartition(ring)) {
          const width = rect.x1 - rect.x0;
          const height = rect.y1 - rect.y0;
          if (width * height >= this.options.minPieceArea) pieces.push({ shape: 'Rectangle', w: width, h: height, area: width * height });
        }
      };
      const pushTriangles = ring => {
        for (const triangle of AreaBreakdown.triangulate(ring)) {
          const dims = AreaBreakdown.triangleDims(triangle);
          if (dims.area >= this.options.minPieceArea) pieces.push({ shape: 'Triangle', base: dims.base, height: dims.height, area: dims.area });
        }
      };

      if (AreaBreakdown.isRectilinear(points)) pushRectangles(points);
      else {
        const squared = points.length === 3 ? null : AreaBreakdown.squareOff(points);
        let reconciled = false;
        if (squared) {
          pushRectangles(squared.pts);
          squared.deducts.forEach(piece => pieces.push(piece));
          reconciled = Math.abs(pieces.reduce((sum, piece) => sum + piece.area, 0) - chordArea) < 0.01;
          if (!reconciled) pieces.length = 0;
        }
        if (!reconciled) pushTriangles(points);
      }

      let arcArea = 0;
      let perimeter = 0;
      for (let i = 0; i < points.length; i++) {
        const next = points[(i + 1) % points.length];
        const segment = AreaBreakdown.segmentArea(points[i], next, edgeBows[i]);
        arcArea += segment.area;
        perimeter += AreaBreakdown.edgeLength(points[i], next, edgeBows[i]);
        if (Math.abs(segment.area) >= this.options.minPieceArea) {
          pieces.push({ shape: 'CircularSegment', radius: segment.radius, angle: segment.angle, area: segment.area });
        }
      }

      const signedGross = chordArea + arcArea;
      // A major segment can outweigh the chord polygon and reverse the
      // boundary winding. Flip every signed component together so the
      // breakdown still reconciles to the positive physical area.
      if (signedGross < 0) pieces.forEach(piece => { piece.area *= -1; });
      const calculatedGross = Math.abs(signedGross);
      const gross = Number.isFinite(poly.gross) ? poly.gross : calculatedGross;
      const pieceSum = pieces.reduce((sum, piece) => sum + piece.area, 0);
      if (Math.abs(gross - pieceSum) > this.options.minPieceArea) {
        pieces.push({ shape: 'Polygon', area: gross - pieceSum, balancing: true });
      }
      const effectiveMultiplier = Number.isFinite(poly.effectiveMultiplier) ? poly.effectiveMultiplier : 1;
      const sign = poly.sign === -1 ? -1 : 1;
      const subtracted = Number(poly.subtracted) || 0;
      const net = Number.isFinite(poly.net) ? poly.net : (gross - subtracted) * effectiveMultiplier * sign;
      return {
        id: poly.id, code: poly.type, name: poly.name || poly.type, description: poly.description,
        page: Number(poly.page) || 0, containerId: poly.containerId ?? null,
        subtractMode: poly.subtractMode || 'auto', subtractedFromId: poly.subtractedFromId ?? null,
        pieces, gross, subtracted, effectiveMultiplier, sign, net,
        perimeter: Number.isFinite(poly.perimeter) ? poly.perimeter : perimeter,
        reconciled: Math.abs(pieces.reduce((sum, piece) => sum + piece.area, 0) - gross) < 0.01
      };
    }

    normalizeModel(model) {
      if (Array.isArray(model)) return { name: 'Untitled', subject: '', commentTables: [], pages: [{ index: 0, polygons: model }], codeTable: this.codeTable };
      const safe = model || {};
      return {
        name: safe.name || 'Untitled',
        subject: safe.subject || '',
        commentTables: safe.commentTables || [],
        pages: safe.pages || [],
        codeTable: safe.codeTable || this.codeTable
      };
    }

    computeAll(model) {
      const normalized = this.normalizeModel(model);
      const results = [];
      normalized.pages.forEach((page, pageIndex) => {
        (page.polygons || []).forEach(poly => {
          if (!poly.type || poly.type === 'UND' || !poly.points || poly.points.length < 3) return;
          results.push(this.computeForPolygon({ ...poly, page: page.index ?? pageIndex }));
        });
      });
      return results;
    }

    rollup(results, codeTable = this.codeTable) {
      const codes = new Map((codeTable || []).map(code => [code.code, code]));
      const totalsByCode = {};
      const totalsByGroup = {};
      let totalGLA = 0;
      let totalAllAreas = 0;
      for (const result of results) {
        // Same rule as the live totals: a deducted area counts as itself
        // unless it is a negative (hole) area.
        if (result.subtractedFromId != null && !((Number(result.net) || 0) > 0)) continue;
        totalsByCode[result.code] = (totalsByCode[result.code] || 0) + result.net;
        const definition = codes.get(result.code) || { group: 'OTH', includeInGLA: false };
        totalsByGroup[definition.group] = (totalsByGroup[definition.group] || 0) + result.net;
        if (definition.includeInGLA) totalGLA += result.net;
        totalAllAreas += result.net;
      }
      return { totalsByCode, totalsByGroup, totalGLA, totalAllAreas };
    }

    fmtDim(value) {
      if (this.options.dimensionStyle === 'feetinches') {
        const negative = value < 0;
        const absolute = Math.abs(value);
        let feet = Math.floor(absolute + 1e-9);
        let inches = Math.round((absolute - feet) * 12);
        if (inches === 12) { feet++; inches = 0; }
        return `${negative ? '-' : ''}${feet}'${inches ? `${inches}\"` : ''}`;
      }
      return Number(value).toFixed(this.options.precision);
    }

    fmtArea(value) { return Number(value).toFixed(this.options.areaPrecision); }

    pieceLine(piece) {
      const area = this.fmtArea(piece.area).padStart(11);
      if (piece.shape === 'Rectangle') return `${this.fmtDim(piece.w).padStart(9)} x ${this.fmtDim(piece.h).padEnd(9)} =${area}`;
      if (piece.shape === 'Triangle') return `0.5 x ${this.fmtDim(piece.base)} x ${this.fmtDim(piece.height)}`.padEnd(21) + ` =${area}`;
      if (piece.shape === 'CircularSegment') return `arc r=${this.fmtDim(piece.radius)} ${piece.angle.toFixed(1)}°`.padEnd(21) + ` =${area}`;
      return `${piece.balancing ? 'balance' : 'polygon'}`.padEnd(21) + ` =${area}`;
    }

    toText(model, options = {}) {
      const normalized = this.normalizeModel(model);
      const results = Array.isArray(model) && model.every(item => item.pieces) ? model : this.computeAll(normalized);
      const showDetail = options.detail !== false;
      const lines = ['AREA CALCULATIONS', ''];
      const byCode = new Map();
      results.forEach(result => {
        if (!byCode.has(result.code)) byCode.set(result.code, []);
        byCode.get(result.code).push(result);
      });
      for (const [code, items] of byCode) {
        for (const result of items) {
          const nested = result.containerId != null ? '  ' : '';
          lines.push(`${nested}${code}  ${result.name || ''}`.trimEnd());
          if (showDetail) {
            result.pieces.forEach(piece => lines.push(`${nested}    ${this.pieceLine(piece)}`));
            lines.push(`${nested}    ${'-'.repeat(33)}`);
          }
          lines.push(`${nested}    ${'Gross'.padEnd(21)} =${this.fmtArea(result.gross).padStart(11)}`);
          if (result.subtracted) lines.push(`${nested}    ${'Subtracted'.padEnd(21)} =${('-' + this.fmtArea(result.subtracted)).padStart(11)}`);
          if (result.effectiveMultiplier !== 1) lines.push(`${nested}    ${'Multiplier'.padEnd(21)}  ${result.effectiveMultiplier.toFixed(2).padStart(10)}`);
          lines.push(`${nested}    ${'Net'.padEnd(21)} =${this.fmtArea(result.net).padStart(11)}`);
          lines.push(`${nested}    ${'Perimeter'.padEnd(21)} =${this.fmtDim(result.perimeter).padStart(11)}`, '');
        }
      }
      const rollup = this.rollup(results, normalized.codeTable);
      lines.push('SUMMARY');
      Object.entries(rollup.totalsByCode).forEach(([code, total]) => lines.push(`  ${code.padEnd(25)}=${this.fmtArea(total).padStart(11)}`));
      lines.push('  ' + '-'.repeat(35));
      lines.push(`  ${'Total Gross Living Area'.padEnd(25)}=${this.fmtArea(rollup.totalGLA).padStart(11)}`);
      lines.push(`  ${'Total All Areas'.padEnd(25)}=${this.fmtArea(rollup.totalAllAreas).padStart(11)}`);
      return lines.join('\n');
    }

    toJSON(model) {
      const normalized = this.normalizeModel(model);
      const results = this.computeAll(normalized);
      const round = (value, places = 2) => +Number(value).toFixed(places);
      return {
        areas: results.map(result => ({
          id: result.id, page: result.page, code: result.code, name: result.name,
          description: result.description, containerId: result.containerId,
          subtractedFromId: result.subtractedFromId, sign: result.sign,
          gross: round(result.gross), subtracted: round(result.subtracted),
          multiplier: result.effectiveMultiplier, net: round(result.net),
          perimeter: round(result.perimeter), reconciled: result.reconciled,
          detail: result.pieces.map(piece => ({ ...piece, area: round(piece.area, 3) }))
        })),
        ...this.rollup(results, normalized.codeTable)
      };
    }

    static escapeXml(value) {
      return String(value == null ? '' : value)
        .replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
        .replace(/\"/g, '&quot;').replace(/'/g, '&apos;');
    }

    static xmlDecimal(value) {
      const number = Number(value) || 0;
      return number.toFixed(1);
    }

    /* XML area-breakdown export namespace. The sketch never reads this XML
       back; it is an export for other tools. */
    toXml(model) {
      const normalized = this.normalizeModel(model);
      const codeMap = new Map((normalized.codeTable || []).map(code => [code.code, code]));
      const lines = ['<?xml version="1.0" encoding="utf-8"?>'];
      lines.push(`<tns:sketch name="${AreaBreakdown.escapeXml(normalized.name)}" unit="foot" xmlns:tns="urn:xmlns:appraisal-sketch:xml">`);
      if (!normalized.subject) lines.push('  <subject />');
      else lines.push(`  <subject value="${AreaBreakdown.escapeXml(normalized.subject)}" />`);
      const comments = Array.from({ length: 3 }, (_, tableIndex) =>
        Array.from({ length: 10 }, (_, commentIndex) => String(normalized.commentTables[tableIndex] && normalized.commentTables[tableIndex][commentIndex] || '')));
      comments.forEach((table, tableIndex) => {
        lines.push(`  <commentTable index="${tableIndex + 1}">`);
        table.forEach(comment => lines.push(`    <comment value="${AreaBreakdown.escapeXml(comment)}" />`));
        lines.push('  </commentTable>');
      });
      normalized.pages.forEach((page, pageIndex) => {
        lines.push(`  <page index="${page.index ?? pageIndex}">`);
        let areaIndex = 0;
        (page.polygons || []).forEach(poly => {
          if (!poly.type || poly.type === 'UND' || !poly.points || poly.points.length < 3) return;
          const definition = codeMap.get(poly.type) || {};
          const attrs = [
            `code="${AreaBreakdown.escapeXml(poly.type)}"`,
            `index="${areaIndex++}"`,
            `name="${AreaBreakdown.escapeXml(poly.name || definition.name || poly.type)}"`,
            `uniqueId="${AreaBreakdown.escapeXml(poly.id)}"`
          ];
          if (poly.subtractedFromId != null) attrs.push(`subtractedFrom="${AreaBreakdown.escapeXml(poly.subtractedFromId)}"`);
          lines.push(`    <area ${attrs.join(' ')}>`);
          lines.push(`      <actualSize>${AreaBreakdown.xmlDecimal(poly.gross)}</actualSize>`);
          lines.push(`      <description>${AreaBreakdown.escapeXml(definition.description || poly.description || poly.type)}</description>`);
          lines.push(`      <multiplier>${Number(poly.effectiveMultiplier ?? definition.multiplier ?? 1)}</multiplier>`);
          lines.push(`      <netSize>${AreaBreakdown.xmlDecimal(poly.net)}</netSize>`);
          lines.push(`      <perimeter>${AreaBreakdown.xmlDecimal(poly.perimeter)}</perimeter>`);
          lines.push(`      <sign>${poly.sign === -1 ? -1 : 1}</sign>`);
          lines.push('    </area>');
        });
        lines.push('  </page>');
      });
      lines.push('</tns:sketch>');
      return '\uFEFF' + lines.join('\r\n');
    }

    previewSvg(model) {
      const normalized = this.normalizeModel(model);
      const pages = normalized.pages.map(page => ({
        ...page,
        polygons: (page.polygons || []).filter(poly => poly.type && poly.type !== 'UND' && poly.points && poly.points.length >= 3)
      })).filter(page => page.polygons.length);
      if (!pages.length) return '<div class="breakdown-preview-empty">No area geometry to preview.</div>';
      const width = 720;
      const height = 520;
      const gap = 24;
      const columns = pages.length > 1 ? 2 : 1;
      const rows = Math.ceil(pages.length / columns);
      const cellWidth = (width - gap * (columns + 1)) / columns;
      const cellHeight = (height - gap * (rows + 1)) / rows;
      const shapes = [];
      pages.forEach((page, pageIndex) => {
        const points = page.polygons.flatMap(poly => poly.points);
        const minX = Math.min(...points.map(point => point.x));
        const maxX = Math.max(...points.map(point => point.x));
        const minY = Math.min(...points.map(point => point.y));
        const maxY = Math.max(...points.map(point => point.y));
        const spanX = Math.max(1, maxX - minX);
        const spanY = Math.max(1, maxY - minY);
        const scale = Math.min((cellWidth - 20) / spanX, (cellHeight - 30) / spanY);
        const column = pageIndex % columns;
        const row = Math.floor(pageIndex / columns);
        const originX = gap + column * (cellWidth + gap) + (cellWidth - spanX * scale) / 2;
        const originY = gap + row * (cellHeight + gap) + (cellHeight - spanY * scale) / 2;
        page.polygons.forEach(poly => {
          const path = poly.points.map((point, index) => {
            const x = originX + (point.x - minX) * scale;
            const y = originY + (maxY - point.y) * scale;
            return `${index ? 'L' : 'M'}${x.toFixed(2)},${y.toFixed(2)}`;
          }).join(' ') + ' Z';
          shapes.push(`<path d="${path}" data-area-id="${AreaBreakdown.escapeXml(poly.id)}"><title>${AreaBreakdown.escapeXml(`${poly.type} - ${poly.name || poly.type}`)}</title></path>`);
        });
        shapes.push(`<text x="${(originX + 4).toFixed(2)}" y="${(originY + cellHeight - 6).toFixed(2)}">Page ${(page.index ?? pageIndex) + 1}</text>`);
      });
      return `<svg class="breakdown-preview-svg" viewBox="0 0 ${width} ${height}" role="img" aria-label="Area geometry preview">${shapes.join('')}</svg>`;
    }

    show(model, options = {}) {
      if (typeof document === 'undefined') return this.toText(model);
      const results = this.computeAll(model);
      if (!results.length) {
        if (options.notify) options.notify('Define at least one area first.');
        return;
      }
      document.getElementById('breakdown-overlay')?.remove();
      const overlay = document.createElement('div');
      overlay.id = 'breakdown-overlay';
      overlay.className = 'breakdown-overlay';
      overlay.innerHTML = `
        <div class="breakdown-dialog breakdown-details-dialog">
          <div class="breakdown-title"><strong>Breakdown Details</strong><button type="button" data-close aria-label="Close">&times;</button></div>
          <div class="breakdown-content"><pre></pre><div class="breakdown-preview">${this.previewSvg(model)}</div></div>
          <div class="breakdown-actions">
            <button type="button" data-copy>Copy for report</button>
            <button type="button" data-style>Feet &amp; inches</button>
            <button type="button" data-json>Download JSON</button>
            <button type="button" data-xml>Download XML</button>
          </div>
        </div>`;
      const pre = overlay.querySelector('pre');
      const refresh = () => { pre.textContent = this.toText(model); };
      const download = (name, body, type) => {
        const url = URL.createObjectURL(new Blob([body], { type }));
        const anchor = document.createElement('a');
        anchor.href = url; anchor.download = name; anchor.click();
        URL.revokeObjectURL(url);
      };
      refresh();
      document.body.appendChild(overlay);
      overlay.querySelector('[data-close]').onclick = () => overlay.remove();
      overlay.onclick = event => { if (event.target === overlay) overlay.remove(); };
      overlay.querySelector('[data-copy]').onclick = async () => {
        try { await navigator.clipboard.writeText(pre.textContent); options.notify?.('Calculations copied.'); }
        catch { options.notify?.('Copy failed. Select the text manually.'); }
      };
      overlay.querySelector('[data-style]').onclick = event => {
        this.options.dimensionStyle = this.options.dimensionStyle === 'decimal' ? 'feetinches' : 'decimal';
        event.target.textContent = this.options.dimensionStyle === 'decimal' ? 'Feet & inches' : 'Decimal feet';
        refresh();
      };
      overlay.querySelector('[data-json]').onclick = () => download('area-breakdown.json', JSON.stringify(this.toJSON(model), null, 2), 'application/json');
      overlay.querySelector('[data-xml]').onclick = () => download('area-breakdown.xml', this.toXml(model), 'application/xml;charset=utf-8');
    }
  }

  return { AreaBreakdown };
});

;
/** Pure area calculation pipeline. */
(function (root, factory) {
  let geometry = root && (root.Geometry || root.window && root.window.Geometry);
  let breakdown = root && (root.AreaBreakdown || root.window && root.window.AreaBreakdown);
  if (typeof module !== 'undefined' && module.exports) {
    geometry = require('./geometry.js').Geometry;
    breakdown = require('./breakdown.js').AreaBreakdown;
  }
  const api = factory(geometry, breakdown);
  if (typeof module !== 'undefined' && module.exports) module.exports = api;
  if (root) {
    root.AreaCalc = api;
    if (root.window && root.window !== root) root.window.AreaCalc = api;
  }
})(typeof globalThis !== 'undefined' ? globalThis : this, function (Geometry, AreaBreakdown) {
  function codeMap(codeTable) {
    return new Map((codeTable || []).map(definition => [definition.code, definition]));
  }

  function normalizePolygon(poly) {
    const points = (poly.points || []).map(point => ({ x: Number(point.x) || 0, y: Number(point.y) || 0 }));
    const edges = points.map((_, index) => ({
      lineId: poly.edges && poly.edges[index] ? poly.edges[index].lineId : null,
      bowAngle: Number(poly.edges && poly.edges[index] && poly.edges[index].bowAngle) || 0
    }));
    return {
      ...poly,
      points,
      edges,
      containerId: null,
      subtractedFromId: null,
      subtractMode: ['auto', 'always', 'never'].includes(poly.subtractMode) ? poly.subtractMode : 'auto',
      subtractTargetId: poly.subtractTargetId ?? null
    };
  }

  function recomputeAreas(polygons) {
    return polygons.map(poly => {
      if (!poly.points || poly.points.length < 3) return { ...poly, gross: 0, area: 0, perimeter: 0 };
      const gross = Geometry.calculateArea(poly.points, poly.edges);
      const perimeter = Geometry.calculatePerimeter(poly.points, poly.edges);
      return { ...poly, gross, area: gross, perimeter };
    });
  }

  // Building level an area code describes: GLA2/GBA2 are the second floor,
  // BSMT codes the basement, other defined codes (garage, porch, deck,
  // patio...) the ground floor, and an undefined outline any level. An area
  // is only deducted from a container on its own level, so a first-floor
  // garage is never taken out of the second floor drawn over it.
  function areaLevel(type) {
    const code = String(type || '').toUpperCase();
    if (!code || code === 'UND') return null;
    if (code.startsWith('BSMT')) return 0;
    const floor = code.match(/^(?:GLA|GBA)(\d+)$/);
    return floor ? Number(floor[1]) : 1;
  }

  function sameLevel(a, b) {
    const first = areaLevel(a.type);
    const second = areaLevel(b.type);
    return first == null || second == null || first === second;
  }

  const idOrder = (a, b) => {
    const x = Number(a.id), y = Number(b.id);
    return Number.isFinite(x) && Number.isFinite(y) ? x - y : String(a.id).localeCompare(String(b.id));
  };

  function buildContainmentTree(polygons) {
    const result = polygons.map(poly => ({ ...poly, containerId: null }));
    const byPage = new Map();
    result.forEach(poly => {
      const page = Number(poly.page) || 0;
      if (!byPage.has(page)) byPage.set(page, []);
      byPage.get(page).push(poly);
    });
    for (const pagePolygons of byPage.values()) {
      for (const inner of pagePolygons) {
        if (!inner.points || inner.points.length < 3) continue;
        const candidates = pagePolygons.filter(outer =>
          outer.id !== inner.id && outer.points && outer.points.length >= 3 &&
          outer.gross > inner.gross + 1e-6 && sameLevel(outer, inner) && Geometry.polygonContains(outer, inner));
        candidates.sort((a, b) => a.gross - b.gross || idOrder(a, b));
        inner.containerId = candidates.length ? candidates[0].id : null;
      }
    }
    return result;
  }

  function cycleMembers(proposed) {
    const color = new Map();
    const cycles = new Set();
    for (const start of proposed.keys()) {
      if (color.get(start) === 2) continue;
      const path = [];
      const position = new Map();
      let current = start;
      while (current != null && proposed.has(current) && color.get(current) !== 2) {
        if (position.has(current)) {
          path.slice(position.get(current)).forEach(id => cycles.add(id));
          break;
        }
        if (color.get(current) === 1) break;
        color.set(current, 1);
        position.set(current, path.length);
        path.push(current);
        current = proposed.get(current);
      }
      path.forEach(id => color.set(id, 2));
    }
    return cycles;
  }

  function resolveSubtraction(polygons) {
    const byId = new Map(polygons.map(poly => [poly.id, poly]));
    const proposed = new Map();
    const warnings = [];
    for (const poly of polygons) {
      let target = null;
      if (poly.subtractMode === 'auto') target = poly.containerId;
      else if (poly.subtractMode === 'always') {
        const explicit = byId.get(poly.subtractTargetId);
        if (poly.subtractTargetId != null && (!explicit || explicit.id === poly.id || (Number(explicit.page) || 0) !== (Number(poly.page) || 0))) {
          warnings.push(`Invalid subtraction target for ${poly.name || poly.type || poly.id}; using spatial containment.`);
          target = poly.containerId;
        } else target = explicit ? explicit.id : poly.containerId;
      }
      proposed.set(poly.id, target ?? null);
    }
    const cyclic = cycleMembers(proposed);
    cyclic.forEach(id => warnings.push(`Cyclic subtraction involving area ${id} was ignored.`));
    return {
      polygons: polygons.map(poly => ({ ...poly, subtractedFromId: cyclic.has(poly.id) ? null : proposed.get(poly.id) })),
      warnings
    };
  }

  function applySubtractions(polygons) {
    const deducted = new Map();
    polygons.forEach(poly => {
      if (poly.subtractedFromId == null) return;
      deducted.set(poly.subtractedFromId, (deducted.get(poly.subtractedFromId) || 0) + (Number(poly.gross) || 0));
    });
    return polygons.map(poly => ({ ...poly, subtracted: deducted.get(poly.id) || 0 }));
  }

  function applyFactors(polygons, definitions) {
    return polygons.map(poly => {
      const definition = definitions.get(poly.type) || { multiplier: 1, positive: true, description: poly.type || '' };
      const override = poly.factorOverride == null ? null : Number(poly.factorOverride);
      const effectiveMultiplier = override == null || !Number.isFinite(override) ? Number(definition.multiplier ?? 1) : override;
      const sign = definition.positive === false ? -1 : 1;
      const net = ((Number(poly.gross) || 0) - (Number(poly.subtracted) || 0)) * effectiveMultiplier * sign;
      return {
        ...poly,
        factorOverride: override == null || !Number.isFinite(override) ? null : override,
        effectiveMultiplier,
        factor: effectiveMultiplier,
        sign,
        description: definition.description || poly.type || '',
        net,
        netArea: net
      };
    });
  }

  function rollup(polygons, definitions) {
    const totalsByCode = {};
    const totalsByGroup = {};
    let totalGLA = 0;
    let totalAllAreas = 0;
    polygons.forEach(poly => {
      if (!poly.type || poly.type === 'UND') return;
      // A deducted area still counts as what it is: a garage inside the
      // house outline is garage area, and a room divided off a floor keeps
      // the floor's total. Only a negative (hole) area stays out, because
      // deducting it from its container already cuts the hole.
      if (poly.subtractedFromId != null && !((Number(poly.net) || 0) > 0)) return;
      const net = Number(poly.net) || 0;
      totalsByCode[poly.type] = (totalsByCode[poly.type] || 0) + net;
      const definition = definitions.get(poly.type) || { group: 'OTH', includeInGLA: false };
      totalsByGroup[definition.group] = (totalsByGroup[definition.group] || 0) + net;
      if (definition.includeInGLA) totalGLA += net;
      totalAllAreas += net;
    });
    return { totalsByCode, totalsByGroup, totalGLA, totalAllAreas };
  }

  function recalculate(inputPolygons, codeTable) {
    const definitions = codeMap(codeTable);
    let polygons = (inputPolygons || []).map(normalizePolygon);
    polygons = recomputeAreas(polygons);
    polygons = buildContainmentTree(polygons);
    const resolved = resolveSubtraction(polygons);
    polygons = applySubtractions(resolved.polygons);
    polygons = applyFactors(polygons, definitions);
    return { polygons, ...rollup(polygons, definitions), warnings: resolved.warnings };
  }

  return {
    recalculate, recomputeAreas, buildContainmentTree, resolveSubtraction,
    applySubtractions, applyFactors, rollup, cycleMembers
  };
});

;
/*
 * Wall Solids - full geometric wall thickness.
 * Computes filled wall polygons (with L/T/X/inline join resolution and
 * hosted-opening gap subtraction) plus interior/exterior face segments for
 * snapping, and polygon-ring face offsets used by the area calculations.
 * All geometry is in world coordinates (feet, y-up).
 */

(function (root) {
  'use strict';

  const EPS = 1e-6;
  const CLUSTER_TOL = 0.02;   // feet: endpoints closer than this share a node
  const MITER_LIMIT = 4;      // clamp miter spikes at sharp corners

  // Bowed walls participate as chords in joins; the renderer approximates
  // their solid with an annular sector (v1 limitation).
  function isStraight(line) {
    return !(line.bowAngle && Math.abs(line.bowAngle) > 0.5);
  }

  function segmentMetrics(line) {
    const dx = line.end.x - line.start.x;
    const dy = line.end.y - line.start.y;
    const length = Math.hypot(dx, dy);
    if (length < EPS) return null;
    return { dx: dx / length, dy: dy / length, length };
  }

  // Signed distance of a point from a segment's centerline, projected along
  // the segment. Returns null when the point is outside the segment interior.
  function interiorProjection(line, metrics, point) {
    const px = point.x - line.start.x;
    const py = point.y - line.start.y;
    const along = px * metrics.dx + py * metrics.dy;
    if (along <= CLUSTER_TOL || along >= metrics.length - CLUSTER_TOL) return null;
    const across = px * -metrics.dy + py * metrics.dx;
    if (Math.abs(across) > CLUSTER_TOL) return null;
    return { along, across };
  }

  // Classify one wall endpoint and return the axial extension applied to the
  // wall solid there:
  //   'free'   - unconnected end, no extension
  //   't'      - butt into a through wall: stop at the through-wall face
  //   'inline' - collinear continuation: extend over the mate's half-thickness
  //   'corner' - L/X node: extend over the mates' half-thickness so the
  //              corner fills
  function classifyJunction(line, metrics, which, thickness, lines, metricsByLine, thicknessOf) {
    const point = line[which];

    // T-junction: the endpoint lands inside another wall's span. The wall
    // terminates at the through wall's near face, accounting for the meeting
    // angle (perpendicular: exactly half the host thickness).
    let host = null;
    for (const other of lines) {
      if (other.id === line.id) continue;
      const otherMetrics = metricsByLine.get(other.id);
      if (!otherMetrics) continue;
      const projection = interiorProjection(other, otherMetrics, point);
      if (!projection) continue;
      const candidate = { line: other, metrics: otherMetrics, projection };
      if (!host || Math.abs(projection.across) < Math.abs(host.projection.across)) host = candidate;
    }
    if (host) {
      const hostThickness = thicknessOf(host.line);
      const sine = Math.abs(metrics.dx * host.metrics.dy - metrics.dy * host.metrics.dx);
      const clampedSine = Math.max(0.25, Math.min(1, sine));
      return { kind: 't', extension: -(hostThickness / 2) / clampedSine };
    }

    // Endpoint cluster: every other wall whose endpoint shares this node.
    const mates = lines.filter(other => {
      if (other.id === line.id) return false;
      return Math.hypot(other.start.x - point.x, other.start.y - point.y) <= CLUSTER_TOL ||
             Math.hypot(other.end.x - point.x, other.end.y - point.y) <= CLUSTER_TOL;
    });
    if (!mates.length) return { kind: 'free', extension: 0 };

    const extension = Math.max(...mates.map(mate => thicknessOf(mate) / 2), thickness / 2);
    const collinearOnly = mates.every(mate => {
      const mateMetrics = metricsByLine.get(mate.id);
      if (!mateMetrics) return false;
      return Math.abs(metrics.dx * mateMetrics.dy - metrics.dy * mateMetrics.dx) < 0.02;
    });
    return { kind: collinearOnly ? 'inline' : 'corner', extension };
  }

  // Axial span of a hosted opening along its wall, clipped to the wall span.
  // SIDE-snapped symbols (single doors) anchor at the hinge, so the opening
  // runs from the hinge toward the side the leaf closes against (the
  // symbol's local +X), which follows the wall's start->end direction unless
  // the swing is flipped. CENTER-snapped symbols anchor mid-opening.
  function gapSpan(symbol, along, length) {
    const width = Number(symbol.wallGapWidth) || 0;
    const library = root.SKETCH_SYMBOL_LIBRARY && root.SKETCH_SYMBOL_LIBRARY.symbols;
    const snap = symbol.wallSnap || (library && library[symbol.symbolType] && library[symbol.symbolType].wallSnap);
    if (snap === 'SIDE') {
      // flipH (Flip Swing) mirrors the leaf's closing side along the wall.
      const dir = (symbol.wallSideFlip ? -1 : 1) * (symbol.flipH ? -1 : 1);
      return dir > 0
        ? { start: Math.max(0, along), end: Math.min(length, along + width) }
        : { start: Math.max(0, along - width), end: Math.min(length, along) };
    }
    const half = width / 2;
    return { start: Math.max(0, along - half), end: Math.min(length, along + half) };
  }

  // Hosted-opening gap intervals along a wall, in axial coordinates relative
  // to the wall start (clipped to the wall span, sorted ascending).
  function wallGaps(line, metrics, symbols) {
    return (symbols || [])
      .filter(symbol => symbol.wallId === line.id && Number(symbol.wallGapWidth) > 0)
      .map(symbol => {
        const along = (symbol.x - line.start.x) * metrics.dx + (symbol.y - line.start.y) * metrics.dy;
        return gapSpan(symbol, along, metrics.length);
      })
      .filter(gap => gap.end > gap.start)
      .sort((a, b) => a.start - b.start);
  }

  /*
   * computeWallSolids(lines, thicknessOf, symbols)
   *   lines       - wall lines (bowed walls skipped here; renderer handles them)
   *   thicknessOf - line -> wall thickness in feet (0 = no solid)
   *   symbols     - hosted symbols that cut gaps
   * Returns {
   *   solids: [{ lineId, type, pieces: [[{x,y} x4], ...] }],
   *   faces:  [{ lineId, side, start, end }],   // side +1/-1 offset faces
   *   junctions: [{ lineId, which, kind }]      // L/T/X/inline classification
   * }
   */
  function computeWallSolids(lines, thicknessOf, symbols = []) {
    const straight = (lines || []).filter(line => isStraight(line) && thicknessOf(line) > 0);
    const metricsByLine = new Map();
    straight.forEach(line => {
      const metrics = segmentMetrics(line);
      if (metrics) metricsByLine.set(line.id, metrics);
    });

    const solids = [];
    const faces = [];
    const junctions = [];

    straight.forEach(line => {
      const metrics = metricsByLine.get(line.id);
      if (!metrics) return;
      const thickness = thicknessOf(line);
      const half = thickness / 2;
      const nx = -metrics.dy, ny = metrics.dx;

      const startJoin = classifyJunction(line, metrics, 'start', thickness, straight, metricsByLine, thicknessOf);
      const endJoin = classifyJunction(line, metrics, 'end', thickness, straight, metricsByLine, thicknessOf);
      junctions.push({ lineId: line.id, which: 'start', kind: startJoin.kind });
      junctions.push({ lineId: line.id, which: 'end', kind: endJoin.kind });

      // Axial bounds of the solid relative to the wall start. Corner/inline
      // joins extend over the mates' half-thickness (backwards at the start,
      // forwards at the end); a T-junction stop (negative extension) keeps
      // the butt wall out of the through wall.
      const solidStart = -startJoin.extension;
      const solidEnd = metrics.length + Math.max(0, endJoin.extension);
      if (solidEnd - solidStart < EPS) return;

      const pointAt = (along, across) => ({
        x: line.start.x + metrics.dx * along + nx * across,
        y: line.start.y + metrics.dy * along + ny * across
      });

      // Cut hosted-opening gaps out of the solid so the opening passes
      // through the full wall thickness.
      const gapIntervals = wallGaps(line, metrics, symbols)
        .map(span => ({ start: Math.max(solidStart, span.start), end: Math.min(solidEnd, span.end) }))
        .filter(span => span.end - span.start > EPS);
      const pieces = [];
      const pushPiece = (a0, a1) => {
        if (a1 - a0 < EPS) return;
        pieces.push([pointAt(a0, half), pointAt(a1, half), pointAt(a1, -half), pointAt(a0, -half)]);
      };
      let cursor = solidStart;
      gapIntervals.forEach(gap => {
        pushPiece(cursor, gap.start);
        cursor = Math.max(cursor, gap.end);
      });
      pushPiece(cursor, solidEnd);
      if (!pieces.length) pushPiece(solidStart, solidEnd);

      solids.push({ lineId: line.id, type: line.type, pieces });

      // Side faces used as snap targets (span the full solid).
      faces.push(
        { lineId: line.id, side: 1, start: pointAt(solidStart, half), end: pointAt(solidEnd, half) },
        { lineId: line.id, side: -1, start: pointAt(solidStart, -half), end: pointAt(solidEnd, -half) }
      );
    });

    return { solids, faces, junctions };
  }

  /*
   * offsetRing(points, offsets)
   * Offsets a polygon ring edge-by-edge along outward normals with miter
   * joins (positive offsets grow the ring, negative shrink it). offsets[i]
   * applies to edge points[i] -> points[i+1]. Returns the offset ring, or
   * null when the offset degenerates (ring collapses or flips).
   */
  function offsetRing(points, offsets) {
    if (!points || points.length < 3 || !offsets || offsets.length !== points.length) return null;
    if (offsets.every(offset => Math.abs(offset) < EPS)) return points.map(p => ({ x: p.x, y: p.y }));

    const signedArea = ring => {
      let area = 0;
      for (let i = 0; i < ring.length; i++) {
        const next = ring[(i + 1) % ring.length];
        area += ring[i].x * next.y - next.x * ring[i].y;
      }
      return area / 2;
    };

    // Work counter-clockwise so the outward normal of edge (a -> b) is
    // (dy, -dx)/len. The result is re-reversed when the input was clockwise.
    const wasClockwise = signedArea(points) < 0;
    const count = points.length;
    const ring = wasClockwise ? points.slice().reverse() : points.slice();
    // Reversed ring edge j (R[j] -> R[j+1]) is original edge (count-2-j)
    // traversed backwards, so that edge's offset rides along with it.
    const edgeOffsets = [];
    for (let j = 0; j < count; j++) {
      edgeOffsets.push(wasClockwise ? offsets[(count - 2 - j + count) % count] : offsets[j]);
    }
    const edges = [];
    for (let i = 0; i < count; i++) {
      const a = ring[i], b = ring[(i + 1) % count];
      const dx = b.x - a.x, dy = b.y - a.y;
      const length = Math.hypot(dx, dy);
      if (length < EPS) return null;
      const ux = dx / length, uy = dy / length;
      const offset = Number(edgeOffsets[i]) || 0;
      // Outward normal for a CCW ring.
      edges.push({ ux, uy, nx: uy, ny: -ux, offset });
    }

    const maxOffset = Math.max(0.01, ...offsets.map(offset => Math.abs(offset)));
    const result = [];
    for (let i = 0; i < count; i++) {
      const prev = edges[(i - 1 + count) % count];
      const next = edges[i];
      // Offset lines: points p with (p - (a + n*o)) . n = 0 for each edge.
      const prevPoint = { x: ring[(i - 1 + count) % count].x + prev.nx * prev.offset, y: ring[(i - 1 + count) % count].y + prev.ny * prev.offset };
      const nextPoint = { x: ring[i].x + next.nx * next.offset, y: ring[i].y + next.ny * next.offset };
      // Turn of the ring at this vertex; near zero means parallel edges.
      const crossU = prev.ux * next.uy - prev.uy * next.ux;
      let vertex;
      if (Math.abs(crossU) < 1e-7) {
        // Parallel edges: project the shared vertex onto the previous offset
        // edge (exact for collinear continuations).
        const shared = ring[i];
        const backAlong = (shared.x - prevPoint.x) * prev.ux + (shared.y - prevPoint.y) * prev.uy;
        vertex = { x: prevPoint.x + prev.ux * backAlong, y: prevPoint.y + prev.uy * backAlong };
      } else {
        // Intersect the two offset edge lines: the vertex lies on the
        // previous edge's offset line where it meets the next edge's.
        const rx = nextPoint.x - prevPoint.x;
        const ry = nextPoint.y - prevPoint.y;
        const t = (rx * next.nx + ry * next.ny) / crossU;
        vertex = { x: prevPoint.x + prev.ux * t, y: prevPoint.y + prev.uy * t };
        // Miter clamp: at sharp corners the intersection can spike far from
        // the original vertex. Limit it along the direction from the vertex.
        const original = ring[i];
        const mvx = vertex.x - original.x, mvy = vertex.y - original.y;
        const miterDistance = Math.hypot(mvx, mvy);
        const limit = MITER_LIMIT * maxOffset;
        if (miterDistance > limit && miterDistance > EPS) {
          vertex = { x: original.x + mvx * limit / miterDistance, y: original.y + mvy * limit / miterDistance };
        }
      }
      result.push(vertex);
    }

    // Degenerate results (collapsed, self-crossed or flipped rings) fall back
    // to the caller. An edge that flips direction means the offset crossed
    // through the ring (e.g. shrinking past the middle of the room).
    const originalSign = Math.sign(signedArea(ring)) || 1;
    if (Math.sign(signedArea(result)) !== originalSign) return null;
    if (Math.abs(signedArea(result)) < EPS) return null;
    for (let i = 0; i < count; i++) {
      const a = result[i], b = result[(i + 1) % count];
      const oa = ring[i], ob = ring[(i + 1) % count];
      if ((b.x - a.x) * (ob.x - oa.x) + (b.y - a.y) * (ob.y - oa.y) <= 0) return null;
    }
    return wasClockwise ? result.reverse() : result;
  }

  // Visible runs of a straight single-line wall once its hosted openings
  // (doors, windows) are cut out, as fractions of the wall from start (0) to
  // end (1). The canvas, the printed report and the SVG/DXF exports all break
  // the wall through this one rule, so an opening drawn on screen is also
  // open on paper.
  function openingRuns(line, symbols) {
    const dx = line.end.x - line.start.x;
    const dy = line.end.y - line.start.y;
    const length = Math.hypot(dx, dy);
    if (length <= 0.001) return [{ start: 0, end: 1 }];
    const gaps = (symbols || [])
      .filter(symbol => symbol.wallId === line.id && Number(symbol.wallGapWidth) > 0)
      .map(symbol => gapSpan(symbol, ((symbol.x - line.start.x) * dx + (symbol.y - line.start.y) * dy) / length, length))
      .filter(gap => gap.end > gap.start)
      .sort((a, b) => a.start - b.start);
    const runs = [];
    let cursor = 0;
    gaps.forEach(gap => {
      if (gap.start > cursor) runs.push({ start: cursor / length, end: gap.start / length });
      cursor = Math.max(cursor, gap.end);
    });
    if (cursor < length) runs.push({ start: cursor / length, end: 1 });
    return runs;
  }

  root.WallSolids = { computeWallSolids, offsetRing, isStraight, gapSpan, openingRuns, CLUSTER_TOL };
  if (typeof module !== 'undefined' && module.exports) module.exports = root.WallSolids;
})(typeof window !== 'undefined' ? window : globalThis);

;
/**
 * Sketch - Input Controller
 * Handles keyboard parsing: distances, Rise/Run, angles, curves, and modifiers.
 *
 * Buffer syntax:
 *   "10"       → 10' in direction of arrow key
 *   "10x5"     → Rise 10, Run 5 (diagonal)
 *   "10<45"    → 10' at 45° absolute angle
 *   Arrow key  → applies buffered distance/angle in that direction
 */

