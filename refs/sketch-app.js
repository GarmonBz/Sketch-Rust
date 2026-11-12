class SketchApp {
  constructor() {
    this.state = {
      mode: 'select',
      penDown: false,
      cursor: { x: 0, y: 0 },
      keyboardPointer: false,
      keyboardPointerAnchor: null,
      pointerWorld: { x: 0, y: 0 },
      zoom: 1.0,
      drawingScale: 10,
      pan: { x: 0, y: 0 },
      snapEnabled: true,
      gridEnabled: true,
      areaGridSnap: false,
      areaGridSnapSize: 1.0,
      gridSize: 1.0,
      dimensionsEnabled: true,
      dimensionPlacement: 'out',
      suppressDimensionNext: false,
      drawColor: '#3182CE',
      canvasTheme: 'light',
      northAngle: 0,   // degrees clockwise from screen-up to true north (per page)
      defineFirst: false,
      pageLayout: JSON.parse(JSON.stringify(DEFAULT_PAGE_LAYOUT)),
      subjectInfo: { generalComments: '', commentTables: [[], [], []], additionalFields: [] },
      subjectInfoAutoFilled: false,
      extendedData: '',
      filters: {
        dimensions: true,
        labels: true,
        symbols: true,
        interior: true,
        exterior: true,
        areas: true
      },
      autoPost: {
        code: false,
        name: true,
        calcs: true,
        base: false,
        factor: false,
        suffix: 'sf'
      },
      sidebar: { active: 'define', open: true, pinned: true },
      textLibraryLabels: null,
      symbolFavorites: null,
      lines: [],
      polygons: [],
      labels: [],
      symbols: [],
      layers: [],
      activeLayerId: null,
      currentPolyPoints: [],
      currentLineIds: [],
      undoStack: [],
      redoStack: [],
      selectedLineIds: new Set(),
      activeLineEndpoint: null,
      selectedPolyIds: new Set(),
      selectedAreaLabelIds: new Set(),
      selectedAreaSizeIds: new Set(),
      selectedLabelIds: new Set(),
      selectedSymbolIds: new Set(),
      selectedDimensionIds: new Set(),
      clipboard: null,
      selectedAreaType: null,
      selectedAreaName: null,
      isPanning: false,
      panStart: null,
      spacePan: false,   // true while the space bar is held (temporary pan)
      spacePanned: false, // true if a pan drag happened during that hold
      fenceStart: null,
      fenceEnd: null,
      placementText: null,
      placementSymbol: null,
      dragSymbol: null,     // symbol type being dragged from the library (live preview)
      pendingShape: null,   // parametric room template awaiting placement
      lassoPoints: [],
      measurements: [],    // finished measure-tool runs (transient, never persisted)
      measureDraft: null,  // run being clicked out right now
      measureCursor: null, // resolved snap under the pointer: { point, type }
      dimensionChains: [],       // chained dimension strings (persisted per page)
      selectedChainIds: new Set(),
      dragState: null,
      zoomFenceStart: null,
      zoomFenceEnd: null,
      curveMode: 'degree',
      options: {
        inputMode: 'pen-ink', promptOnDeletion: false, inkConfirmation: true,
        unitSystem: 'imperial', decimalPrecision: 0.1, stepUnit: 'whole', dimensionOption: 8.5,
        orthoAngle: 5, autoPlacement: true, stripTrailingZeroes: false, sameColorAsLine: false,
        areaLabelSuffix: 'sf', alignmentLines: true, alignmentPoints: true, testCloseIndicators: true,
        alignmentColor: '#808080', alignmentStyle: 'dash', gridBackground: '#FFFFFF',
        majorGridColor: '#334155', minorGridColor: '#1f2937', minimumDimensionEnabled: false,
        minimumDimension: 12, autoShade: false, autoShadePercent: 50, selectionHighlight: true,
        distoDiscovery: 'automatic', distoRounding: 'none', distoDrawingMode: 'standard',
        autoSaveEnabled: true, autoSaveInterval: 5, imageSize: '2048x1536', imageDepth: 24,
        appendPageNumber: true, imageFormats: ['png'], disableVideoTutorial: false, autoHealFiles: true,
        dimensionFont: 'Arial', dimensionFontSize: 12, dimensionBold: false, dimensionItalic: false, dimensionUnderline: false,
        textFont: 'Arial', textFontSize: 12, textBold: false, textItalic: false, textUnderline: false, textColor: '#e2e8f0',
        autoPostFont: 'Arial', autoPostFontSize: 16, autoPostBold: false, autoPostItalic: false, autoPostUnderline: false,
        circleAreaCalcs: false, textBackground: '#111111', textBackgroundOpacity: 0,
        exteriorColor: '#2B6CB0', interiorColor: '#63B3ED', exteriorWidth: 4, interiorWidth: 2,
        wallSolids: false, // master switch for geometric wall solids; thicknesses below are inches and only apply when enabled
        exteriorThickness: 6, interiorThickness: 4, // geometric wall thickness in inches; 0 = legacy single-line render
        witnessBaseColor: '#ef4444', witnessInteriorColor: '#2563eb', witnessCurveColor: '#22c55e', labelOpacity: 50,
        displayOnClose: false, defineFirst: false, autoSubtractOverlapped: true, quickShapeDefine: false, redefine: false,
        panZoomDelete: true, saveImageWithSketch: false, imageQuality: 85, suppressMinimizeConfirmation: false,
        pointerSize: 128
      }
    };

    // Entity ids are sketch-global and monotonic. They are deliberately not
    // part of page data or undo snapshots.
    this.nextId = 1;
    this.areaCodes = window.AreaCodes
      ? window.AreaCodes.cloneAreaCodes(window.AREA_CODES)
      : JSON.parse(JSON.stringify(window.AREA_CODES || []));
    this.areaTotals = { totalsByCode: {}, totalsByGroup: {}, totalGLA: 0, totalAllAreas: 0 };
    this.breakdown = new window.AreaBreakdown({ codeTable: this.areaCodes });
    this.debugAreaCalculations = false;

    // Multi-page support
    this.pages = [this.getPageData()];
    this.currentPage = 0;
    // View-only underlay: another page ghosted beneath the current one.
    // Never persisted, never part of undo snapshots.
    this.underlayPageIndex = null;
    this.fileName = 'Untitled';
    this.ensureLayers();

    // Background image (photometrics)
    this.bgImage = {
      img: null,           // HTMLImageElement
      visible: false,
      x: 0, y: 0,         // world position (center)
      scale: 1.0,          // image scale (px per world unit)
      rotation: 0,         // degrees
      opacity: 0.5,        // 0-1
      flipH: false,
      flipV: false,
      scaleMode: false,    // true when in Set Scale mode
      scaleP1: null,
      scaleP2: null,
      editMode: false,
      orientMode: false,
      orientP1: null,
      orientP2: null
    };

    this.ui = new window.UIManager(this);
    this.input = new window.InputController(this);
    this.canvas = new window.CanvasEngine(this, 'sketchCanvas');
    
    this.state.pan = {
      x: this.canvas.viewWidth / 2,
      y: this.canvas.viewHeight / 2
    };

    this.canvas.setBackgroundTheme(this.state.canvasTheme);
    if (Object.keys(this.ui.storedDefaultOptions || {}).length) this.applyOptions(this.ui.storedDefaultOptions);
    this.ui.updateUndoRedoControls();
    this.ui.updatePageDisplay();
    this.ui.updateLayersPanel?.();
    this.updateTitle();
  }

  genId() {
    return this.nextId++;
  }

  getDefaultPageLayout() {
    return JSON.parse(JSON.stringify(DEFAULT_PAGE_LAYOUT));
  }

  normalizePageLayout(layout) {
    const defaults = this.getDefaultPageLayout();
    if (!layout || typeof layout !== 'object') return defaults;
    const paperSize = ['legal', 'letter', 'a4'].includes(layout.paperSize) ? layout.paperSize : defaults.paperSize;
    const prefix = String(layout.currentForm || '').split('-').slice(0, -1).join('-') || 'standard';
    const allowedForm = ['standard', 'full-page', 'sketch-only', 'with-calculations'].includes(prefix) ? prefix : 'standard';
    return {
      ...defaults,
      ...layout,
      paperSize,
      currentForm: `${allowedForm}-${paperSize}`,
      captions: { ...defaults.captions, ...(layout.captions || {}) }
    };
  }

  setPageLayout(layout) {
    this.state.pageLayout = this.normalizePageLayout(layout);
    return this.state.pageLayout;
  }

  // --- Mode Management ---

  setMode(mode) {
    const previousMode = this.state.mode;
    // A drawing chain belongs to one wall type. Finish it before changing tools
    // (including switching directly between exterior and interior walls).
    if (this.state.penDown && mode !== this.state.mode) {
      this.penUp();
    }

    if ((this.state.mode === 'fence' || this.state.mode === 'zoom-fence') && mode !== this.state.mode) {
      this.state.fenceStart = null;
      this.state.fenceEnd = null;
      this.state.zoomFenceStart = null;
      this.state.zoomFenceEnd = null;
    }

    // Clear placement when leaving placement modes
    if (mode !== 'place-text') this.state.placementText = null;
    if (mode !== 'place-symbol') this.state.placementSymbol = null;
    if (mode !== 'place-shape') this.state.pendingShape = null;

    this.state.mode = mode;

    if (mode === 'select') {
      this.ui.updateHUD('', '', false);
    } else if (mode === 'curve') {
      this.ui.updateHUD('', '', false);
      this.ui.notify('Curve Wall: drag a wall outward or inward. Calculations update live.');
    } else if (mode === 'measure') {
      this.ui.updateHUD('', '', false);
      this.ui.notify('Measure: click points to chain legs. Enter finishes, Shift+Enter closes for area, Ctrl+C copies.');
    } else if (mode === 'chain-dim') {
      this.ui.updateHUD('', '', false);
      this.ui.notify('Chain Dimensions: click a wall to dimension its full run. Esc exits.');
    } else if (mode === 'place-shape') {
      this.ui.updateHUD('', '', false);
      this.ui.notify('Place Shape: click to anchor the room template. Esc exits.');
    }

    // Temporary measurements belong to the measure tool session.
    if (previousMode === 'measure' && mode !== 'measure') {
      this.state.measurements = [];
      this.state.measureDraft = null;
      this.state.measureCursor = null;
    }
    this.canvas.setCursor();
    
    document.querySelectorAll('.tool-btn').forEach(b => b.classList.remove('active'));
    const btn = document.getElementById('tool-' + mode);
    if (btn) btn.classList.add('active');

    // Status bar magnification (Zoom Fence) button reflects its mode
    document.getElementById('btn-zoom-fence')?.classList.toggle('active', mode === 'zoom-fence');

    this.render();
  }

  // --- Pen Control ---

  togglePen() {
    if (this.state.penDown) this.penUp();
    else this.penDown();
  }

  penDown(worldPt) {
    if (this.state.penDown) return;
    const start = worldPt ? this.snapPoint(worldPt) : { ...this.state.cursor };
    this.state.penDown = true;
    this.state.keyboardPointer = false;
    this.state.keyboardPointerAnchor = null;
    this.state.cursor = { ...start };
    this.state.currentPolyPoints = [{ ...start }];
    this.state.currentLineIds = [];
    if (this.ui) this.ui.updateHUD(this.input ? this.input.buffer : '', null, true);
    this.render();
  }

  penUp() {
    if (!this.state.penDown) return;
    this.state.penDown = false;
    this.state.keyboardPointer = false;
    this.state.keyboardPointerAnchor = null;
    this.state.currentPolyPoints = [];
    this.state.currentLineIds = [];
    if (this.ui) this.ui.updateHUD('', '', false);
    this.canvas.setCursor();
    this.render();
  }

  // --- Snap ---
  getScreenToleranceWorld(pixels = 10) {
    const scale = window.Geometry.PIXELS_PER_FOOT * this.state.zoom;
    return pixels / Math.max(scale, 0.0001);
  }

  pointKey(point) {
    return `${Number(point.x).toFixed(5)},${Number(point.y).toFixed(5)}`;
  }

  // Return the live boundary walls only when every polygon edge still maps to
  // the same geometric segment. A stale UND polygon can survive old/imported
  // precision mismatches even after one of its walls is gone; treating that
  // record as closed blacklists the two visibly free ends and prevents the
  // replacement wall from ever joining them.
  polygonBoundaryLines(poly, linesById = null) {
    if (!poly || !Array.isArray(poly.points) || poly.points.length < 3 ||
        !Array.isArray(poly.edges) || poly.edges.length !== poly.points.length ||
        window.Geometry.calculateArea(poly.points, poly.edges) < this.minimumClosedArea()) return null;
    const byId = linesById || new Map(this.state.lines.map(line => [line.id, line]));
    const used = new Set();
    const boundary = [];
    for (let index = 0; index < poly.points.length; index++) {
      const edge = poly.edges[index];
      const line = edge && byId.get(edge.lineId);
      const start = poly.points[index];
      const end = poly.points[(index + 1) % poly.points.length];
      const matches = line && line.type === 'exterior' &&
        ((this.pointsEqual(line.start, start) && this.pointsEqual(line.end, end)) ||
         (this.pointsEqual(line.start, end) && this.pointsEqual(line.end, start)));
      if (!matches || used.has(line.id)) return null;
      used.add(line.id);
      boundary.push(line);
    }
    return boundary;
  }

  isPolygonBoundaryIntact(poly, linesById = null) {
    return Boolean(this.polygonBoundaryLines(poly, linesById));
  }

  // Vertices of genuinely closed areas must not attract new walls: a line
  // started next to a complete area stays independent. Incomplete/stale area
  // records deliberately do not veto their live free endpoints, so a missing
  // boundary can be drawn back into the wall graph.
  closedAreaVertexKeys() {
    const keys = new Set();
    const linesById = new Map(this.state.lines.map(line => [line.id, line]));
    this.state.polygons.forEach(poly => {
      const boundary = this.polygonBoundaryLines(poly, linesById);
      if (!boundary) return;
      // Protect the canonical live endpoints rather than the nearly-equal
      // polygon copies used to validate them. Otherwise a 0.00004-foot saved
      // residue can pass validation but miss pointKey's exact lookup.
      boundary.forEach(line => {
        keys.add(this.pointKey(line.start));
        keys.add(this.pointKey(line.end));
      });
    });
    return keys;
  }

  snapPoint(worldPt) {
    const pt = { ...worldPt };
    if (!this.state.snapEnabled) return pt;
    return this.structuralSnap(pt) || window.Geometry.snapToGrid(pt, this.state.gridSize);
  }

  // Endpoint and wall-face snaps. These outrank grid and polar snapping so a
  // wall that visibly meets another one is also joined to it.
  structuralSnap(worldPt) {
    const pt = { ...worldPt };

    // Keep snap feel constant at every zoom level by measuring its capture
    // radius in screen pixels rather than feet.
    const tolerance = this.getScreenToleranceWorld(10);
    const vertexKeys = this.closedAreaVertexKeys();
    const nearPt = window.Geometry.findNearestEndpoint(pt, this.state.lines, tolerance,
      candidate => vertexKeys.has(this.pointKey(candidate)));
    if (nearPt) return nearPt;

    // Wall-face snap: with geometric wall thickness enabled, the interior and
    // exterior faces of every wall are snap targets.
    const solids = this.getWallSolids();
    if (solids) {
      let faceHit = null;
      for (const face of solids.faces) {
        const dx = face.end.x - face.start.x;
        const dy = face.end.y - face.start.y;
        const lengthSq = dx * dx + dy * dy;
        if (lengthSq < 1e-9) continue;
        const t = Math.max(0, Math.min(1, ((pt.x - face.start.x) * dx + (pt.y - face.start.y) * dy) / lengthSq));
        const projection = { x: face.start.x + dx * t, y: face.start.y + dy * t };
        const distance = Math.hypot(pt.x - projection.x, pt.y - projection.y);
        if (distance <= tolerance && (!faceHit || distance < faceHit.distance)) {
          faceHit = { projection, distance };
        }
      }
      if (faceHit) return faceHit.projection;
    }
    return null;
  }

  getDrawingTarget(worldPt) {
    let target = this.snapPoint(worldPt);
    // While a wall chain is live, a free-angle leg snaps in polar steps from
    // its anchor (whole degrees in 5-degree steps, whole grid-length runs)
    // rather than to the nearest grid node. An anchor that is
    // already off the grid (after an angled wall) therefore keeps producing
    // clean lengths instead of fractional ones.
    const points = this.state.currentPolyPoints || [];
    if (this.state.snapEnabled && this.state.penDown && points.length &&
        (this.state.mode === 'draw-exterior' || this.state.mode === 'draw-interior') &&
        !this.structuralSnap(worldPt)) {
      const polar = window.Geometry.polarSnap(points[points.length - 1], worldPt, this.state.gridSize, 5);
      if (polar) target = polar;
    }
    // Joining a wall endpoint is structural, not cosmetic. Always capture an
    // existing endpoint within the on-screen join radius, even when ordinary
    // grid/endpoint snapping is disabled, so a visibly joined wall is also
    // topologically joined and can close an appraisal area. Closed areas are
    // excluded so a new chain drawn next to one does not attach to it.
    const joinTolerance = this.getScreenToleranceWorld(16);
    const vertexKeys = this.closedAreaVertexKeys();
    const joinedEndpoint = window.Geometry.findNearestEndpoint(worldPt, this.state.lines, joinTolerance,
      candidate => vertexKeys.has(this.pointKey(candidate)));
    if (joinedEndpoint) target = { ...joinedEndpoint };
    if (this.state.penDown && this.state.currentPolyPoints.length >= 3) {
      const start = this.state.currentPolyPoints[0];
      if (window.Geometry.distance(worldPt, start) <= joinTolerance ||
          window.Geometry.distance(target, start) <= joinTolerance) {
        target = { ...start };
      }
    }
    return target;
  }

  toggleGrid() {
    this.state.gridEnabled = !this.state.gridEnabled;
    const btn = document.getElementById('btn-grid');
    if (btn) btn.classList.toggle('active', this.state.gridEnabled);
    this.render();
  }

  toggleSnap() {
    this.state.snapEnabled = !this.state.snapEnabled;
    const btn = document.getElementById('btn-snap');
    if (btn) btn.classList.toggle('active', this.state.snapEnabled);
  }

  toggleAreaGridSnap() {
    this.state.areaGridSnap = !this.state.areaGridSnap;
    const btn = document.getElementById('btn-area-snap');
    if (btn) btn.classList.toggle('active', this.state.areaGridSnap);
  }

  setAreaGridSnapSize(value) {
    const size = parseFloat(value);
    if (!Number.isFinite(size) || size <= 0) return;
    this.state.areaGridSnapSize = size;
    // The pitch menu sets the snap grid for drawing and endpoint dragging too,
    // otherwise those paths keep snapping at the default one-foot grid.
    this.state.gridSize = size;
  }

  cycleDimensions() {
    if (!this.state.dimensionsEnabled) {
      this.state.dimensionsEnabled = true;
      this.state.dimensionPlacement = 'out';
    } else if (this.state.dimensionPlacement === 'out') {
      this.state.dimensionPlacement = 'in';
    } else {
      this.state.dimensionsEnabled = false;
    }

    const dims = document.getElementById('autopost-dims');
    const placement = document.getElementById('dimension-placement');
    if (dims) dims.checked = this.state.dimensionsEnabled;
    if (placement) placement.value = this.state.dimensionPlacement;
    this.ui.notify(this.state.dimensionsEnabled
      ? `Dimensions: ${this.state.dimensionPlacement === 'out' ? 'Outside' : 'Inside'}`
      : 'Dimensions: Hidden');
    this.render();
  }

  // --- Selection ---

  // Whether a wall currently shows its measurement label.
  isDimensionVisible(line) {
    if (line.dimensionHidden) return false;
    if (this.state.filters.dimensions === false) return false;
    const isActive = this.state.penDown && this.state.currentLineIds.includes(line.id);
    if (!this.state.dimensionsEnabled && !isActive) return false;
    const dist = window.Geometry.distance(line.start, line.end);
    if (dist < 0.1) return false;
    const options = this.state.options || {};
    if (options.minimumDimensionEnabled && dist < (Number(options.minimumDimension) || 0)) return false;
    return true;
  }

  // Space while a wall (or its label) is selected toggles that measurement
  // label on or off - this is also how a deleted label is restored.
  toggleSelectedDimensions() {
    const ids = new Set([...this.state.selectedLineIds, ...this.state.selectedDimensionIds]);
    if (!ids.size) return;
    this.pushUndo('Toggle Dimensions');
    this.state.lines.forEach(line => {
      if (!ids.has(line.id)) return;
      line.dimensionHidden = !line.dimensionHidden;
      if (line.dimensionHidden) { delete line.dimensionPosition; delete line.dimensionAnchor; }
      this.state.selectedDimensionIds.delete(line.id);
    });
    this.render();
  }

  clearSelection(shouldRender = true) {
    this.state.selectedLineIds.clear();
    this.state.selectedPolyIds.clear();
    this.state.selectedAreaLabelIds.clear();
    this.state.selectedAreaSizeIds.clear();
    this.state.selectedLabelIds.clear();
    this.state.selectedSymbolIds.clear();
    this.state.selectedDimensionIds.clear();
    this.state.selectedChainIds.clear();
    this.state.activeLineEndpoint = null;
    this.ui.updateAreaActionStates?.();
    if (shouldRender) this.render();
  }

  // The options a saved sketch carries, merged over the current ones. The
  // wall-solid switch is the sketch's own: one that never set it draws wall
  // lines, rather than inheriting whatever the previous assignment left on.
  mergeSavedOptions(saved) {
    const own = saved || {};
    return { ...this.state.options, ...own, wallSolids: Boolean(own.wallSolids) };
  }

  applyOptions(next) {
    this.state.options = { ...this.state.options, ...(next || {}) };
    const options = this.state.options;
    options.interiorWidth = Math.max(1, Number(options.interiorWidth) || 2);
    options.exteriorWidth = Math.max(2, Number(options.exteriorWidth) || options.interiorWidth * 2);
    // Wall solid thicknesses are stored in inches; zero disables solids.
    options.wallSolids = Boolean(options.wallSolids);
    options.exteriorThickness = Math.max(0, Number(options.exteriorThickness ?? 6) || 0);
    options.interiorThickness = Math.max(0, Number(options.interiorThickness ?? 4) || 0);
    this.wallSolidsCache = null;
    this.state.defineFirst = Boolean(options.defineFirst);
    this.state.gridEnabled = options.gridEnabled !== false;
    this.state.autoPost.suffix = options.areaLabelSuffix || this.state.autoPost.suffix;
    this.canvas?.applyOptions?.();
    this.render();
  }

  formatLength(value) {
    const options = this.state.options || {};
    const precision = Math.max(0.0001, Number(options.decimalPrecision) || 0.1);
    const decimals = Math.max(0, Math.ceil(-Math.log10(precision)));
    const converted = options.unitSystem === 'metric' ? Number(value) * 0.3048 : Number(value);
    let text = converted.toFixed(decimals);
    if (options.stripTrailingZeroes) text = text.replace(/\.0+$|(?<=\.[0-9]*?)0+$/g, '').replace(/\.$/, '');
    return `${text}${options.unitSystem === 'metric' ? ' m' : "'"}`;
  }

  // Feet and whole inches, the form a tape reads in.
  formatFeetInches(value) {
    const number = Number(value) || 0;
    const absolute = Math.abs(number);
    let feet = Math.floor(absolute + 1e-9);
    let inches = Math.round((absolute - feet) * 12);
    if (inches === 12) { feet += 1; inches = 0; }
    return `${number < 0 ? '-' : ''}${feet}'${inches ? ` ${inches}"` : ''}`;
  }

  // Decimal length in the sketch's units, with the feet-and-inches equivalent
  // alongside it for imperial sketches.
  formatLengthDual(value) {
    const text = this.formatLength(value);
    if ((this.state.options || {}).unitSystem === 'metric') return text;
    return `${text} (${this.formatFeetInches(value)})`;
  }

  formatArea(squareFeet) {
    const options = this.state.options || {};
    const metric = options.unitSystem === 'metric';
    const value = metric ? Number(squareFeet) * 0.09290304 : Number(squareFeet);
    const [whole, fraction] = (Number.isFinite(value) ? value : 0).toFixed(metric ? 2 : 1).split('.');
    const grouped = whole.replace(/\B(?=(\d{3})+$)/g, ',');
    return `${grouped}.${fraction} ${metric ? 'm\u00b2' : (options.areaLabelSuffix || 'sf')}`;
  }

  getDefaultLineStyle(type) {
    const options = this.state.options || {};
    const interior = type === 'interior';
    return {
      color: interior ? options.interiorColor : options.exteriorColor,
      lineThickness: interior ? options.interiorWidth : options.exteriorWidth
    };
  }

  // --- Wall solids (full geometric wall thickness) ---

  // Wall thickness of a line in FEET. Per-line `thickness` overrides (stored
  // in inches) win over the exterior/interior option defaults.
  wallThicknessOf(line) {
    if (Number(line?.thickness) > 0) return Number(line.thickness) / 12;
    const options = this.state.options || {};
    const inches = line?.type === 'interior'
      ? Number(options.interiorThickness) || 0
      : Number(options.exteriorThickness) || 0;
    return inches / 12;
  }

  // Face-mode offset ring for a polygon: when measureTo is 'interior' or
  // 'exterior', offset each edge by half its wall's thickness toward that
  // face before the area decomposition. Returns null when the polygon keeps
  // its centerline ring (default mode, solids disabled, or degenerate).
  // Bowed edges are treated as chords in the offset (v1 limitation).
  measureToOffsetRing(poly, lines) {
    const mode = poly.measureTo;
    if (mode !== 'interior' && mode !== 'exterior') return null;
    if (!this.wallSolidsEnabled() || !window.WallSolids) return null;
    const byId = new Map((lines || this.state.lines).map(line => [line.id, line]));
    const sign = mode === 'interior' ? -1 : 1;
    const offsets = (poly.edges || []).map(edge => {
      const line = edge?.lineId != null ? byId.get(edge.lineId) : null;
      const thickness = line ? this.wallThicknessOf(line) : this.wallThicknessOf({ type: 'exterior' });
      return sign * thickness / 2;
    });
    if (offsets.length !== poly.points.length) return null;
    return window.WallSolids.offsetRing(poly.points, offsets);
  }

  wallSolidsEnabled() {
    const options = this.state.options || {};
    return Boolean(options.wallSolids) &&
      (Number(options.exteriorThickness) > 0 || Number(options.interiorThickness) > 0) &&
      Boolean(window.WallSolids);
  }

  // Cached wall-solid geometry for the current page. The cache key mixes
  // geometry and thickness signatures so drags (which mutate lines in place)
  // invalidate it automatically.
  getWallSolids() {
    if (!this.wallSolidsEnabled()) return null;
    const s = this.state;
    const options = s.options || {};
    const signature = [s.lines.length, s.symbols.length, options.exteriorThickness, options.interiorThickness];
    s.lines.forEach(line => signature.push(
      line.start.x, line.start.y, line.end.x, line.end.y, line.bowAngle || 0, line.thickness || 0, line.type));
    s.symbols.forEach(symbol => {
      if (symbol.wallId) signature.push(symbol.x, symbol.y, symbol.wallGapWidth || 0,
        symbol.flipH ? 1 : 0, symbol.wallSideFlip ? 1 : 0);
    });
    const key = signature.join('|');
    if (this.wallSolidsCache && this.wallSolidsCache.key === key) return this.wallSolidsCache.result;
    const result = window.WallSolids.computeWallSolids(s.lines, line => this.wallThicknessOf(line), s.symbols);
    this.wallSolidsCache = { key, result };
    return result;
  }

  getAreaLabelPosition(poly) {
    if (Number.isFinite(Number(poly?.labelPosition?.x)) && Number.isFinite(Number(poly?.labelPosition?.y))) {
      return { x: Number(poly.labelPosition.x), y: Number(poly.labelPosition.y) };
    }
    const points = poly?.points || [];
    if (!points.length) return { x: 0, y: 0 };
    return window.Geometry.areaLabelAnchor(window.Geometry.flattenRing(poly));
  }

  getAreaSizePosition(poly) {
    if (Number.isFinite(Number(poly?.areaSizePosition?.x)) && Number.isFinite(Number(poly?.areaSizePosition?.y))) {
      return { x: Number(poly.areaSizePosition.x), y: Number(poly.areaSizePosition.y) };
    }
    const labelPosition = this.getAreaLabelPosition(poly);
    const autoPost = { ...(this.state.autoPost || {}), ...(poly?.autoPost || {}) };
    if (!autoPost.code && !autoPost.name) return labelPosition;
    const labelLineCount = Number(Boolean(autoPost.code)) + Number(Boolean(autoPost.name));
    const sizeLineCount = Number(Boolean(autoPost.calcs)) + Number(Boolean(autoPost.base)) + Number(Boolean(autoPost.factor));
    const labelScale = Math.max(0.5, Math.min(4, Number(poly?.labelScale) || 1));
    const sizeScale = Math.max(0.5, Math.min(4, Number(poly?.areaSizeScale ?? labelScale) || 1));
    const separationPixels = (labelLineCount * labelScale + Math.max(1, sizeLineCount) * sizeScale) * 7.5;
    // World Y increases upward, so a smaller Y places the size below the label.
    return { x: labelPosition.x, y: labelPosition.y - separationPixels / (window.Geometry.PIXELS_PER_FOOT || 20) };
  }

  distanceToWall(worldPt, line) {
    const bowAngle = Number(line && line.bowAngle) || 0;
    if (Math.abs(bowAngle) < 0.5) {
      return window.Geometry.distanceToSegment(worldPt, line.start, line.end);
    }
    const path = window.Geometry.flattenRing({
      points: [line.start, line.end],
      edges: [{ bowAngle }, { bowAngle: 0 }]
    }, undefined, Math.max(0.005, this.getScreenToleranceWorld(1) / 2));
    let distance = Infinity;
    for (let index = 0; index < path.length - 1; index++) {
      distance = Math.min(distance,
        window.Geometry.distanceToSegment(worldPt, path[index], path[index + 1]));
    }
    return distance;
  }

  pointInSymbol(worldPt, symbol, paddingPixels = 6) {
    const definition = window.SKETCH_SYMBOL_LIBRARY?.symbols?.[symbol.symbolType];
    if (!definition?.bounds) {
      return window.Geometry.distance(worldPt, symbol) <= this.getScreenToleranceWorld(18);
    }
    const dx = worldPt.x - symbol.x;
    const dy = worldPt.y - symbol.y;
    const radians = (Number(symbol.rotation) || 0) * Math.PI / 180;
    const screenX = dx;
    const screenY = -dy;
    const localScreenX = screenX * Math.cos(-radians) - screenY * Math.sin(-radians);
    const localScreenY = screenX * Math.sin(-radians) + screenY * Math.cos(-radians);
    const scale = Math.max(0.001, Number(symbol.scale) || 1) / 12;
    const localX = localScreenX / scale;
    const localY = -localScreenY / scale;
    const padding = this.getScreenToleranceWorld(paddingPixels) / scale;
    const bounds = definition.bounds;
    return localX >= bounds.minX - padding && localX <= bounds.maxX + padding &&
      localY >= bounds.minY - padding && localY <= bounds.maxY + padding;
  }

  hitTestAt(worldPt) {
    for (const polygon of this.paintOrderDescending(this.state.polygons)) {
      if (!polygon.type || polygon.type === 'UND') continue;
      if (!this.isEntityVisible(polygon) || this.isEntityLocked(polygon)) continue;
      const labelPosition = this.getAreaLabelPosition(polygon);
      const sizePosition = this.getAreaSizePosition(polygon);
      const preciseLabelHit = this.canvas?.pointInAreaLabel?.(worldPt, polygon);
      const preciseSizeHit = this.canvas?.pointInAreaSize?.(worldPt, polygon);
      if (preciseLabelHit || preciseSizeHit) {
        const labelDistance = preciseLabelHit ? window.Geometry.distance(worldPt, labelPosition) : Infinity;
        const sizeDistance = preciseSizeHit ? window.Geometry.distance(worldPt, sizePosition) : Infinity;
        return { type: sizeDistance < labelDistance ? 'areaSize' : 'areaLabel', id: polygon.id };
      }
      const labelFallback = window.Geometry.distance(worldPt, labelPosition) <= this.getScreenToleranceWorld(22);
      const sizeFallback = window.Geometry.distance(worldPt, sizePosition) <= this.getScreenToleranceWorld(22);
      if (labelFallback || sizeFallback) {
        const labelDistance = labelFallback ? window.Geometry.distance(worldPt, labelPosition) : Infinity;
        const sizeDistance = sizeFallback ? window.Geometry.distance(worldPt, sizePosition) : Infinity;
        return { type: sizeDistance < labelDistance ? 'areaSize' : 'areaLabel', id: polygon.id };
      }
    }
    const pointThreshold = this.getScreenToleranceWorld(18);
    let bestLabel = null;
    let bestLabelDist = pointThreshold;
    this.state.labels.forEach(label => {
      if (!this.isEntityVisible(label) || this.isEntityLocked(label)) return;
      const d = window.Geometry.distance(worldPt, label);
      if (d < bestLabelDist) { bestLabel = label; bestLabelDist = d; }
    });
    if (bestLabel) return { type: 'label', id: bestLabel.id };

    for (const symbol of this.paintOrderDescending(this.state.symbols)) {
      if (!this.isEntityVisible(symbol) || this.isEntityLocked(symbol)) continue;
      if (this.pointInSymbol(worldPt, symbol)) return { type: 'symbol', id: symbol.id };
    }

    // A click right on a wall line selects the wall even where the wall's own
    // dimension text overlaps it (at fit-to-screen zoom the label sits a few
    // pixels off the line and used to win, so the wall could not be picked).
    const onWallThreshold = this.getScreenToleranceWorld(4);
    let onWall = null;
    let onWallDistance = onWallThreshold;
    for (const line of this.paintOrderDescending(this.state.lines)) {
      if (!this.isEntityVisible(line) || this.isEntityLocked(line)) continue;
      const distance = this.distanceToWall(worldPt, line);
      if (distance <= onWallDistance) { onWall = line; onWallDistance = distance; }
    }
    if (onWall) return { type: 'line', id: onWall.id };

    // Measurement labels are only clickable with the pointer tool, so curve
    // clicks keep reaching the wall underneath.
    if (this.state.mode === 'select') {
      const dimensionId = this.canvas?.hitDimensionAt?.(worldPt);
      if (dimensionId != null) {
        const dimLine = this.state.lines.find(line => line.id === dimensionId);
        if (dimLine && this.isEntityVisible(dimLine) && !this.isEntityLocked(dimLine)) {
          return { type: 'dimension', id: dimensionId };
        }
      }
    }

    const lineThreshold = this.getScreenToleranceWorld(9);
    for (const line of this.paintOrderDescending(this.state.lines)) {
      if (!this.isEntityVisible(line) || this.isEntityLocked(line)) continue;
      if (this.distanceToWall(worldPt, line) <= lineThreshold) return { type: 'line', id: line.id };
    }

    for (const polygon of this.paintOrderDescending(this.state.polygons)) {
      if (!this.isEntityVisible(polygon) || this.isEntityLocked(polygon)) continue;
      if (window.Geometry.pointInPolygon(worldPt, window.Geometry.flattenRing(polygon))) {
        return { type: 'polygon', id: polygon.id };
      }
    }
    return null;
  }

  isHitSelected(hit) {
    if (!hit) return false;
    const sets = {
      line: this.state.selectedLineIds,
      polygon: this.state.selectedPolyIds,
      areaLabel: this.state.selectedAreaLabelIds,
      areaSize: this.state.selectedAreaSizeIds,
      label: this.state.selectedLabelIds,
      symbol: this.state.selectedSymbolIds,
      dimension: this.state.selectedDimensionIds
    };
    return sets[hit.type]?.has(hit.id) || false;
  }

  selectLineAt(worldPt, additive) {
    const hit = this.hitTestAt(worldPt);
    if (!additive) this.clearSelection(false);
    if (!hit) { this.render(); return null; }

    const sets = {
      line: this.state.selectedLineIds,
      polygon: this.state.selectedPolyIds,
      areaLabel: this.state.selectedAreaLabelIds,
      areaSize: this.state.selectedAreaSizeIds,
      label: this.state.selectedLabelIds,
      symbol: this.state.selectedSymbolIds,
      dimension: this.state.selectedDimensionIds
    };
    const target = sets[hit.type];
    if (additive && target.has(hit.id)) target.delete(hit.id);
    else target.add(hit.id);
    if (hit.type === 'line') {
      const line = this.state.lines.find(item => item.id === hit.id);
      if (line) this.state.activeLineEndpoint = {
        lineId: line.id,
        endpoint: window.Geometry.distance(worldPt, line.start) <= window.Geometry.distance(worldPt, line.end) ? 'start' : 'end'
      };
    }
    this.ui.updateAreaActionStates?.();
    this.render();
    return hit;
  }

  selectAll() {
    const selectable = entity => !this.isEntityLocked(entity) && this.isEntityVisible(entity);
    this.state.lines.filter(selectable).forEach(l => this.state.selectedLineIds.add(l.id));
    this.state.polygons.filter(selectable).forEach(p => this.state.selectedPolyIds.add(p.id));
    this.state.polygons.filter(p => selectable(p) && p.type && p.type !== 'UND').forEach(p => this.state.selectedAreaLabelIds.add(p.id));
    this.state.polygons.filter(p => selectable(p) && p.type && p.type !== 'UND').forEach(p => this.state.selectedAreaSizeIds.add(p.id));
    this.state.labels.filter(selectable).forEach(l => this.state.selectedLabelIds.add(l.id));
    this.state.symbols.filter(selectable).forEach(s => this.state.selectedSymbolIds.add(s.id));
    this.ui.updateAreaActionStates?.();
    this.render();
  }

  selectByType(type) {
    this.clearSelection(false);
    const selectable = entity => !this.isEntityLocked(entity) && this.isEntityVisible(entity);
    if (type === 'areas') {
      this.state.polygons.filter(selectable).forEach(p => this.state.selectedPolyIds.add(p.id));
    } else if (type === 'text') {
      this.state.polygons.filter(p => selectable(p) && p.type && p.type !== 'UND').forEach(p => this.state.selectedAreaLabelIds.add(p.id));
      this.state.polygons.filter(p => selectable(p) && p.type && p.type !== 'UND').forEach(p => this.state.selectedAreaSizeIds.add(p.id));
      this.state.labels.filter(selectable).forEach(l => this.state.selectedLabelIds.add(l.id));
    } else if (type === 'symbols') {
      this.state.symbols.filter(selectable).forEach(s => this.state.selectedSymbolIds.add(s.id));
    } else if (type === 'freeform') {
      this.state.lines.filter(l => l.type === 'interior' && selectable(l)).forEach(l => this.state.selectedLineIds.add(l.id));
    } else if (type === 'dimensions') {
      // Select each visible measurement label so it can be moved or deleted.
      this.state.lines.forEach(line => {
        if (this.isDimensionVisible(line) && selectable(line)) this.state.selectedDimensionIds.add(line.id);
      });
    } else {
      this.selectAll();
      return;
    }
    this.ui.updateAreaActionStates?.();
    this.render();
  }

  getAllSketchPoints() {
    const points = [];
    this.state.lines.forEach(line => points.push(line.start, line.end));
    this.state.polygons.forEach(poly => points.push(...poly.points));
    this.state.polygons.filter(poly => poly.labelPosition).forEach(poly => points.push(poly.labelPosition));
    this.state.polygons.filter(poly => poly.areaSizePosition).forEach(poly => points.push(poly.areaSizePosition));
    this.state.labels.forEach(label => points.push(label));
    this.state.symbols.forEach(symbol => points.push(symbol));
    return points;
  }

  transformSketch(transformPoint, rotationDelta = 0) {
    this.state.lines.forEach(line => {
      line.start = transformPoint(line.start);
      line.end = transformPoint(line.end);
      if (line.dimensionPosition) line.dimensionPosition = transformPoint(line.dimensionPosition);
      if (line.dimensionAnchor) line.dimensionAnchor = transformPoint(line.dimensionAnchor);
    });
    this.state.polygons.forEach(poly => {
      poly.points = poly.points.map(transformPoint);
      const labelRotation = Number(poly.labelRotation) || 0;
      const sizeRotation = Number(poly.areaSizeRotation ?? labelRotation) || 0;
      if (poly.labelPosition) {
        poly.labelPosition = transformPoint(poly.labelPosition);
        poly.labelRotation = labelRotation - rotationDelta;
      }
      if (poly.areaSizePosition) {
        poly.areaSizePosition = transformPoint(poly.areaSizePosition);
        poly.areaSizeRotation = sizeRotation - rotationDelta;
      }
    });
    this.state.labels.forEach(label => {
      const transformed = transformPoint(label);
      label.x = transformed.x;
      label.y = transformed.y;
      label.rotation = (label.rotation || 0) + rotationDelta;
    });
    this.state.symbols.forEach(symbol => {
      const transformed = transformPoint(symbol);
      symbol.x = transformed.x;
      symbol.y = transformed.y;
      symbol.rotation = (symbol.rotation || 0) + rotationDelta;
    });
    this.recalcAutoSubtract();
  }

  rotateSketch(degrees = 90) {
    const points = this.getAllSketchPoints();
    if (points.length === 0) return;
    this.pushUndo('Rotate Sketch');
    const minX = Math.min(...points.map(p => p.x));
    const maxX = Math.max(...points.map(p => p.x));
    const minY = Math.min(...points.map(p => p.y));
    const maxY = Math.max(...points.map(p => p.y));
    const center = { x: (minX + maxX) / 2, y: (minY + maxY) / 2 };
    const radians = degrees * Math.PI / 180;
    const cos = Math.cos(radians);
    const sin = Math.sin(radians);
    this.transformSketch(point => {
      const dx = point.x - center.x;
      const dy = point.y - center.y;
      return {
        x: center.x + dx * cos - dy * sin,
        y: center.y + dx * sin + dy * cos
      };
    }, degrees);
    // North rotates with the sketch geometry.
    this.state.northAngle = this.normalizeNorthAngle(this.state.northAngle + degrees);
    this.clearSelection();
    this.ui.updateCalcsPanel();
    this.ui.updateNorthDisplay?.();
    this.render();
  }

  flipSketch(axis = 'horizontal') {
    const points = this.getAllSketchPoints();
    if (points.length === 0) return;
    this.pushUndo('Flip Sketch');
    const minX = Math.min(...points.map(p => p.x));
    const maxX = Math.max(...points.map(p => p.x));
    const minY = Math.min(...points.map(p => p.y));
    const maxY = Math.max(...points.map(p => p.y));
    const center = { x: (minX + maxX) / 2, y: (minY + maxY) / 2 };
    // A reflection reverses handedness, so each signed curve bow must reverse
    // as well or the arc will appear on the wrong side of its chord.
    this.state.lines.forEach(line => {
      line.bowAngle = -(Number(line.bowAngle) || 0);
      line.arcHeight = -(Number(line.arcHeight) || 0);
    });
    this.state.polygons.forEach(poly => {
      (poly.edges || []).forEach(edge => {
        edge.bowAngle = -(Number(edge.bowAngle) || 0);
      });
    });
    this.transformSketch(point => ({
      x: axis === 'horizontal' ? 2 * center.x - point.x : point.x,
      y: axis === 'vertical' ? 2 * center.y - point.y : point.y
    }));
    // A mirror reverses the compass bearing: across the vertical axis a
    // bearing a becomes 360 - a; across the horizontal axis 180 - a.
    this.state.northAngle = this.normalizeNorthAngle(
      axis === 'horizontal' ? -this.state.northAngle : 180 - this.state.northAngle
    );
    this.clearSelection();
    this.ui.updateCalcsPanel();
    this.ui.updateNorthDisplay?.();
    this.render();
  }

  normalizeNorthAngle(degrees) {
    const value = Number(degrees) || 0;
    return ((value % 360) + 360) % 360;
  }

  setNorthAngle(degrees) {
    const value = parseFloat(degrees);
    if (!Number.isFinite(value)) return;
    this.state.northAngle = this.normalizeNorthAngle(value);
    this.ui.updateNorthDisplay?.();
    this.render();
  }

  getSelectedPolygons() {
    return this.state.polygons.filter(poly => this.state.selectedPolyIds.has(poly.id));
  }

  setSelectedAreaStyle(property, value) {
    const selected = this.getSelectedPolygons();
    if (!selected.length) return;
    this.pushUndo('Area Style');
    const lineIds = new Set();
    selected.forEach(poly => {
      poly[property] = value;
      (poly.edges || []).forEach(edge => { if (edge.lineId != null) lineIds.add(edge.lineId); });
    });
    const boundaryProperty = ({ lineStyle: 'lineStyle', lineThickness: 'lineThickness', lineColor: 'color' })[property];
    if (boundaryProperty) this.state.lines.forEach(line => {
      if (lineIds.has(line.id)) line[boundaryProperty] = value;
    });
    this.render();
  }

  updateSelectedAreaProperties(properties) {
    const selected = this.getSelectedPolygons();
    if (!selected.length) return;
    this.pushUndo('Area Properties');
    selected.forEach(poly => {
      poly.type = properties.type || poly.type;
      poly.name = properties.name || poly.name;
      poly.factorOverride = Number.isFinite(Number(properties.factor)) ? Number(properties.factor) : null;
      poly.factor = poly.factorOverride == null ? (poly.factor || 1) : poly.factorOverride;
      poly.subtractMode = properties.autoSubtract === false ? 'never' : 'auto';
      if (properties.measureTo != null) poly.measureTo = ['center', 'interior', 'exterior'].includes(properties.measureTo) ? properties.measureTo : 'center';
      poly.autoPost = { ...(poly.autoPost || {}), ...(properties.autoPost || {}) };
    });
    const dimensionLineIds = new Set(selected.flatMap(poly => (poly.edges || []).map(edge => edge.lineId)));
    if (properties.autoPost && typeof properties.autoPost.dims === 'boolean') {
      this.state.lines.forEach(line => { if (dimensionLineIds.has(line.id)) line.dimensionHidden = !properties.autoPost.dims; });
    }
    this.recalcAutoSubtract();
    this.ui.updateCalcsPanel();
    this.render();
  }

  updateSelectedAreaAttributes(attributes) {
    const selected = this.getSelectedPolygons();
    if (!selected.length) return;
    this.pushUndo('Area Attributes');
    selected.forEach(poly => { poly.attributes = { ...(poly.attributes || {}), ...attributes }; });
    this.render();
  }

  toggleSelectedAreaAutoSubtract() {
    const selected = this.getSelectedPolygons();
    if (!selected.length) return;
    this.pushUndo('Auto-Subtract');
    const enable = selected.some(poly => poly.subtractMode === 'never');
    selected.forEach(poly => { poly.subtractMode = enable ? 'auto' : 'never'; });
    this.recalcAutoSubtract();
    this.ui.updateCalcsPanel();
    this.ui.notify(`Auto-Subtract ${enable ? 'enabled' : 'disabled'} for selected area${selected.length === 1 ? '' : 's'}.`);
    this.render();
  }

  // Toggles the calculation sign of the selected areas, matching the
  // reference app's Make Negative command. A negative factorOverride makes
  // the area's net value subtract from the totals in the calc engine.
  makeNegative() {
    const selected = this.getSelectedPolygons();
    if (!selected.length) { this.ui.notify('Select an area to make negative.'); return; }
    this.pushUndo('Make Negative');
    const makeNegative = selected.some(poly => ((poly.factorOverride ?? poly.factor) ?? 1) >= 0);
    selected.forEach(poly => {
      const magnitude = Math.abs(poly.factorOverride ?? poly.factor ?? 1) || 1;
      poly.factorOverride = makeNegative ? -magnitude : magnitude;
      poly.factor = poly.factorOverride;
    });
    this.recalcAutoSubtract();
    this.ui.updateCalcsPanel();
    this.ui.notify(`Area${selected.length === 1 ? '' : 's'} ${makeNegative ? 'made negative' : 'made positive'}.`);
    this.render();
  }

  flipSelectedAreas(axis = 'horizontal') {
    const selected = this.getSelectedPolygons();
    if (!selected.length) return;
    const model = this.getSelectedGeometryRotationModel();
    if (!model) return;
    this.pushUndo('Flip Area');
    const { key, selectedKeys, center } = model;
    const lineIds = new Set();
    selected.forEach(poly => (poly.edges || []).forEach(edge => { if (edge.lineId != null) lineIds.add(edge.lineId); }));
    const flipPoint = point => {
      if (!selectedKeys.has(key(point))) return;
      if (axis === 'horizontal') point.x = 2 * center.x - point.x;
      else point.y = 2 * center.y - point.y;
    };
    this.state.lines.forEach(line => {
      if (!lineIds.has(line.id)) return;
      flipPoint(line.start); flipPoint(line.end);
      line.bowAngle = -(Number(line.bowAngle) || 0);
      line.arcHeight = -(Number(line.arcHeight) || 0);
    });
    selected.forEach(poly => {
      poly.points.forEach(flipPoint);
      (poly.edges || []).forEach(edge => { edge.bowAngle = -(Number(edge.bowAngle) || 0); });
    });
    this.state.symbols.forEach(symbol => {
      if (!lineIds.has(symbol.wallId)) return;
      if (axis === 'horizontal') symbol.x = 2 * center.x - symbol.x;
      else symbol.y = 2 * center.y - symbol.y;
      symbol.rotation = axis === 'horizontal' ? -(Number(symbol.rotation) || 0) : 180 - (Number(symbol.rotation) || 0);
    });
    this.recalcAutoSubtract();
    this.ui.updateCalcsPanel();
    this.render();
  }

  // --- Layer System ---
  //
  // A simplified Photoshop-style layer stack. state.layers is a flat array
  // in paint order (index 0 is the bottom). Groups are layers with
  // isGroup:true; child layers reference them via parentId. The visible
  // paint order is computed by flattenLayerOrder(). Every drawing entity
  // carries a layerId; entities from files saved before layers existed
  // fall back to the default (bottom) layer.

  ensureLayers(layers, activeLayerId) {
    if (Array.isArray(layers) && layers.length) {
      this.state.layers = layers.map((layer, index) => ({
        id: layer.id != null ? String(layer.id) : 'layer-' + (index + 1),
        name: layer.name || 'Layer ' + (index + 1),
        visible: layer.visible !== false,
        locked: !!layer.locked,
        isGroup: !!layer.isGroup,
        collapsed: !!layer.collapsed,
        parentId: layer.parentId != null ? String(layer.parentId) : null,
        areaPolyId: layer.areaPolyId != null ? String(layer.areaPolyId) : null
      }));
    } else {
      this.state.layers = [{
        id: 'layer-1', name: 'Layer 1', visible: true,
        locked: false, isGroup: false, collapsed: false, parentId: null
      }];
    }
    const fallback = this.getDefaultLayerId();
    this.state.activeLayerId =
      activeLayerId != null && this.getLayerById(String(activeLayerId))
        ? String(activeLayerId) : fallback;
  }

  flattenLayerOrder() {
    const order = [];
    const visit = parentId => {
      this.state.layers.forEach(layer => {
        if ((layer.parentId || null) === parentId) {
          order.push(layer);
          visit(layer.id);
        }
      });
    };
    visit(null);
    return order;
  }

  // Entities in reverse paint order - topmost layer first, last drawn first
  // within a layer - so hit testing picks what the user actually sees on top.
  paintOrderDescending(array) {
    const order = this.flattenLayerOrder().map(layer => layer.id);
    const rank = entity => {
      const index = order.indexOf(this.getEntityLayerId(entity));
      return index < 0 ? -1 : index;
    };
    return array
      .map((entity, index) => ({ entity, rank: rank(entity), index }))
      .sort((a, b) => b.rank - a.rank || b.index - a.index)
      .map(item => item.entity);
  }

  getLayerById(layerId) {
    return this.state.layers.find(layer => layer.id === layerId) || null;
  }

  getDefaultLayerId() {
    const flat = this.flattenLayerOrder();
    const first = flat.find(layer => !layer.isGroup) || flat[0] || this.state.layers[0];
    return first ? first.id : null;
  }

  getEntityLayerId(entity) {
    if (entity && entity.layerId && this.getLayerById(entity.layerId)) return entity.layerId;
    return this.getDefaultLayerId();
  }

  // The smallest closed area containing the point owns content dropped there.
  // paintOrderDescending supplies a deterministic tie-breaker for coincident
  // or equally sized areas while the area sort favors an inset over its host.
  getAreaAtPoint(worldPt) {
    if (!worldPt) return null;
    return this.paintOrderDescending(this.state.polygons)
      .filter(poly => (poly.points || []).length >= 3 &&
        window.Geometry.pointInPolygon(worldPt, window.Geometry.flattenRing(poly)))
      .map((poly, paintRank) => ({ poly, paintRank }))
      .sort((a, b) => {
        const areaA = Number.isFinite(Number(a.poly.area)) ? Math.abs(Number(a.poly.area)) : Infinity;
        const areaB = Number.isFinite(Number(b.poly.area)) ? Math.abs(Number(b.poly.area)) : Infinity;
        return areaA - areaB || a.paintRank - b.paintRank;
      })[0]?.poly || null;
  }

  getAreaLayerIdAt(worldPt) {
    const area = this.getAreaAtPoint(worldPt);
    if (!area) return this.state.activeLayerId || this.getDefaultLayerId();
    const layer = this.assignAreaToDedicatedLayer(area);
    return layer ? layer.id : (this.state.activeLayerId || this.getDefaultLayerId());
  }

  // The Layers tab represents the current drawing page, not every internal
  // paint layer accumulated by old files or other pages.
  getAreaLayersForCurrentPage() {
    this.state.polygons.forEach(poly => {
      const layer = this.getLayerById(poly.layerId);
      if (!layer || String(layer.areaPolyId) !== String(poly.id)) {
        this.assignAreaToDedicatedLayer(poly, { updatePanel: false });
      }
    });
    const polygonIds = new Set(this.state.polygons.map(poly => String(poly.id)));
    const polygonLayerIds = new Set(this.state.polygons.map(poly => String(poly.layerId || '')));
    return this.flattenLayerOrder().filter(layer =>
      layer.areaPolyId != null &&
      polygonIds.has(String(layer.areaPolyId)) &&
      polygonLayerIds.has(String(layer.id))
    );
  }

  // Upgrade legacy pages lazily as they are opened. Each closed polygon gets
  // one dedicated layer; existing text inside an area follows that area too.
  ensureAreaLayersForCurrentPage() {
    this.state.polygons.forEach(poly => this.assignAreaToDedicatedLayer(poly, { updatePanel: false }));
    this.state.labels.forEach(label => {
      const area = this.getAreaAtPoint(label);
      if (area) label.layerId = area.layerId;
    });
    this.ui.updateLayersPanel?.();
  }

  layerAncestry(layerId) {
    const chain = [];
    let current = this.getLayerById(layerId);
    while (current) {
      chain.unshift(current);
      current = current.parentId ? this.getLayerById(current.parentId) : null;
    }
    return chain;
  }

  isLayerVisible(layerId) {
    return this.layerAncestry(layerId).every(layer => layer.visible);
  }

  isLayerLocked(layerId) {
    return this.layerAncestry(layerId).some(layer => layer.locked);
  }

  isEntityVisible(entity) {
    return this.isLayerVisible(this.getEntityLayerId(entity));
  }

  isEntityLocked(entity) {
    return this.isLayerLocked(this.getEntityLayerId(entity));
  }

  descendantLayerIds(layerId) {
    const ids = [layerId];
    this.state.layers.forEach(layer => {
      if (layer.parentId === layerId) ids.push(...this.descendantLayerIds(layer.id));
    });
    return ids;
  }

  clearLayerSelection(layerIds) {
    const ids = new Set(layerIds);
    const onLayer = entity => ids.has(this.getEntityLayerId(entity));
    this.state.lines.forEach(line => {
      if (!onLayer(line)) return;
      this.state.selectedLineIds.delete(line.id);
      this.state.selectedDimensionIds.delete(line.id);
    });
    this.state.polygons.forEach(poly => {
      if (!onLayer(poly)) return;
      this.state.selectedPolyIds.delete(poly.id);
      this.state.selectedAreaLabelIds.delete(poly.id);
      this.state.selectedAreaSizeIds.delete(poly.id);
    });
    this.state.labels.forEach(label => {
      if (onLayer(label)) this.state.selectedLabelIds.delete(label.id);
    });
    this.state.symbols.forEach(symbol => {
      if (onLayer(symbol)) this.state.selectedSymbolIds.delete(symbol.id);
    });
  }
  addLayer(name, isGroup = false) {
    const layer = this.createLayerRaw(name, isGroup);
    this.state.activeLayerId = layer.id;
    this.ui.updateLayersPanel?.();
    this.render();
    return layer;
  }

  // Pushes a layer onto the stack without rendering or touching the active
  // layer - callers like assignAreaToDedicatedLayer manage those themselves.
  createLayerRaw(name, isGroup = false) {
    let id;
    do { id = 'layer-' + this.genId(); } while (this.getLayerById(id));
    const layer = {
      id,
      name: name || (isGroup ? 'Group' : 'Layer ' + (this.state.layers.length + 1)),
      visible: true,
      locked: false,
      isGroup,
      collapsed: false,
      parentId: null
    };
    this.state.layers.push(layer);
    return layer;
  }

  // Returns the given layer name unless it is taken, then "name 2", "name 3", ...
  uniqueLayerName(base) {
    const names = new Set(this.state.layers.map(layer => layer.name));
    if (!names.has(base)) return base;
    let index = 2;
    while (names.has(base + ' ' + index)) index++;
    return base + ' ' + index;
  }

  // Every area lives on its own dedicated layer: the polygon and its boundary
  // walls move onto a fresh layer at the top of the stack, so overlapping
  // areas can be re-stacked independently and a foreground area's fill and
  // walls always paint over the layers below it. When activate is set the new
  // layer also becomes the active drawing layer. Dedicated area layers carry
  // areaPolyId so they can be reused on re-close and removed with their area.
  assignAreaToDedicatedLayer(poly, { activate = false, updatePanel = true } = {}) {
    if (!poly) return null;
    const edgeIds = new Set((poly.edges || []).map(edge => edge.lineId).filter(id => id != null));
    const sourceLayerIds = new Set();
    this.state.lines.forEach(line => { if (edgeIds.has(line.id)) sourceLayerIds.add(this.getEntityLayerId(line)); });

    // Repeated synchronization of an already dedicated area is idempotent.
    let layer = this.getLayerById(poly.layerId);
    if (!layer || String(layer.areaPolyId) !== String(poly.id)) layer = null;

    // A re-closed (reopened) area keeps its original layer when all of its
    // walls already sit on that dedicated layer and its polygon is gone.
    if (!layer && sourceLayerIds.size === 1) {
      const candidate = this.getLayerById([...sourceLayerIds][0]);
      if (candidate && candidate.areaPolyId != null &&
          !this.state.polygons.some(item => String(item.id) === String(candidate.areaPolyId))) {
        layer = candidate;
      }
    }
    if (!layer) {
      const base = poly.name && poly.name !== 'Undefined Area' ? poly.name : 'Area';
      layer = this.createLayerRaw(this.uniqueLayerName(base), false);
    }
    layer.areaPolyId = poly.id;
    const layerId = layer.id;
    poly.layerId = layerId;
    this.state.lines.forEach(line => { if (edgeIds.has(line.id)) line.layerId = layerId; });
    if (activate) this.state.activeLayerId = layerId;
    // Dedicated area layers left empty by the move go away with their area.
    sourceLayerIds.forEach(id => { if (id !== layerId) this.removeEmptyAreaLayer(id); });
    if (updatePanel) this.ui.updateLayersPanel?.();
    return layer;
  }

  // All symbols share one dedicated "Symbols" layer so they stack above (or
  // below) the area layers as a group. Created on first use, never activated.
  ensureSymbolsLayer() {
    const existing = this.state.layers.find(layer => layer.name === 'Symbols' && !layer.isGroup && !layer.parentId);
    if (existing) return existing;
    const layer = this.createLayerRaw('Symbols', false);
    this.ui.updateLayersPanel?.();
    return layer;
  }

  // A dedicated area layer disappears with its area once nothing is left on
  // it. Called after an area is deleted, or after its walls move onto another
  // area's layer. Layers that still hold any entity are left alone.
  removeEmptyAreaLayer(layerId) {
    const layer = this.getLayerById(layerId);
    if (!layer || layer.areaPolyId == null) return;
    const occupied =
      this.state.polygons.some(item => this.getEntityLayerId(item) === layerId) ||
      this.state.lines.some(item => this.getEntityLayerId(item) === layerId) ||
      this.state.labels.some(item => this.getEntityLayerId(item) === layerId) ||
      this.state.symbols.some(item => this.getEntityLayerId(item) === layerId);
    if (occupied) return;
    this.clearLayerSelection([layerId]);
    this.state.layers = this.state.layers.filter(item => item.id !== layerId && item.parentId !== layerId);
    if (!this.state.layers.length) this.ensureLayers();
    if (this.state.activeLayerId === layerId || !this.getLayerById(this.state.activeLayerId)) {
      this.state.activeLayerId = this.getDefaultLayerId();
    }
  }

  renameLayer(layerId, name) {
    const layer = this.getLayerById(layerId);
    if (!layer) return;
    const clean = String(name || '').trim();
    if (!clean) return;
    layer.name = clean;
    this.ui.updateLayersPanel?.();
  }

  removeLayer(layerId) {
    const layer = this.getLayerById(layerId);
    if (!layer) return;
    const ids = new Set(this.descendantLayerIds(layerId));
    const defaultId = this.getDefaultLayerId();
    const reassign = entity => {
      if (ids.has(this.getEntityLayerId(entity))) entity.layerId = defaultId;
    };
    this.state.lines.forEach(reassign);
    this.state.polygons.forEach(reassign);
    this.state.labels.forEach(reassign);
    this.state.symbols.forEach(reassign);
    this.state.layers = this.state.layers.filter(item => !ids.has(item.id));
    this.clearLayerSelection([...ids]);
    if (!this.state.layers.length) this.ensureLayers();
    if (ids.has(this.state.activeLayerId)) this.state.activeLayerId = this.getDefaultLayerId();
    this.ui.updateLayersPanel?.();
    this.render();
  }

  setLayerVisible(layerId, visible) {
    const layer = this.getLayerById(layerId);
    if (!layer || layer.visible === visible) return;
    this.pushUndo('Layer Visibility');
    layer.visible = visible;
    if (!visible) this.clearLayerSelection(this.descendantLayerIds(layerId));
    this.ui.updateLayersPanel?.();
    this.render();
  }

  setLayerLocked(layerId, locked) {
    const layer = this.getLayerById(layerId);
    if (!layer || layer.locked === locked) return;
    this.pushUndo('Layer Lock');
    layer.locked = locked;
    if (locked) this.clearLayerSelection(this.descendantLayerIds(layerId));
    this.ui.updateLayersPanel?.();
    this.render();
  }

  setLayerCollapsed(layerId, collapsed) {
    const layer = this.getLayerById(layerId);
    if (!layer) return;
    layer.collapsed = collapsed;
    this.ui.updateLayersPanel?.();
  }

  setActiveLayer(layerId) {
    if (!this.getLayerById(layerId)) return;
    this.state.activeLayerId = layerId;
    this.ui.updateLayersPanel?.();
  }

  moveLayer(layerId, targetId, position) {
    const layer = this.getLayerById(layerId);
    const target = this.getLayerById(targetId);
    if (!layer || !target || layerId === targetId) return;
    const moving = new Set(this.descendantLayerIds(layerId));
    if (moving.has(targetId)) return; // cannot move a layer inside itself
    this.pushUndo('Move Layer');
    layer.parentId = null;
    const targetChildren = new Set(this.descendantLayerIds(targetId));
    const others = this.state.layers.filter(item => !moving.has(item.id));
    const insertAt = others.findIndex(item => item.id === targetId);
    if (position === 'inside') {
      target.collapsed = false;
      layer.parentId = target.id;
      others.splice(insertAt + 1, 0, layer); // becomes the group's first child
    } else if (position === 'before') {
      others.splice(insertAt, 0, layer);
    } else {
      let at = insertAt + 1;
      while (at < others.length && targetChildren.has(others[at].id)) at++;
      others.splice(at, 0, layer);
    }
    this.state.layers = others;
    this.ui.updateLayersPanel?.();
    this.render();
  }

  moveSelectionToLayer(layerId) {
    if (!this.getLayerById(layerId)) return;
    const s = this.state;
    const hasAny = s.selectedLineIds.size > 0 || s.selectedPolyIds.size > 0 ||
      s.selectedAreaLabelIds.size > 0 || s.selectedAreaSizeIds.size > 0 ||
      s.selectedLabelIds.size > 0 || s.selectedSymbolIds.size > 0;
    if (!hasAny) return;
    this.pushUndo('Move to Layer');
    const assign = (entity, ids) => { if (ids.has(entity.id)) entity.layerId = layerId; };
    s.lines.forEach(line => assign(line, s.selectedLineIds));
    s.lines.forEach(line => assign(line, s.selectedDimensionIds));
    s.polygons.forEach(poly => assign(poly, s.selectedPolyIds));
    s.polygons.forEach(poly => assign(poly, s.selectedAreaLabelIds));
    s.polygons.forEach(poly => assign(poly, s.selectedAreaSizeIds));
    s.labels.forEach(label => assign(label, s.selectedLabelIds));
    s.symbols.forEach(symbol => assign(symbol, s.selectedSymbolIds));
    this.ui.updateLayersPanel?.();
    this.render();
  }

  // Duplicates a layer (and, for groups, all of its children) together with
  // every entity sitting on them. Copies land right after their originals in
  // the stack, keep the same nesting, and the new root layer becomes active.
  duplicateLayer(layerId) {
    const layer = this.getLayerById(layerId);
    if (!layer) return null;
    this.pushUndo('Duplicate Layer');
    const sourceIds = new Set(this.descendantLayerIds(layerId));
    const layerIdMap = new Map();
    sourceIds.forEach(id => {
      let newId;
      do { newId = 'layer-' + this.genId(); } while (this.getLayerById(newId));
      layerIdMap.set(id, newId);
    });
    const orderedSources = this.state.layers.filter(item => layerIdMap.has(item.id));
    const clones = orderedSources.map(src => ({
      ...JSON.parse(JSON.stringify(src)),
      id: layerIdMap.get(src.id),
      parentId: layerIdMap.has(src.parentId) ? layerIdMap.get(src.parentId) : (src.parentId || null),
      name: this.uniqueLayerName(src.name + ' copy')
    }));
    const insertAt = this.state.layers.indexOf(orderedSources[orderedSources.length - 1]);
    this.state.layers.splice(insertAt + 1, 0, ...clones);
    const onSource = entity => layerIdMap.has(this.getEntityLayerId(entity));
    const remappedLayer = entity => layerIdMap.get(this.getEntityLayerId(entity));
    const lineIdMap = new Map();
    const polyIdMap = new Map();
    this.state.lines.filter(onSource).forEach(line => {
      const copy = JSON.parse(JSON.stringify(line));
      copy.id = this.genId();
      lineIdMap.set(line.id, copy.id);
      copy.layerId = remappedLayer(line);
      this.state.lines.push(copy);
    });
    this.state.polygons.filter(onSource).forEach(poly => polyIdMap.set(poly.id, this.genId()));
    this.state.polygons.filter(onSource).forEach(poly => {
      const copy = JSON.parse(JSON.stringify(poly));
      copy.id = polyIdMap.get(poly.id);
      copy.layerId = remappedLayer(poly);
      copy.edges = (copy.edges || []).map(edge => ({ ...edge, lineId: lineIdMap.get(edge.lineId) ?? null }));
      copy.containerId = polyIdMap.get(copy.containerId) ?? null;
      copy.subtractedFromId = polyIdMap.get(copy.subtractedFromId) ?? null;
      copy.subtractTargetId = polyIdMap.get(copy.subtractTargetId) ?? null;
      this.state.polygons.push(copy);
    });
    this.state.labels.filter(onSource).forEach(label => {
      const copy = JSON.parse(JSON.stringify(label));
      copy.id = this.genId();
      copy.layerId = remappedLayer(label);
      this.state.labels.push(copy);
    });
    this.state.symbols.filter(onSource).forEach(symbol => {
      const copy = JSON.parse(JSON.stringify(symbol));
      copy.id = this.genId();
      copy.layerId = remappedLayer(symbol);
      this.state.symbols.push(copy);
    });
    this.state.activeLayerId = layerIdMap.get(layerId);
    this.ui.updateLayersPanel?.();
    this.render();
    return this.getLayerById(this.state.activeLayerId);
  }

  // Serializes a layer (and, for groups, its children) together with every
  // entity on them into a JSON file that importLayerFile() can read back.
  exportLayer(layerId) {
    const layer = this.getLayerById(layerId);
    if (!layer) return;
    const layerIds = new Set(this.descendantLayerIds(layerId));
    const onLayer = entity => layerIds.has(this.getEntityLayerId(entity));
    const data = {
      version: 'Sketch-Layer',
      layers: this.state.layers.filter(item => layerIds.has(item.id)),
      lines: this.state.lines.filter(onLayer),
      polygons: this.state.polygons.filter(onLayer),
      labels: this.state.labels.filter(onLayer),
      symbols: this.state.symbols.filter(onLayer)
    };
    const safeName = String(layer.name || 'layer').replace(/[^\w\- ]+/g, '').trim() || 'layer';
    const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${safeName}-layer.json`;
    a.click();
    URL.revokeObjectURL(url);
    this.ui.notify(`Exported layer "${layer.name}"`);
  }

  importLayerFile(file) {
    const reader = new FileReader();
    reader.onload = async (e) => {
      try {
        const data = JSON.parse(e.target.result);
        if (!Array.isArray(data.layers) || !data.layers.length) {
          throw new Error('No layer data found');
        }
        this.pushUndo('Import Layer');
        const layerIdMap = new Map();
        data.layers.forEach(src => {
          let newId;
          do { newId = 'layer-' + this.genId(); } while (this.getLayerById(newId));
          layerIdMap.set(src.id, newId);
        });
        data.layers.forEach(src => {
          this.state.layers.push({
            ...JSON.parse(JSON.stringify(src)),
            id: layerIdMap.get(src.id),
            parentId: layerIdMap.has(src.parentId) ? layerIdMap.get(src.parentId) : null,
            name: this.uniqueLayerName(src.name || 'Imported layer')
          });
        });
        const mappedLayerId = id => layerIdMap.get(id) ?? this.getDefaultLayerId();
        const lineIdMap = new Map();
        const polyIdMap = new Map();
        (data.lines || []).forEach(line => {
          const newId = this.genId();
          lineIdMap.set(line.id, newId);
          this.state.lines.push({ ...JSON.parse(JSON.stringify(line)), id: newId, layerId: mappedLayerId(line.layerId) });
        });
        (data.polygons || []).forEach(poly => polyIdMap.set(poly.id, this.genId()));
        (data.polygons || []).forEach(poly => {
          this.state.polygons.push({
            ...JSON.parse(JSON.stringify(poly)),
            edges: (poly.edges || []).map(edge => ({ ...edge, lineId: lineIdMap.get(edge.lineId) ?? null })),
            containerId: polyIdMap.get(poly.containerId) ?? null,
            subtractedFromId: polyIdMap.get(poly.subtractedFromId) ?? null,
            subtractTargetId: polyIdMap.get(poly.subtractTargetId) ?? null,
            id: polyIdMap.get(poly.id),
            layerId: mappedLayerId(poly.layerId)
          });
        });
        (data.labels || []).forEach(label => {
          this.state.labels.push({ ...JSON.parse(JSON.stringify(label)), id: this.genId(), layerId: mappedLayerId(label.layerId) });
        });
        (data.symbols || []).forEach(symbol => {
          this.state.symbols.push({ ...JSON.parse(JSON.stringify(symbol)), id: this.genId(), layerId: mappedLayerId(symbol.layerId) });
        });
        this.state.polygons = this.ensurePageModel(this.getPageData(), this.currentPage).polygons;
        this.state.activeLayerId = layerIdMap.get(data.layers[0].id) || this.state.activeLayerId;
        this.recalcAutoSubtract();
        this.ui.updateCalcsPanel();
        this.ui.updateLayersPanel?.();
        this.render();
        this.ui.notify(`Imported layer from ${file.name}`);
      } catch (err) {
        await this.ui.alertDialog('Error importing layer: ' + err.message);
      }
    };
    reader.readAsText(file);
  }

  applyLayerOrder() {
    const orderMap = new Map();
    this.flattenLayerOrder().forEach((layer, index) => orderMap.set(layer.id, index));
    const layerIndex = entity => orderMap.get(this.getEntityLayerId(entity));
    const byLayer = (a, b) => layerIndex(a) - layerIndex(b);
    const s = this.state;
    s.lines.sort(byLayer);
    s.polygons.sort(byLayer);
    s.labels.sort(byLayer);
    s.symbols.sort(byLayer);
  }

  // Context-menu Send to Back / Bring to Front: auto-assigns the current
  // selection to a dedicated layer placed at the very bottom or top of the
  // layer stack, so the z-order change is visible both on the canvas and in
  // the Layers panel. The dedicated layer is reused if it still exists,
  // ungrouped, with the same name.
  sendToBack() {
    this.autoAssignLayer('Send to Back', true);
  }

  bringToFront() {
    this.autoAssignLayer('Bring to Front', false);
  }

  hasSelection() {
    const s = this.state;
    return s.selectedLineIds.size > 0 || s.selectedPolyIds.size > 0 ||
      s.selectedAreaLabelIds.size > 0 || s.selectedAreaSizeIds.size > 0 ||
      s.selectedLabelIds.size > 0 || s.selectedSymbolIds.size > 0;
  }

  autoAssignLayer(name, toBack) {
    const s = this.state;
    if (!this.hasSelection()) return;
    this.pushUndo('Auto-Assign Layer');
    let layer = s.layers.find(item => item.name === name && !item.isGroup && !item.parentId);
    if (!layer) layer = this.addLayer(name, false);
    layer.visible = true;
    layer.locked = false;
    const others = s.layers.filter(item => item.id !== layer.id);
    s.layers = toBack ? [layer, ...others] : [...others, layer];
    const assign = (entity, ids) => { if (ids.has(entity.id)) entity.layerId = layer.id; };
    s.lines.forEach(line => assign(line, s.selectedLineIds));
    s.lines.forEach(line => assign(line, s.selectedDimensionIds));
    s.polygons.forEach(poly => assign(poly, s.selectedPolyIds));
    s.polygons.forEach(poly => assign(poly, s.selectedAreaLabelIds));
    s.polygons.forEach(poly => assign(poly, s.selectedAreaSizeIds));
    s.labels.forEach(label => assign(label, s.selectedLabelIds));
    s.symbols.forEach(symbol => assign(symbol, s.selectedSymbolIds));
    this.setActiveLayer(layer.id);
    this.ui.updateLayersPanel?.();
    this.reorderSelectionWithinLayers(!toBack);
    this.ui?.notify?.('Moved selection to layer ' + layer.name + '.');
    this.render();
  }

  reorderSelected(toFront) {
    if (!this.hasSelection()) return;
    this.pushUndo('Reorder Selection');
    this.reorderSelectionWithinLayers(toFront);
    this.render();
  }

  // Shared by reorderSelected and autoAssignLayer: moves the selected
  // entities to the front or back of their own layer segment. No undo push
  // and no render - callers manage those.
  reorderSelectionWithinLayers(toFront) {
    const s = this.state;
    const reorderWithinLayers = (items, isSelected) => {
      const selected = items.filter(isSelected);
      if (!selected.length) return items;
      const order = this.flattenLayerOrder().map(layer => layer.id);
      const out = [];
      order.forEach(layerId => {
        const segment = items.filter(item => this.getEntityLayerId(item) === layerId);
        const segSelected = segment.filter(isSelected);
        out.push(...(segSelected.length
          ? (toFront
              ? [...segment.filter(item => !isSelected(item)), ...segSelected]
              : [...segSelected, ...segment.filter(item => !isSelected(item))])
          : segment));
      });
      return out;
    };
    s.polygons = reorderWithinLayers(s.polygons, poly => s.selectedPolyIds.has(poly.id) || s.selectedAreaLabelIds.has(poly.id) || s.selectedAreaSizeIds.has(poly.id));
    s.lines = reorderWithinLayers(s.lines, line => s.selectedLineIds.has(line.id) || s.selectedDimensionIds.has(line.id));
    s.labels = reorderWithinLayers(s.labels, label => s.selectedLabelIds.has(label.id));
    s.symbols = reorderWithinLayers(s.symbols, symbol => s.selectedSymbolIds.has(symbol.id));
    this.render();
  }


  setSelectedTextStyle(property, value) {
    const selected = this.state.labels.filter(label => this.state.selectedLabelIds.has(label.id));
    const areaLabels = this.state.polygons.filter(poly =>
      this.state.selectedAreaLabelIds.has(poly.id) || this.state.selectedAreaSizeIds.has(poly.id));
    if (!selected.length && !areaLabels.length) return;
    this.pushUndo('Text Style');
    selected.forEach(label => { label[property] = property === 'fontSize' ? Math.max(6, Math.min(72, Number(value) || 12)) : value; });
    const areaProperty = {
      fontFamily: 'labelFontFamily', fontSize: 'labelFontSize', color: 'labelColor',
      bold: 'labelBold', italic: 'labelItalic', underline: 'labelUnderline', text: 'name'
    }[property];
    if (areaProperty) areaLabels.forEach(poly => {
      if (property === 'text' && !this.state.selectedAreaLabelIds.has(poly.id)) return;
      poly[areaProperty] = property === 'fontSize' ? Math.max(6, Math.min(72, Number(value) || 12)) : value;
    });
    this.render();
  }

  toggleSelectedTextStyle(property) {
    const selected = this.state.labels.filter(label => this.state.selectedLabelIds.has(label.id));
    const areaLabels = this.state.polygons.filter(poly =>
      this.state.selectedAreaLabelIds.has(poly.id) || this.state.selectedAreaSizeIds.has(poly.id));
    if (!selected.length && !areaLabels.length) return;
    const areaProperty = { bold: 'labelBold', italic: 'labelItalic', underline: 'labelUnderline' }[property];
    const next = !(selected.every(label => Boolean(label[property])) && areaLabels.every(poly => Boolean(poly[areaProperty])));
    this.setSelectedTextStyle(property, next);
  }

  adjustSelectedTextSize(delta) {
    const selected = this.state.labels.filter(label => this.state.selectedLabelIds.has(label.id));
    const areaLabels = this.state.polygons.filter(poly =>
      this.state.selectedAreaLabelIds.has(poly.id) || this.state.selectedAreaSizeIds.has(poly.id));
    if (!selected.length && !areaLabels.length) return;
    this.pushUndo('Text Size');
    selected.forEach(label => { label.fontSize = Math.max(6, Math.min(72, (Number(label.fontSize) || 12) + delta)); });
    areaLabels.forEach(poly => { poly.labelFontSize = Math.max(6, Math.min(72, (Number(poly.labelFontSize) || Number(this.state.options?.autoPostFontSize) || 16) + delta)); });
    this.render();
  }

  // --- Placement ---

  startTextPlacement(text) {
    this.state.placementText = text;
    this.setMode('place-text');
  }

  startSymbolPlacement(symbolType) {
    this.state.placementSymbol = symbolType;
    this.setMode('place-symbol');
  }

  resolveSymbolPlacement(worldPt, symbolType = this.state.placementSymbol, { currentSideFlip = null } = {}) {
    const definition = window.SKETCH_SYMBOL_LIBRARY?.symbols?.[symbolType];
    const placement = {
      x: worldPt.x,
      y: worldPt.y,
      rotation: 0,
      wallId: null,
      wallSnap: definition?.wallSnap || 'NONE',
      wallGapWidth: (Number(definition?.wallGapWidth) || 0) / 12
    };
    if (!definition?.wallSnap || definition.wallSnap === 'NONE') return placement;

    const tolerance = Math.max(1.5, this.getScreenToleranceWorld(28));
    let nearest = null;
    this.state.lines.forEach(line => {
      if (Math.abs(Number(line.bowAngle) || 0) > 0.001 || Math.abs(Number(line.arcHeight) || 0) > 0.001) return;
      const dx = line.end.x - line.start.x;
      const dy = line.end.y - line.start.y;
      const lengthSq = dx * dx + dy * dy;
      if (lengthSq < 1e-8) return;
      const t = Math.max(0, Math.min(1,
        ((worldPt.x - line.start.x) * dx + (worldPt.y - line.start.y) * dy) / lengthSq));
      const point = { x: line.start.x + dx * t, y: line.start.y + dy * t };
      const distance = Math.hypot(worldPt.x - point.x, worldPt.y - point.y);
      if (distance <= tolerance && (!nearest || distance < nearest.distance)) {
        nearest = { line, point, distance, angle: Math.atan2(dy, dx), t };
      }
    });
    if (nearest) {
      placement.x = nearest.point.x;
      placement.y = nearest.point.y;
      placement.rotation = -nearest.angle * 180 / Math.PI;
      placement.wallId = nearest.line.id;
      placement.wallFraction = nearest.t;
      let sideFlip = false;

      // Door symbols extend in their +Y direction, which the renderer
      // (Y-flipped) maps to the LEFT side of the wall direction in world
      // coordinates. A SIDE-snapped opening swings toward the
      // side of the wall it is dropped on. A drop right on the wall line
      // keeps a re-snapped opening's swing, and otherwise swings an exterior
      // door into its area rather than outward.
      const wallLength = Math.hypot(nearest.line.end.x - nearest.line.start.x, nearest.line.end.y - nearest.line.start.y);
      const offset = ((nearest.line.end.x - nearest.line.start.x) * (worldPt.y - nearest.line.start.y) -
        (nearest.line.end.y - nearest.line.start.y) * (worldPt.x - nearest.line.start.x)) / wallLength;
      if (definition.wallSnap === 'SIDE' && Math.abs(offset) > 0.05) {
        sideFlip = offset < 0;
      } else if (definition.wallSnap === 'SIDE' && currentSideFlip != null) {
        sideFlip = Boolean(currentSideFlip);
      } else if (nearest.line.type === 'exterior' && definition.wallSnap === 'SIDE' &&
          this.exteriorSwingFlip(nearest.line, this.state.polygons)) {
        sideFlip = true;
      }
      if (sideFlip) placement.rotation += 180;
      placement.wallSideFlip = sideFlip;
    }
    return placement;
  }

  // True when a SIDE-snapped opening hosted on `line` must be flipped 180°
  // so its swing (local +Y, the left side of the wall direction) stays inside
  // the adjoining area instead of facing the exterior.
  exteriorSwingFlip(line, polygons) {
    const polygon = (polygons || []).find(poly => {
      for (let i = 0; i < (poly.points || []).length; i++) {
        const next = poly.points[(i + 1) % poly.points.length];
        if ((this.pointsEqual(poly.points[i], line.start) && this.pointsEqual(next, line.end)) ||
            (this.pointsEqual(poly.points[i], line.end) && this.pointsEqual(next, line.start))) {
          return true;
        }
      }
      return false;
    });
    if (!polygon || (polygon.points || []).length < 3) return false;
    const signedArea = window.Geometry.calculateSignedArea(polygon.points);
    let followsPath = true;
    for (let i = 0; i < polygon.points.length; i++) {
      const next = polygon.points[(i + 1) % polygon.points.length];
      if (this.pointsEqual(polygon.points[i], line.start) &&
          this.pointsEqual(next, line.end)) {
        followsPath = true;
        break;
      }
      if (this.pointsEqual(polygon.points[i], line.end) &&
          this.pointsEqual(next, line.start)) {
        followsPath = false;
        break;
      }
    }
    // For a counter-clockwise polygon the interior lies to the left of an
    // edge that follows the path, i.e. on the door's swing side, so no flip
    // is needed there; clockwise windings (or a wall traversed against the
    // path) put the exterior on the swing side instead.
    let exteriorOnSwingSide = signedArea >= 0 ? false : true;
    if (!followsPath) exteriorOnSwingSide = !exteriorOnSwingSide;
    return exteriorOnSwingSide;
  }

  resnapSelectedSymbols() {
    this.state.symbols.forEach(symbol => {
      if (!this.state.selectedSymbolIds.has(symbol.id)) return;
      const placement = this.resolveSymbolPlacement(symbol, symbol.symbolType,
        { currentSideFlip: symbol.wallId ? Boolean(symbol.wallSideFlip) : null });
      symbol.x = placement.x;
      symbol.y = placement.y;
      symbol.wallId = placement.wallId;
      if (placement.wallId) symbol.rotation = placement.rotation;
      if (placement.wallId && placement.wallFraction != null) {
        symbol.wallFraction = placement.wallFraction;
        symbol.wallSideFlip = Boolean(placement.wallSideFlip);
        symbol.wallSideExplicit = true;
        // Symbols dropped before wall hosting existed never received an
        // opening width; grant the definition's default once.
        if (symbol.wallGapWidth == null && Number(placement.wallGapWidth) > 0) {
          symbol.wallGapWidth = placement.wallGapWidth;
        }
      }
    });
  }

  // Recompute a hosted opening's position and rotation from its wall and the
  // stored along-wall fraction, so it follows the wall as it moves or resizes.
  repositionHostedSymbol(symbol) {
    if (!symbol.wallId || symbol.wallFraction == null) return false;
    const line = this.state.lines.find(item => item.id === symbol.wallId);
    if (!line) return false;
    const dx = line.end.x - line.start.x;
    const dy = line.end.y - line.start.y;
    if (Math.hypot(dx, dy) < 1e-6) return false;
    const t = Math.max(0, Math.min(1, symbol.wallFraction));
    symbol.x = line.start.x + dx * t;
    symbol.y = line.start.y + dy * t;
    const base = -Math.atan2(dy, dx) * 180 / Math.PI;
    symbol.rotation = base + (symbol.wallSideFlip ? 180 : 0);
    return true;
  }

  // Follow-along: keep every non-selected hosted opening glued to its wall.
  refreshHostedSymbols() {
    this.state.symbols.forEach(symbol => {
      if (symbol.wallId && symbol.wallFraction != null &&
          !this.state.selectedSymbolIds.has(symbol.id)) {
        this.repositionHostedSymbol(symbol);
      }
    });
  }

  // Swing flip: mirror the selected wall-hosted openings across the wall axis.
  flipOpening() {
    const targets = this.state.symbols.filter(sym =>
      this.state.selectedSymbolIds.has(sym.id) && sym.wallId);
    if (!targets.length) { this.ui.notify('Select a wall-hosted opening to flip.'); return; }
    this.pushUndo('Flip Opening');
    targets.forEach(sym => { sym.flipH = !sym.flipH; });
    this.render();
  }

  // Per-instance opening width: grow/shrink the gap cut in the host wall.
  adjustOpeningWidth(deltaFeet) {
    const targets = this.state.symbols.filter(sym =>
      this.state.selectedSymbolIds.has(sym.id) && sym.wallId);
    if (!targets.length) { this.ui.notify('Select a wall-hosted opening to resize.'); return; }
    this.pushUndo('Opening Width');
    targets.forEach(sym => {
      sym.wallGapWidth = Math.max(0, (Number(sym.wallGapWidth) || 0) + deltaFeet);
    });
    this.recalculate();
    this.render();
  }

  placeTextAt(worldPt) {
    if (!this.state.placementText) return;
    this.pushUndo('Place Text');
    const textOptions = this.state.options || {};
    this.state.labels.push({
      x: worldPt.x,
      y: worldPt.y,
      text: this.state.placementText,
      layerId: this.getAreaLayerIdAt(worldPt),
      fontSize: Number(textOptions.textFontSize) || 12,
      fontFamily: textOptions.textFont || 'Arial',
      bold: Boolean(textOptions.textBold),
      italic: Boolean(textOptions.textItalic),
      underline: Boolean(textOptions.textUnderline),
      // The old dark-theme default (#e2e8f0) is "automatic": dark on the
      // light canvas and black in print. Stamping it made labels near-white.
      color: textOptions.textColor && String(textOptions.textColor).toLowerCase() !== '#e2e8f0' ? textOptions.textColor : null,
      rotation: 0,
      id: this.genId()
    });
    // Stay in placement mode for rapid placement
    this.render();
    if (this.state.sidebar?.pinned === false) this.ui.setSidebarOpen?.(false);
  }

  placeSymbolAt(worldPt, symbolType = this.state.placementSymbol) {
    if (!symbolType) return;
    const placement = this.resolveSymbolPlacement(worldPt, symbolType);

    this.pushUndo('Place Symbol');
    const symbol = {
      x: placement.x,
      y: placement.y,
      symbolType,
      layerId: this.ensureSymbolsLayer().id,
      rotation: placement.rotation,
      scale: 1,
      wallId: placement.wallId,
      wallSnap: placement.wallSnap,
      wallGapWidth: placement.wallGapWidth,
      wallFraction: placement.wallFraction != null ? placement.wallFraction : null,
      wallSideFlip: Boolean(placement.wallSideFlip),
      wallSideExplicit: Boolean(placement.wallId),
      id: this.genId()
    };
    this.state.symbols.push(symbol);
    this.clearSelection(false);
    this.state.selectedSymbolIds.add(symbol.id);
    this.setMode('select');
    if (this.state.sidebar?.pinned === false) this.ui.setSidebarOpen?.(false);
  }

  // --- Measure tool (transient, view-only measurements) ---

  // Snap candidates for measuring, strongest first: wall and area corners,
  // then wall midpoints, then the nearest point along any wall, wall face, or
  // area edge. The kind of snap travels with the point so the cursor can show
  // what it caught.
  measureSnapTarget(worldPt) {
    const tolerance = this.getScreenToleranceWorld(12);
    const corners = [];
    const midpoints = [];
    const edges = [];

    this.state.lines.forEach(line => {
      if (!this.isEntityVisible(line)) return;
      corners.push(line.start, line.end);
      midpoints.push({ x: (line.start.x + line.end.x) / 2, y: (line.start.y + line.end.y) / 2 });
      edges.push([line.start, line.end]);
    });
    this.state.polygons.forEach(poly => {
      if (!this.isEntityVisible(poly)) return;
      const points = poly.points || [];
      points.forEach((point, index) => {
        corners.push(point);
        edges.push([point, points[(index + 1) % points.length]]);
      });
    });
    const solids = this.getWallSolids();
    if (solids) solids.faces.forEach(face => edges.push([face.start, face.end]));

    const nearestPoint = (candidates, type) => {
      let best = null;
      candidates.forEach(candidate => {
        const distance = window.Geometry.distance(worldPt, candidate);
        if (distance <= tolerance && (!best || distance < best.distance)) {
          best = { point: { x: candidate.x, y: candidate.y }, distance, type };
        }
      });
      return best;
    };

    const corner = nearestPoint(corners, 'corner');
    if (corner) return corner;
    const midpoint = nearestPoint(midpoints, 'midpoint');
    if (midpoint) return midpoint;

    let edgeHit = null;
    edges.forEach(([start, end]) => {
      const projection = window.Geometry.projectPointToSegment(worldPt, start, end);
      const distance = window.Geometry.distance(worldPt, projection);
      if (distance <= tolerance && (!edgeHit || distance < edgeHit.distance)) {
        edgeHit = { point: projection, distance, type: 'edge' };
      }
    });
    if (edgeHit) return edgeHit;

    if (this.state.snapEnabled) {
      return { point: window.Geometry.snapToGrid({ ...worldPt }, this.state.gridSize), distance: 0, type: 'grid' };
    }
    return { point: { ...worldPt }, distance: 0, type: 'free' };
  }

  // Where the next measured point would land. Holding the angle lock keeps the
  // leg on a 45-degree ray from the previous point and ignores object snaps,
  // which is what makes a square room read square.
  measureTarget(worldPt, ortho = false) {
    const anchor = this.lastMeasurePoint();
    if (ortho && anchor) {
      return { point: window.Geometry.constrainToAngleStep(anchor, worldPt, 45), distance: 0, type: 'ortho' };
    }
    return this.measureSnapTarget(worldPt);
  }

  lastMeasurePoint() {
    const points = this.state.measureDraft?.points;
    return points && points.length ? points[points.length - 1] : null;
  }

  handleMeasureClick(worldPt, { ortho = false } = {}) {
    if (this.state.mode !== 'measure') return;
    const target = this.measureTarget(worldPt, ortho).point;
    const draft = this.state.measureDraft;
    if (!draft) {
      this.state.measureDraft = { id: this.genId(), points: [{ ...target }], closed: false };
      this.render();
      return;
    }
    // Landing back on the first point closes the run into an area reading
    // rather than adding a zero-length leg on top of it.
    if (draft.points.length >= 3 &&
        window.Geometry.distance(target, draft.points[0]) <= this.getScreenToleranceWorld(10)) {
      this.finishMeasureRun(true);
      return;
    }
    if (window.Geometry.distance(target, this.lastMeasurePoint()) <= 1e-6) return;
    draft.points.push({ ...target });
    this.render();
  }

  // Commits the run in progress. Runs shorter than one leg are dropped.
  finishMeasureRun(closed = false) {
    const draft = this.state.measureDraft;
    const complete = Boolean(draft && draft.points.length >= 2);
    this.state.measureDraft = null;
    if (complete) {
      draft.closed = Boolean(closed) && draft.points.length >= 3;
      this.state.measurements.push(draft);
    }
    this.ui.updateHUD('', '', false);
    this.render();
    return complete;
  }

  cancelMeasureRun() {
    if (!this.state.measureDraft) return false;
    this.state.measureDraft = null;
    this.ui.updateHUD('', '', false);
    this.render();
    return true;
  }

  undoMeasurePoint() {
    const draft = this.state.measureDraft;
    if (!draft) return false;
    draft.points.pop();
    if (!draft.points.length) this.state.measureDraft = null;
    this.render();
    return true;
  }

  clearMeasurements() {
    if (!this.state.measurements.length && !this.state.measureDraft) return false;
    this.state.measurements = [];
    this.state.measureDraft = null;
    this.ui.updateHUD('', '', false);
    this.render();
    return true;
  }

  // Every measurement on screen as plain text, for pasting into notes.
  measureSummaryText() {
    const runs = [...this.state.measurements];
    if (this.state.measureDraft?.points.length >= 2) runs.push(this.state.measureDraft);
    const lines = [];
    runs.forEach((run, index) => {
      const metrics = window.Geometry.polylineMetrics(run.points, run.closed);
      lines.push(`Measurement ${index + 1}${metrics.closed ? ' (closed)' : ''}`);
      metrics.segments.forEach((segment, leg) => {
        lines.push(`  ${leg + 1}. ${this.formatLengthDual(segment.length)} at ${segment.bearing.toFixed(1)}\u00b0`);
      });
      lines.push(`  ${metrics.closed ? 'Perimeter' : 'Total'}: ${this.formatLengthDual(metrics.total)}`);
      if (metrics.closed) lines.push(`  Area: ${this.formatArea(metrics.area)}`);
    });
    return lines.join('\n');
  }

  async copyMeasurements() {
    const text = this.measureSummaryText();
    if (!text) {
      this.ui.notify('Nothing measured yet.');
      return false;
    }
    try {
      await navigator.clipboard.writeText(text);
      this.ui.notify('Measurements copied.');
      return true;
    } catch (_) {
      this.ui.notify('Copy failed. The clipboard is unavailable here.');
      return false;
    }
  }

  fenceSelect(worldMin, worldMax, additive = false) {
    if (!additive) this.clearSelection(false);
    const selectable = entity => !this.isEntityLocked(entity) && this.isEntityVisible(entity);

    this.state.lines.forEach(line => {
      if (!selectable(line)) return;
      if (window.Geometry.pointInRect(line.start, worldMin, worldMax) &&
          window.Geometry.pointInRect(line.end, worldMin, worldMax)) {
        this.state.selectedLineIds.add(line.id);
      }
    });

    this.state.polygons.forEach(poly => {
      if (!selectable(poly)) return;
      const allInside = poly.points.every(pt => 
        window.Geometry.pointInRect(pt, worldMin, worldMax)
      );
      if (allInside) this.state.selectedPolyIds.add(poly.id);
      if (poly.type && poly.type !== 'UND' && window.Geometry.pointInRect(this.getAreaLabelPosition(poly), worldMin, worldMax)) {
        this.state.selectedAreaLabelIds.add(poly.id);
      }
      if (poly.type && poly.type !== 'UND' && window.Geometry.pointInRect(this.getAreaSizePosition(poly), worldMin, worldMax)) {
        this.state.selectedAreaSizeIds.add(poly.id);
      }
    });

    this.state.labels.forEach(lbl => {
      if (!selectable(lbl)) return;
      if (window.Geometry.pointInRect({ x: lbl.x, y: lbl.y }, worldMin, worldMax))
        this.state.selectedLabelIds.add(lbl.id);
    });

    this.state.symbols.forEach(sym => {
      if (!selectable(sym)) return;
      if (window.Geometry.pointInRect({ x: sym.x, y: sym.y }, worldMin, worldMax))
        this.state.selectedSymbolIds.add(sym.id);
    });

    this.state.labels.forEach(label => {
      if (!selectable(label)) return;
      if (window.Geometry.pointInRect(label, worldMin, worldMax)) this.state.selectedLabelIds.add(label.id);
    });

    this.state.lines.forEach(line => {
      if (!selectable(line) || !this.isDimensionVisible(line)) return;
      const position = this.canvas?.getDimensionLabelWorld?.(line);
      if (position && window.Geometry.pointInRect(position, worldMin, worldMax)) {
        this.state.selectedDimensionIds.add(line.id);
      }
    });

    this.render();
  }

  // --- Delete Selected ---

  async deleteSelected() {
    const hasAny = this.state.selectedLineIds.size > 0 || this.state.selectedPolyIds.size > 0 ||
      this.state.selectedAreaLabelIds.size > 0 || this.state.selectedAreaSizeIds.size > 0 ||
      this.state.selectedLabelIds.size > 0 || this.state.selectedSymbolIds.size > 0 ||
      this.state.selectedDimensionIds.size > 0 || this.state.selectedChainIds.size > 0;
    if (!hasAny) return;
    if (this.state.options?.promptOnDeletion && !(await this.ui.confirmDialog('Delete the selected sketch items?'))) return;
    
    this.pushUndo('Delete');

    this.state.dimensionChains = this.state.dimensionChains.filter(chain => !this.state.selectedChainIds.has(chain.id));

    const pointKey = point => `${point.x.toFixed(5)},${point.y.toFixed(5)}`;
    const edgeKey = (a, b) => [pointKey(a), pointKey(b)].sort().join('|');
    const selectedLineIds = new Set(this.state.selectedLineIds);
    const selectedEdges = new Set(
      this.state.lines
        .filter(line => selectedLineIds.has(line.id))
        .map(line => edgeKey(line.start, line.end))
    );
    const selectedAreaLineIds = new Set();
    const selectedAreaEdges = new Set();
    this.state.polygons.forEach(poly => {
      if (!this.state.selectedPolyIds.has(poly.id)) return;
      (poly.edges || []).forEach(edge => {
        if (edge.lineId != null) selectedAreaLineIds.add(edge.lineId);
      });
      for (let i = 0; i < poly.points.length; i++) {
        selectedAreaEdges.add(edgeKey(poly.points[i], poly.points[(i + 1) % poly.points.length]));
      }
    });
    // Which areas go: the selected ones, plus any area whose own wall is
    // explicitly deleted (deleting a boundary wall reopens its area) or that
    // runs along an explicitly deleted wall. An area is never removed merely
    // because a *deleted area's* outline coincides with it: deleting a second
    // floor drawn over the first floor used to wipe out the first floor too.
    const removedPolys = new Set(this.state.polygons.filter(poly => {
      if (this.state.selectedPolyIds.has(poly.id)) return true;
      if ((poly.edges || []).some(edge => selectedLineIds.has(edge.lineId))) return true;
      for (let i = 0; i < poly.points.length; i++) {
        if (selectedEdges.has(edgeKey(poly.points[i], poly.points[(i + 1) % poly.points.length]))) return true;
      }
      return false;
    }));
    // Layers of the areas about to vanish, so their dedicated layers can be
    // removed too once nothing is left on them.
    const removedAreaLayerIds = new Set([...removedPolys].map(poly => this.getEntityLayerId(poly)));
    // Walls still used by a surviving area are kept even when a deleted area
    // also ran along them.
    const keptLineIds = new Set();
    this.state.polygons.forEach(poly => {
      if (removedPolys.has(poly)) return;
      (poly.edges || []).forEach(edge => { if (edge.lineId != null) keptLineIds.add(edge.lineId); });
    });

    // Deleting a boundary wall reopens its calculated area. Deleting an area
    // removes its associated boundary walls as documented by the desktop app.
    this.state.lines = this.state.lines.filter(line => {
      if (selectedLineIds.has(line.id)) return false;
      if (keptLineIds.has(line.id)) return true;
      return !selectedAreaLineIds.has(line.id) && !selectedAreaEdges.has(edgeKey(line.start, line.end));
    });
    this.state.polygons = this.state.polygons.filter(poly => !removedPolys.has(poly));
    this.state.labels = this.state.labels.filter(l => !this.state.selectedLabelIds.has(l.id));
    this.state.symbols = this.state.symbols.filter(s => !this.state.selectedSymbolIds.has(s.id));
    // Deleting a measurement label only hides it; the wall itself stays put.
    // Select the wall and press Space to bring the label back.
    this.state.lines.forEach(line => {
      if (this.state.selectedDimensionIds.has(line.id)) {
        line.dimensionHidden = true;
        delete line.dimensionPosition;
        delete line.dimensionAnchor;
      }
    });
    this.clearSelection(false);
    // Deleting an area also deletes its dedicated layer once it is empty.
    removedAreaLayerIds.forEach(id => this.removeEmptyAreaLayer(id));
    this.ui.updateLayersPanel?.();
    this.recalcAutoSubtract();
    this.ui.updateCalcsPanel();
    this.render();
  }

  // --- Move Selected ---

  // Bounding box of everything currently selected, used as the reference
  // point for absolute grid snapping while dragging areas.
  getSelectionBounds() {
    const s = this.state;
    const pts = [];
    s.lines.forEach(line => {
      if (s.selectedLineIds.has(line.id)) { pts.push(line.start, line.end); }
      else if (s.selectedDimensionIds.has(line.id) && line.dimensionPosition) {
        pts.push(window.Geometry.pinnedDimensionPoint(line));
      }
    });
    s.polygons.forEach(poly => {
      if (s.selectedPolyIds.has(poly.id)) {
        (poly.points || []).forEach(pt => pts.push(pt));
      }
      if (s.selectedAreaLabelIds.has(poly.id) && poly.labelPosition) pts.push(poly.labelPosition);
      if (s.selectedAreaSizeIds.has(poly.id) && poly.areaSizePosition) pts.push(poly.areaSizePosition);
    });
    s.labels.forEach(lbl => { if (s.selectedLabelIds.has(lbl.id)) pts.push({ x: lbl.x, y: lbl.y }); });
    s.symbols.forEach(sym => { if (s.selectedSymbolIds.has(sym.id)) pts.push({ x: sym.x, y: sym.y }); });
    if (!pts.length) return null;
    const xs = pts.map(pt => pt.x);
    const ys = pts.map(pt => pt.y);
    return {
      x: Math.min(...xs),
      y: Math.min(...ys),
      w: Math.max(...xs) - Math.min(...xs),
      h: Math.max(...ys) - Math.min(...ys)
    };
  }

  moveSelected(dx, dy, recordHistory = true) {
    const hasAny = this.state.selectedLineIds.size > 0 || this.state.selectedPolyIds.size > 0 ||
      this.state.selectedAreaLabelIds.size > 0 || this.state.selectedAreaSizeIds.size > 0 ||
      this.state.selectedLabelIds.size > 0 || this.state.selectedSymbolIds.size > 0 ||
      this.state.selectedDimensionIds.size > 0;
    if (!hasAny) return;
    
    if (recordHistory) this.pushUndo('Move');

    const pointKey = point => `${point.x.toFixed(5)},${point.y.toFixed(5)}`;
    const selectionHasAreas = this.state.selectedPolyIds.size > 0;

    // Moving whole areas: a wall the moving area shares with an area that
    // stays put (or only partly reuses) is detached first - the moving area
    // takes its own copy. Otherwise the neighbour was dragged out of shape
    // and its old dimensions stayed behind where its wall used to be.
    if (selectionHasAreas) {
      const stayingLineIds = new Set();
      this.state.polygons.forEach(poly => {
        if (this.state.selectedPolyIds.has(poly.id)) return;
        (poly.edges || []).forEach(edge => { if (edge.lineId != null) stayingLineIds.add(edge.lineId); });
      });
      this.state.polygons.forEach(poly => {
        if (!this.state.selectedPolyIds.has(poly.id)) return;
        (poly.edges || []).forEach((edge, index) => {
          if (edge.lineId == null || this.state.selectedLineIds.has(edge.lineId)) return;
          if (stayingLineIds.has(edge.lineId) || this.edgeBorrowsPartOfWall(poly, index)) {
            this.ownBoundaryLine(poly, index);
          }
        });
      });
    }
    const linesById = new Map(this.state.lines.map(line => [line.id, line]));

    // Walls that belong to the selection: explicitly selected lines plus the
    // boundary walls of the selected areas. Only these walls move with an
    // area; coincidental vertices of other areas must not drag their walls.
    const movingLineIds = new Set(this.state.selectedLineIds);
    this.state.polygons.forEach(poly => {
      if (!this.state.selectedPolyIds.has(poly.id)) return;
      (poly.edges || []).forEach(edge => {
        if (edge.lineId != null) movingLineIds.add(edge.lineId);
      });
    });

    const movingPoints = new Set();
    this.state.lines.forEach(line => {
      if (this.state.selectedLineIds.has(line.id)) {
        movingPoints.add(pointKey(line.start));
        movingPoints.add(pointKey(line.end));
      }
    });
    this.state.polygons.forEach(poly => {
      if (this.state.selectedPolyIds.has(poly.id)) {
        poly.points.forEach(point => movingPoints.add(pointKey(point)));
      }
    });

    // Non-selected areas follow only walls they genuinely share with the
    // selection, so an adjacent area stays joined to the moved boundary. A
    // mere coincident vertex is left alone: a subtraction area drawn against
    // another area's corner can move without grabbing the other area.
    const gluePointsByPoly = new Map();
    if (selectionHasAreas) {
      this.state.polygons.forEach(poly => {
        if (this.state.selectedPolyIds.has(poly.id)) return;
        const gluePoints = new Set();
        (poly.edges || []).forEach(edge => {
          if (!movingLineIds.has(edge.lineId)) return;
          const line = linesById.get(edge.lineId);
          if (line) {
            gluePoints.add(pointKey(line.start));
            gluePoints.add(pointKey(line.end));
          }
        });
        if (gluePoints.size) gluePointsByPoly.set(poly.id, gluePoints);
      });
    }

    // Moving a wall also moves the matching endpoints of its connected walls,
    // so the adjoining geometry stretches rather than becoming disconnected.
    // Area moves are the exception: only the area's own boundary walls move.
    this.state.lines.forEach(line => {
      if (movingLineIds.has(line.id)) {
        line.start.x += dx; line.start.y += dy;
        line.end.x += dx; line.end.y += dy;
        return;
      }
      if (selectionHasAreas) return;
      // A connected (non-selected) wall stretches: only the endpoint shared
      // with the selection follows; its far end stays anchored.
      if (movingPoints.has(pointKey(line.start))) { line.start.x += dx; line.start.y += dy; }
      if (movingPoints.has(pointKey(line.end))) { line.end.x += dx; line.end.y += dy; }
    });
    this.state.polygons.forEach(poly => {
      const moveWithArea = this.state.selectedPolyIds.has(poly.id);
      const gluePoints = gluePointsByPoly.get(poly.id);
      poly.points.forEach(point => {
        if (moveWithArea || (gluePoints && gluePoints.has(pointKey(point))) ||
            (!selectionHasAreas && movingPoints.has(pointKey(point)))) {
          point.x += dx; point.y += dy;
        }
      });
      const moveLabelOnly = this.state.selectedAreaLabelIds.has(poly.id) && !moveWithArea;
      const moveSizeOnly = this.state.selectedAreaSizeIds.has(poly.id) && !moveWithArea;
      if (moveWithArea) {
        if (poly.labelPosition) {
          poly.labelPosition.x += dx;
          poly.labelPosition.y += dy;
        }
        if (poly.areaSizePosition) {
          poly.areaSizePosition.x += dx;
          poly.areaSizePosition.y += dy;
        }
      } else {
        const labelPosition = this.getAreaLabelPosition(poly);
        const sizePosition = this.getAreaSizePosition(poly);
        if (moveLabelOnly) {
          // The name and its area figure travel as one block;
          // freezing the figure left "1200.0 sf" behind wherever the name was
          // dragged from. A figure placed on its own keeps its offset.
          poly.labelPosition = { x: labelPosition.x + dx, y: labelPosition.y + dy };
          if (poly.areaSizePosition && !moveSizeOnly) {
            poly.areaSizePosition = { x: poly.areaSizePosition.x + dx, y: poly.areaSizePosition.y + dy };
          }
        }
        if (moveSizeOnly) {
          poly.areaSizePosition = { x: sizePosition.x + dx, y: sizePosition.y + dy };
        }
      }
    });
    this.state.labels.forEach(lbl => {
      if (this.state.selectedLabelIds.has(lbl.id)) {
        lbl.x += dx; lbl.y += dy;
      }
    });
    this.state.symbols.forEach(sym => {
      if (this.state.selectedSymbolIds.has(sym.id)) {
        sym.x += dx; sym.y += dy;
      }
    });
    // Hosted openings ride along with their walls as the selection moves.
    this.refreshHostedSymbols();
    // Selected measurement labels detach from their wall on the first move.
    this.state.lines.forEach(line => {
      if (!this.state.selectedDimensionIds.has(line.id)) return;
      // Re-pin from where the label draws now, against the wall's current
      // midpoint, so it follows the wall afterwards.
      const current = window.Geometry.pinnedDimensionPoint(line) || this.canvas?.getDimensionLabelWorld?.(line) || null;
      if (current) {
        line.dimensionPosition = { x: current.x + dx, y: current.y + dy };
        line.dimensionAnchor = { x: (line.start.x + line.end.x) / 2, y: (line.start.y + line.end.y) / 2 };
      }
    });

    this.recalcAutoSubtract();
    this.ui.updateCalcsPanel();
    this.render();
  }

  // --- Cut / Copy / Paste ---

  copySelected() {
    const polys = this.state.polygons.filter(p => this.state.selectedPolyIds.has(p.id));
    const boundaryIds = new Set(polys.flatMap(poly => (poly.edges || []).map(edge => edge.lineId)));
    const lines = this.state.lines.filter(line => this.state.selectedLineIds.has(line.id) || boundaryIds.has(line.id));
    const labels = this.state.labels.filter(l => this.state.selectedLabelIds.has(l.id));
    const symbols = this.state.symbols.filter(s => this.state.selectedSymbolIds.has(s.id));
    if (lines.length === 0 && polys.length === 0 && labels.length === 0 && symbols.length === 0) return;

    this.state.clipboard = {
      lines: JSON.parse(JSON.stringify(lines)),
      polygons: JSON.parse(JSON.stringify(polys)),
      labels: JSON.parse(JSON.stringify(labels)),
      symbols: JSON.parse(JSON.stringify(symbols))
    };
  }

  cutSelected() {
    this.copySelected();
    this.deleteSelected();
  }

  paste() {
    if (!this.state.clipboard) return;
    
    this.pushUndo('Paste');

    // Offset pasted items slightly so they don't overlap originals
    const offset = 2;

    this.clearSelection(false);
    const lineIdMap = new Map();
    const polygonIdMap = new Map();
    (this.state.clipboard.lines || []).forEach(line => {
      const newId = this.genId();
      lineIdMap.set(line.id, newId);
      const newLine = {
        ...JSON.parse(JSON.stringify(line)),
        start: { x: line.start.x + offset, y: line.start.y + offset },
        end: { x: line.end.x + offset, y: line.end.y + offset },
        id: newId
      };
      this.state.lines.push(newLine);
      this.state.selectedLineIds.add(newLine.id);
    });

    (this.state.clipboard.polygons || []).forEach(poly => polygonIdMap.set(poly.id, this.genId()));
    (this.state.clipboard.polygons || []).forEach(poly => {
      const points = poly.points.map(pt => ({ x: pt.x + offset, y: pt.y + offset }));
      const newPoly = {
        ...JSON.parse(JSON.stringify(poly)),
        points,
        edges: (poly.edges || []).map(edge => ({ ...edge, lineId: lineIdMap.get(edge.lineId) ?? null })),
        containerId: null,
        subtractedFromId: null,
        subtractTargetId: polygonIdMap.get(poly.subtractTargetId) ??
          (this.state.polygons.some(item => item.id === poly.subtractTargetId) ? poly.subtractTargetId : null),
        id: polygonIdMap.get(poly.id)
      };
      if (newPoly.labelPosition) {
        newPoly.labelPosition.x += offset;
        newPoly.labelPosition.y += offset;
      }
      if (newPoly.areaSizePosition) {
        newPoly.areaSizePosition.x += offset;
        newPoly.areaSizePosition.y += offset;
      }
      if (newPoly.edges.length !== points.length || newPoly.edges.some(edge => edge.lineId == null)) {
        newPoly.edges = this.buildPolygonEdges(newPoly, this.state.lines);
      }
      this.state.polygons.push(newPoly);
      this.assignAreaToDedicatedLayer(newPoly);
      this.state.selectedPolyIds.add(newPoly.id);
    });

    (this.state.clipboard.labels || []).forEach(label => {
      const copy = { ...JSON.parse(JSON.stringify(label)), x: label.x + offset, y: label.y + offset, id: this.genId() };
      this.state.labels.push(copy);
      this.state.selectedLabelIds.add(copy.id);
    });
    (this.state.clipboard.symbols || []).forEach(symbol => {
      const copy = { ...JSON.parse(JSON.stringify(symbol)), x: symbol.x + offset, y: symbol.y + offset, id: this.genId() };
      this.state.symbols.push(copy);
      this.state.selectedSymbolIds.add(copy.id);
    });

    this.recalcAutoSubtract();
    this.ui.updateCalcsPanel();
    this.render();
  }

  // Clones the selected entities exactly in place, leaving the copies
  // selected so they can be dragged away. Boundary walls of cloned areas
  // are cloned too, and copied symbols detach from their wall like rotated
  // ones do.
  cloneSelected() {
    const s = this.state;
    const hasAny = s.selectedLineIds.size > 0 || s.selectedPolyIds.size > 0 ||
      s.selectedLabelIds.size > 0 || s.selectedSymbolIds.size > 0;
    if (!hasAny) return;
    this.pushUndo('Clone');

    const lineIdMap = new Map();
    const polygonIdMap = new Map();
    const cloneLines = s.lines.filter(line => s.selectedLineIds.has(line.id));
    const clonePolys = s.polygons.filter(poly => s.selectedPolyIds.has(poly.id));
    const boundaryLineIds = new Set(clonePolys.flatMap(poly => (poly.edges || []).map(edge => edge.lineId).filter(id => id != null)));
    const boundaryLines = s.lines.filter(line => boundaryLineIds.has(line.id) && !s.selectedLineIds.has(line.id));

    this.clearSelection(false);

    [...cloneLines, ...boundaryLines].forEach(line => {
      const newId = this.genId();
      lineIdMap.set(line.id, newId);
      const copy = { ...JSON.parse(JSON.stringify(line)), id: newId };
      s.lines.push(copy);
      s.selectedLineIds.add(newId);
    });

    clonePolys.forEach(poly => polygonIdMap.set(poly.id, this.genId()));
    clonePolys.forEach(poly => {
      const newPoly = {
        ...JSON.parse(JSON.stringify(poly)),
        edges: (poly.edges || []).map(edge => ({ ...edge, lineId: lineIdMap.get(edge.lineId) ?? null })),
        containerId: null,
        subtractedFromId: null,
        subtractTargetId: polygonIdMap.get(poly.subtractTargetId) ?? null,
        id: polygonIdMap.get(poly.id)
      };
      if (newPoly.edges.length !== newPoly.points.length || newPoly.edges.some(edge => edge.lineId == null)) {
        newPoly.edges = this.buildPolygonEdges(newPoly, s.lines);
      }
      s.polygons.push(newPoly);
      this.assignAreaToDedicatedLayer(newPoly);
      s.selectedPolyIds.add(newPoly.id);
    });

    s.labels.filter(label => s.selectedLabelIds.has(label.id)).forEach(label => {
      const copy = { ...JSON.parse(JSON.stringify(label)), id: this.genId() };
      s.labels.push(copy);
      s.selectedLabelIds.add(copy.id);
    });
    s.symbols.filter(symbol => s.selectedSymbolIds.has(symbol.id)).forEach(symbol => {
      const copy = { ...JSON.parse(JSON.stringify(symbol)), id: this.genId(), wallId: null };
      s.symbols.push(copy);
      s.selectedSymbolIds.add(copy.id);
    });

    this.recalcAutoSubtract();
    this.ui.updateCalcsPanel();
    this.ui.notify(`Cloned ${clonePolys.length ? `${clonePolys.length} area${clonePolys.length === 1 ? '' : 's'}` : 'selection'}.`);
    this.render();
  }

  // --- Mouse-Click Drawing ---

  handleCanvasClick(worldPt) {
    this.state.pointerWorld = { ...worldPt };
    let targetPt = this.getDrawingTarget(worldPt);

    if (!this.state.penDown) {
      if (this.state.defineFirst && this.state.mode === 'draw-exterior' && !this.state.selectedAreaType) {
        this.ui.showPanel('panel-define-area');
        this.ui.notify('Define First is enabled. Choose an area type before drawing.');
        return;
      }
      this.penDown(targetPt);
      return;
    }

    // Closing is a structural snap and remains available even when general
    // snapping is off. It uses a screen-space tolerance so zoom does not
    // change how hard it is to hit the starting point.
    if (this.state.currentPolyPoints.length >= 3) {
      const startPt = this.state.currentPolyPoints[0];
      if (window.Geometry.distance(targetPt, startPt) < 0.0001) {
        this.autoClose();
        return;
      }
    }

    const dist = window.Geometry.distance(this.state.cursor, targetPt);
    if (dist < 0.1) return;

    this.pushUndo('Draw');

    const lineId = this.genId();
    const lineStyle = this.getDefaultLineStyle(this.state.mode === 'draw-interior' ? 'interior' : 'exterior');
    this.state.lines.push({
      start: { ...this.state.cursor },
      end: { ...targetPt },
      type: this.state.mode === 'draw-interior' ? 'interior' : 'exterior',
      color: lineStyle.color || this.state.drawColor,
      lineThickness: lineStyle.lineThickness,
      dimensionHidden: this.state.suppressDimensionNext,
      layerId: this.state.activeLayerId,
      id: lineId
    });
    this.state.suppressDimensionNext = false;
    this.state.currentLineIds.push(lineId);
    this.state.currentPolyPoints.push({ ...targetPt });
    this.state.cursor = { ...targetPt };

    const closedAreas = this.finishCommittedWall(lineId);
    if (closedAreas) return;
    this.ui.updateHUD(dist.toFixed(1), '', true);
    this.render();
  }

  // --- Keyboard-Driven Drawing ---

  // Trig-based commands (<angle, l/r turns, bare lengths) leave ~1e-15 residue
  // that splits a shared corner into two point keys and blocks loop closing.
  // Snap an endpoint coordinate to the display grid so chained walls meet
  // exactly and areas can close.
  cleanCoord(value) {
    // toFixed drops the binary residue of the multiplication itself
    // (24.1422 * 1e-4 steps can come back as 24.142200000000003).
    const rounded = Number((Math.round(value / 1e-4) * 1e-4).toFixed(4));
    return rounded === 0 ? 0 : rounded;
  }

  drawLine(newPos, dist, angle, bowAngle) {
    newPos = { x: this.cleanCoord(newPos.x), y: this.cleanCoord(newPos.y) };
    if (this.state.penDown && dist > 0) {
      this.pushUndo('Draw Wall');
      const lineStyle = this.getDefaultLineStyle(this.state.mode === 'draw-interior' ? 'interior' : 'exterior');
      const line = {
        start: { ...this.state.cursor },
        end: { ...newPos },
        type: this.state.mode === 'draw-interior' ? 'interior' : 'exterior',
        color: lineStyle.color || this.state.drawColor,
        lineThickness: lineStyle.lineThickness,
        dimensionHidden: this.state.suppressDimensionNext,
        layerId: this.state.activeLayerId,
        id: this.genId()
      };
      this.state.suppressDimensionNext = false;
      // If bowed, store the arc angle
      if (bowAngle && bowAngle !== 0) {
        line.bowAngle = window.Geometry.clampBowAngle(bowAngle);
        const chord = window.Geometry.distance(line.start, line.end);
        line.arcHeight = chord / 2 * Math.tan(line.bowAngle * Math.PI / 720);
      }
      this.state.lines.push(line);
      this.state.currentLineIds.push(line.id);
      this.state.currentPolyPoints.push({ ...newPos });
      this.state.cursor = { ...newPos };
      if (this.finishCommittedWall(line.id)) return;
    }
    this.state.cursor = newPos;
    this.ui.updateHUD(dist ? dist.toFixed(1) : null, angle, this.state.penDown);
    this.render();
  }

  // --- Bay Window Mirror ---
  // After drawing an angled wall, B mirrors the last wall across
  // the perpendicular axis of the wall before it, creating the
  // second half of a bay window.
  bayWindowMirror() {
    if (this.state.lines.length < 2) return;
    if (!this.state.penDown) return;

    const lastLine = this.state.lines[this.state.lines.length - 1];
    const prevLine = this.state.lines[this.state.lines.length - 2];

    // Direction of the previous line (the base wall)
    const baseAngle = Math.atan2(
      prevLine.end.y - prevLine.start.y,
      prevLine.end.x - prevLine.start.x
    );

    // Direction of the last line (the angled bay)
    const bayDx = lastLine.end.x - lastLine.start.x;
    const bayDy = lastLine.end.y - lastLine.start.y;
    const bayAngle = Math.atan2(bayDy, bayDx);
    const bayLen = Math.hypot(bayDx, bayDy);

    // Mirror the bay angle across the base angle
    const mirrorAngle = 2 * baseAngle - bayAngle;

    const newEnd = {
      x: this.state.cursor.x + bayLen * Math.cos(mirrorAngle),
      y: this.state.cursor.y + bayLen * Math.sin(mirrorAngle)
    };

    this.pushUndo('Bay Mirror');
    const lineId = this.genId();
    this.state.lines.push({
      start: { ...this.state.cursor },
      end: { ...newEnd },
      type: lastLine.type,
      color: lastLine.color || this.state.drawColor,
      layerId: lastLine.layerId || this.state.activeLayerId,
      id: lineId
    });
    this.state.currentLineIds.push(lineId);
    this.state.currentPolyPoints.push({ ...newEnd });
    this.state.cursor = { ...newEnd };

    const angleDeg = Math.round(mirrorAngle * 180 / Math.PI);
    this.ui.updateHUD(bayLen.toFixed(1), angleDeg, true);
    this.render();
  }

  // --- Auto-Close Polygon ---

  autoClose() {
    if (this.state.currentPolyPoints.length < 3) return;

    // Closing the chain is one undoable action, separate from the previously
    // committed wall.
    this.pushUndo('Auto Close');
    
    const startPt = this.state.currentPolyPoints[0];
    const dist = window.Geometry.distance(this.state.cursor, startPt);
    
    if (dist > 0.1) {
      const closingId = this.genId();
      const lineStyle = this.getDefaultLineStyle(this.state.mode === 'draw-interior' ? 'interior' : 'exterior');
      this.state.lines.push({
        start: { ...this.state.cursor },
        end: { ...startPt },
        type: this.state.mode === 'draw-interior' ? 'interior' : 'exterior',
        color: lineStyle.color || this.state.drawColor,
        lineThickness: lineStyle.lineThickness,
        dimensionHidden: this.state.suppressDimensionNext,
        layerId: this.state.activeLayerId,
        id: closingId
      });
      this.state.suppressDimensionNext = false;
      this.state.currentLineIds.push(closingId);
    }
    
    this.state.cursor = { ...startPt };
    this.state.penDown = false;
    this.state.keyboardPointer = false;
    this.state.keyboardPointerAnchor = null;
    this.canvas.setCursor();

    // Exterior walls calculate enclosed areas. Interior walls are illustration
    // lines and may form a loop without creating an appraisal area.
    if (this.state.mode === 'draw-exterior') {
      const edges = this.state.currentLineIds.map((lineId, index) => {
        const line = this.state.lines.find(item => item.id === lineId);
        const point = this.state.currentPolyPoints[index];
        const followsLine = line && this.pointsEqual(line.start, point);
        return { lineId, bowAngle: line ? (followsLine ? (line.bowAngle || 0) : -(line.bowAngle || 0)) : 0 };
      });
      const area = window.Geometry.calculateArea(this.state.currentPolyPoints, edges);
      if (area >= this.minimumClosedArea()) {
        this.state.polygons.push({
          points: this.state.currentPolyPoints.map(pt => ({ ...pt })),
          edges,
          type: this.state.defineFirst ? this.state.selectedAreaType : 'UND',
          name: this.state.defineFirst ? (this.state.selectedAreaName || this.state.selectedAreaType) : 'Undefined Area',
          gross: area,
          area,
          factorOverride: null,
          subtractMode: this.state.options?.autoSubtractOverlapped === false ? 'never' : 'auto',
          subtractTargetId: null,
          layerId: this.state.activeLayerId,
          id: this.genId()
        });
        // New areas get their own dedicated layer (fill + boundary walls).
        this.assignAreaToDedicatedLayer(this.state.polygons[this.state.polygons.length - 1], { activate: true });
      }
    }
    
    this.state.currentPolyPoints = [];
    this.state.currentLineIds = [];
    this.ui.updateHUD('', '', false);
    this.recalcAutoSubtract();
    this.render();
    this.ui.updateCalcsPanel();
  }

  toggleLastDimension() {
    const activeId = this.state.currentLineIds[this.state.currentLineIds.length - 1];
    const line = this.state.lines.find(item => item.id === activeId);
    if (line) {
      this.pushUndo('Toggle Dimension');
      line.dimensionHidden = !line.dimensionHidden;
      this.ui.notify(line.dimensionHidden ? 'Dimension suppressed for the last wall.' : 'Dimension restored.');
    } else {
      this.state.suppressDimensionNext = !this.state.suppressDimensionNext;
      this.ui.notify(this.state.suppressDimensionNext ? 'The next wall dimension will be suppressed.' : 'Next wall dimension restored.');
    }
    this.render();
  }

  insertVertexAt(worldPt) {
    const tolerance = this.getScreenToleranceWorld(10);
    let best = null;
    let bestDistance = tolerance;
    this.state.lines.forEach((line, index) => {
      const projection = window.Geometry.projectPointToSegment(worldPt, line.start, line.end);
      const distance = window.Geometry.distance(worldPt, projection);
      if (distance < bestDistance) {
        best = { line, index, projection };
        bestDistance = distance;
      }
    });
    if (!best) { this.ui.notify('Click directly on a wall to insert a vertex.'); return false; }
    if (window.Geometry.distance(best.projection, best.line.start) < 0.05 ||
        window.Geometry.distance(best.projection, best.line.end) < 0.05) {
      this.ui.notify('That point is already a vertex.');
      return false;
    }
    if (Math.abs(best.line.bowAngle || 0) > 0.5) {
      this.ui.notify('Straighten the curved wall before inserting a vertex.');
      return false;
    }

    this.pushUndo('Insert Vertex');
    const originalEnd = { ...best.line.end };
    // Each new wall gets its own automatic dimension; a pinned label must not
    // be copied onto both halves.
    delete best.line.dimensionPosition;
    delete best.line.dimensionAnchor;
    best.line.end = { ...best.projection };
    const second = {
      ...JSON.parse(JSON.stringify(best.line)),
      start: { ...best.projection },
      end: originalEnd,
      id: this.genId()
    };
    this.state.lines.splice(best.index + 1, 0, second);

    this.state.polygons.forEach(poly => {
      for (let i = 0; i < poly.points.length; i++) {
        const nextIndex = (i + 1) % poly.points.length;
        const a = poly.points[i];
        const b = poly.points[nextIndex];
        const followsLine = this.pointsEqual(a, best.line.start) && this.pointsEqual(b, originalEnd);
        const reversesLine = this.pointsEqual(a, originalEnd) && this.pointsEqual(b, best.line.start);
        const sameEdge = followsLine || reversesLine;
        if (!sameEdge) continue;
        poly.points.splice(i + 1, 0, { ...best.projection });
        if (!Array.isArray(poly.edges)) poly.edges = this.buildPolygonEdges(poly, this.state.lines);
        const splitEdges = followsLine
          ? [{ lineId: best.line.id, bowAngle: 0 }, { lineId: second.id, bowAngle: 0 }]
          : [{ lineId: second.id, bowAngle: 0 }, { lineId: best.line.id, bowAngle: 0 }];
        poly.edges.splice(i, 1, ...splitEdges);
        break;
      }
    });
    this.recalcAutoSubtract();
    this.ui.updateCalcsPanel();
    this.render();
    return true;
  }

  pointsEqual(a, b, tolerance = 0.0001) {
    return Math.abs(a.x - b.x) <= tolerance && Math.abs(a.y - b.y) <= tolerance;
  }

  // Area values are displayed at much coarser precision than coordinates.
  // Reject sub-hundredth-square-foot rings so floating-point residue cannot
  // create a protected UND sliver that blocks later endpoint joins.
  minimumClosedArea() {
    return 0.01;
  }

  // Cyclic polygon comparison used when a newly valid wall loop repairs an
  // older polygon whose boundary metadata is missing or stale. Accept either
  // winding and any starting vertex while keeping the tolerance tight enough
  // not to merge neighboring appraisal areas.
  polygonRingsEqual(first, second, tolerance = 0.02) {
    if (!Array.isArray(first) || !Array.isArray(second) || first.length < 3 || second.length < 3) {
      return false;
    }
    if (first.length === second.length) {
      const count = first.length;
      for (let offset = 0; offset < count; offset++) {
        if (!this.pointsEqual(first[0], second[offset], tolerance)) continue;
        let forward = true;
        let reverse = true;
        for (let index = 1; index < count && (forward || reverse); index++) {
          if (forward && !this.pointsEqual(first[index], second[(offset + index) % count], tolerance)) forward = false;
          if (reverse && !this.pointsEqual(first[index], second[(offset - index + count) % count], tolerance)) reverse = false;
        }
        if (forward || reverse) return true;
      }
    }

    // A T-junction can split one straight boundary wall and add a collinear
    // vertex without changing the area geometry. Compare both rings against
    // the other's segments so that stale four-corner metadata can be repaired
    // by a live five-edge boundary instead of being duplicated.
    const liesOnRing = (point, ring) => ring.some((start, index) =>
      window.Geometry.distanceToSegment(point, start, ring[(index + 1) % ring.length]) <= tolerance);
    const followsBoundary = (source, target) => source.every((start, index) => {
      const end = source[(index + 1) % source.length];
      const midpoint = { x: (start.x + end.x) / 2, y: (start.y + end.y) / 2 };
      return liesOnRing(start, target) && liesOnRing(midpoint, target);
    });
    return followsBoundary(first, second) && followsBoundary(second, first);
  }

  lassoSelect(points, additive = false) {
    if (!points || points.length < 3) return;
    if (!additive) this.clearSelection(false);
    const selectable = entity => !this.isEntityLocked(entity) && this.isEntityVisible(entity);
    this.state.lines.forEach(line => {
      if (!selectable(line)) return;
      if (window.Geometry.pointInPolygon(line.start, points) && window.Geometry.pointInPolygon(line.end, points)) {
        this.state.selectedLineIds.add(line.id);
      }
    });
    this.state.polygons.forEach(poly => {
      if (!selectable(poly)) return;
      if (poly.points.every(point => window.Geometry.pointInPolygon(point, points))) this.state.selectedPolyIds.add(poly.id);
      if (poly.type && poly.type !== 'UND' && window.Geometry.pointInPolygon(this.getAreaLabelPosition(poly), points)) {
        this.state.selectedAreaLabelIds.add(poly.id);
      }
      if (poly.type && poly.type !== 'UND' && window.Geometry.pointInPolygon(this.getAreaSizePosition(poly), points)) {
        this.state.selectedAreaSizeIds.add(poly.id);
      }
    });
    this.state.labels.forEach(label => {
      if (!selectable(label)) return;
      if (window.Geometry.pointInPolygon(label, points)) this.state.selectedLabelIds.add(label.id);
    });
    this.state.symbols.forEach(symbol => {
      if (!selectable(symbol)) return;
      if (window.Geometry.pointInPolygon(symbol, points)) this.state.selectedSymbolIds.add(symbol.id);
    });
    this.state.lines.forEach(line => {
      if (!selectable(line) || !this.isDimensionVisible(line)) return;
      const position = this.canvas?.getDimensionLabelWorld?.(line);
      if (position && window.Geometry.pointInPolygon(position, points)) {
        this.state.selectedDimensionIds.add(line.id);
      }
    });
    this.render();
  }

  cloneSelectedAreas() {
    const selected = this.state.polygons.filter(poly => this.state.selectedPolyIds.has(poly.id));
    if (selected.length === 0) { this.ui.notify('Select one or more areas to clone.'); return; }
    this.pushUndo('Clone Area');
    this.clearSelection(false);
    const polygonIdMap = new Map(selected.map(poly => [poly.id, this.genId()]));
    selected.forEach(poly => {
      const points = poly.points.map(point => ({ x: point.x + 2, y: point.y + 2 }));
      const edges = [];
      for (let i = 0; i < points.length; i++) {
        const sourceEdge = poly.edges && poly.edges[i] || { bowAngle: 0 };
        const sourceLine = this.state.lines.find(line => line.id === sourceEdge.lineId);
        const lineId = this.genId();
        const line = {
          ...(sourceLine ? JSON.parse(JSON.stringify(sourceLine)) : {}),
          layerId: sourceLine && sourceLine.layerId ? sourceLine.layerId : this.state.activeLayerId,
          start: { ...points[i] },
          end: { ...points[(i + 1) % points.length] },
          type: sourceLine && sourceLine.type || 'exterior',
          color: sourceLine && sourceLine.color || this.state.drawColor,
          bowAngle: Number(sourceEdge.bowAngle) || 0,
          id: lineId
        };
        if (line.bowAngle) {
          const chord = window.Geometry.distance(line.start, line.end);
          line.arcHeight = chord / 2 * Math.tan(line.bowAngle * Math.PI / 720);
        }
        this.state.lines.push(line);
        edges.push({ lineId, bowAngle: line.bowAngle || 0 });
      }
      const clone = {
        ...JSON.parse(JSON.stringify(poly)),
        points,
        edges,
        containerId: null,
        subtractedFromId: null,
        subtractTargetId: polygonIdMap.get(poly.subtractTargetId) ??
          (this.state.polygons.some(item => item.id === poly.subtractTargetId) ? poly.subtractTargetId : null),
        id: polygonIdMap.get(poly.id)
      };
      if (clone.labelPosition) {
        clone.labelPosition.x += 2;
        clone.labelPosition.y += 2;
      }
      if (clone.areaSizePosition) {
        clone.areaSizePosition.x += 2;
        clone.areaSizePosition.y += 2;
      }
      this.state.polygons.push(clone);
      this.assignAreaToDedicatedLayer(clone);
      this.state.selectedPolyIds.add(clone.id);
    });
    this.recalcAutoSubtract();
    this.ui.updateCalcsPanel();
    this.render();
  }

  defineAreaAt(worldPt, code, name) {
    const target = this.regionAreaAt(worldPt).poly;
    if (!target) { this.ui.notify('Drop the area definition inside a closed shape.'); return; }
    this.defineAreaTarget(target, code, name);
  }

  // The area a define click means: the tightest region the exterior walls
  // enclose around the point. An exterior divider drawn across an
  // area makes each side definable on its own. Read-only: returns the
  // existing area to use ({ poly }) or the wall face to materialize
  // ({ face }); the hover preview and the click both resolve through here.
  resolveRegionAt(worldPt) {
    const visible = this.state.polygons.filter(poly => this.isEntityVisible(poly) &&
      window.Geometry.pointInPolygon(worldPt, window.Geometry.flattenRing(poly)));
    const byArea = poly => Math.abs(poly.area ?? poly.gross ?? 0);
    const smallest = visible.slice().sort((a, b) => byArea(a) - byArea(b))[0] || null;
    const exterior = this.state.lines.filter(line => line.type === 'exterior' && this.isEntityVisible(line) &&
      window.Geometry.distance(line.start, line.end) > 1e-6);
    // Pointer hover asks on every move; reuse faces until a wall changes.
    const key = exterior.map(line => `${line.id}:${line.start.x},${line.start.y},${line.end.x},${line.end.y}`).join('|');
    if (!this.regionFaceCache || this.regionFaceCache.key !== key) {
      this.regionFaceCache = { key, faces: window.Geometry.findEnclosedFaces(exterior, 1) };
    }
    const face = this.regionFaceCache.faces
      .filter(item => window.Geometry.pointInPolygon(worldPt, item.points))
      .sort((a, b) => a.area - b.area)[0];
    if (!face) return { poly: smallest, face: null };
    const same = visible.find(poly => this.polygonRingsEqual(poly.points, face.points));
    if (same) return { poly: same, face: null };
    if (smallest && byArea(smallest) <= face.area + 1e-6) return { poly: smallest, face: null };
    if (smallest && this.isEntityLocked(smallest)) return { poly: smallest, face: null };
    return { poly: null, face };
  }

  // Give edge `index` of `poly` a wall of its own running exactly along that
  // edge, styled like the wall it borrowed. Used where an area must not share
  // or partially reuse another area's wall: moving the porch then moves only
  // the porch, never the house wall it was drawn against.
  ownBoundaryLine(poly, index) {
    const edge = poly.edges[index];
    const source = this.state.lines.find(line => line.id === edge?.lineId);
    const start = poly.points[index];
    const end = poly.points[(index + 1) % poly.points.length];
    const line = {
      ...(source ? JSON.parse(JSON.stringify(source)) : { type: 'exterior' }),
      start: { x: start.x, y: start.y },
      end: { x: end.x, y: end.y },
      bowAngle: Number(edge?.bowAngle) || 0,
      id: this.genId()
    };
    delete line.dimensionPosition;
    delete line.dimensionAnchor;
    if (line.bowAngle) {
      const chord = window.Geometry.distance(line.start, line.end);
      line.arcHeight = chord / 2 * Math.tan(line.bowAngle * Math.PI / 720);
    } else {
      delete line.arcHeight;
    }
    if (poly.layerId) line.layerId = poly.layerId;
    this.state.lines.push(line);
    poly.edges[index] = { lineId: line.id, bowAngle: line.bowAngle };
    return line;
  }

  // True when edge `index` of `poly` runs along only part of its wall.
  edgeBorrowsPartOfWall(poly, index) {
    const line = this.state.lines.find(item => item.id === poly.edges[index]?.lineId);
    if (!line) return false;
    const start = poly.points[index];
    const end = poly.points[(index + 1) % poly.points.length];
    const exact = (this.pointsEqual(line.start, start) && this.pointsEqual(line.end, end)) ||
      (this.pointsEqual(line.start, end) && this.pointsEqual(line.end, start));
    return !exact;
  }

  // Resolves a define click and materializes a new wall face as an undefined
  // area (one undo step). Returns { poly, created }.
  regionAreaAt(worldPt) {
    const region = this.resolveRegionAt(worldPt);
    if (!region.face) return { poly: region.poly, created: false };
    const face = region.face;
    this.pushUndo('Define Region');
    const poly = {
      points: face.points.map(point => ({ x: point.x, y: point.y })),
      edges: (face.edges || []).map(edge => ({ lineId: edge.lineId, bowAngle: 0 })),
      type: 'UND',
      name: 'Undefined Area',
      gross: face.area,
      area: face.area,
      factorOverride: null,
      subtractMode: this.state.options?.autoSubtractOverlapped === false ? 'never' : 'auto',
      subtractTargetId: null,
      layerId: this.state.activeLayerId,
      id: this.genId()
    };
    if (poly.edges.length !== poly.points.length) poly.edges = this.buildPolygonEdges(poly, this.state.lines);
    // An edge that runs along only part of a wall (the porch's side of the
    // house wall) gets its own overlapping wall, as a hand-closed addition
    // does, so the new area's boundary is exactly its own.
    poly.edges.forEach((edge, index) => {
      if (this.edgeBorrowsPartOfWall(poly, index)) this.ownBoundaryLine(poly, index);
    });
    this.state.polygons.push(poly);
    this.assignAreaToDedicatedLayer(poly);
    this.recalculate();
    return { poly: this.state.polygons.find(item => item.id === poly.id) || poly, created: true };
  }

  defineAreaTarget(target, code, name, autoPost = null) {
    if (!this.state.polygons.includes(target)) return;
    this.pushUndo('Define Area');
    if (autoPost) {
      target.autoPost = { ...(target.autoPost || {}), ...autoPost };
      const lineIds = new Set((target.edges || []).map(edge => edge.lineId));
      if (typeof autoPost.dims === 'boolean') this.state.lines.forEach(line => {
        if (lineIds.has(line.id)) line.dimensionHidden = !autoPost.dims;
      });
    }
    target.type = code;
    target.name = name;
    target.factor = target.factor || 1;
    // Auto-generated area layers pick up the area's name once it is defined.
    const areaLayer = this.getLayerById(target.layerId);
    if (areaLayer && /^Area( \d+)?$/.test(areaLayer.name)) {
      areaLayer.name = this.uniqueLayerName(name || 'Area');
      this.ui.updateLayersPanel?.();
    }
    this.recalcAutoSubtract();
    this.ui.updateCalcsPanel();
    this.render();
  }

  // Auto-detect enclosed regions formed by the walls and create an undefined
  // (UND) area for each one not already covered by an existing polygon. Runs
  // face traversal over the wall graph; a single undo covers every created area.
  detectRegions() {
    const MIN_REGION_AREA = 4; // sq ft; drop sliver faces
    const faces = window.Geometry.findEnclosedFaces(this.state.lines, MIN_REGION_AREA);
    if (!faces.length) { this.ui.notify('No enclosed regions found.'); return; }
    const existing = this.state.polygons;
    const toCreate = faces.filter(face => {
      const centroid = window.Geometry.polygonCentroid(face.points);
      return !existing.some(poly =>
        window.Geometry.pointInPolygon(centroid, window.Geometry.flattenRing(poly)));
    });
    if (!toCreate.length) { this.ui.notify('All enclosed regions are already defined.'); return; }
    this.pushUndo('Detect Regions');
    toCreate.forEach(face => {
      const poly = {
        points: face.points.map(point => ({ x: point.x, y: point.y })),
        edges: (face.edges || []).map(edge => ({ lineId: edge.lineId, bowAngle: 0 })),
        type: 'UND',
        name: 'Undefined Area',
        gross: face.area,
        area: face.area,
        factorOverride: null,
        subtractMode: this.state.options?.autoSubtractOverlapped === false ? 'never' : 'auto',
        subtractTargetId: null,
        layerId: this.state.activeLayerId,
        id: this.genId()
      };
      if (poly.edges.length !== poly.points.length) {
        poly.edges = this.buildPolygonEdges(poly, this.state.lines);
      }
      this.state.polygons.push(poly);
      this.assignAreaToDedicatedLayer(poly);
    });
    this.recalcAutoSubtract();
    this.ui.updateCalcsPanel();
    this.render();
    this.ui.notify(`Detected ${toCreate.length} enclosed region${toCreate.length === 1 ? '' : 's'}.`);
  }

  // Parametric room templates. Returns the unrotated outline points for the
  // given shape config (anchored at the local origin, bottom-left).
  getShapePoints(shape) {
    const w = Math.max(0.1, Number(shape.width) || 0);
    const d = Math.max(0.1, Number(shape.depth) || 0);
    if (shape.kind === 'rectangle') {
      return [{ x: 0, y: 0 }, { x: w, y: 0 }, { x: w, y: d }, { x: 0, y: d }];
    }
    // L-shape: rectangle with a corner notch cut from the top-right.
    const cw = Math.min(w, Math.max(0, Number(shape.cutWidth) || 0));
    const cd = Math.min(d, Math.max(0, Number(shape.cutDepth) || 0));
    return [
      { x: 0, y: 0 }, { x: w, y: 0 }, { x: w, y: d - cd },
      { x: w - cw, y: d - cd }, { x: w - cw, y: d }, { x: 0, y: d }
    ];
  }

  // Anchor the pending room template at a world point, creating a closed
  // exterior-wall loop with shared endpoints and (optionally) an area polygon.
  placeShapeAt(worldPt) {
    const shape = this.state.pendingShape;
    if (!shape) return;
    const base = this.getShapePoints(shape);
    const theta = (Number(shape.rotation) || 0) * Math.PI / 180;
    const cos = Math.cos(theta), sin = Math.sin(theta);
    let pts = base.map(p => ({ x: p.x * cos - p.y * sin, y: p.x * sin + p.y * cos }));
    const minX = Math.min(...pts.map(p => p.x));
    const minY = Math.min(...pts.map(p => p.y));
    pts = pts.map(p => ({ x: worldPt.x + (p.x - minX), y: worldPt.y + (p.y - minY) }));

    this.pushUndo('Place Shape');
    const style = this.getDefaultLineStyle('exterior');
    const created = [];
    for (let i = 0; i < pts.length; i++) {
      const a = pts[i], b = pts[(i + 1) % pts.length];
      const line = {
        start: { ...a }, end: { ...b }, type: 'exterior', bowAngle: 0,
        color: style.color, lineThickness: style.lineThickness,
        layerId: this.state.activeLayerId, id: this.genId()
      };
      this.state.lines.push(line);
      created.push(line);
    }
    if (shape.defineAsArea) {
      const edges = created.map(line => ({ lineId: line.id, bowAngle: 0 }));
      const area = window.Geometry.calculateArea(pts, edges);
      const poly = {
        points: pts.map(p => ({ ...p })), edges,
        type: 'UND', name: 'Undefined Area', gross: area, area,
        factorOverride: null,
        subtractMode: this.state.options?.autoSubtractOverlapped === false ? 'never' : 'auto',
        subtractTargetId: null, layerId: this.state.activeLayerId, id: this.genId()
      };
      this.state.polygons.push(poly);
      this.assignAreaToDedicatedLayer(poly);
    }
    this.recalcAutoSubtract();
    this.ui.updateCalcsPanel();
    this.render();
  }

  reopenSelectedArea() {
    const poly = this.state.polygons.find(item => this.state.selectedPolyIds.has(item.id));
    if (!poly || poly.points.length < 3) { this.ui.notify('Select an area to reopen.'); return; }
    this.pushUndo('Reopen Area');
    const openStart = { ...poly.points[0] };
    const pathStart = { ...poly.points[1] };
    const boundaryId = poly.edges && poly.edges[0] && poly.edges[0].lineId;
    const edgeMatches = line => line.id === boundaryId ||
      ((this.pointsEqual(line.start, openStart) && this.pointsEqual(line.end, pathStart)) ||
       (this.pointsEqual(line.start, pathStart) && this.pointsEqual(line.end, openStart)));
    const boundary = this.state.lines.find(edgeMatches);
    if (boundary) this.state.lines = this.state.lines.filter(line => line.id !== boundary.id);
    this.state.polygons = this.state.polygons.filter(item => item.id !== poly.id);
    // Keep editing on the area's own layer so the re-closed area reuses it
    // instead of stacking a fresh one on top.
    const polyLayerId = this.getEntityLayerId(poly);
    if (this.getLayerById(polyLayerId)) this.state.activeLayerId = polyLayerId;
    this.recalculate();
    this.clearSelection(false);
    this.setMode('draw-exterior');
    this.state.penDown = true;
    this.state.currentPolyPoints = [
      ...poly.points.slice(1).map(point => ({ ...point })),
      { ...openStart }
    ];
    this.state.currentLineIds = (poly.edges || []).slice(1).map(edge => edge.lineId).filter(id => id != null);
    this.state.cursor = { ...openStart };
    this.state.pointerWorld = { ...openStart };
    this.ui.updateHUD('', '', true);
    this.ui.notify('Area reopened. Continue drawing from the open end, then close back to the starting point.');
    this.ui.updateCalcsPanel();
    this.render();
  }

  getSelectedGeometryRotationModel() {
    const key = point => `${point.x.toFixed(5)},${point.y.toFixed(5)}`;
    const selectedKeys = new Set();
    this.state.lines.forEach(line => {
      if (this.state.selectedLineIds.has(line.id)) {
        selectedKeys.add(key(line.start));
        selectedKeys.add(key(line.end));
      }
    });
    this.state.polygons.forEach(poly => {
      if (this.state.selectedPolyIds.has(poly.id)) poly.points.forEach(point => selectedKeys.add(key(point)));
    });
    if (selectedKeys.size === 0) return null;

    const uniquePoints = new Map();
    this.state.lines.forEach(line => [line.start, line.end].forEach(point => {
      const pointKey = key(point);
      if (selectedKeys.has(pointKey) && !uniquePoints.has(pointKey)) uniquePoints.set(pointKey, point);
    }));
    this.state.polygons.forEach(poly => poly.points.forEach(point => {
      const pointKey = key(point);
      if (selectedKeys.has(pointKey) && !uniquePoints.has(pointKey)) uniquePoints.set(pointKey, point);
    }));
    const points = [...uniquePoints.values()];
    if (!points.length) return null;
    const bounds = {
      minX: Math.min(...points.map(point => point.x)),
      maxX: Math.max(...points.map(point => point.x)),
      minY: Math.min(...points.map(point => point.y)),
      maxY: Math.max(...points.map(point => point.y))
    };
    return {
      key,
      selectedKeys,
      points,
      bounds,
      center: { x: (bounds.minX + bounds.maxX) / 2, y: (bounds.minY + bounds.maxY) / 2 }
    };
  }

  rotateSelected(degrees) {
    const entities = [
      ...this.state.polygons.filter(item => this.state.selectedAreaLabelIds.has(item.id)),
      ...this.state.polygons.filter(item => this.state.selectedAreaSizeIds.has(item.id)),
      ...this.state.labels.filter(item => this.state.selectedLabelIds.has(item.id)),
      ...this.state.symbols.filter(item => this.state.selectedSymbolIds.has(item.id))
    ];
    if (entities.length === 0 && this.state.selectedLineIds.size === 0 && this.state.selectedPolyIds.size === 0) return;
    this.pushUndo('Rotate');
    if (this.state.selectedLineIds.size === 0 && this.state.selectedPolyIds.size === 0) {
      this.state.polygons.forEach(poly => {
        const labelRotation = Number(poly.labelRotation) || 0;
        const sizeRotation = Number(poly.areaSizeRotation ?? labelRotation) || 0;
        if (this.state.selectedAreaLabelIds.has(poly.id)) poly.labelRotation = labelRotation + degrees;
        if (this.state.selectedAreaSizeIds.has(poly.id)) {
          poly.areaSizeRotation = sizeRotation + degrees;
        }
      });
      this.state.labels.forEach(label => {
        if (this.state.selectedLabelIds.has(label.id)) label.rotation = (label.rotation || 0) + degrees;
      });
      this.state.symbols.forEach(symbol => {
        if (this.state.selectedSymbolIds.has(symbol.id)) {
          symbol.rotation = (symbol.rotation || 0) + degrees;
          symbol.wallId = null;
        }
      });
      this.render();
      return;
    }

    const model = this.getSelectedGeometryRotationModel();
    if (!model) return;
    const { key, selectedKeys, center } = model;
    const radians = degrees * Math.PI / 180;
    const cos = Math.cos(radians), sin = Math.sin(radians);
    const rotatingLineIds = new Set(this.state.lines
      .filter(line => selectedKeys.has(key(line.start)) && selectedKeys.has(key(line.end)))
      .map(line => line.id));
    this.state.symbols.forEach(symbol => {
      if (!rotatingLineIds.has(symbol.wallId) && !this.state.selectedSymbolIds.has(symbol.id)) return;
      const dx = symbol.x - center.x;
      const dy = symbol.y - center.y;
      symbol.x = center.x + dx * cos - dy * sin;
      symbol.y = center.y + dx * sin + dy * cos;
      symbol.rotation = (Number(symbol.rotation) || 0) - degrees;
    });
    const rotatePoint = point => {
      const originalKey = key(point);
      if (!selectedKeys.has(originalKey)) return;
      const dx = point.x - center.x, dy = point.y - center.y;
      point.x = center.x + dx * cos - dy * sin;
      point.y = center.y + dx * sin + dy * cos;
    };
    this.state.labels.forEach(label => {
      if (!this.state.selectedLabelIds.has(label.id)) return;
      const dx = label.x - center.x, dy = label.y - center.y;
      label.x = center.x + dx * cos - dy * sin;
      label.y = center.y + dx * sin + dy * cos;
      label.rotation = (Number(label.rotation) || 0) - degrees;
    });
    this.state.polygons.forEach(poly => {
      const rotateWithArea = this.state.selectedPolyIds.has(poly.id);
      const rotateLabelOnly = this.state.selectedAreaLabelIds.has(poly.id) && !rotateWithArea;
      const rotateSizeOnly = this.state.selectedAreaSizeIds.has(poly.id) && !rotateWithArea;
      const labelRotation = Number(poly.labelRotation) || 0;
      const sizeRotation = Number(poly.areaSizeRotation ?? labelRotation) || 0;
      if (rotateWithArea) {
        if (poly.labelPosition) {
          const labelPoint = { ...poly.labelPosition };
          const dx = labelPoint.x - center.x, dy = labelPoint.y - center.y;
          poly.labelPosition = { x: center.x + dx * cos - dy * sin, y: center.y + dx * sin + dy * cos };
        }
        if (poly.areaSizePosition) {
          const sizePoint = { ...poly.areaSizePosition };
          const dx = sizePoint.x - center.x, dy = sizePoint.y - center.y;
          poly.areaSizePosition = { x: center.x + dx * cos - dy * sin, y: center.y + dx * sin + dy * cos };
        }
        poly.labelRotation = labelRotation - degrees;
        poly.areaSizeRotation = sizeRotation - degrees;
      } else {
        if (rotateLabelOnly) {
          const labelPoint = this.getAreaLabelPosition(poly);
          const dx = labelPoint.x - center.x, dy = labelPoint.y - center.y;
          poly.labelPosition = { x: center.x + dx * cos - dy * sin, y: center.y + dx * sin + dy * cos };
          poly.labelRotation = labelRotation - degrees;
        }
        if (rotateSizeOnly) {
          const sizePoint = this.getAreaSizePosition(poly);
          const dx = sizePoint.x - center.x, dy = sizePoint.y - center.y;
          poly.areaSizePosition = { x: center.x + dx * cos - dy * sin, y: center.y + dx * sin + dy * cos };
          poly.areaSizeRotation = sizeRotation - degrees;
        }
      }
    });
    this.state.lines.forEach(line => { rotatePoint(line.start); rotatePoint(line.end); });
    this.state.polygons.forEach(poly => {
      poly.points.forEach(rotatePoint);
    });
    this.recalcAutoSubtract();
    this.ui.updateCalcsPanel();
    this.render();
  }

  orientSelected(axis) {
    this.pushUndo('Orient');
    const rotation = axis === 'horizontal' ? 0 : 90;
    this.state.polygons.forEach(poly => { if (this.state.selectedAreaLabelIds.has(poly.id)) poly.labelRotation = rotation; });
    this.state.polygons.forEach(poly => { if (this.state.selectedAreaSizeIds.has(poly.id)) poly.areaSizeRotation = rotation; });
    this.state.labels.forEach(label => { if (this.state.selectedLabelIds.has(label.id)) label.rotation = rotation; });
    this.state.symbols.forEach(symbol => {
      if (this.state.selectedSymbolIds.has(symbol.id)) {
        symbol.rotation = rotation;
        symbol.wallId = null;
      }
    });
    this.render();
  }

  scaleSelected(factor) {
    const hasTargets = this.state.selectedAreaLabelIds.size > 0 || this.state.selectedAreaSizeIds.size > 0 ||
      this.state.selectedLabelIds.size > 0 || this.state.selectedSymbolIds.size > 0;
    if (!hasTargets) return;
    this.pushUndo('Scale');
    this.state.labels.forEach(label => {
      if (this.state.selectedLabelIds.has(label.id)) label.fontSize = Math.max(6, Math.min(72, (label.fontSize || 12) * factor));
    });
    this.state.polygons.forEach(poly => {
      const labelScale = Number(poly.labelScale) || 1;
      const sizeScale = Number(poly.areaSizeScale ?? labelScale) || 1;
      if (this.state.selectedAreaLabelIds.has(poly.id)) poly.labelScale = Math.max(0.5, Math.min(4, labelScale * factor));
      if (this.state.selectedAreaSizeIds.has(poly.id)) {
        poly.areaSizeScale = Math.max(0.5, Math.min(4, sizeScale * factor));
      }
    });
    this.state.symbols.forEach(symbol => {
      if (this.state.selectedSymbolIds.has(symbol.id)) symbol.scale = Math.max(0.25, Math.min(8, (symbol.scale || 1) * factor));
    });
    this.render();
  }

  moveLineEndpoint(lineId, endpoint, nextPoint, recordHistory = true) {
    const line = this.state.lines.find(item => item.id === lineId);
    if (!line || !['start', 'end'].includes(endpoint) || !nextPoint) return false;
    const oldPoint = { ...line[endpoint] };
    if (Math.hypot(nextPoint.x - oldPoint.x, nextPoint.y - oldPoint.y) < 1e-8) return false;
    const matches = point => Math.hypot(point.x - oldPoint.x, point.y - oldPoint.y) < 1e-5;
    if (recordHistory) this.pushUndo('Edit Wall Endpoint');

    // A wall endpoint is a shared vertex. Move only that vertex everywhere it
    // occurs so adjoining walls stretch while the opposite endpoint stays put.
    this.state.lines.forEach(item => [item.start, item.end].forEach(point => {
      if (matches(point)) { point.x = nextPoint.x; point.y = nextPoint.y; }
    }));
    this.state.polygons.forEach(poly => poly.points.forEach(point => {
      if (matches(point)) { point.x = nextPoint.x; point.y = nextPoint.y; }
    }));
    this.state.activeLineEndpoint = { lineId, endpoint };
    if (Math.abs(Number(line.bowAngle) || 0) > .001) {
      const chord = window.Geometry.distance(line.start, line.end);
      line.arcHeight = chord / 2 * Math.tan(Number(line.bowAngle) * Math.PI / 720);
    }
    if (recordHistory) this.closeJoinedExteriorLoops();
    this.refreshHostedSymbols();
    this.recalculate();
    this.ui.updateCalcsPanel();
    this.render();
    return true;
  }

  getLineEndpointRefs(point) {
    const refs = [];
    this.state.lines.forEach(line => ['start', 'end'].forEach(endpoint => {
      if (this.pointsEqual(line[endpoint], point)) refs.push(`${line.id}:${endpoint}`);
    }));
    return refs;
  }

  snapLineEndpoint(worldPt, lineId, endpoint, excludedRefs = null) {
    // Honor the Snap toggle for endpoint dragging, just like drawing: when
    // snapping is off, the endpoint follows the pointer freely.
    if (!this.state.snapEnabled) return { ...worldPt };
    const movingLine = this.state.lines.find(line => line.id === lineId);
    const excluded = new Set(excludedRefs || this.getLineEndpointRefs(movingLine?.[endpoint]));
    const tolerance = this.getScreenToleranceWorld(14);
    let nearest = null;
    this.state.lines.forEach(line => ['start', 'end'].forEach(side => {
      if (excluded.has(`${line.id}:${side}`)) return;
      const point = line[side];
      const distance = window.Geometry.distance(worldPt, point);
      if (distance <= tolerance && (!nearest || distance < nearest.distance)) nearest = { point, distance };
    }));
    if (nearest) return { ...nearest.point };
    return window.Geometry.snapToGrid(worldPt, this.state.gridSize);
  }

  // Intersection point of two segments that properly cross each other.
  // Endpoint touches and collinear overlaps are deliberately not crossings.
  segmentCrossPoint(a1, a2, b1, b2) {
    const r1x = a2.x - a1.x, r1y = a2.y - a1.y;
    const r2x = b2.x - b1.x, r2y = b2.y - b1.y;
    const den = r1x * r2y - r1y * r2x;
    if (Math.abs(den) < 1e-9) return null;
    const t = ((b1.x - a1.x) * r2y - (b1.y - a1.y) * r2x) / den;
    const u = ((b1.x - a1.x) * r1y - (b1.y - a1.y) * r1x) / den;
    // The intersection must lie on BOTH segments. Without this, an extended
    // (extrapolated) crossing registers as a junction, and the wall healer
    // fabricates phantom walls / truncates real ones mid-draw.
    const EPS = 1e-6;
    if (t < -EPS || t > 1 + EPS || u < -EPS || u > 1 + EPS) return null;
    // Two segments that only share a corner are not a crossing.
    if ((t <= EPS || t >= 1 - EPS) && (u <= EPS || u >= 1 - EPS)) return null;
    return { x: a1.x + t * r1x, y: a1.y + t * r1y };
  }

  alongLineT(line, point) {
    const dx = line.end.x - line.start.x, dy = line.end.y - line.start.y;
    return ((point.x - line.start.x) * dx + (point.y - line.start.y) * dy) / (dx * dx + dy * dy);
  }

  // Walk a closed loop through the wall graph starting along firstId from
  // startKey. At junction nodes the straightest unused continuation is tried
  // first, and a wall may be passed straight through where a junction sits
  // on its body, so T-junctions and healed crossings traverse correctly.
  linePassesThrough(point, line) {
    return this.pointKey(point) !== this.pointKey(line.start) &&
      this.pointKey(point) !== this.pointKey(line.end);
  }

  exitKeyFrom(nodePoint, fromPoint, line) {
    const pointKey = point => this.pointKey(point);
    const dirX = nodePoint.x - fromPoint.x, dirY = nodePoint.y - fromPoint.y;
    const dotStart = dirX * (line.start.x - nodePoint.x) + dirY * (line.start.y - nodePoint.y);
    const dotEnd = dirX * (line.end.x - nodePoint.x) + dirY * (line.end.y - nodePoint.y);
    return dotStart > dotEnd ? pointKey(line.start) : pointKey(line.end);
  }

  walkClosedLoop(nodes, byId, startKey, firstId) {
    const pointKey = point => this.pointKey(point);
    const deviation = (fromPoint, nodePoint, line) => {
      const inX = nodePoint.x - fromPoint.x, inY = nodePoint.y - fromPoint.y;
      const outKey = this.exitKeyFrom(nodePoint, fromPoint, line);
      const out = pointKey(line.start) === outKey ? line.start : line.end;
      const outX = out.x - nodePoint.x, outY = out.y - nodePoint.y;
      const inLen = Math.hypot(inX, inY) || 1, outLen = Math.hypot(outX, outY) || 1;
      return Math.abs((inX / inLen) * (outY / outLen) - (inY / inLen) * (outX / outLen));
    };
    const rec = (currentKey, currentId, usedIds, points, edges, fromPoint) => {
      const node = nodes.get(currentKey);
      const line = byId.get(currentId);
      if (!node || !line || usedIds.has(currentId)) return null;
      const startK = pointKey(line.start), endK = pointKey(line.end);
      // Arriving mid-wall (T-junction pass-through) either way is viable.
      const exits = currentKey === startK ? [endK] : currentKey === endK ? [startK] : [startK, endK];
      for (const nextKey of exits) {
        const nextPoints = points.concat([{ ...node.point }]);
        const nextEdges = edges.concat([{
          lineId: line.id,
          bowAngle: nextKey === endK ? (line.bowAngle || 0) : -(line.bowAngle || 0)
        }]);
        const used = new Set(usedIds); used.add(currentId);
        if (nextKey === startKey) {
          if (used.size >= 3) return { points: nextPoints, edges: nextEdges, used };
          continue;
        }
        const nextNode = nodes.get(nextKey);
        if (!nextNode) continue;
        const candidates = nextNode.ids.filter(id => id === currentId
          ? Math.abs(Number(line.bowAngle) || 0) < 1e-6 && this.linePassesThrough(nextNode.point, line)
          : !used.has(id));
        candidates.sort((a, b) => deviation(node.point, nextNode.point, byId.get(a)) -
          deviation(node.point, nextNode.point, byId.get(b)));
        for (const cand of candidates) {
          const result = rec(nextKey, cand, used, nextPoints, nextEdges, node.point);
          if (result) return result;
        }
      }
      return null;
    };
    return rec(startKey, firstId, new Set(), [], [], null);
  }

  // Split a straight wall at an interior point so polygon vertices always
  // sit on real wall endpoints, mirroring insertVertexAt.
  splitLineAtPoint(lineId, point) {
    // After earlier splits on the same wall the requested id may now belong
    // to the other half, so fall back to whichever wall contains the point.
    const contains = item => Math.abs(Number(item.bowAngle) || 0) < 1e-6 &&
      this.linePassesThrough(point, item) &&
      window.Geometry.distanceToSegment(point, item.start, item.end) < 1e-6;
    const line = this.state.lines.find(item => item.id === lineId && contains(item)) ||
      this.state.lines.find(item => contains(item));
    if (!line) return;
    const index = this.state.lines.indexOf(line);
    const originalEnd = { ...line.end };
    line.end = { ...point };
    // Each half gets its own automatic dimension; one pinned label must not
    // become two stacked copies.
    delete line.dimensionPosition;
    delete line.dimensionAnchor;
    const second = { ...JSON.parse(JSON.stringify(line)), start: { ...point }, end: originalEnd, id: this.genId() };
    this.state.lines.splice(index + 1, 0, second);
    this.state.polygons.forEach(poly => {
      for (let i = 0; i < poly.points.length; i++) {
        const nextIndex = (i + 1) % poly.points.length;
        const a = poly.points[i], b = poly.points[nextIndex];
        const followsLine = this.pointsEqual(a, line.start) && this.pointsEqual(b, originalEnd);
        const reversesLine = this.pointsEqual(a, originalEnd) && this.pointsEqual(b, line.start);
        if (!followsLine && !reversesLine) continue;
        poly.points.splice(i + 1, 0, { ...point });
        const splitEdges = followsLine
          ? [{ lineId: line.id, bowAngle: 0 }, { lineId: second.id, bowAngle: 0 }]
          : [{ lineId: second.id, bowAngle: 0 }, { lineId: line.id, bowAngle: 0 }];
        poly.edges.splice(i, 1, ...splitEdges);
        break;
      }
      // Partial edges that now run along the far half take the new wall id,
      // whichever direction the polygon traverses the junction.
      poly.edges.forEach((edge, i) => {
        if (edge.lineId !== line.id) return;
        const a = poly.points[i], b = poly.points[(i + 1) % poly.points.length];
        if (this.pointsEqual(a, line.start) && this.pointsEqual(b, line.end)) return;
        if (this.pointsEqual(a, line.end) && this.pointsEqual(b, line.start)) return;
        const far = this.pointsEqual(a, point) ? b : this.pointsEqual(b, point) ? a : null;
        if (far && window.Geometry.distanceToSegment(far, point, originalEnd) < 1e-6) edge.lineId = second.id;
      });
    });
  }

  // Exterior walls sketched with overshooting or nearly touching ends still
  // enclose an appraisal area on screen. Heal the wall graph before loop
  // detection: weld free ends that land on another wall's body and trim
  // dangling overshoot tails back to the crossing point. The mutations are
  // applied only when they actually produce a closed loop that does not
  // exist yet, so open sketches and existing areas stay untouched.
  healExteriorWallJoins() {
    const exterior = this.state.lines.filter(line => line.type === 'exterior' &&
      window.Geometry.distance(line.start, line.end) > 1e-6 &&
      Math.abs(Number(line.bowAngle) || 0) < 0.5);
    if (exterior.length < 3) return;
    const pointKey = point => this.pointKey(point);
    const protectedKeys = this.closedAreaVertexKeys();
    const existing = new Set(this.state.polygons.filter(poly => this.isPolygonBoundaryIntact(poly)).map(poly => (poly.edges || [])
      .map(edge => edge.lineId).filter(id => id != null)
      .sort((a, b) => String(a).localeCompare(String(b))).join('|')));
    const lineIndex = new Map(exterior.map((line, index) => [line.id, index]));
    const endpointShared = (line, side) => exterior.some(other => other !== line &&
      (pointKey(other.start) === pointKey(line[side]) || pointKey(other.end) === pointKey(line[side])));

    // Welds: a free end that nearly lands on another wall's body joins it.
    const weldTolerance = this.getScreenToleranceWorld(16);
    const welds = [];
    exterior.forEach(line => ['start', 'end'].forEach(side => {
      if (endpointShared(line, side) || protectedKeys.has(pointKey(line[side]))) return;
      // An end that already crosses another wall joins the graph at that
      // crossing; welding it elsewhere would pull the wall to a wrong node.
      const alreadyJoined = exterior.some(other => {
        if (other === line) return false;
        const cross = this.segmentCrossPoint(line.start, line.end, other.start, other.end);
        if (!cross) return false;
        const onLine = this.alongLineT(line, cross), onOther = this.alongLineT(other, cross);
        return onLine > 1e-6 && onLine < 1 - 1e-6 && onOther > 1e-6 && onOther < 1 - 1e-6 &&
          window.Geometry.distance(line[side], cross) <= weldTolerance;
      });
      if (alreadyJoined) return;
      let best = null;
      const consider = candidate => {
        if (!best || candidate.distance < best.distance - 1e-9 ||
            (Math.abs(candidate.distance - best.distance) <= 1e-9 && candidate.atEndpoint && !best.atEndpoint)) {
          best = candidate;
        }
      };
      exterior.forEach(host => {
        if (host === line) return;
        // Canonicalize near endpoint-to-endpoint joins onto the older wall.
        // This handles sub-grid residue that is visibly coincident but falls
        // on opposite sides of pointKey's five-decimal boundary. The reverse
        // direction is skipped so a pair cannot weld to each other.
        if (lineIndex.get(host.id) < lineIndex.get(line.id)) {
          ['start', 'end'].forEach(hostSide => {
            const target = host[hostSide];
            if (protectedKeys.has(pointKey(target))) return;
            const distance = window.Geometry.distance(line[side], target);
            if (distance > 1e-9 && distance <= weldTolerance) {
              consider({ host, target, distance, atEndpoint: true });
            }
          });
        }
        const target = window.Geometry.projectPointToSegment(line[side], host.start, host.end);
        if (window.Geometry.distance(target, host.start) <= 1e-4 ||
            window.Geometry.distance(target, host.end) <= 1e-4) return;
        const distance = window.Geometry.distance(line[side], target);
        if (distance <= 1e-9 || distance > weldTolerance) return;
        consider({ host, target, distance, atEndpoint: false });
      });
      if (best) welds.push({ line, side, ...best });
    }));

    // Proper crossings split both walls they join; weld targets split hosts.
    const splitsByLine = new Map();
    const addSplit = (line, point) => {
      if (!splitsByLine.has(line.id)) splitsByLine.set(line.id, []);
      splitsByLine.get(line.id).push(point);
    };
    for (let i = 0; i < exterior.length; i++) {
      for (let j = i + 1; j < exterior.length; j++) {
        const point = this.segmentCrossPoint(exterior[i].start, exterior[i].end, exterior[j].start, exterior[j].end);
        if (!point) continue;
        addSplit(exterior[i], point);
        addSplit(exterior[j], point);
      }
    }
    welds.forEach(weld => { if (!weld.atEndpoint) addSplit(weld.host, weld.target); });
    if (!welds.length && !splitsByLine.size) return;

    // Virtual graph: nodes are endpoints (welded ends merge into their host
    // split node) plus split points; sub-edges are the wall pieces between.
    const nodes = new Map();
    const subEdges = [];
    const subEdgeCount = new Map();
    const weldKey = new Map();
    welds.forEach(weld => weldKey.set(`${weld.line.id}:${weld.side}`, pointKey(weld.target)));
    const nodeFor = (point, overrideKey) => {
      const key = overrideKey || pointKey(point);
      if (!nodes.has(key)) nodes.set(key, { point: { ...point }, ids: [] });
      return key;
    };
    // Pre-seed junction nodes so a welded end merged under an override key
    // resolves to a node positioned at the junction, not at the free end.
    welds.forEach(weld => nodeFor(weld.target));
    splitsByLine.forEach(splits => splits.forEach(point => nodeFor(point)));
    exterior.forEach(line => {
      const splits = (splitsByLine.get(line.id) || [])
        .map(point => ({ t: this.alongLineT(line, point), point }))
        .sort((a, b) => a.t - b.t);
      const keys = [nodeFor(line.start, weldKey.get(`${line.id}:start`))];
      splits.forEach(split => keys.push(nodeFor(split.point)));
      keys.push(nodeFor(line.end, weldKey.get(`${line.id}:end`)));
      for (let index = 0; index < keys.length - 1; index++) {
        if (keys[index] === keys[index + 1]) continue;
        const edge = {
          id: `${line.id}#${subEdges.length}`, lineId: line.id,
          aKey: keys[index], bKey: keys[index + 1]
        };
        subEdges.push(edge);
        subEdgeCount.set(line.id, (subEdgeCount.get(line.id) || 0) + 1);
        nodes.get(edge.aKey).ids.push(edge.id);
        nodes.get(edge.bKey).ids.push(edge.id);
      }
    });

    // Prune dangling pieces. A wall keeps at least one piece and vertices of
    // existing areas are never trimmed away.
    const endPointNode = new Map();
    exterior.forEach(line => {
      endPointNode.set(`${line.id}:start`, weldKey.get(`${line.id}:start`) || pointKey(line.start));
      endPointNode.set(`${line.id}:end`, weldKey.get(`${line.id}:end`) || pointKey(line.end));
    });
    const queue = [...nodes.keys()].filter(key => nodes.get(key).ids.length === 1);
    const stuck = new Set();
    while (queue.length) {
      const key = queue.pop();
      if (stuck.has(key) || protectedKeys.has(key)) continue;
      const node = nodes.get(key);
      if (node.ids.length !== 1) continue;
      const edge = subEdges.find(item => item.id === node.ids[0]);
      if ((subEdgeCount.get(edge.lineId) || 0) <= 1) { stuck.add(key); continue; }
      const otherKey = edge.aKey === key ? edge.bKey : edge.aKey;
      ['start', 'end'].forEach(side => {
        if (endPointNode.get(`${edge.lineId}:${side}`) === key) endPointNode.set(`${edge.lineId}:${side}`, otherKey);
      });
      node.ids = [];
      nodes.get(otherKey).ids = nodes.get(otherKey).ids.filter(item => item !== edge.id);
      subEdgeCount.set(edge.lineId, subEdgeCount.get(edge.lineId) - 1);
      if (nodes.get(otherKey).ids.length === 1) queue.push(otherKey);
    }

    // Look for a closed loop in the surviving graph; virtual lines carry the
    // real wall id so signatures compare against existing polygons.
    const virtualNodes = new Map();
    const virtualById = new Map();
    subEdges.forEach(edge => {
      if (!nodes.get(edge.aKey).ids.includes(edge.id) || !nodes.get(edge.bKey).ids.includes(edge.id)) return;
      virtualById.set(edge.id, {
        id: edge.id, realId: edge.lineId, bowAngle: 0,
        start: nodes.get(edge.aKey).point, end: nodes.get(edge.bKey).point
      });
    });
    nodes.forEach((node, key) => {
      if (!node.ids.length) return;
      virtualNodes.set(key, { point: node.point, ids: node.ids.slice() });
    });
    let newLoop = null;
    for (const edge of subEdges) {
      if (!virtualById.has(edge.id)) continue;
      [edge.aKey, edge.bKey].forEach(startKey => {
        if (newLoop) return;
        const loop = this.walkClosedLoop(virtualNodes, virtualById, startKey, edge.id);
        if (!loop) return;
        const realIds = new Set([...loop.used].map(id => virtualById.get(id).realId));
        const signature = [...realIds]
          .sort((a, b) => String(a).localeCompare(String(b))).join('|');
        if (existing.has(signature)) return;
        if (window.Geometry.calculateArea(loop.points, loop.edges) < this.minimumClosedArea()) return;
        newLoop = { signature, realIds, virtualIds: new Set(loop.used) };
      });
      if (newLoop) break;
    }
    if (!newLoop) return;

    // Commit the heals: move endpoints to their healed nodes and let any
    // area vertex sitting on a moved endpoint follow, like endpoint edits do.
    exterior.forEach(line => ['start', 'end'].forEach(side => {
      const targetKey = endPointNode.get(`${line.id}:${side}`);
      if (!targetKey || targetKey === pointKey(line[side])) return;
      // A wall may have several virtual pieces after a T-junction split. Only
      // move the endpoint attached to a piece used by the accepted loop; use
      // of the same real wall elsewhere must not authorize a speculative weld
      // or trim at its opposite tail.
      const usedAtTarget = subEdges.some(edge => newLoop.virtualIds.has(edge.id) &&
        edge.lineId === line.id && (edge.aKey === targetKey || edge.bKey === targetKey));
      if (!usedAtTarget) return;
      const target = nodes.get(targetKey);
      if (!target || protectedKeys.has(pointKey(line[side]))) return;
      const old = { ...line[side] };
      line[side].x = target.point.x; line[side].y = target.point.y;
      this.state.polygons.forEach(poly => poly.points.forEach(point => {
        if (this.pointsEqual(point, old, 1e-5)) { point.x = target.point.x; point.y = target.point.y; }
      }));
    }));
  }

  closeJoinedExteriorLoops() {
    this.healExteriorWallJoins();
    let exterior = this.state.lines.filter(line => line.type === 'exterior' &&
      window.Geometry.distance(line.start, line.end) > 1e-6);
    let byId = new Map(exterior.map(line => [line.id, line]));
    let faces = window.Geometry.findEnclosedFaces(exterior, 1e-9, 1e-5);

    // Materialize the virtual subdivisions returned by the planar face walk.
    // A divider may terminate halfway along a long perimeter wall; splitting
    // that wall first gives every bounded face its own real edge IDs and keeps
    // repaired polygon metadata topologically exact. Process points from the
    // far end back so the original line ID continues to contain each split.
    const splitPointsByLine = new Map();
    faces.forEach(face => face.edges.forEach((edge, index) => {
      const line = byId.get(edge.lineId);
      if (!line || Math.abs(Number(line.bowAngle) || 0) >= 1e-6) return;
      const points = [face.points[index], face.points[(index + 1) % face.points.length]];
      points.forEach(point => {
        if (!this.linePassesThrough(point, line) ||
            window.Geometry.distanceToSegment(point, line.start, line.end) > 1e-6) return;
        if (!splitPointsByLine.has(line.id)) splitPointsByLine.set(line.id, new Map());
        splitPointsByLine.get(line.id).set(this.pointKey(point), { ...point });
      });
    }));
    if (this.state.options?.splitWallsAtJunctions) {
      splitPointsByLine.forEach((points, lineId) => {
        const line = byId.get(lineId);
        [...points.values()]
          .sort((a, b) => this.alongLineT(line, b) - this.alongLineT(line, a))
          .forEach(point => this.splitLineAtPoint(lineId, point));
      });
    }
    if (splitPointsByLine.size) {
      exterior = this.state.lines.filter(line => line.type === 'exterior' &&
        window.Geometry.distance(line.start, line.end) > 1e-6);
      byId = new Map(exterior.map(line => [line.id, line]));
      faces = window.Geometry.findEnclosedFaces(exterior, 1e-9, 1e-5);
    }

    const intactPolygons = this.state.polygons.filter(poly => this.isPolygonBoundaryIntact(poly, byId));
    const intactIds = new Set(intactPolygons.map(poly => poly.id));
    const repairablePolygons = this.state.polygons.filter(poly => !intactIds.has(poly.id));
    const claimedLineIds = new Set(intactPolygons.flatMap(poly =>
      (poly.edges || []).map(edge => edge.lineId).filter(id => id != null)));
    const existing = new Set(intactPolygons.map(poly => (poly.edges || [])
      .map(edge => edge.lineId).filter(id => id != null).sort((a,b) => String(a).localeCompare(String(b))).join('|')));
    const seen = new Set();
    const candidates = [];
    let created = 0;

    faces.forEach(face => {
      let materialized = true;
      const edges = face.edges.map((edge, index) => {
        const line = byId.get(edge.lineId);
        if (!line) {
          materialized = false;
          return { lineId: edge.lineId, bowAngle: 0 };
        }
        const start = face.points[index];
        const end = face.points[(index + 1) % face.points.length];
        const follows = this.pointsEqual(line.start, start) && this.pointsEqual(line.end, end);
        const reverses = this.pointsEqual(line.end, start) && this.pointsEqual(line.start, end);
        if (!follows && !reverses) materialized = false;
        return { lineId: edge.lineId, bowAngle: follows ? (line.bowAngle || 0) :
          reverses ? -(line.bowAngle || 0) : 0 };
      });
      // Bowed walls are intentionally not split at T-junctions; until curved
      // sub-edges are modeled, do not persist a face whose edge is only part
      // of a live wall because it could never validate as a closed boundary.
      if (!materialized) return;
      const loop = { points: face.points, edges };
      const loopLineIds = [...new Set(edges.map(edge => edge.lineId))];
      const signature = [...loopLineIds].sort((a, b) => String(a).localeCompare(String(b))).join('|');
      if (existing.has(signature) || seen.has(signature)) return;
      if (loopLineIds.every(id => claimedLineIds.has(id))) return;
      const area = window.Geometry.calculateArea(loop.points, loop.edges);
      if (area < this.minimumClosedArea()) return;
      seen.add(signature);
      candidates.push({ loop, lineIds: new Set(loopLineIds), signature, area });
    });

    // The wall graph also exposes outer rings made by combining adjacent
    // faces. Select candidates before mutating polygons, and discard a larger
    // candidate only when it both contains a smaller ring and reuses some of
    // that ring's boundary walls. Disconnected nested areas share no walls and
    // remain independently valid.
    const sharesCandidateBoundary = (first, second) => {
      for (const id of second.lineIds) if (first.lineIds.has(id)) return true;
      return false;
    };
    const sharesPolygonBoundary = (candidate, poly) => (poly.edges || [])
      .some(edge => candidate.lineIds.has(edge.lineId));
    const acceptedCandidates = candidates.filter(candidate => {
      const overlapsIntactFace = intactPolygons.some(poly => {
        if (!sharesPolygonBoundary(candidate, poly)) return false;
        // Do not create a composite around an existing face, or subdivide an
        // appraiser-defined intact area merely because an exterior divider was
        // drawn through it. Both would leave overlapping calculated areas.
        return window.Geometry.polygonContains(candidate.loop, poly) ||
          window.Geometry.polygonContains(poly, candidate.loop);
      });
      if (overlapsIntactFace) return false;
      return !candidates.some(other => other !== candidate &&
        other.area < candidate.area - 1e-6 && sharesCandidateBoundary(candidate, other) &&
        window.Geometry.polygonContains(candidate.loop, other.loop));
    }).sort((a, b) => a.area - b.area || String(a.signature).localeCompare(String(b.signature)));

    acceptedCandidates.forEach(candidate => {
        const { loop, signature, area } = candidate;
        const repairIndex = repairablePolygons.findIndex(poly => this.polygonRingsEqual(poly.points, loop.points));
        const repaired = repairIndex >= 0 ? repairablePolygons.splice(repairIndex, 1)[0] : null;
        if (repaired) {
          // Preserve the area identity, definition, overrides, and label
          // placement while replacing only its stale geometry relationship.
          repaired.points = loop.points.map(point => ({ ...point }));
          repaired.edges = loop.edges.map(edge => ({ ...edge }));
          repaired.gross = area;
          repaired.area = area;
          const layer = this.getLayerById(repaired.layerId);
          if (layer) {
            layer.areaPolyId = repaired.id;
            const edgeIds = new Set(repaired.edges.map(edge => edge.lineId));
            this.state.lines.forEach(line => { if (edgeIds.has(line.id)) line.layerId = layer.id; });
            this.state.activeLayerId = layer.id;
            this.ui.updateLayersPanel?.();
          } else {
            this.assignAreaToDedicatedLayer(repaired, { activate: true });
          }
        } else {
          const defined = this.state.defineFirst && this.state.selectedAreaType;
          this.state.polygons.push({
            points: loop.points.map(point => ({ ...point })),
            edges: loop.edges.map(edge => ({ ...edge })),
            type: defined ? this.state.selectedAreaType : 'UND',
            name: defined ? (this.state.selectedAreaName || this.state.selectedAreaType) : 'Undefined Area',
            gross: area, area, factorOverride: null,
            subtractMode: this.state.options?.autoSubtractOverlapped === false ? 'never' : 'auto',
            subtractTargetId: null, id: this.genId()
          });
          this.assignAreaToDedicatedLayer(this.state.polygons[this.state.polygons.length - 1], { activate: true });
        }
        existing.add(signature);
        created += 1;
    });
    if (created) this.ui.updateAreaActionStates?.();
    return created;
  }

  // After a selection move finishes, walls may now enclose an area (or an
  // overshoot may have been healed); run the same closing pass that endpoint
  // edits use.
  finalizeSelectionMove() {
    const created = this.closeJoinedExteriorLoops();
    this.recalculate();
    this.ui.updateCalcsPanel();
    this.ui.updateAreaActionStates?.();
    this.render();
    if (created) this.ui.notify(`${created === 1 ? 'Area' : `${created} areas`} closed and ready to define.`);
    return created;
  }

  finishCommittedWall(lineId) {
    const line = this.state.lines.find(item => item.id === lineId);
    if (!line || line.type !== 'exterior') return 0;
    const created = this.closeJoinedExteriorLoops();
    if (!created) return 0;

    // An older imported loop may be discovered while the appraiser is drawing
    // somewhere else. Recalculate that area, but do not terminate the active
    // run unless the wall just committed belongs to one of the closed rings.
    const committedWallClosed = this.state.polygons.some(poly =>
      (poly.edges || []).some(edge => edge.lineId === lineId) && this.isPolygonBoundaryIntact(poly));
    if (!committedWallClosed) {
      this.recalculate();
      this.ui.updateCalcsPanel();
      this.ui.updateAreaActionStates?.();
      this.ui.notify(`${created === 1 ? 'An area was' : `${created} areas were`} detected elsewhere in the sketch.`);
      this.render();
      return 0;
    }

    // A replacement wall can close a pre-existing three-sided component with
    // only one newly drawn segment. End that short drawing chain just as a
    // normal polygon auto-close does, and make the recreated UND area
    // immediately available for selection and definition.
    this.state.penDown = false;
    this.state.keyboardPointer = false;
    this.state.keyboardPointerAnchor = null;
    this.state.currentPolyPoints = [];
    this.state.currentLineIds = [];
    this.canvas.setCursor();
    this.recalculate();
    this.ui.updateHUD('', '', false);
    this.ui.updateCalcsPanel();
    this.ui.updateAreaActionStates?.();
    this.ui.notify(`${created === 1 ? 'Area' : `${created} areas`} closed and ready to define.`);
    this.render();
    return created;
  }

  finalizeLineEndpointEdit(lineId) {
    const created = this.closeJoinedExteriorLoops();
    this.recalculate();
    this.ui.updateCalcsPanel();
    this.ui.updateAreaActionStates?.();
    this.render();
    if (created) this.ui.notify(`${created === 1 ? 'Area' : `${created} areas`} closed and ready to define.`);
    return created;
  }

  setSelectedLineLength(length) {
    const line = this.state.lines.find(item => this.state.selectedLineIds.has(item.id));
    const nextLength = Number(length);
    if (!line || !Number.isFinite(nextLength) || nextLength <= 0) return false;
    const currentLength = window.Geometry.distance(line.start, line.end);
    if (currentLength < 1e-8) return false;
    const endpoint = this.state.activeLineEndpoint?.lineId === line.id
      ? this.state.activeLineEndpoint.endpoint : 'end';
    const fixed = endpoint === 'start' ? line.end : line.start;
    const moving = line[endpoint];
    const nextPoint = {
      x: fixed.x + (moving.x - fixed.x) / currentLength * nextLength,
      y: fixed.y + (moving.y - fixed.y) / currentLength * nextLength
    };
    return this.moveLineEndpoint(line.id, endpoint, nextPoint, true);
  }

  setSelectedLineAppearance(properties) {
    const selected = this.state.lines.filter(line => this.state.selectedLineIds.has(line.id));
    if (!selected.length) return;
    this.pushUndo('Wall Appearance');
    selected.forEach(line => Object.assign(line, properties));
    this.render();
  }

  addWallShape(shape, wallType = 'exterior', settings = {}) {
    const centerScreen = { x: this.canvas.viewWidth / 2, y: this.canvas.viewHeight / 2 };
    const center = window.Geometry.toWorld(centerScreen, this.state.pan, this.state.zoom);
    const width = Math.max(.5, Number(settings.width) || 20);
    const height = Math.max(.5, Number(settings.height) || width);
    const sides = shape === 'circle' ? Math.max(24, Number(settings.sides) || 36)
      : shape === 'polygon' ? Math.max(3, Math.min(24, Number(settings.sides) || 8)) : 4;
    let points;
    if (shape === 'rectangle') {
      points = [
        { x:center.x-width/2, y:center.y-height/2 }, { x:center.x+width/2, y:center.y-height/2 },
        { x:center.x+width/2, y:center.y+height/2 }, { x:center.x-width/2, y:center.y+height/2 }
      ];
    } else {
      const rx = width / 2, ry = height / 2;
      points = Array.from({ length:sides }, (_, index) => {
        const angle = -Math.PI / 2 + index * Math.PI * 2 / sides;
        return { x:center.x + Math.cos(angle)*rx, y:center.y + Math.sin(angle)*ry };
      });
    }
    this.pushUndo('Wall Shape');
    const lineStyle = this.getDefaultLineStyle(wallType);
    const lines = points.map((point, index) => ({
      start:{ ...point }, end:{ ...points[(index + 1) % points.length] }, type:wallType,
      color:lineStyle.color || this.state.drawColor, lineThickness:lineStyle.lineThickness,
      layerId:this.state.activeLayerId, id:this.genId()
    }));
    this.state.lines.push(...lines);
    if (wallType === 'exterior') {
      const edges = lines.map(line => ({ lineId:line.id, bowAngle:0 }));
      const area = window.Geometry.calculateArea(points, edges);
      this.state.polygons.push({ points:points.map(point => ({ ...point })), edges,
        layerId:this.state.activeLayerId,
        type:this.state.defineFirst && this.state.selectedAreaType ? this.state.selectedAreaType : 'UND',
        name:this.state.defineFirst && this.state.selectedAreaType ? (this.state.selectedAreaName || this.state.selectedAreaType) : 'Undefined Area',
        gross:area, area, factorOverride:null, subtractMode:this.state.options?.autoSubtractOverlapped === false ? 'never' : 'auto',
        subtractTargetId:null, id:this.genId() });
      this.assignAreaToDedicatedLayer(this.state.polygons[this.state.polygons.length - 1], { activate: true });
      this.recalcAutoSubtract();
      this.ui.updateCalcsPanel();
    }
    this.render();
  }

  adjustSelectedCurves(delta) {
    const selected = this.state.lines.filter(line => this.state.selectedLineIds.has(line.id));
    if (selected.length === 0) return false;
    this.pushUndo('Curve Wall');
    selected.forEach(line => {
      let bowAngle;
      if (this.state.curveMode === 'height') {
        const chord = window.Geometry.distance(line.start, line.end);
        const arcHeight = (line.arcHeight || 0) + delta;
        bowAngle = window.Geometry.clampBowAngle(
          4 * Math.atan2(2 * arcHeight, chord) * 180 / Math.PI);
      } else {
        bowAngle = window.Geometry.clampBowAngle((line.bowAngle || 0) + delta);
      }
      this.commitCurveBow(line.id, bowAngle, false);
    });
    this.recalculate();
    this.ui.updateCalcsPanel();
    this.render();
    return true;
  }

  previewCurveBow(lineId, bowAngle) {
    const line = this.state.lines.find(item => item.id === lineId);
    if (!line) return false;
    this.commitCurveBow(lineId, bowAngle, false);
    // Curve dragging is a live geometry edit: update gross/net/GLA and the
    // calculation panel on every pointer move, not only after mouse-up.
    this.recalculate();
    this.ui.updateCalcsPanel();
    return true;
  }

  commitCurveBow(lineId, bowAngle, shouldRecalculate = true) {
    const clamped = window.Geometry.clampBowAngle(bowAngle);
    const syncPage = page => {
      const line = (page.lines || []).find(item => item.id === lineId);
      if (!line) return false;
      line.bowAngle = clamped;
      const chord = window.Geometry.distance(line.start, line.end);
      line.arcHeight = chord / 2 * Math.tan(clamped * Math.PI / 720);
      (page.polygons || []).forEach(poly => {
        if (!Array.isArray(poly.edges)) return;
        poly.edges.forEach((edge, index) => {
          if (edge.lineId !== lineId) return;
          const start = poly.points[index];
          const followsLine = start && this.pointsEqual(start, line.start);
          edge.bowAngle = followsLine ? clamped : -clamped;
        });
      });
      return true;
    };

    const activeChanged = syncPage(this.state);
    this.pages.forEach((page, index) => {
      if (index !== this.currentPage) syncPage(page);
    });
    if (shouldRecalculate) {
      if (activeChanged) {
        this.recalculate();
        this.ui.updateCalcsPanel();
        this.render();
      } else {
        this.recalculateSavedPages();
      }
    }
    return clamped;
  }

  // Compatibility entry point retained for older UI paths.
  recalcAutoSubtract() {
    return this.recalculate();
  }

  // Polygon copy prepared for area calculation: when a face measure mode is
  // active, the ring is offset to that face (bows reduced to chords).
  faceMeasurePolygon(poly, lines) {
    const offset = this.measureToOffsetRing(poly, lines);
    if (!offset) return poly;
    return { ...poly, points: offset, edges: (poly.edges || []).map(edge => ({ ...edge, bowAngle: 0 })) };
  }

  /* BSMT = finished below-grade area, BSMT2 = unfinished. Both are excluded
     from GLA but both count toward the 1004's basement line, so the total is
     the sum and the finished portion is BSMT alone. */
  basementRollupFrom(totalsByCode) { return basementRollupFromTotals(totalsByCode); }

  recalculate() {
    const originals = this.state.polygons;
    const input = originals.map(poly => this.faceMeasurePolygon(poly));
    const result = window.AreaCalc.recalculate(
      input.map(poly => ({ ...poly, page: this.currentPage })),
      this.areaCodes
    );
    // Areas may be measured to a wall face, but the stored geometry stays on
    // the centerline ring so it keeps following its walls.
    this.state.polygons = result.polygons.map((poly, index) => ({
      ...poly, points: originals[index].points, edges: originals[index].edges, page: this.currentPage
    }));
    this.areaTotals = {
      totalsByCode: result.totalsByCode,
      totalsByGroup: result.totalsByGroup,
      totalGLA: result.totalGLA,
      totalAllAreas: result.totalAllAreas
    };
    /* Completeness scheduling reads this: it changes only on a real area
       recalculation, so pointer movement and panning cannot make the suite
       think the GLA moved and trigger a validation pass. */
    this.areaCalcRevision = (this.areaCalcRevision || 0) + 1;
    /* Tell the suite the GLA total was recalculated. Emitted only here, at the
       genuine commit point, so drawing strokes never reach the painter.
       Below-grade area rides the same event: BSMT is finished basement and
       BSMT2 unfinished, so the sketch - not an assessor or MLS import - is
       the source for the Sales Grid's basement line when a sketch exists. */
    try {
      if (window.parent && window.parent !== window && window.parent.dispatchEvent) {
        const bsmtRollup = window.app && window.app.basementRollupFrom
          ? window.app.basementRollupFrom(result.totalsByCode)
          : basementRollupFromTotals(result.totalsByCode);
        window.parent.dispatchEvent(new CustomEvent('suite:sketchrecalculated',
          { detail: { totalGLA: result.totalGLA, revision: this.areaCalcRevision,
            basementTotal: bsmtRollup.total, basementFinished: bsmtRollup.finished } }));
      }
    } catch (_) {}
    if (result.warnings.length && this.ui) this.ui.notify(result.warnings[0]);
    this.assertAreaInvariants();
    return result;
  }

  assertAreaInvariants() {
    if (!this.debugAreaCalculations) return true;
    const byLineId = new Map(this.state.lines.map(line => [line.id, line]));
    this.state.polygons.forEach(poly => {
      if (poly.edges.length !== poly.points.length) throw new Error(`Area ${poly.id}: edge/point count mismatch`);
      poly.edges.forEach((edge, index) => {
        const line = byLineId.get(edge.lineId);
        if (!line) throw new Error(`Area ${poly.id}: missing line ${edge.lineId}`);
        const expected = this.pointsEqual(poly.points[index], line.start) ? (line.bowAngle || 0) : -(line.bowAngle || 0);
        if (Math.abs(expected - edge.bowAngle) > 1e-6) throw new Error(`Area ${poly.id}: stale bow metadata`);
      });
      const detail = this.breakdown.computeForPolygon(poly);
      const sum = detail.pieces.reduce((total, piece) => total + piece.area, 0);
      if (Math.abs(sum - poly.gross) >= 0.01) throw new Error(`Area ${poly.id}: breakdown does not reconcile`);
    });
    return true;
  }

  buildPolygonEdges(poly, lines) {
    if (!poly || !Array.isArray(poly.points)) return [];
    const available = lines || this.state.lines;
    return poly.points.map((start, index) => {
      const end = poly.points[(index + 1) % poly.points.length];
      const line = available.find(item =>
        (this.pointsEqual(item.start, start) && this.pointsEqual(item.end, end)) ||
        (this.pointsEqual(item.start, end) && this.pointsEqual(item.end, start)));
      if (!line) return { lineId: null, bowAngle: 0 };
      const forward = this.pointsEqual(line.start, start);
      return { lineId: line.id, bowAngle: forward ? (line.bowAngle || 0) : -(line.bowAngle || 0) };
    });
  }

  ensurePageModel(page, pageIndex = 0) {
    const normalized = {
      lines: page && Array.isArray(page.lines) ? page.lines : [],
      polygons: page && Array.isArray(page.polygons) ? page.polygons : [],
      labels: page && Array.isArray(page.labels) ? page.labels : [],
      symbols: page && Array.isArray(page.symbols) ? page.symbols : [],
      northAngle: this.normalizeNorthAngle(page && page.northAngle),
      dimensionChains: page && Array.isArray(page.dimensionChains) ? page.dimensionChains : []
    };
    normalized.lines.forEach(line => {
      // One-time migration: a dimension label pinned before pins carried an
      // anchor keeps its saved spot by anchoring it to the wall as saved.
      if (line.dimensionPosition && !line.dimensionAnchor) {
        line.dimensionAnchor = { x: (line.start.x + line.end.x) / 2, y: (line.start.y + line.end.y) / 2 };
      }
      line.bowAngle = window.Geometry.clampBowAngle(line.bowAngle);
      if (line.bowAngle) {
        const chord = window.Geometry.distance(line.start, line.end);
        line.arcHeight = chord / 2 * Math.tan(line.bowAngle * Math.PI / 720);
      }
    });
    normalized.polygons.forEach(poly => {
      if (!Array.isArray(poly.edges) || poly.edges.length !== (poly.points || []).length) {
        poly.edges = this.buildPolygonEdges(poly, normalized.lines);
      }
      poly.edges = (poly.points || []).map((_, index) => ({
        lineId: poly.edges[index] ? poly.edges[index].lineId : null,
        bowAngle: window.Geometry.clampBowAngle(poly.edges[index] && poly.edges[index].bowAngle)
      }));
      if (poly.factorOverride === undefined) {
        poly.factorOverride = Number.isFinite(poly.factor) && poly.factor !== 1 ? poly.factor : null;
      }
      poly.subtractMode = ['auto', 'always', 'never'].includes(poly.subtractMode) ? poly.subtractMode : 'auto';
      poly.subtractTargetId = poly.subtractTargetId ?? null;
      poly.page = pageIndex;
    });
    // Repair hosted exterior openings saved with the old outward-swing snap:
    // recompute the side flip (and rotation) from the page geometry. This is
    // a one-time migration. Openings placed or re-snapped since then carry
    // wallSideExplicit, so an appraiser's chosen swing (an out-swinging door)
    // survives undo, page switches and reopening instead of being reset.
    const linesById = new Map(normalized.lines.map(line => [line.id, line]));
    normalized.symbols.forEach(symbol => {
      if (!symbol.wallId || symbol.wallFraction == null) return;
      if (symbol.wallSideExplicit) return;
      symbol.wallSideExplicit = true;
      const snap = symbol.wallSnap ||
        window.SKETCH_SYMBOL_LIBRARY?.symbols?.[symbol.symbolType]?.wallSnap;
      if (snap !== 'SIDE') return;
      const line = linesById.get(symbol.wallId);
      if (!line || line.type !== 'exterior') return;
      const flip = this.exteriorSwingFlip(line, normalized.polygons);
      symbol.wallSideFlip = flip;
      if (symbol.wallGapWidth == null) {
        const defGap = Number(window.SKETCH_SYMBOL_LIBRARY?.symbols?.[symbol.symbolType]?.wallGapWidth) || 0;
        if (defGap > 0) symbol.wallGapWidth = defGap / 12;
      }
      const dx = line.end.x - line.start.x;
      const dy = line.end.y - line.start.y;
      if (Math.hypot(dx, dy) > 1e-6) {
        symbol.rotation = -Math.atan2(dy, dx) * 180 / Math.PI + (flip ? 180 : 0);
      }
    });
    return normalized;
  }

  recalculateSavedPages() {
    this.saveCurrentPage();
    this.pages = this.pages.map((page, index) => {
      const normalized = this.ensurePageModel(page, index);
      const input = normalized.polygons.map(poly => this.faceMeasurePolygon(poly, normalized.lines));
      const result = window.AreaCalc.recalculate(input, this.areaCodes);
      return { ...normalized, polygons: result.polygons.map((poly, i) => ({
        ...poly, points: normalized.polygons[i].points, edges: normalized.polygons[i].edges, page: index
      })) };
    });
    const active = this.pages[this.currentPage];
    this.state.lines = JSON.parse(JSON.stringify(active.lines));
    this.state.polygons = JSON.parse(JSON.stringify(active.polygons));
    this.state.labels = JSON.parse(JSON.stringify(active.labels));
    this.state.symbols = JSON.parse(JSON.stringify(active.symbols));
    this.recalculate();
  }

  setAreaCodes(codes) {
    this.areaCodes = window.AreaCodes.validateAreaCodes(codes);
    window.AREA_CODES = window.AreaCodes.cloneAreaCodes(this.areaCodes);
    this.breakdown.codeTable = this.areaCodes;
    this.recalculateSavedPages();
    this.ui.populateAreaCodes();
    this.ui.updateCalcsPanel();
    this.render();
  }

  buildExportModel() {
    this.recalculateSavedPages();
    const subjectInfo = typeof this.state.subjectInfo === 'object' && this.state.subjectInfo
      ? this.state.subjectInfo
      : { generalComments: String(this.state.subjectInfo || '') };
    return {
      name: this.fileName || 'Untitled',
      subject: subjectInfo.generalComments || subjectInfo.propertyAddress || '',
      subjectInfo,
      layout: JSON.parse(JSON.stringify(this.normalizePageLayout(this.state.pageLayout))),
      options: JSON.parse(JSON.stringify(this.state.options)),
      commentTables: Array.from({ length: 3 }, (_, index) => {
        const table = subjectInfo.commentTables?.[index];
        return Array.isArray(table) ? table.slice(0, 10) : String(table || '').split(/\r?\n/).slice(0, 10);
      }).map(table => [...table, ...Array(Math.max(0, 10 - table.length)).fill('')]),
      pages: this.pages.map((page, index) => ({
        index,
        northAngle: this.normalizeNorthAngle(page.northAngle),
        lines: JSON.parse(JSON.stringify((page.lines || []).filter(item => this.isEntityVisible(item)))),
        polygons: page.polygons.filter(poly => this.isEntityVisible(poly)).map(poly => ({ ...JSON.parse(JSON.stringify(poly)), page: index })),
        labels: JSON.parse(JSON.stringify((page.labels || []).filter(item => this.isEntityVisible(item)))),
        symbols: JSON.parse(JSON.stringify((page.symbols || []).filter(item => this.isEntityVisible(item)))),
        dimensionChains: JSON.parse(JSON.stringify(page.dimensionChains || []))
      }))
    };
  }

  getMaxLiveId(pages = this.pages) {
    let maximum = 0;
    pages.forEach(page => ['lines', 'polygons', 'labels', 'symbols'].forEach(key => {
      (page[key] || []).forEach(entity => { maximum = Math.max(maximum, Number(entity.id) || 0); });
    }));
    return maximum;
  }

  migratePageIds(pages) {
    let next = 1;
    return pages.map((rawPage, pageIndex) => {
      const page = this.ensurePageModel(JSON.parse(JSON.stringify(rawPage || {})), pageIndex);
      const idMap = new Map();
      ['lines', 'polygons', 'labels', 'symbols'].forEach(key => {
        page[key].forEach(entity => {
          const oldId = entity.id;
          const newId = next++;
          idMap.set(oldId, newId);
          entity.id = newId;
        });
      });
      page.polygons.forEach(poly => {
        poly.edges.forEach(edge => { edge.lineId = idMap.get(edge.lineId) ?? null; });
        poly.containerId = idMap.get(poly.containerId) ?? null;
        poly.subtractedFromId = idMap.get(poly.subtractedFromId) ?? null;
        poly.subtractTargetId = idMap.get(poly.subtractTargetId) ?? null;
      });
      return page;
    });
  }

  // --- Zoom ---
  zoomTo(newZoom, centerX, centerY) {
    const oldZoom = this.state.zoom;
    newZoom = Math.max(0.02, Math.min(10, newZoom));
    this.state.pan.x = centerX - (centerX - this.state.pan.x) * (newZoom / oldZoom);
    this.state.pan.y = centerY - (centerY - this.state.pan.y) * (newZoom / oldZoom);
    this.state.zoom = newZoom;
    this.ui.updateZoomDisplay();
    this.render();
  }

  // Zoom that 100% means at the current 1:N drawing scale (1:10 is 1.0).
  drawingScaleFactor() {
    const scale = Number(this.state.drawingScale) || 10;
    return 10 / scale;
  }

  setDrawingScale(scale) {
    let next = Number(scale);
    if (!Number.isFinite(next) || next <= 0) { this.ui.updateZoomDisplay(); return; }
    // 1:1 through 1:100 covers residential sketches and keeps every scale
    // inside the zoom range, so the percentage readout stays truthful.
    next = Math.max(1, Math.min(100, next));
    const previous = this.drawingScaleFactor();
    this.state.drawingScale = next;
    const cx = this.canvas.viewWidth / 2;
    const cy = this.canvas.viewHeight / 2;
    this.zoomTo(this.state.zoom * this.drawingScaleFactor() / previous, cx, cy);
  }

  zoomIn() {
    const cx = this.canvas.viewWidth / 2;
    const cy = this.canvas.viewHeight / 2;
    this.zoomTo(this.state.zoom * 1.2, cx, cy);
  }

  zoomOut() {
    const cx = this.canvas.viewWidth / 2;
    const cy = this.canvas.viewHeight / 2;
    this.zoomTo(this.state.zoom / 1.2, cx, cy);
  }

  zoomToWorldRect(worldMin, worldMax) {
    const width = worldMax.x - worldMin.x;
    const height = worldMax.y - worldMin.y;
    if (width <= 0 || height <= 0) return;
    const margin = 30;
    const zoomX = (this.canvas.viewWidth - margin * 2) / (width * window.Geometry.PIXELS_PER_FOOT);
    const zoomY = (this.canvas.viewHeight - margin * 2) / (height * window.Geometry.PIXELS_PER_FOOT);
    this.state.zoom = Math.max(0.1, Math.min(10, Math.min(zoomX, zoomY)));
    const centerX = (worldMin.x + worldMax.x) / 2;
    const centerY = (worldMin.y + worldMax.y) / 2;
    this.state.pan.x = this.canvas.viewWidth / 2 - centerX * window.Geometry.PIXELS_PER_FOOT * this.state.zoom;
    this.state.pan.y = this.canvas.viewHeight / 2 + centerY * window.Geometry.PIXELS_PER_FOOT * this.state.zoom;
    this.ui.updateZoomDisplay();
    this.setMode('select');
  }

  centerSketch() {
    if (this.state.lines.length === 0) return;
    const bounds = this.getSketchBounds();
    const cx = (bounds.minX + bounds.maxX) / 2;
    const cy = (bounds.minY + bounds.maxY) / 2;
    // Pan is expressed in CSS pixels; use the view size, not the DPR-scaled
    // backing store, or centering shifts off-screen on high-DPI displays.
    this.state.pan.x = this.canvas.viewWidth / 2 - cx * window.Geometry.PIXELS_PER_FOOT * this.state.zoom;
    this.state.pan.y = this.canvas.viewHeight / 2 + cy * window.Geometry.PIXELS_PER_FOOT * this.state.zoom;
    this.render();
  }

  centerAndFit() {
    if (this.state.lines.length === 0) return;
    const bounds = this.getSketchBounds();
    const worldW = bounds.maxX - bounds.minX;
    const worldH = bounds.maxY - bounds.minY;
    if (worldW === 0 && worldH === 0) return;
    const margin = 30;
    // Use the CSS-pixel view size, not the DPR-scaled backing store.
    const zoomX = (this.canvas.viewWidth - margin * 2) / (Math.max(worldW, 1) * window.Geometry.PIXELS_PER_FOOT);
    const zoomY = (this.canvas.viewHeight - margin * 2) / (Math.max(worldH, 1) * window.Geometry.PIXELS_PER_FOOT);
    this.state.zoom = Math.max(0.1, Math.min(10, Math.min(zoomX, zoomY)));
    const cx = (bounds.minX + bounds.maxX) / 2;
    const cy = (bounds.minY + bounds.maxY) / 2;
    this.state.pan.x = this.canvas.viewWidth / 2 - cx * window.Geometry.PIXELS_PER_FOOT * this.state.zoom;
    this.state.pan.y = this.canvas.viewHeight / 2 + cy * window.Geometry.PIXELS_PER_FOOT * this.state.zoom;
    this.ui.updateZoomDisplay();
    this.render();
  }

  getSketchBounds() {
    let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;
    this.state.lines.forEach(l => {
      [l.start, l.end].forEach(p => {
        minX = Math.min(minX, p.x); maxX = Math.max(maxX, p.x);
        minY = Math.min(minY, p.y); maxY = Math.max(maxY, p.y);
      });
    });
    return { minX, maxX, minY, maxY };
  }

  // --- Undo / Redo ---

  // Clone the mutable model into an undo/redo snapshot. An optional label
  // describes the upcoming mutation for the undo history panel.
  snapshotModel(label = 'Edit') {
    return {
      label,
      lines: JSON.parse(JSON.stringify(this.state.lines)),
      polygons: JSON.parse(JSON.stringify(this.state.polygons)),
      labels: JSON.parse(JSON.stringify(this.state.labels)),
      symbols: JSON.parse(JSON.stringify(this.state.symbols)),
      layers: JSON.parse(JSON.stringify(this.state.layers)),
      activeLayerId: this.state.activeLayerId,
      northAngle: this.state.northAngle,
      dimensionChains: JSON.parse(JSON.stringify(this.state.dimensionChains)),
      cursor: { ...this.state.cursor },
      currentPolyPoints: JSON.parse(JSON.stringify(this.state.currentPolyPoints)),
      currentLineIds: [...this.state.currentLineIds],
      penDown: this.state.penDown
    };
  }

  pushUndo(label = 'Edit') {
    this.state.undoStack.push(this.snapshotModel(label));
    this.state.redoStack = [];
    this.ui.updateUndoRedoControls();
  }

  // Undo/redo step back along a wall chain only while it is still being
  // drawn. Once the chain was finished (pen up), restoring the snapshot's
  // pen-down state left an invisible chain start behind, so the next click
  // drew a stray wall from that old point.
  restoreDrawingChain(snapshot) {
    this.state.cursor = snapshot.cursor;
    if (this.state.penDown && snapshot.penDown) {
      this.state.currentPolyPoints = snapshot.currentPolyPoints;
      this.state.currentLineIds = snapshot.currentLineIds || [];
      this.state.penDown = true;
      return;
    }
    this.state.penDown = false;
    this.state.keyboardPointer = false;
    this.state.keyboardPointerAnchor = null;
    this.state.currentPolyPoints = [];
    this.state.currentLineIds = [];
  }

  undo() {
    if (this.state.undoStack.length === 0) return;
    // The redo entry describes the action it will replay.
    const currentLabel = this.state.undoStack[this.state.undoStack.length - 1]?.label || 'Edit';
    this.state.redoStack.push(this.snapshotModel(currentLabel));
    const prev = this.state.undoStack.pop();
    if (prev.layers) {
      this.state.layers = prev.layers;
      this.state.activeLayerId = prev.activeLayerId || this.getDefaultLayerId();
      this.ui.updateLayersPanel?.();
    }
    this.state.lines = prev.lines;
    this.state.polygons = prev.polygons;
    this.state.labels = prev.labels || [];
    this.state.symbols = prev.symbols || [];
    this.state.dimensionChains = prev.dimensionChains || [];
    this.state.northAngle = Number.isFinite(prev.northAngle) ? prev.northAngle : this.state.northAngle;
    this.restoreDrawingChain(prev);
    this.clearSelection();
    this.recalculate();
    this.ui.updateUndoRedoControls();
    this.ui.updateNorthDisplay?.();
    this.ui.updateHUD('', '', this.state.penDown);
    this.ui.updateCalcsPanel();
    this.render();
  }

  redo() {
    if (this.state.redoStack.length === 0) return;
    const currentLabel = this.state.redoStack[this.state.redoStack.length - 1]?.label || 'Edit';
    this.state.undoStack.push(this.snapshotModel(currentLabel));
    const next = this.state.redoStack.pop();
    if (next.layers) {
      this.state.layers = next.layers;
      this.state.activeLayerId = next.activeLayerId || this.getDefaultLayerId();
      this.ui.updateLayersPanel?.();
    }
    this.state.lines = next.lines;
    this.state.polygons = next.polygons;
    this.state.labels = next.labels || [];
    this.state.symbols = next.symbols || [];
    this.state.dimensionChains = next.dimensionChains || [];
    this.state.northAngle = Number.isFinite(next.northAngle) ? next.northAngle : this.state.northAngle;
    this.restoreDrawingChain(next);
    this.clearSelection();
    this.recalculate();
    this.ui.updateUndoRedoControls();
    this.ui.updateNorthDisplay?.();
    this.ui.updateHUD('', '', this.state.penDown);
    this.ui.updateCalcsPanel();
    this.render();
  }

  // Rewind to the snapshot at the given undo-stack index (undo history panel).
  // Each snapshot records the state BEFORE its action, so applying entry i
  // means undoing until entry i itself is popped and restored.
  undoTo(index) {
    const target = Math.max(0, Math.min(index, this.state.undoStack.length - 1));
    let guard = this.state.undoStack.length + 1;
    while (this.state.undoStack.length > target && guard-- > 0) this.undo();
  }

  // --- Reference image ---

  loadBgImage(file) {
    const reader = new FileReader();
    reader.onload = (e) => {
      const img = new Image();
      img.onload = () => {
        this.bgImage.img = img;
        this.bgImage.visible = true;
        this.bgImage.x = 0;
        this.bgImage.y = 0;
        // Auto-scale: fit image to ~60 feet wide by default
        const PPF = window.Geometry.PIXELS_PER_FOOT;
        this.bgImage.scale = (60 * PPF) / img.width;
        this.bgImage.rotation = 0;
        this.bgImage.flipH = false;
        this.bgImage.flipV = false;
        this.bgImage.opacity = 0.5;
        const slider = document.getElementById('photo-intensity');
        if (slider) slider.value = 50;
        this.render();
      };
      img.src = e.target.result;
    };
    reader.readAsDataURL(file);
  }

  async showBgImage() {
    if (!this.bgImage.img) { await this.ui.alertDialog('No image loaded. Use Load Image first.'); return; }
    this.bgImage.visible = true;
    this.render();
  }

  hideBgImage() {
    this.bgImage.visible = false;
    this.render();
  }

  async removeBgImage() {
    if (!this.bgImage.img) return;
    if (!(await this.ui.confirmDialog('Remove the background image?'))) return;
    this.bgImage.img = null;
    this.bgImage.visible = false;
    this.bgImage.scaleMode = false;
    this.bgImage.scaleP1 = null;
    this.bgImage.scaleP2 = null;
    this.render();
  }

  rotateBgImage(degrees) {
    if (!this.bgImage.img) return;
    this.bgImage.rotation = (this.bgImage.rotation + degrees) % 360;
    this.render();
  }

  flipBgImageH() {
    if (!this.bgImage.img) return;
    this.bgImage.flipH = !this.bgImage.flipH;
    this.render();
  }

  flipBgImageV() {
    if (!this.bgImage.img) return;
    this.bgImage.flipV = !this.bgImage.flipV;
    this.render();
  }

  scaleBgImageBy(factor) {
    if (!this.bgImage.img) return;
    this.bgImage.scale *= factor;
    this.render();
  }

  setBgIntensity(value) {
    this.bgImage.opacity = value / 100;
    this.render();
  }

  async startSetScale() {
    if (!this.bgImage.img) { await this.ui.alertDialog('No image loaded.'); return; }
    this.bgImage.scaleMode = true;
    this.bgImage.scaleP1 = null;
    this.bgImage.scaleP2 = null;
    this.setMode('select');
    await this.ui.alertDialog('Click two points on the image that represent a known distance, then enter the measurement.');
  }

  togglePhotoEditMode() {
    if (!this.bgImage.img) {
      this.ui.notify('Load an image before entering Image Edit Mode.');
      return;
    }
    this.bgImage.editMode = !this.bgImage.editMode;
    this.bgImage.orientMode = false;
    document.getElementById('btn-photo-edit-mode')?.classList.toggle('active', this.bgImage.editMode);
    this.ui.notify(this.bgImage.editMode
      ? 'Image Edit Mode: drag the image to reposition it; use the wheel to resize.'
      : 'Image Edit Mode off.');
    this.render();
  }

  startPhotoOrient() {
    if (!this.bgImage.img) {
      this.ui.notify('Load an image before using Orient Guide.');
      return;
    }
    this.bgImage.editMode = false;
    this.bgImage.orientMode = true;
    this.bgImage.orientP1 = null;
    this.bgImage.orientP2 = null;
    document.getElementById('btn-photo-edit-mode')?.classList.remove('active');
    this.setMode('select');
    this.ui.notify('Orient Guide: click two points along a line in the image.');
    this.render();
  }

  handleOrientClick(worldPt) {
    if (!this.bgImage.orientMode) return false;
    if (!this.bgImage.orientP1) {
      this.bgImage.orientP1 = { ...worldPt };
      this.render();
      return true;
    }

    this.bgImage.orientP2 = { ...worldPt };
    const dx = this.bgImage.orientP2.x - this.bgImage.orientP1.x;
    const dy = this.bgImage.orientP2.y - this.bgImage.orientP1.y;
    const currentAngle = Math.atan2(dy, dx) * 180 / Math.PI;
    this.bgImage.orientMode = false;
    this.bgImage.orientP1 = null;
    this.bgImage.orientP2 = null;
    this.ui.promptDialog('Orient this guide Horizontal or Vertical?', 'Horizontal').then(orientation => {
      const value = (orientation || '').trim().toLowerCase();
      if (value.startsWith('v')) this.bgImage.rotation += 90 - currentAngle;
      else this.bgImage.rotation -= currentAngle;
      this.ui.notify('Image orientation applied.');
      this.render();
    });
    return true;
  }

  async handleScaleClick(worldPt) {
    if (!this.bgImage.scaleMode) return false;

    if (!this.bgImage.scaleP1) {
      this.bgImage.scaleP1 = { ...worldPt };
      this.render();
      return true;
    }

    this.bgImage.scaleP2 = { ...worldPt };
    this.render();

    // Calculate pixel distance on image between the two points
    const dx = this.bgImage.scaleP2.x - this.bgImage.scaleP1.x;
    const dy = this.bgImage.scaleP2.y - this.bgImage.scaleP1.y;
    const worldDist = Math.sqrt(dx * dx + dy * dy);

    const measurement = await this.ui.promptDialog('Enter the known distance between the two points (in feet):');
    if (!measurement || isNaN(parseFloat(measurement)) || parseFloat(measurement) <= 0) {
      this.bgImage.scaleMode = false;
      this.bgImage.scaleP1 = null;
      this.bgImage.scaleP2 = null;
      this.render();
      return true;
    }

    const knownFeet = parseFloat(measurement);
    // worldDist is in world units (feet). We need to adjust image scale so the
    // distance on the image matches knownFeet.
    // Current ratio: worldDist feet on screen = actual knownFeet
    // Scale factor adjustment:
    const scaleFactor = knownFeet / worldDist;
    this.bgImage.scale *= scaleFactor;

    this.bgImage.scaleMode = false;
    this.bgImage.scaleP1 = null;
    this.bgImage.scaleP2 = null;
    this.render();
    return true;
  }

  // --- Page Management ---

  getPageData() {
    return {
      lines: JSON.parse(JSON.stringify(this.state.lines)),
      polygons: JSON.parse(JSON.stringify(this.state.polygons)),
      labels: JSON.parse(JSON.stringify(this.state.labels)),
      symbols: JSON.parse(JSON.stringify(this.state.symbols)),
      northAngle: this.normalizeNorthAngle(this.state.northAngle),
      dimensionChains: JSON.parse(JSON.stringify(this.state.dimensionChains))
    };
  }

  loadPageData(data) {
    const page = this.ensurePageModel(JSON.parse(JSON.stringify(data || {})), this.currentPage);
    // New walls must never reuse an id already on this page: two walls with
    // one id corrupt each other on the next join or edit.
    this.nextId = Math.max(Number(this.nextId) || 1, this.getMaxLiveId([page]) + 1);
    this.state.lines = page.lines;
    this.state.polygons = page.polygons;
    this.state.labels = page.labels;
    this.state.symbols = page.symbols;
    this.state.northAngle = page.northAngle;
    this.state.dimensionChains = page.dimensionChains;
    this.ensureAreaLayersForCurrentPage();
    this.state.currentPolyPoints = [];
    this.state.currentLineIds = [];
    this.state.penDown = false;
    this.state.undoStack = [];
    this.state.redoStack = [];
    this.ui.updateUndoRedoControls();
    this.ui.updateNorthDisplay?.();
    this.clearSelection();
    this.recalculate();
    this.ui.updateCalcsPanel();
    this.render();
  }

  saveCurrentPage() {
    this.pages[this.currentPage] = this.getPageData();
  }

  goToPage(index) {
    if (index < 0 || index >= this.pages.length) return;
    this.saveCurrentPage();
    this.currentPage = index;
    this.loadPageData(this.pages[index]);
    if (this.underlayPageIndex === this.currentPage) this.underlayPageIndex = null;
    this.ui.updatePageDisplay();
  }

  // Underlay (ghost floor) support — view-only reference to another page.
  setUnderlay(index) {
    const parsed = Number(index);
    this.underlayPageIndex = Number.isInteger(parsed) && parsed >= 0 && parsed < this.pages.length && parsed !== this.currentPage
      ? parsed : null;
    const select = document.getElementById('underlay-select');
    if (select) select.value = String(this.underlayPageIndex == null ? -1 : this.underlayPageIndex);
    this.render();
  }

  getUnderlayPage() {
    if (this.underlayPageIndex == null || this.underlayPageIndex === this.currentPage) return null;
    return this.pages[this.underlayPageIndex] || null;
  }

  // --- Chained Dimensions ---

  // Live geometry for a chain record; spans are always recomputed from the
  // current walls so chains track edits automatically.
  computeChain(chain) {
    return window.Geometry.computeDimensionChain(this.state.lines, this.state.symbols, chain.seedLineId, 0.25, this.state.options);
  }

  // Wall ids covered by any chain. Render-time only: per-wall labels for
  // these walls are suppressed while the chain exists, never mutated.
  getChainCoverage() {
    const covered = new Set();
    this.state.dimensionChains.forEach(chain => {
      const computed = this.computeChain(chain);
      (computed?.lineIds || []).forEach(id => covered.add(id));
    });
    return covered;
  }

  handleChainClick(worldPt) {
    let best = null;
    let bestDist = this.getScreenToleranceWorld(8);
    this.state.lines.forEach(line => {
      if (Math.abs(Number(line.bowAngle) || 0) >= 0.5) return; // straight runs only (v1)
      const d = window.Geometry.distanceToSegment(worldPt, line.start, line.end);
      if (d < bestDist) { bestDist = d; best = line; }
    });
    if (!best) {
      this.ui.notify('Chain dimensions attach to straight walls.');
      return;
    }
    const dx = best.end.x - best.start.x;
    const dy = best.end.y - best.start.y;
    const cross = dx * (worldPt.y - best.start.y) - dy * (worldPt.x - best.start.x);
    this.pushUndo('Dimension Chain');
    this.state.dimensionChains.push({
      id: this.genId(),
      seedLineId: best.id,
      side: cross >= 0 ? 1 : -1,
      offset: Number(this.state.options.dimensionOption) || 3,
      showOverall: true,
      hiddenSpans: []
    });
    this.render();
  }

  addPage() {
    this.saveCurrentPage();
    this.currentPage += 1;
    this.pages.splice(this.currentPage, 0, {
      lines: [], polygons: [], labels: [], symbols: [], northAngle: 0, dimensionChains: []
    });
    // Keep the underlay reference pointed at the same page after the insert.
    if (this.underlayPageIndex != null && this.underlayPageIndex >= this.currentPage) this.underlayPageIndex += 1;
    this.loadPageData(this.pages[this.currentPage]);
    this.ui.updatePageDisplay();
  }

  async deletePage() {
    if (this.pages.length <= 1) {
      await this.ui.alertDialog('Cannot delete the last page.');
      return;
    }
    if (!(await this.ui.confirmDialog(`Delete page ${this.currentPage + 1}?`))) return;
    this.pages.splice(this.currentPage, 1);
    // Repoint or drop the underlay reference around the removed page.
    if (this.underlayPageIndex === this.currentPage) this.underlayPageIndex = null;
    else if (this.underlayPageIndex != null && this.underlayPageIndex > this.currentPage) this.underlayPageIndex -= 1;
    if (this.currentPage >= this.pages.length) {
      this.currentPage = this.pages.length - 1;
    }
    // Landing on the page that was the underlay must not leave it ghosting
    // itself (a later Add Page would bring the stale reference back).
    if (this.underlayPageIndex === this.currentPage) this.underlayPageIndex = null;
    this.loadPageData(this.pages[this.currentPage]);
    this.ui.updatePageDisplay();
  }

  // --- File Operations ---

  async newSketch() {
    if (!(await this.ui.confirmDialog('Start a new sketch? Unsaved changes will be lost.'))) return;
    this.resetToBlank();
    document.getElementById('file-menu').style.display = 'none';
    const submenu = document.getElementById('file-recent-submenu');
    if (submenu) submenu.style.display = 'none';
  }

  /* Drop every page and return to a blank drawing without prompting. Used by
     File > New and by suite workfiles that were saved without a sketch. */
  resetToBlank() {
    this.state.lines = [];
    this.state.polygons = [];
    this.state.labels = [];
    this.state.symbols = [];
    this.state.dimensionChains = [];
    this.state.currentPolyPoints = [];
    this.state.currentLineIds = [];
    this.state.undoStack = [];
    this.state.redoStack = [];
    this.ui.updateUndoRedoControls();
    this.state.penDown = false;
    this.state.cursor = { x: 0, y: 0 };
    this.state.northAngle = 0;
    this.state.subjectInfoAutoFilled = false;
    this.nextId = 1;
    this.ensureLayers();
    this.pages = [this.getPageData()];
    this.currentPage = 0;
    this.underlayPageIndex = null;
    this.fileName = 'Untitled';
    this.setMode('select');
    this.ui.updateCalcsPanel();
    this.ui.updatePageDisplay();
    this.updateTitle();
    this.pushSuitePreview();
  }

  /* True when any page carries drawable entities. An empty drawing has no
     print preview to place on the report sketch addendum page. */
  hasSketchContent() {
    return (this.pages || []).some(page =>
      (page.lines && page.lines.length) || (page.polygons && page.polygons.length) ||
      (page.labels && page.labels.length) || (page.symbols && page.symbols.length));
  }

  /* Rasterize each print-preview sheet (sketch body and calculations,
     below the subject header) into its own data URL. Keeping page boundaries
     here is what lets the 1004 and Appraise-It exporters emit repeatable
     Sketch sheets instead of shrinking a tall, stacked image onto one page.
     Each raster runs at 2x CSS resolution (192 DPI). */
  async renderSuitePreviews() {
    if (!window.SketchReport || !this.hasSketchContent()) return [];
    const model = this.buildExportModel();
    const layout = window.SketchReport.normalizeLayout(model.layout);
    const size = window.SketchReport.PAGE_SIZES[layout.paperSize] || window.SketchReport.PAGE_SIZES.letter;
    const pages = model.pages || [];
    if (!pages.length) return [];
    const renderScale = 2;
    const ptToPx = 4 / 3;
    const pageW = Math.round(size.width * ptToPx);
    const pageH = Math.round(size.height * ptToPx);
    const cropTop = Math.round(110.4 * ptToPx);
    const cropH = pageH - cropTop;
    const previews = [];
    for (const page of pages) {
      /* The outer report sheet owns pagination, so suppress the sketch
         renderer's internal page number inside the placed image. */
      const pageModel = { ...model, pages: [page], layout: { ...(model.layout || {}), showPageNumbers: false } };
      const html = window.SketchReport.render(pageModel, this.areaCodes);
      const wrapped = `<div xmlns="http://www.w3.org/1999/xhtml" style="width:${pageW}px;height:${pageH}px;overflow:hidden"><style>${window.SketchReport.REPORT_CSS}</style>${html}</div>`;
      const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${pageW * renderScale}" height="${pageH * renderScale}" viewBox="0 0 ${pageW} ${pageH}"><foreignObject x="0" y="0" width="${pageW}" height="${pageH}">${wrapped}</foreignObject></svg>`;
      const image = await new Promise((resolve, reject) => {
        const im = new Image();
        im.onload = () => resolve(im);
        im.onerror = () => reject(new Error('Sketch preview rasterization failed'));
        im.src = 'data:image/svg+xml;charset=utf-8,' + encodeURIComponent(svg);
      });
      const canvas = document.createElement('canvas');
      canvas.width = pageW * renderScale;
      canvas.height = cropH * renderScale;
      const ctx = canvas.getContext('2d');
      ctx.fillStyle = '#fff';
      ctx.fillRect(0, 0, canvas.width, canvas.height);
      ctx.drawImage(image, 0, cropTop * renderScale, pageW * renderScale, cropH * renderScale, 0, 0, pageW * renderScale, cropH * renderScale);
      previews.push(canvas.toDataURL('image/jpeg', 0.92));
    }
    return previews;
  }

  /* Legacy callers asked for one preview. Keep that read contract while new
     report consumers use renderSuitePreviews()/__exportPreviews. */
  async renderSuitePreview() {
    const previews = await this.renderSuitePreviews();
    return previews[0] || null;
  }

  /* Push the rendered preview (or a clear signal) to the suite so the 1004 and
     1004C sketch addendum pages place it whenever the sketch is saved. */
  async pushSuitePreview() {
    let previews = [];
    try { previews = await this.renderSuitePreviews(); } catch (_) { previews = []; }
    this.suitePreviewCache = previews;
    try {
      if (window.parent && window.parent !== window) {
        window.parent.dispatchEvent(new CustomEvent('suite:sketchpreview', { detail: { previews, preview: previews[0] || '' } }));
      }
    } catch (_) { /* standalone sketch session */ }
    return previews;
  }

  updateTitle() {
    const title = `Appraisal Sketch - ${this.fileName}`;
    document.title = title;
    const el = document.querySelector('.window-title');
    if (el) el.textContent = title;
    const pill = document.getElementById('doc-save-state');
    if (pill && this.fileName && this.fileName !== 'Untitled') pill.textContent = this.fileName;
  }

  /* Serializable drawing model shared by the standalone sketch-file save,
     the suite workfile export, and the recent-files cache. */
  buildSaveModel() {
    this.saveCurrentPage();
    return {
      version: 'Sketch-Web',
      areaModelVersion: 1,
      fileName: this.fileName,
      nextId: this.nextId,
      areaCodes: this.areaCodes,
      layers: this.state.layers,
      activeLayerId: this.state.activeLayerId,
      pages: this.pages,
      subjectInfo: this.state.subjectInfo,
      subjectInfoAutoFilled: Boolean(this.state.subjectInfoAutoFilled),
      extendedData: this.state.extendedData,
      settings: {
        defineFirst: this.state.defineFirst,
        dimensionsEnabled: this.state.dimensionsEnabled,
        dimensionPlacement: this.state.dimensionPlacement,
        drawColor: this.state.drawColor,
        canvasTheme: this.state.canvasTheme,
        pageLayout: this.state.pageLayout,
        autoPost: this.state.autoPost,
        sidebar: this.state.sidebar,
        textLibraryLabels: this.state.textLibraryLabels,
        symbolFavorites: this.state.symbolFavorites,
        options: this.state.options
      },
      savedAt: new Date().toISOString()
    };
  }

  async saveToFile() {
    if (this.fileName === 'Untitled') {
      const name = await this.ui.promptDialog('Name your sketch:', '');
      if (!name || !name.trim()) return;
      this.fileName = name.trim();
    }
    const data = this.buildSaveModel();
    /* The recent-files cache keeps the plain model. loadFromFile identifies by
       content, so a cached entry opens through the legacy branch regardless of
       the .skx name it is stored under. */
    this.ui.recordRecentFile?.(`${this.fileName}.skx`, JSON.stringify(data));

    let blob;
    let extension = '.skx';
    try {
      if (!window.SketchFile) throw new Error('SketchFile module unavailable');
      blob = await window.SketchFile.write(data);
    } catch (err) {
      /* A container write that fails must not cost the appraiser the drawing.
         Fall back to the legacy JSON file, which every version can still read,
         and say so rather than writing a .skx that is not one. */
      this.ui.notify(`Could not write .skx (${err && err.message ? err.message : err}) - saved as JSON instead.`);
      blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
      extension = '.json';
    }

    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${this.fileName}${extension}`;
    a.click();
    URL.revokeObjectURL(url);
    document.getElementById('file-menu').style.display = 'none';
    const submenu = document.getElementById('file-recent-submenu');
    if (submenu) submenu.style.display = 'none';
    this.updateTitle();
    this.pushSuitePreview();
}

  async saveAs() {
    const name = await this.ui.promptDialog('Save as:', this.fileName === 'Untitled' ? '' : this.fileName);
    if (!name || !name.trim()) return;
    this.fileName = name.trim();
    await this.saveToFile();
  }

  /* Open a sketch. The format is decided by the file's own bytes, never by its
     extension: ZIP magic means a .skx (or any workfile container carrying a
     sketch part), anything else is a legacy .json sketch. A file renamed in
     transit still opens. */
  async loadFromFile(file) {
    try {
      const model = await this.readSketchSource(file);
      await this.applySaveModel(model, (file.name || '').replace(/\.[^.]+$/, ''));
      /* Cache the model, not the raw bytes - a container's bytes are binary and
         would not survive the string-keyed recent-files store. */
      this.ui.recordRecentFile?.(file.name, JSON.stringify(model));
    } catch (err) {
      await this.ui.alertDialog('Error reading file: ' + (err && err.message ? err.message : err));
    }
  }

  /* Decode a sketch file to its model object, container or legacy JSON. */
  async readSketchSource(file) {
    if (window.SketchFile) {
      const result = await window.SketchFile.readAny(file);
      return result.model;
    }
    return JSON.parse(await file.text());
  }

  /* Restore a model produced by buildSaveModel(). Backs both the standalone
     sketch-file open and the suite workfile restore. */
  async applySaveModel(data, fallbackName) {
    if (!data || !Array.isArray(data.pages)) {
      await this.ui.alertDialog('Invalid sketch file format.');
      return false;
    }
    const incomingPages = JSON.parse(JSON.stringify(data.pages));
    this.pages = data.areaModelVersion >= 1
      ? incomingPages.map((page, index) => this.ensurePageModel(page, index))
      : this.migratePageIds(incomingPages);
    this.nextId = Math.max(Number(data.nextId) || 1, this.getMaxLiveId(this.pages) + 1);
    if (data.areaCodes && window.AreaCodes) {
      this.areaCodes = window.AreaCodes.validateAreaCodes(data.areaCodes);
      window.AREA_CODES = window.AreaCodes.cloneAreaCodes(this.areaCodes);
      this.breakdown.codeTable = this.areaCodes;
    }
    this.currentPage = 0;
    this.underlayPageIndex = null;
    const savedName = typeof data.fileName === 'string' && data.fileName !== 'Untitled' ? data.fileName : '';
    this.fileName = savedName || fallbackName || 'Untitled';
    this.updateTitle();
    this.ensureLayers(data.layers, data.activeLayerId);
    this.state.subjectInfo = data.subjectInfo || '';
    this.state.subjectInfoAutoFilled = Boolean(data.subjectInfoAutoFilled);
    this.state.extendedData = data.extendedData || '';
    if (data.settings) {
      this.state.defineFirst = Boolean(data.settings.defineFirst);
      this.state.dimensionsEnabled = data.settings.dimensionsEnabled !== false;
      this.state.dimensionPlacement = data.settings.dimensionPlacement || 'out';
      this.state.drawColor = data.settings.drawColor || this.state.drawColor;
      this.state.canvasTheme = ['dark', 'blue'].includes(data.settings.canvasTheme) ? data.settings.canvasTheme : 'light';
      this.state.pageLayout = this.normalizePageLayout(data.settings.pageLayout);
      this.state.autoPost = { ...this.state.autoPost, ...(data.settings.autoPost || {}) };
      this.state.sidebar = { ...this.state.sidebar, ...(data.settings.sidebar || {}) };
      this.state.textLibraryLabels = Array.isArray(data.settings.textLibraryLabels)
        ? data.settings.textLibraryLabels.slice() : this.state.textLibraryLabels;
      this.state.symbolFavorites = Array.isArray(data.settings.symbolFavorites)
        ? data.settings.symbolFavorites.slice() : this.state.symbolFavorites;
      this.state.options = this.mergeSavedOptions(data.settings.options);
      this.wallSolidsCache = null;
      this.state.defineFirst = Boolean(this.state.options.defineFirst ?? this.state.defineFirst);
      this.state.gridEnabled = this.state.options.gridEnabled !== false;
    } else {
      // An older sketch with no saved settings still must not inherit solids.
      this.state.options = this.mergeSavedOptions(null);
      this.wallSolidsCache = null;
    }
    this.canvas.setBackgroundTheme(this.state.canvasTheme);
    this.canvas.applyOptions?.();
    this.ui.populateAreaCodes();
    this.ui.loadLibraryState();
    this.ui.restoreSidebarState();
    this.ui.syncSettingsControls();
    this.loadPageData(this.pages[0]);
    this.centerAndFit();
    this.ui.updatePageDisplay();
    this.recalcAutoSubtract();
    document.getElementById('file-menu').style.display = 'none';
    const submenu = document.getElementById('file-recent-submenu');
    if (submenu) submenu.style.display = 'none';
    this.pushSuitePreview();
    return true;
  }

  /* Suite workfile entry point. Works on a private copy so the shell's cached
     payload can never be mutated by the live drawing. */
  async importState(data) {
    if (!data || !Array.isArray(data.pages)) return false;
    return this.applySaveModel(JSON.parse(JSON.stringify(data)), '');
  }

  async importFromFile(file) {
    try {
      const data = await this.readSketchSource(file);
      const importedPage = Array.isArray(data.pages) ? data.pages[0] : data;
      if (!importedPage || !Array.isArray(importedPage.lines)) {
        throw new Error('No sketch geometry found');
      }

      this.pushUndo('Import');
      const offset = 2;
      const lineIdMap = new Map();
      const polygonIdMap = new Map();
      (importedPage.lines || []).forEach(line => {
        const newId = this.genId();
        lineIdMap.set(line.id, newId);
        this.state.lines.push({
          ...JSON.parse(JSON.stringify(line)),
          start: { x: line.start.x + offset, y: line.start.y + offset },
          end: { x: line.end.x + offset, y: line.end.y + offset },
          id: newId
        });
      });
      (importedPage.polygons || []).forEach(poly => polygonIdMap.set(poly.id, this.genId()));
      (importedPage.polygons || []).forEach(poly => {
        const points = poly.points.map(point => ({ x: point.x + offset, y: point.y + offset }));
        const importedPoly = {
          ...JSON.parse(JSON.stringify(poly)),
          points,
          edges: (poly.edges || []).map(edge => ({ ...edge, lineId: lineIdMap.get(edge.lineId) ?? null })),
          containerId: null,
          subtractedFromId: null,
          subtractTargetId: polygonIdMap.get(poly.subtractTargetId) ?? null,
          id: polygonIdMap.get(poly.id)
        };
        if (importedPoly.labelPosition) {
          importedPoly.labelPosition.x += offset;
          importedPoly.labelPosition.y += offset;
        }
        if (importedPoly.areaSizePosition) {
          importedPoly.areaSizePosition.x += offset;
          importedPoly.areaSizePosition.y += offset;
        }
        this.state.polygons.push(importedPoly);
        this.assignAreaToDedicatedLayer(importedPoly);
      });
      this.state.polygons = this.ensurePageModel(this.getPageData(), this.currentPage).polygons;
      (importedPage.labels || []).forEach(label => {
        this.state.labels.push({ ...JSON.parse(JSON.stringify(label)), x: label.x + offset, y: label.y + offset, id: this.genId() });
      });
      (importedPage.symbols || []).forEach(symbol => {
        this.state.symbols.push({ ...JSON.parse(JSON.stringify(symbol)), x: symbol.x + offset, y: symbol.y + offset, id: this.genId() });
      });
      this.recalcAutoSubtract();
      this.ui.updateCalcsPanel();
      this.render();
      this.ui.notify(`Imported ${file.name}`);
      document.getElementById('file-menu').style.display = 'none';
      const submenu = document.getElementById('file-recent-submenu');
      if (submenu) submenu.style.display = 'none';
    } catch (err) {
      await this.ui.alertDialog('Error importing file: ' + (err && err.message ? err.message : err));
    }
  }

  async connectDisto() {
    if (!navigator.bluetooth?.requestDevice) {
      this.ui.notify('Bluetooth device access is not available in this browser.');
      return;
    }
    try {
      const device = await navigator.bluetooth.requestDevice({ acceptAllDevices: true });
      const status = document.getElementById('disto-status');
      if (status) {
        status.querySelector('span').textContent = device.name || 'Connected';
        status.classList.add('active');
      }
      this.ui.notify(`Connected to ${device.name || 'Bluetooth device'}.`);
    } catch (err) {
      if (err.name !== 'NotFoundError') this.ui.notify('Bluetooth connection was not completed.');
    }
  }

  emailSupport() {
    window.location.href = 'mailto:thatappraiserx@gmail.com?subject=Appraisal%20Sketch%20Support';
  }

  async exitApp() {
    if (!(await this.ui.confirmDialog('Close this sketch? Save your work first if needed.'))) return;
    window.close();
    this.ui.notify('This browser does not allow the application to close its own tab.');
  }

  exportPNG() {
    const transientState = {
      selectedLineIds: this.state.selectedLineIds,
      selectedPolyIds: this.state.selectedPolyIds,
      selectedAreaLabelIds: this.state.selectedAreaLabelIds,
      selectedAreaSizeIds: this.state.selectedAreaSizeIds,
      selectedLabelIds: this.state.selectedLabelIds,
      selectedSymbolIds: this.state.selectedSymbolIds,
      activeLineEndpoint: this.state.activeLineEndpoint,
      penDown: this.state.penDown,
      dragState: this.state.dragState,
      pan: { ...this.state.pan },
      zoom: this.state.zoom
    };
    try {
      // Export drawing content only. Selection styling is state-driven, so use
      // empty temporary sets while preserving the user's live selection.
      this.state.selectedLineIds = new Set();
      this.state.selectedPolyIds = new Set();
      this.state.selectedAreaLabelIds = new Set();
      this.state.selectedAreaSizeIds = new Set();
      this.state.selectedLabelIds = new Set();
      this.state.selectedSymbolIds = new Set();
      this.state.activeLineEndpoint = null;
      this.state.penDown = false;
      this.state.dragState = null;

      // Parse the selected export resolution and create an off-screen canvas
      // at that exact size so the PNG matches the option chosen by the user.
      const sizeStr = this.state.options.imageSize || '2048x1536';
      const [targetWidth, targetHeight] = sizeStr.split('x').map(Number);
      const exportCanvas = document.createElement('canvas');
      exportCanvas.width = targetWidth;
      exportCanvas.height = targetHeight;

      // Fit all sketch content into the export canvas while preserving aspect ratio.
      const hasContent = this.state.lines.length || this.state.polygons.length ||
        this.state.labels.length || this.state.symbols.length ||
        (this.state.options.saveImageWithSketch && this.bgImage.img);
      if (hasContent) {
        const bounds = this.getExportBounds();
        const worldW = bounds.maxX - bounds.minX;
        const worldH = bounds.maxY - bounds.minY;
        const margin = 60;
        const zoomX = (targetWidth - margin * 2) / (Math.max(worldW, 1) * window.Geometry.PIXELS_PER_FOOT);
        const zoomY = (targetHeight - margin * 2) / (Math.max(worldH, 1) * window.Geometry.PIXELS_PER_FOOT);
        this.state.zoom = Math.max(0.1, Math.min(10, Math.min(zoomX, zoomY)));
        const cx = (bounds.minX + bounds.maxX) / 2;
        const cy = (bounds.minY + bounds.maxY) / 2;
        this.state.pan.x = targetWidth / 2 - cx * window.Geometry.PIXELS_PER_FOOT * this.state.zoom;
        this.state.pan.y = targetHeight / 2 + cy * window.Geometry.PIXELS_PER_FOOT * this.state.zoom;
      } else {
        // Empty sketch: center the origin in the exported image.
        this.state.zoom = 1;
        this.state.pan.x = targetWidth / 2;
        this.state.pan.y = targetHeight / 2;
      }

      this.canvas.renderTo(exportCanvas, { transparent: true, clean: true });

      const link = document.createElement('a');
      link.download = `${this.fileName}.png`;
      link.href = exportCanvas.toDataURL('image/png');
      link.click();
    } finally {
      Object.assign(this.state, transientState);
      this.state.pan = transientState.pan;
      this.render();
      document.getElementById('file-menu').style.display = 'none';
      const submenu = document.getElementById('file-recent-submenu');
      if (submenu) submenu.style.display = 'none';
    }
  }

  getExportBounds() {
    let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;
    const expand = (x, y) => {
      minX = Math.min(minX, x); maxX = Math.max(maxX, x);
      minY = Math.min(minY, y); maxY = Math.max(maxY, y);
    };
    this.state.lines.forEach(line => {
      expand(line.start.x, line.start.y);
      expand(line.end.x, line.end.y);
    });
    this.state.polygons.forEach(poly => poly.points.forEach(p => expand(p.x, p.y)));
    this.state.labels.forEach(label => expand(label.x, label.y));
    this.state.symbols.forEach(symbol => expand(symbol.x, symbol.y));
    if (this.state.options.saveImageWithSketch && this.bgImage?.img && this.bgImage.visible) {
      const halfW = (this.bgImage.img.width / this.bgImage.scale) / 2;
      const halfH = (this.bgImage.img.height / this.bgImage.scale) / 2;
      const halfDiag = Math.hypot(halfW, halfH);
      expand(this.bgImage.x - halfDiag, this.bgImage.y - halfDiag);
      expand(this.bgImage.x + halfDiag, this.bgImage.y + halfDiag);
    }
    if (!isFinite(minX)) {
      minX = maxX = minY = maxY = 0;
    }
    return { minX, maxX, minY, maxY };
  }

  exportPDF() {
    const model = this.buildExportModel();
    const printWindow = window.open('', '_blank');
    if (!printWindow) {
      this.ui.notify('Allow pop-ups to open the appraisal report.');
      return;
    }
    printWindow.document.write(window.SketchReport.documentHTML(model, this.areaCodes, true));
    printWindow.document.close();
    document.getElementById('file-menu').style.display = 'none';
    const submenu = document.getElementById('file-recent-submenu');
    if (submenu) submenu.style.display = 'none';
  }

  closeFileMenu() {
    document.getElementById('file-menu').style.display = 'none';
    const submenu = document.getElementById('file-recent-submenu');
    if (submenu) submenu.style.display = 'none';
  }

  exportSVG() {
    this.closeFileMenu();
    new window.Exporter(this).exportSVG();
  }

  exportDXF() {
    this.closeFileMenu();
    new window.Exporter(this).exportDXF();
  }

  render() {
    this.applyLayerOrder();
    this.ui?.updateContextualTools?.();
    if (this.canvas) this.canvas.render();
  }
}

/* Merge suite-provided subject/appraiser values into the Subject Info model.
   Comments, comment tables and additional fields are preserved, and blank
   incoming values never overwrite an existing entry. */
/* Format a stored lat/lon pair for the single GPS field; empty when absent. */
function formatGpsPair(lat, lon) {
  const latText = String(lat == null ? '' : lat).trim();
  const lonText = String(lon == null ? '' : lon).trim();
  return latText && lonText ? `${latText}, ${lonText}` : '';
}

/* Parse either decimal degrees ("47.7737, -122.6539") or a DMS pair
   ("47°46'24.92\"N 122°39'14.18\"W") into decimal { lat, lon }; null when invalid. */
function parseGpsPair(value) {
  const text = String(value == null ? '' : value).trim();
  if (!text) return null;
  const dms = [...text.matchAll(/(\d{1,3})\s*°\s*(\d{1,2})\s*['′]\s*(\d{1,2}(?:\.\d+)?)\s*(?:"|″|'')\s*([NSEW])/gi)];
  if (dms.length === 2) {
    const toDecimal = group => {
      const sign = /[SW]/i.test(group[4]) ? -1 : 1;
      return sign * (Number(group[1]) + Number(group[2]) / 60 + Number(group[3]) / 3600);
    };
    const firstIsLat = /[NS]/i.test(dms[0][4]);
    return {
      lat: firstIsLat ? toDecimal(dms[0]) : toDecimal(dms[1]),
      lon: firstIsLat ? toDecimal(dms[1]) : toDecimal(dms[0])
    };
  }
  const decimal = text.split(',').map(item => item.trim());
  if (decimal.length === 2 && decimal.every(item => /^[+-]?\d+(\.\d+)?$/.test(item))) {
    return { lat: Number(decimal[0]), lon: Number(decimal[1]) };
  }
  return null;
}

function mergeSubjectInfo(state, info) {
  const legacy = typeof state.subjectInfo === 'string' ? state.subjectInfo : '';
  const current = typeof state.subjectInfo === 'object' && state.subjectInfo
    ? state.subjectInfo : { generalComments: legacy };
  const next = { ...current };
  let changed = false;
  Object.keys(info || {}).forEach(key => {
    const value = info[key] == null ? '' : String(info[key]);
    if (!value.trim()) return;
    if (next[key] !== value) { next[key] = value; changed = true; }
  });
  /* Keep the single GPS field in step with a freshly pushed lat/lon pair. */
  const pair = formatGpsPair(info.gpsLatitude, info.gpsLongitude);
  if (pair && next.gpsCoordinates !== pair) { next.gpsCoordinates = pair; changed = true; }
  if (changed) state.subjectInfo = next;
  return changed;
}

window.onload = () => {
  window.app = new SketchApp();
  /* Suite shell hooks: the workfile save/restore pipeline reads and writes the
     full drawing model through these entry points. */
  window.__exportState = () => { try { return window.app ? window.app.buildSaveModel() : null; } catch (_) { return null; } };
  /* Live-validation projection (report completeness). buildSaveModel above
     serializes the entire drawing (polygons, lines, pages, images); the
     completeness rules need two numbers. The revision changes only when
     recalculate() runs, so drawing strokes and pointer movement cannot
     schedule a validation pass. */
  window.__exportValidationState = () => {
    try {
      if (!window.app) return null;
      const totals = window.app.areaTotals;
      if (!totals) return null;
      const gla = Number(totals.totalGLA);
      return {
        totalGLA: Number.isFinite(gla) ? gla : null,
        revision: Number(window.app.areaCalcRevision) || 0
      };
    } catch (_) { return null; }
  };
  /* Suite shell hook: the sketch's own Gross Living Area rollup (sum of
     includeInGLA areas across pages). Consumed by AppShell.sketchGlaChanged
     to auto-populate the Sales Grid subject GLA when the grid cell is still
     empty; an appraiser-entered cell is never overwritten. */
  /* Suite shell hook: the sketch's below-grade rollup, the companion to
     __exportGla. Returns null when the sketch draws no basement at all so a
     sketch without one can never zero a reported basement. */
  window.__exportBasement = () => {
    try {
      if (!window.app || !window.app.areaTotals) return null;
      const rollup = basementRollupFromTotals(window.app.areaTotals.totalsByCode);
      return rollup.total > 0 ? rollup : null;
    } catch (_) { return null; }
  };
  window.__exportGla = () => {
    try {
      if (!window.app || !window.app.areaTotals) return null;
      const gla = Number(window.app.areaTotals.totalGLA);
      return Number.isFinite(gla) && gla > 0 ? gla : null;
    } catch (_) { return null; }
  };
  window.__importState = (data) => { try { return window.app ? window.app.importState(data) : false; } catch (_) { return false; } };
  /* Suite shell hook: populate the Subject Info dialog from the report's
     subject and appraiser data without touching comments or custom fields. */
  window.__setSubjectInfo = (info) => {
    try {
      if (!window.app || !info || typeof info !== 'object') return false;
      const hasData = Object.keys(info).some(key => info[key] != null && String(info[key]).trim());
      const changed = mergeSubjectInfo(window.app.state, info);
      if (hasData) window.app.state.subjectInfoAutoFilled = true;
      if (changed || hasData) {
        /* Refresh the dialog when open so the appraiser sees synced values. */
        const overlay = document.getElementById('subject-info-overlay');
        const model = window.app.state.subjectInfo;
        if (overlay && model && typeof model === 'object') {
          Object.keys(model).forEach(key => {
            const input = overlay.querySelector(`[name="${key}"]`);
            if (input) input.value = model[key];
          });
          if (hasData) overlay.querySelector('[data-autofill-row]')?.remove();
        }
      }
      return true;
    } catch (_) { return false; }
  };
  window.__clearSketch = () => { try { if (window.app) window.app.resetToBlank(); } catch (_) { /* not ready yet */ } };
  window.__refreshSuitePreview = () => { try { return window.app ? window.app.pushSuitePreview() : Promise.resolve(null); } catch (_) { return Promise.resolve(null); } };
  window.__exportPreviews = () => { try { return window.app && Array.isArray(window.app.suitePreviewCache) ? window.app.suitePreviewCache.slice() : []; } catch (_) { return []; } };
  window.__exportPreview = () => { try { const pages = window.app && window.app.suitePreviewCache; return Array.isArray(pages) ? (pages[0] || null) : (pages || null); } catch (_) { return null; } };
};

  </script>
<!-- CAD IMPORT LAYER: generated by tools/install-cad-import.js -->
<script>
