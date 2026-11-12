/* ====== CAD IMPORT ======
   Adds DXF (and, with a one-time external conversion, DWG) import to the sketch
   app. Converted entities land in state.lines / state.polygons / state.symbols,
   so every downstream feature -- areas, dimensions, Define, layers, undo --
   works on imported geometry exactly as it does on hand-drawn geometry.

   Why this lives on the PREVIEW layer rather than inside the bundle's own
   importFromFile(): sketch.bundle.html is a generated artefact. Patching it
   directly means the next upstream build silently reverts the feature. This
   module runs after the bundle, installs window.CadImport, and is picked up by
   both apps from one file.

   UNIT HANDLING -- read this before changing anything.
   The sketch app is FOOT-based: state.lines coordinates are decimal feet and
   Geometry.PIXELS_PER_FOOT (default 20) converts them to screen pixels.
   CAD files are usually INCH-based (DXF $INSUNITS 1 = inches) or metric.
   Insunits -> feet scale factors:
     1 inches  -> 1/12      4 mm      -> 1/304.8
     2 feet    -> 1         5 cm      -> 1/30.48
     3 miles   -> 5280      6 m       -> 1/0.3048
     0/other   -> assume inches (1/12), the common case for US plan sets.
   Metric drawings are auto-detected by extent when the file is unitless or
   millimetre-scaled and the geometry is building-sized, because a 12000-unit
   wall is metres far more often than it is 1000 feet.
*/

(function (root) {
  'use strict';

  var UNNAMED_LAYER = '0';

  /* ---------- minimal DXF reader (no dependencies) ----------
     We only need enough of DXF to pull 2D entities out of the ENTITIES
     section: a (group code, value) pair stream. Group codes 0 start an
     entity, 8 is its layer, 10/20/11/21 are points, 42 is a polyline bulge,
     70 is a flag word, 40 a radius, 1 a text string. That covers the
     entities this app can represent; everything else is skipped safely. */

  function parseDxfPairs(text) {
    var lines = String(text).split(/\r\n|\r|\n/);
    var pairs = [];
    for (var i = 0; i + 1 < lines.length; i += 2) {
      var code = parseInt(lines[i], 10);
      if (isNaN(code)) continue;
      pairs.push({ code: code, value: lines[i + 1] });
    }
    return pairs;
  }

  function sectionRanges(pairs) {
    var ranges = {};
    var current = null;
    for (var i = 0; i < pairs.length; i++) {
      var p = pairs[i];
      if (p.code === 0 && p.value === 'SECTION') { current = null; continue; }
      if (p.code === 2 && current === null) { current = p.value; ranges[current] = { start: i, end: pairs.length }; continue; }
      if (p.code === 0 && p.value === 'ENDSEC' && current) { ranges[current].end = i; current = null; }
    }
    return ranges;
  }

  function headerInsunits(pairs, ranges) {
    var hdr = ranges.HEADER;
    if (!hdr) return null;
    for (var i = hdr.start; i < hdr.end - 1; i++) {
      if (pairs[i].code === 9 && pairs[i].value === '$INSUNITS') {
        return parseInt(pairs[i + 1].value, 10);
      }
    }
    return null;
  }

  /* Read the 2D entities we can represent. Coordinates are kept in the file's
     own units; scaling happens later once the unit is decided.

     Vertex accumulation is an explicit state machine rather than a side effect
     of the generic switch: LWPOLYLINE and legacy POLYLINE both emit repeated
     10/20 pairs, and an earlier version wrote them through shared per-entity
     fields, which collapsed a 4-vertex rectangle into two duplicated points. */
  function readEntities(pairs, ranges) {
    var ents = ranges.ENTITIES;
    if (!ents) return [];
    var out = [];
    var cur = null;

    function num(v) { var n = parseFloat(v); return isFinite(n) ? n : 0; }

    /* Open a new vertex on the entity we are currently reading. */
    function pushVertex(e) {
      if (!e.vertices) e.vertices = [];
      var vtx = { x: 0, y: 0, bulge: 0 };
      e.vertices.push(vtx);
      return vtx;
    }

    for (var i = ents.start; i < ents.end; i++) {
      var p = pairs[i];
      var code = p.code, v = p.value;

      if (code === 0) {
        var t = String(v).trim().toUpperCase();
        if (t === 'LINE' || t === 'LWPOLYLINE' || t === 'POLYLINE' ||
            t === 'CIRCLE' || t === 'ARC' || t === 'TEXT' || t === 'MTEXT' ||
            t === 'INSERT' || t === 'POINT') {
          cur = { type: t, layer: UNNAMED_LAYER, flag: 0 };
          out.push(cur);
        } else if (t === 'VERTEX') {
          /* Legacy POLYLINE: vertices are separate VERTEX entities belonging to
             the preceding POLYLINE, terminated by SEQEND. Open its vertex here
             so the following 10/20 FILL that vertex instead of appending new
             ones -- opening it lazily in `case 10` would create one vertex per
             coordinate and double the vertex list. */
          if (cur && cur.type === 'POLYLINE') {
            cur._vtx = pushVertex(cur);
          } else {
            cur = null;
          }
        } else if (t === 'SEQEND') {
          if (cur && cur.type === 'POLYLINE') cur._vtx = null;
          cur = null;
        } else {
          cur = null;
        }
        continue;
      }
      if (!cur) continue;

      var isPolyline = (cur.type === 'LWPOLYLINE' || cur.type === 'POLYLINE');

      switch (code) {
        case 8: cur.layer = String(v || UNNAMED_LAYER); break;
        case 10:
          if (isPolyline) {
            /* Legacy POLYLINE already opened this vertex at the VERTEX entity,
               so fill it; LWPOLYLINE opens one per 10 group. */
            if (cur.type === 'LWPOLYLINE' || !cur._vtx) cur._vtx = pushVertex(cur);
            cur._vtx.x = num(v);
            cur.x = num(v);
          } else {
            cur.x = num(v);
          }
          break;
        case 20:
          if (isPolyline) {
            /* 20 completes the vertex opened by the preceding 10 (or VERTEX).
               If a 20 ever arrives first, open a vertex so the Y is not lost. */
            if (!cur._vtx) cur._vtx = pushVertex(cur);
            cur._vtx.y = num(v);
            cur.y = num(v);
          } else {
            cur.y = num(v);
          }
          break;
        case 11: cur.x2 = num(v); break;
        case 21: cur.y2 = num(v); break;
        case 40: cur.radius = num(v); break;
        case 42:
          cur.bulge = num(v);
          /* Bulge belongs to the vertex it follows, and describes the arc from
             that vertex to the next one. */
          if (cur._vtx) cur._vtx.bulge = num(v);
          break;
        case 50: cur.startAngle = num(v); break;
        case 51: cur.endAngle = num(v); break;
        case 70: cur.flag = parseInt(v, 10) || 0; break;
        case 1: cur.text = (cur.text || '') + String(v); break;
        case 3: cur.text = (cur.text || '') + String(v); break;
        case 2: cur.name = String(v); break;
        default: break;
      }
    }

    /* Drop the scratch pointer; vertices are the source of truth. */
    out.forEach(function (e) { delete e._vtx; });
    return out;
  }

  /* ---------- unit detection ---------- */

  function extentOf(entities) {
    var minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity, n = 0;
    entities.forEach(function (e) {
      function add(x, y) {
        if (!isFinite(x) || !isFinite(y)) return;
        minX = Math.min(minX, x); minY = Math.min(minY, y);
        maxX = Math.max(maxX, x); maxY = Math.max(maxY, y);
        n++;
      }
      if (e.vertices && e.vertices.length) {
        e.vertices.forEach(function (v) { add(v.x, v.y); });
      } else if (e.radius != null && e.x != null) {
        add(e.x - e.radius, e.y - e.radius);
        add(e.x + e.radius, e.y + e.radius);
      } else {
        add(e.x, e.y);
        add(e.x2, e.y2);
      }
    });
    if (!n) return null;
    return { minX: minX, minY: minY, maxX: maxX, maxY: maxY, width: maxX - minX, height: maxY - minY };
  }

  /* Decide how to turn file units into the app's feet. */
  function pickScale(insunits, extent) {
    var INCH = 1 / 12, FEET = 1, MILE = 5280,
        MM = 1 / 304.8, CM = 1 / 30.48, M = 1 / 0.3048;

    if (insunits === 2) return { scale: FEET, units: 'feet', source: 'insunits' };
    if (insunits === 3) return { scale: MILE, units: 'miles', source: 'insunits' };
    if (insunits === 4) return { scale: MM, units: 'mm', source: 'insunits' };
    if (insunits === 5) return { scale: CM, units: 'cm', source: 'insunits' };
    if (insunits === 6) return { scale: M, units: 'm', source: 'insunits' };
    if (insunits === 1) return { scale: INCH, units: 'inches', source: 'insunits' };

    /* Unitless or unknown: infer from size. A house is tens of metres or
       hundreds of inches; both are four-digit numbers, so disambiguate by
       asking which reading gives the more plausible building. */
    if (extent) {
      var span = Math.max(extent.width, extent.height);
      if (span > 0) {
        var asInches = span * INCH, asFeet = span * FEET, asMm = span * MM, asM = span * M;
        function plausible(ft) { return ft > 8 && ft < 2000; }
        if (plausible(asMm) && span > 1000) return { scale: MM, units: 'mm', source: 'inferred' };
        if (plausible(asInches)) return { scale: INCH, units: 'inches', source: 'inferred' };
        if (plausible(asFeet)) return { scale: FEET, units: 'feet', source: 'inferred' };
        if (plausible(asM)) return { scale: M, units: 'm', source: 'inferred' };
      }
    }
    return { scale: INCH, units: 'inches', source: 'default' };
  }

  /* ---------- geometry conversion ---------- */

  function bulgeToArcPoints(v0, v1, bulge, segments) {
    /* Flatten a DXF bulge into a polyline arc. bulge = tan(sweep/4). */
    var pts = [];
    var b = Number(bulge) || 0;
    if (Math.abs(b) < 1e-9) return pts;
    var chord = Math.hypot(v1.x - v0.x, v1.y - v0.y);
    if (chord < 1e-12) return pts;
    var sweep = 4 * Math.atan(b);
    var radius = chord / (2 * Math.sin(Math.abs(sweep) / 2) || 1e-12);
    /* Centre lies on the perpendicular bisector; sign of bulge picks the side. */
    var mx = (v0.x + v1.x) / 2, my = (v0.y + v1.y) / 2;
    var nx = -(v1.y - v0.y) / chord, ny = (v1.x - v0.x) / chord;
    var h = Math.sqrt(Math.max(0, radius * radius - (chord / 2) * (chord / 2)));
    var cx = mx + nx * h * (b > 0 ? 1 : -1);
    var cy = my + ny * h * (b > 0 ? 1 : -1);
    var a0 = Math.atan2(v0.y - cy, v0.x - cx);
    var steps = Math.max(2, segments || Math.min(48, Math.max(6, Math.round(Math.abs(sweep) / (Math.PI / 18)))));
    for (var s = 1; s < steps; s++) {
      var a = a0 + sweep * (s / steps);
      pts.push({ x: cx + radius * Math.cos(a), y: cy + radius * Math.sin(a) });
    }
    return pts;
  }

  function circlePoints(cx, cy, r, steps) {
    var pts = [];
    steps = steps || 32;
    for (var i = 0; i < steps; i++) {
      var a = (i / steps) * Math.PI * 2;
      pts.push({ x: cx + r * Math.cos(a), y: cy + r * Math.sin(a) });
    }
    return pts;
  }

  function arcPoints(cx, cy, r, startDeg, endDeg, steps) {
    var pts = [];
    var a0 = startDeg * Math.PI / 180;
    var a1 = endDeg * Math.PI / 180;
    /* DXF arcs run counter-clockwise; unwrap so a 350->10 arc sweeps 20 deg. */
    while (a1 < a0) a1 += Math.PI * 2;
    var sweep = a1 - a0;
    steps = steps || Math.max(6, Math.min(64, Math.round(Math.abs(sweep) / (Math.PI / 36)) + 1));
    for (var i = 0; i <= steps; i++) {
      var a = a0 + sweep * (i / steps);
      pts.push({ x: cx + r * Math.cos(a), y: cy + r * Math.sin(a) });
    }
    return pts;
  }

  /* Convert DXF entities to app-space (feet) polylines, circles and text.
     Returns { lines, polygons, labels, stats }. */
  function entitiesToSketch(entities, scale, offset) {
    var lines = [], polygons = [], labels = [], skipped = {};
    var dx = offset ? offset.x : 0, dy = offset ? offset.y : 0;
    function T(x, y) { return { x: x * scale + dx, y: y * scale + dy }; }

    entities.forEach(function (e) {
      var type = e.type;

      if (type === 'LINE') {
        var a = T(e.x, e.y), b = T(e.x2, e.y2);
        if (Math.hypot(b.x - a.x, b.y - a.y) > 1e-9) {
          lines.push({ start: a, end: b, layer: e.layer });
        } else { skipped.degenerate = (skipped.degenerate || 0) + 1; }
        return;
      }

      if (type === 'LWPOLYLINE' || type === 'POLYLINE') {
        var verts = e.vertices || [];
        if (verts.length < 2) { skipped.short = (skipped.short || 0) + 1; return; }
        var pts = [];
        for (var i = 0; i < verts.length; i++) {
          var v0 = verts[i];
          pts.push(T(v0.x, v0.y));
          var v1 = verts[i + 1];
          /* Bulge on vertex i describes the arc to vertex i+1. */
          if (v1 && v0.bulge) {
            bulgeToArcPoints({ x: v0.x, y: v0.y }, { x: v1.x, y: v1.y }, v0.bulge)
              .forEach(function (p) { pts.push(T(p.x, p.y)); });
          } else if (!v1 && v0.bulge && (e.flag & 1)) {
            /* Closed polyline: last vertex bulges back to the first. */
            var first = verts[0];
            bulgeToArcPoints({ x: v0.x, y: v0.y }, { x: first.x, y: first.y }, v0.bulge)
              .forEach(function (p) { pts.push(T(p.x, p.y)); });
          }
        }
        if ((e.flag & 1) && pts.length > 2) {
          polygons.push({ points: pts, layer: e.layer, closed: true });
        } else {
          for (var k = 0; k + 1 < pts.length; k++) {
            lines.push({ start: pts[k], end: pts[k + 1], layer: e.layer });
          }
        }
        return;
      }

      if (type === 'CIRCLE') {
        var cp = circlePoints(e.x, e.y, e.radius);
        if (cp.length) polygons.push({ points: cp.map(function (p) { return T(p.x, p.y); }), layer: e.layer, closed: true });
        return;
      }

      if (type === 'ARC') {
        var ap = arcPoints(e.x, e.y, e.radius, e.startAngle || 0, e.endAngle || 0);
        for (var j = 0; j + 1 < ap.length; j++) {
          lines.push({ start: T(ap[j].x, ap[j].y), end: T(ap[j + 1].x, ap[j + 1].y), layer: e.layer });
        }
        return;
      }

      if (type === 'TEXT' || type === 'MTEXT') {
        if (e.text && String(e.text).trim()) {
          labels.push({ x: T(e.x, e.y).x, y: T(e.x, e.y).y, text: String(e.text).replace(/\\[A-Za-z0-9.| ;]+;?/g, '').trim(), layer: e.layer });
        }
        return;
      }

      if (type === 'POINT') {
        labels.push({ x: T(e.x, e.y).x, y: T(e.x, e.y).y, text: '', layer: e.layer });
        return;
      }

      skipped[type ? type.toLowerCase() : 'unknown'] = (skipped[type ? type.toLowerCase() : 'unknown'] || 0) + 1;
    });

    return { lines: lines, polygons: polygons, labels: labels, skipped: skipped };
  }

  /* ---------- insert into the live app ---------- */

  function insertIntoApp(app, converted, options) {
    options = options || {};
    var state = app.state;
    if (!state || !Array.isArray(state.lines)) throw new Error('App is not ready.');

    var genId = function () {
      if (typeof app.genId === 'function') return app.genId();
      return 'cad-' + Math.random().toString(36).slice(2, 10);
    };

    app.pushUndo ? app.pushUndo('Import CAD') : null;

    var addedLines = 0, addedPolys = 0, addedLabels = 0;

    (converted.lines || []).forEach(function (ln) {
      state.lines.push({
        id: genId(),
        start: { x: ln.start.x, y: ln.start.y },
        end: { x: ln.end.x, y: ln.end.y },
        layer: options.layerName || ln.layer || null,
        exterior: options.asExterior !== false,
        interior: false
      });
      addedLines++;
    });

    (converted.polygons || []).forEach(function (poly) {
      if (!poly.points || poly.points.length < 3) return;
      state.polygons.push({
        id: genId(),
        points: poly.points.map(function (p) { return { x: p.x, y: p.y }; }),
        code: '',
        name: '',
        closed: true,
        layer: options.layerName || poly.layer || null
      });
      addedPolys++;
    });

    (converted.labels || []).forEach(function (lb) {
      if (!lb.text) return;
      if (!Array.isArray(state.labels)) return;
      state.labels.push({
        id: genId(),
        x: lb.x,
        y: lb.y,
        text: lb.text,
        fontSize: 10,
        layer: options.layerName || lb.layer || null
      });
      addedLabels++;
    });

    if (app.fitToContent) app.fitToContent();
    else if (app.zoomToFit) app.zoomToFit();
    if (app.render) app.render();
    else if (app.requestRender) app.requestRender();

    return { lines: addedLines, polygons: addedPolys, labels: addedLabels };
  }

  /* ---------- public API ---------- */

  var CadImport = {
    /* Parse DXF text into a normalized, scaled sketch payload. */
    parseDxf: function (text, options) {
      options = options || {};
      var pairs = parseDxfPairs(text);
      var ranges = sectionRanges(pairs);
      var insunits = headerInsunits(pairs, ranges);
      var entities = readEntities(pairs, ranges);
      if (!entities.length) throw new Error('No supported entities found in DXF.');
      var extent = extentOf(entities);
      var unit = options.scale != null
        ? { scale: options.scale, units: 'custom', source: 'override' }
        : pickScale(insunits, extent);
      var converted = entitiesToSketch(entities, unit.scale, options.offset || { x: 0, y: 0 });
      return {
        extent: extent,
        units: unit.units,
        unitSource: unit.source,
        scale: unit.scale,
        entityCount: entities.length,
        layers: Array.from(new Set(entities.map(function (e) { return e.layer || UNNAMED_LAYER; }))),
        lines: converted.lines,
        polygons: converted.polygons,
        labels: converted.labels,
        skipped: converted.skipped
      };
    },

    /* Import a DXF File/Blob into the running app. */
    importDxfFile: function (app, file, options) {
      return file.text().then(function (text) {
        var parsed = CadImport.parseDxf(text, options);
        var inserted = insertIntoApp(app, parsed, options);
        return Object.assign({}, parsed, { inserted: inserted });
      });
    },

    /* DWG cannot be parsed in-browser without a large WASM dependency, so we
       ask the user to convert once, out of band. See cadImportDwgHint(). */
    cadImportDwgHint: function () {
      return 'DWG is a binary format this app cannot read directly. Convert it ' +
        'to DXF first (free options: the ODA File Converter on Windows, or ' +
        'LibreCAD / QCAD on any platform), then use Import CAD on the .dxf.';
    },

    _internals: { parseDxfPairs: parseDxfPairs, sectionRanges: sectionRanges, pickScale: pickScale, entitiesToSketch: entitiesToSketch }
  };

  root.CadImport = CadImport;
})(typeof window !== 'undefined' ? window : this);

</script>
<script>
/* ====== CAD IMPORT UI LAYER ======
   Adds "Import CAD..." (.dxf) to the file menu of both the desktop and mobile
   apps, next to the existing Import/Export entries.

   It is injected AFTER the bundle's main script (see scripts/sync-bundle.mjs,
   which appends layers before the last </body>) and depends only on:
     - window.CadImport   (modules/cad-import.js)
     - window.app         (the SketchApp instance the bundle creates)
   Neither the bundle nor its generated artefacts are modified, so an upstream
   rebuild cannot silently revert this feature.

   DWG: shown in the picker but rejected with a conversion hint, because DWG is
   a closed binary format with no viable dependency-free browser parser.
*/

(function () {
  'use strict';

  var ACCEPT = '.dxf,.dwg,application/dxf';

  function ready() {
    return !!(window.CadImport && window.app && window.app.state && Array.isArray(window.app.state.lines));
  }

  function notify(message, isError) {
    try {
      if (window.app && window.app.ui && typeof window.app.ui.alertDialog === 'function') {
        window.app.ui.alertDialog(message);
        return;
      }
    } catch (_) { /* fall through */ }
    if (isError) console.error('[cad-import]', message);
    else console.log('[cad-import]', message);
  }

  function describe(result) {
    var parts = [];
    if (result.inserted.lines) parts.push(result.inserted.lines + ' line' + (result.inserted.lines === 1 ? '' : 's'));
    if (result.inserted.polygons) parts.push(result.inserted.polygons + ' shape' + (result.inserted.polygons === 1 ? '' : 's'));
    if (result.inserted.labels) parts.push(result.inserted.labels + ' label' + (result.inserted.labels === 1 ? '' : 's'));
    var summary = parts.length ? parts.join(', ') : 'no supported geometry';
    var unitNote = 'Units detected: ' + result.units + ' (' + result.unitSource + ').';
    var skipped = result.skipped && Object.keys(result.skipped).length
      ? ' Skipped: ' + Object.keys(result.skipped).map(function (k) { return k + ' x' + result.skipped[k]; }).join(', ') + '.'
      : '';
    return 'Imported ' + summary + ' from ' + result.entityCount + ' CAD entities.\n' + unitNote + skipped;
  }

  function runImport(file) {
    var name = (file && file.name) || 'file';
    var isDwg = /\.dwg$/i.test(name);

    if (isDwg) {
      notify(window.CadImport.cadImportDwgHint(), true);
      return;
    }

    window.CadImport.importDxfFile(window.app, file, {})
      .then(function (result) {
        notify(describe(result), false);
      })
      .catch(function (err) {
        notify('Could not import ' + name + ': ' + (err && err.message ? err.message : err), true);
      });
  }

  function install() {
    if (!ready()) return false;
    if (document.getElementById('cad-import-input')) return true;

    var input = document.createElement('input');
    input.type = 'file';
    input.id = 'cad-import-input';
    input.accept = ACCEPT;
    input.style.display = 'none';
    input.addEventListener('change', function (e) {
      var file = e.target.files && e.target.files[0];
      e.target.value = '';
      if (file) runImport(file);
    });

    /* Place the entry directly after the existing Import row so it reads as
       part of the same group: Import..., Import CAD..., Export as PNG...
       The input is NOT appended to body first -- insertBefore moves it, and
       appending then inserting leaves a duplicate element behind. */
    var anchor = document.getElementById('file-import');
    var button = document.createElement('button');
    button.className = 'file-menu-item';
    button.id = 'file-import-cad';
    button.innerHTML = '<i class="ph-fill ph-blueprint" style="color:#B794F4;"></i> <span>Import CAD...</span>';
    button.addEventListener('click', function () { input.click(); });

    if (anchor && anchor.parentNode) {
      anchor.parentNode.insertBefore(button, anchor.nextSibling);
      anchor.parentNode.insertBefore(input, button.nextSibling);
    } else {
      document.body.appendChild(button);
      document.body.appendChild(input);
    }

    /* Mobile runs the same bundle but a different shell; if it exposes a
       command bus, register there too so the entry is reachable on tablet. */
    if (window.Slate && window.Slate.registerCommand) {
      window.Slate.registerCommand('import-cad', function () { input.click(); });
    }
    return true;
  }

  /* The bundle builds window.app at the end of its main script, so poll briefly
     rather than assuming load order. */
  var attempts = 0;
  var timer = setInterval(function () {
    attempts++;
    if (install() || attempts > 100) clearInterval(timer);
  }, 100);

  window.CadImportUI = { install: install, runImport: runImport };
})();

</script>
<script>
