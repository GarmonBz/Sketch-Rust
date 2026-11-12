class CanvasEngine {
  constructor(app, canvasId) {
    this.app = app;
    this.canvas = document.getElementById(canvasId);
    this.ctx = this.canvas.getContext('2d');
    this.mouseScreen = { x: 0, y: 0 };
    this.ignoreClick = false;
    
    this.colors = {
      bg: '#111111',
      grid10: 'rgba(255, 255, 255, 0.05)',
      grid50: 'rgba(255, 255, 255, 0.12)',
      wallExt: '#2B6CB0',
      wallInt: '#63B3ED',
      wallActive: '#D946EF',
      wallPreview: '#EF4444',
      dimLabel: '#A0AEC0',
      vertex: '#E53E3E',
      vertexStroke: '#FC8181',
      fillUndef: 'rgba(255, 0, 0, 0.08)',
      fillGLA: 'rgba(43, 108, 176, 0.08)',
      polyOutline: '#2B6CB0',
      witnessLine: 'rgba(99, 179, 237, 0.3)',
      snapDot: '#22C55E',
      // Selection colors
      selectLine: '#C05AFA',       // Purple highlight for selected walls
      selectPoly: 'rgba(192, 90, 250, 0.15)',
      selectPolyOutline: '#C05AFA',
      fenceFill: 'rgba(99, 179, 237, 0.1)',
      fenceStroke: '#63B3ED'
    };

    // Measure tool palette. Deliberately outside `colors` so the amber
    // measuring overlay never follows the canvas theme's wall colors.
    this.measureColors = {
      line: '#F6AD55',
      snap: '#63B3ED',
      chipFill: 'rgba(17, 24, 39, 0.85)',
      panelFill: 'rgba(17, 24, 39, 0.92)',
      accentFill: 'rgba(246, 173, 85, 0.92)',
      accentText: '#111827',
      areaFill: 'rgba(246, 173, 85, 0.12)',
      muted: '#A0AEC0',
      text: '#E2E8F0'
    };

    // Default wall widths: exterior walls render twice as thick as
    // interior walls unless the user overrides either in Options.
    this.wallWidths = {
      exterior: 4,
      interior: 2
    };

    // The drawing pointer variants (cursor-<size>[-light].png) are pre-built
    // by tools/build-cursors.js. Static files work even on file:// pages,
    // where generating cursor data URLs at runtime would taint the canvas.
    this.cursorImg = new Image();
    this.cursorImg.src = __cursorUrl('cursor.png');
    this.cursorCache = new Map();
    this.cursorDrawCache = new Map();
    this.cursorImg.onload = () => this.setCursor();

    this.setupEvents();
    window.addEventListener('resize', this.resize.bind(this));
    this.resize();
  }

  setBackgroundTheme(theme) {
    const valid = theme === 'dark' || theme === 'blue' ? theme : 'light';
    this.app.state.canvasTheme = valid;
    if (valid === 'light') {
      this.colors.bg = '#FFFFFF';
      this.colors.grid10 = 'rgba(30, 41, 59, 0.09)';
      this.colors.grid50 = 'rgba(30, 41, 59, 0.20)';
      this.colors.dimLabel = '#475569';
      this.colors.witnessLine = 'rgba(37, 99, 235, 0.38)';
    } else if (valid === 'blue') {
      this.colors.bg = '#0b1728';
      this.colors.grid10 = 'rgba(147, 197, 253, 0.08)';
      this.colors.grid50 = 'rgba(147, 197, 253, 0.18)';
      this.colors.dimLabel = '#93C5FD';
      this.colors.witnessLine = 'rgba(99, 179, 237, 0.32)';
    } else {
      this.colors.bg = '#111111';
      this.colors.grid10 = 'rgba(255, 255, 255, 0.05)';
      this.colors.grid50 = 'rgba(255, 255, 255, 0.12)';
      this.colors.dimLabel = '#A0AEC0';
      this.colors.witnessLine = 'rgba(99, 179, 237, 0.3)';
    }
    this.canvas.dataset.backgroundTheme = valid;
    if (this.canvas.parentElement) this.canvas.parentElement.dataset.backgroundTheme = valid;
    this.setCursor();
    this.render();
  }

  colorWithAlpha(color, alpha) {
    const match = /^#([0-9a-f]{6})$/i.exec(String(color || ''));
    if (!match) return color;
    const value = parseInt(match[1], 16);
    return `rgba(${value >> 16}, ${(value >> 8) & 255}, ${value & 255}, ${alpha})`;
  }

  applyOptions() {
    const options = this.app.state.options || {};
    const interior = Math.max(1, Number(options.interiorWidth) || 2);
    // Exterior thickness defaults to twice interior but can be overridden
    // independently from the desktop-style dialog.
    const exterior = Math.max(2, Number(options.exteriorWidth) || interior * 2);
    this.wallWidths.interior = interior;
    this.wallWidths.exterior = exterior;
    options.interiorWidth = interior;
    options.exteriorWidth = exterior;
    this.colors.wallExt = options.exteriorColor || this.colors.wallExt;
    this.colors.wallInt = options.interiorColor || this.colors.wallInt;
    this.colors.wallPreview = options.witnessCurveColor || this.colors.wallPreview;
    this.colors.vertex = options.witnessBaseColor || this.colors.vertex;
    this.colors.snapDot = options.witnessInteriorColor || this.colors.snapDot;
    this.setCursor();
    this.render();
  }

  normalizeDegrees(value) {
    let result = value % 360;
    if (result > 180) result -= 360;
    if (result < -180) result += 360;
    return result;
  }

  // Central cursor policy: an active drag wins, then holding space
  // (temporary hand tool), then the active mode's cursor.
  setCursor() {
    const s = this.app.state;
    if (s.isPanning || s.dragState) { this.canvas.style.cursor = 'grabbing'; return; }
    if (s.spacePan || s.mode === 'pan') { this.canvas.style.cursor = 'grab'; return; }
    if (s.mode === 'draw-exterior' || s.mode === 'draw-interior') {
      // Keyboard-pointer mode hides the OS cursor; the pointer is painted at
      // the keyboard cursor instead (see drawKeyboardPointer).
      if (s.keyboardPointer) { this.canvas.style.cursor = 'none'; return; }
      this.canvas.style.cursor = this.getDrawingCursor() || `url('${__cursorUrl('cursor.png')}') 64 64, crosshair`;
      return;
    }
    if (s.mode === 'fence' || s.mode === 'lasso' || s.mode === 'insert-vertex' || s.mode === 'zoom-fence' || s.mode === 'curve' || s.mode === 'measure' || s.mode === 'chain-dim' || s.mode === 'place-shape') {
      this.canvas.style.cursor = 'crosshair';
      return;
    }
    if (s.mode === 'place-text' || s.mode === 'place-symbol') { this.canvas.style.cursor = 'copy'; return; }
    this.canvas.style.cursor = 'default';
  }

  // Resolves the drawing pointer for the current size option and background
  // theme. Prefers the pre-built static variant; if that is missing it falls
  // back to a runtime data URL, and finally to the original pointer.
  getDrawingCursor() {
    const size = Number(this.app.state.options?.pointerSize);
    const valid = [32, 64, 128].includes(size) ? size : 128;
    const invert = this.app.state.canvasTheme === 'light';
    const key = `${valid}|${invert}`;
    const src = __cursorUrl(`cursor-${valid}${invert ? '-light' : ''}.png`);
    let entry = this.cursorCache.get(key);
    if (!entry) {
      entry = { img: null, resolved: null };
      this.cursorCache.set(key, entry);
    }
    if (entry.resolved) return entry.resolved;
    if (!entry.img) {
      const img = new Image();
      img.src = src;
      entry.img = img;
      img.onload = () => this.setCursor();
    }
    const img = entry.img;
    const half = Math.round(valid / 2);
    if (img.complete && img.naturalWidth > 0) {
      entry.resolved = `url('${src}') ${half} ${half}, crosshair`;
      return entry.resolved;
    }
    if (img.complete) {
      // Variant file not built; generate a data URL instead.
      entry.resolved = this.runtimeCursor(valid, invert);
      return entry.resolved;
    }
    return null; // still loading
  }

  // Runtime fallback that scales and inverts cursor.png via canvas. Guarded
  // because toDataURL throws when the canvas is tainted (e.g. file:// pages).
  runtimeCursor(size, invert) {
    try {
      if (!this.cursorImg || !this.cursorImg.complete || !this.cursorImg.naturalWidth) {
        return `url('${__cursorUrl('cursor.png')}') 64 64, crosshair`;
      }
      const canvas = document.createElement('canvas');
      canvas.width = size;
      canvas.height = size;
      const ctx = canvas.getContext('2d');
      if (invert) ctx.filter = 'invert(1)';
      ctx.drawImage(this.cursorImg, 0, 0, size, size);
      const half = Math.round(size / 2);
      return `url("${canvas.toDataURL('image/png')}") ${half} ${half}, crosshair`;
    } catch (err) {
      return `url('${__cursorUrl('cursor.png')}') 64 64, crosshair`;
    }
  }

  getSelectedAreaTransform() {
    if (!this.app.state.selectedPolyIds.size) return null;
    const model = this.app.getSelectedGeometryRotationModel?.();
    if (!model) return null;
    const centerScreen = window.Geometry.toScreen(model.center, this.app.state.pan, this.app.state.zoom);
    const topScreen = window.Geometry.toScreen(
      { x: model.center.x, y: model.bounds.maxY }, this.app.state.pan, this.app.state.zoom
    );
    const handleScreen = { x: centerScreen.x, y: topScreen.y - 34 };
    const handleWorld = window.Geometry.toWorld(handleScreen, this.app.state.pan, this.app.state.zoom);
    return { ...model, centerScreen, topScreen, handleScreen, handleWorld };
  }

  getSymbolTransform(symbol) {
    const definition = this.getSymbolDefinition(symbol.symbolType);
    if (!definition?.bounds) return null;
    const centerScreen = window.Geometry.toScreen(symbol, this.app.state.pan, this.app.state.zoom);
    const pixelScale = window.Geometry.PIXELS_PER_FOOT * this.app.state.zoom * (symbol.scale || 1) / 12;
    const bounds = definition.bounds;
    const rect = {
      x: bounds.minX * pixelScale,
      y: -bounds.maxY * pixelScale,
      width: (bounds.maxX - bounds.minX) * pixelScale,
      height: (bounds.maxY - bounds.minY) * pixelScale
    };
    const rotation = (Number(symbol.rotation) || 0) * Math.PI / 180;
    const rotate = point => ({
      x: centerScreen.x + point.x * Math.cos(rotation) - point.y * Math.sin(rotation),
      y: centerScreen.y + point.x * Math.sin(rotation) + point.y * Math.cos(rotation)
    });
    const topAnchorLocal = { x: rect.x + rect.width / 2, y: rect.y };
    const handleLocal = { x: topAnchorLocal.x, y: rect.y - 30 };
    return {
      symbol, definition, centerScreen, pixelScale, rect, rotation, rotate,
      corners: [
        { x: rect.x, y: rect.y }, { x: rect.x + rect.width, y: rect.y },
        { x: rect.x + rect.width, y: rect.y + rect.height }, { x: rect.x, y: rect.y + rect.height }
      ].map(rotate),
      topAnchor: rotate(topAnchorLocal),
      handleScreen: rotate(handleLocal)
    };
  }

  getTextBoxTransform(centerWorld, width, height, rotation, entity, type) {
    const centerScreen = window.Geometry.toScreen(centerWorld, this.app.state.pan, this.app.state.zoom);
    const radians = (Number(rotation) || 0) * Math.PI / 180;
    const rotate = point => ({
      x: centerScreen.x + point.x * Math.cos(radians) - point.y * Math.sin(radians),
      y: centerScreen.y + point.x * Math.sin(radians) + point.y * Math.cos(radians)
    });
    const halfWidth = Math.max(8, width / 2);
    const halfHeight = Math.max(7, height / 2);
    const topAnchorLocal = { x: 0, y: -halfHeight };
    const handleLocal = { x: 0, y: -halfHeight - 30 };
    return {
      type, entity, centerWorld, centerScreen, rotation: radians, rotate, halfWidth, halfHeight,
      corners: [
        { x: -halfWidth, y: -halfHeight }, { x: halfWidth, y: -halfHeight },
        { x: halfWidth, y: halfHeight }, { x: -halfWidth, y: halfHeight }
      ].map(rotate),
      topAnchor: rotate(topAnchorLocal),
      handleScreen: rotate(handleLocal)
    };
  }

  getLabelTransform(label) {
    // Labels scale with the drawing, but never below a readable minimum
    // (matches the desktop reference behavior at typical zoom levels).
    const fontSize = Math.max(9, (label.fontSize || 12) * this.getTextZoom());
    this.ctx.save();
    this.ctx.font = `${label.italic ? 'italic ' : ''}${label.bold ? 'bold ' : ''}${fontSize}px "${label.fontFamily || 'Arial'}", sans-serif`;
    const width = this.ctx.measureText(label.text || '').width + 8;
    this.ctx.restore();
    return this.getTextBoxTransform(label, width, fontSize + 8, label.rotation, label, 'label');
  }

  getAreaLabelLines(poly) {
    const groups = this.getAreaLabelGroups(poly);
    return [...groups.label, ...groups.size];
  }

  getAreaLabelGroups(poly) {
    const state = this.app.state;
    const autoPost = { ...(state.autoPost || {}), ...(poly.autoPost || {}) };
    const label = [];
    const size = [];
    if (autoPost.code) label.push(poly.type);
    if (autoPost.name) label.push(poly.name);
    if (autoPost.calcs) size.push(`${(poly.netArea ?? poly.area).toFixed(1)} ${autoPost.suffix || 'sf'}`);
    if (autoPost.base) size.push(`Base: ${poly.area.toFixed(1)}`);
    if (autoPost.factor) size.push(`Factor: ${(poly.factor ?? 1).toFixed(2)}`);
    return { label, size };
  }

  getAreaTextTransform(poly, type) {
    if (!poly?.type || poly.type === 'UND') return null;
    const groups = this.getAreaLabelGroups(poly);
    const lines = type === 'areaSize' ? groups.size : groups.label;
    if (!lines.length) return null;
    const options = this.app.state.options || {};
    const zoom = this.getTextZoom();
    const scale = Number(type === 'areaSize' ? (poly.areaSizeScale ?? poly.labelScale) : poly.labelScale) || 1;
    let width = 0;
    lines.forEach((text, index) => {
      const isFirstAutoPostLine = type === 'areaLabel' ? index === 0 : !groups.label.length && index === 0;
      const weight = ((poly.labelBold ?? options.autoPostBold) || isFirstAutoPostLine) ? 'bold ' : '';
      const size = Math.max(9, (Number(poly.labelFontSize) || Number(options.autoPostFontSize) || (index === 0 ? 12 : 11)) * scale * zoom);
      this.ctx.save();
      this.ctx.font = `${(poly.labelItalic ?? options.autoPostItalic) ? 'italic ' : ''}${weight}${size}px "${poly.labelFontFamily || options.autoPostFont || 'Inter'}", sans-serif`;
      width = Math.max(width, this.ctx.measureText(text || '').width);
      this.ctx.restore();
    });
    const height = lines.length * Math.max(9, 15 * scale * zoom);
    return this.getTextBoxTransform(
      type === 'areaSize' ? (this.app.getAreaSizePosition?.(poly) || this.app.getAreaLabelPosition(poly)) : this.app.getAreaLabelPosition(poly),
      width + 10, height + 8,
      type === 'areaSize' ? (poly.areaSizeRotation ?? poly.labelRotation) : poly.labelRotation,
      poly, type
    );
  }

  getAreaLabelTransform(poly) {
    return this.getAreaTextTransform(poly, 'areaLabel');
  }

  getAreaSizeTransform(poly) {
    return this.getAreaTextTransform(poly, 'areaSize');
  }

  pointInTextTransform(worldPt, transform) {
    if (!transform) return false;
    const screen = window.Geometry.toScreen(worldPt, this.app.state.pan, this.app.state.zoom);
    const dx = screen.x - transform.centerScreen.x;
    const dy = screen.y - transform.centerScreen.y;
    const cos = Math.cos(-transform.rotation), sin = Math.sin(-transform.rotation);
    const local = { x: dx * cos - dy * sin, y: dx * sin + dy * cos };
    return Math.abs(local.x) <= transform.halfWidth + 5 &&
      Math.abs(local.y) <= transform.halfHeight + 5;
  }

  pointInAreaLabel(worldPt, poly) {
    return this.pointInTextTransform(worldPt, this.getAreaLabelTransform(poly));
  }

  pointInAreaSize(worldPt, poly) {
    return this.pointInTextTransform(worldPt, this.getAreaSizeTransform(poly));
  }

  /* Corner squares drawn on a selected symbol are its resize handles. */
  getSymbolResizeHandleHit(screenPt) {
    const selectedSymbols = this.app.state.symbols
      .filter(symbol => this.app.state.selectedSymbolIds.has(symbol.id));
    for (let index = selectedSymbols.length - 1; index >= 0; index--) {
      const transform = this.getSymbolTransform(selectedSymbols[index]);
      if (!transform) continue;
      const corner = transform.corners.findIndex(point =>
        Math.abs(screenPt.x - point.x) <= 7 && Math.abs(screenPt.y - point.y) <= 7);
      if (corner >= 0) return { transform, corner };
    }
    return null;
  }

  getRotationHandleHit(screenPt) {
    const selectedAreaSizes = this.app.state.polygons
      .filter(poly => this.app.state.selectedAreaSizeIds?.has(poly.id));
    for (let index = selectedAreaSizes.length - 1; index >= 0; index--) {
      const transform = this.getAreaSizeTransform(selectedAreaSizes[index]);
      if (transform && Math.hypot(screenPt.x - transform.handleScreen.x, screenPt.y - transform.handleScreen.y) <= 11) {
        return { type: 'areaSize', transform };
      }
    }
    const selectedAreaLabels = this.app.state.polygons
      .filter(poly => this.app.state.selectedAreaLabelIds.has(poly.id));
    for (let index = selectedAreaLabels.length - 1; index >= 0; index--) {
      const transform = this.getAreaLabelTransform(selectedAreaLabels[index]);
      if (transform && Math.hypot(screenPt.x - transform.handleScreen.x, screenPt.y - transform.handleScreen.y) <= 11) {
        return { type: 'areaLabel', transform };
      }
    }
    const selectedLabels = this.app.state.labels
      .filter(label => this.app.state.selectedLabelIds.has(label.id));
    for (let index = selectedLabels.length - 1; index >= 0; index--) {
      const transform = this.getLabelTransform(selectedLabels[index]);
      if (Math.hypot(screenPt.x - transform.handleScreen.x, screenPt.y - transform.handleScreen.y) <= 11) {
        return { type: 'label', transform };
      }
    }
    const selectedSymbols = this.app.state.symbols
      .filter(symbol => this.app.state.selectedSymbolIds.has(symbol.id));
    for (let index = selectedSymbols.length - 1; index >= 0; index--) {
      const transform = this.getSymbolTransform(selectedSymbols[index]);
      if (transform && Math.hypot(screenPt.x - transform.handleScreen.x, screenPt.y - transform.handleScreen.y) <= 11) {
        return { type: 'symbol', transform };
      }
    }
    const area = this.getSelectedAreaTransform();
    if (area && Math.hypot(screenPt.x - area.handleScreen.x, screenPt.y - area.handleScreen.y) <= 11) {
      return { type: 'area', transform: area };
    }
    return null;
  }

  getSelectedLineEndpointHit(worldPt) {
    const tolerance = this.app.getScreenToleranceWorld(11);
    let nearest = null;
    this.app.state.lines.forEach(line => {
      if (!this.app.state.selectedLineIds.has(line.id)) return;
      ['start', 'end'].forEach(endpoint => {
        const distance = window.Geometry.distance(worldPt, line[endpoint]);
        if (distance <= tolerance && (!nearest || distance < nearest.distance)) nearest = { line, endpoint, distance };
      });
    });
    return nearest;
  }

  drawLineEndpointHandles() {
    const state = this.app.state;
    if (state.mode !== 'select' || state.selectedLineIds.size !== 1) return;
    const line = state.lines.find(item => state.selectedLineIds.has(item.id));
    if (!line) return;
    const active = state.activeLineEndpoint?.lineId === line.id ? state.activeLineEndpoint.endpoint : 'end';
    const ctx = this.ctx;
    ctx.save();
    ['start', 'end'].forEach(endpoint => {
      const screen = window.Geometry.toScreen(line[endpoint], state.pan, state.zoom);
      const isActive = endpoint === active;
      ctx.beginPath();
      ctx.arc(screen.x, screen.y, isActive ? 7 : 5, 0, Math.PI * 2);
      ctx.fillStyle = isActive ? '#56e36b' : '#e53e3e';
      ctx.fill();
      ctx.strokeStyle = isActive ? '#168b2d' : '#8f1f1f';
      ctx.lineWidth = 2;
      ctx.stroke();
      if (isActive) {
        ctx.beginPath();
        ctx.arc(screen.x, screen.y, 2, 0, Math.PI * 2);
        ctx.fillStyle = '#ffffff';
        ctx.fill();
      }
    });
    ctx.restore();
  }

  setupEvents() {
    // Track mouse
    this.canvas.addEventListener('pointermove', (e) => {
      const rect = this.canvas.getBoundingClientRect();
      this.mouseScreen.x = e.clientX - rect.left;
      this.mouseScreen.y = e.clientY - rect.top;
      
      // Panning
      if (this.app.state.isPanning && this.app.state.panStart) {
        this.app.state.pan.x += e.clientX - this.app.state.panStart.x;
        this.app.state.pan.y += e.clientY - this.app.state.panStart.y;
        this.app.state.panStart = { x: e.clientX, y: e.clientY };
      }

      // Hover position is transient. Never overwrite the committed drawing
      // anchor (`state.cursor`) while merely moving the pointer.
      this.app.state.pointerWorld = window.Geometry.toWorld(
        this.mouseScreen, this.app.state.pan, this.app.state.zoom
      );

      // Moving the mouse hands control back to the pointer: leave
      // keyboard-pointer mode so the OS cursor reappears at the mouse.
      if (this.app.state.keyboardPointer && this.app.state.keyboardPointerAnchor) {
        const anchor = this.app.state.keyboardPointerAnchor;
        const moved = Math.hypot(this.mouseScreen.x - anchor.x, this.mouseScreen.y - anchor.y);
        if (moved > 3) {
          this.app.state.keyboardPointer = false;
          this.app.state.keyboardPointerAnchor = null;
          this.setCursor();
        }
      }

      if (this.app.state.dragState?.type === 'rotate-area') {
        const drag = this.app.state.dragState;
        const currentAngle = Math.atan2(
          this.app.state.pointerWorld.y - drag.center.y,
          this.app.state.pointerWorld.x - drag.center.x
        );
        const step = e.ctrlKey ? 1 : 15;
        const rawDegrees = this.normalizeDegrees((currentAngle - drag.startAngle) * 180 / Math.PI);
        drag.previewDegrees = Math.round(rawDegrees / step) * step;
        drag.moved = drag.moved || Math.abs(drag.previewDegrees) >= step;
        this.canvas.style.cursor = 'grabbing';
      }

      if (this.app.state.dragState?.type === 'resize-symbol') {
        const drag = this.app.state.dragState;
        const symbol = this.app.state.symbols.find(item => item.id === drag.symbolId);
        if (symbol) {
          const distance = Math.hypot(
            this.app.state.pointerWorld.x - drag.center.x,
            this.app.state.pointerWorld.y - drag.center.y
          );
          // Same limits as the +/- and wheel scaling. Snap to 5% steps unless
          // Ctrl is held for a fine adjustment.
          const step = e.ctrlKey ? 0.01 : 0.05;
          const raw = drag.startScale * distance / drag.startDistance;
          const nextScale = Math.max(0.25, Math.min(8, Math.round(raw / step) * step));
          if (drag.moved || Math.abs(nextScale - drag.startScale) >= step / 2) {
            if (!drag.moved) { this.app.pushUndo('Scale'); drag.moved = true; }
            symbol.scale = nextScale;
          }
        }
        this.canvas.style.cursor = 'nwse-resize';
      } else if (this.app.state.mode === 'select' && !this.app.state.dragState && !this.app.state.isPanning) {
        const overResize = Boolean(this.getSymbolResizeHandleHit(this.mouseScreen));
        if (overResize) this.canvas.style.cursor = 'nwse-resize';
        else if (this.overSymbolResizeHandle) this.setCursor();
        this.overSymbolResizeHandle = overResize;
      }

      if (this.app.state.dragState?.type === 'rotate-symbol') {
        const drag = this.app.state.dragState;
        const symbol = this.app.state.symbols.find(item => item.id === drag.symbolId);
        if (symbol) {
          const currentAngle = Math.atan2(
            this.app.state.pointerWorld.y - drag.center.y,
            this.app.state.pointerWorld.x - drag.center.x
          );
          const step = e.ctrlKey ? 1 : 15;
          const pointerDelta = this.normalizeDegrees((currentAngle - drag.startAngle) * 180 / Math.PI);
          const nextRotation = drag.startRotation - Math.round(pointerDelta / step) * step;
          if (drag.moved || Math.abs(nextRotation - drag.startRotation) >= step) {
            if (!drag.moved) { this.app.pushUndo(); drag.moved = true; }
            symbol.rotation = nextRotation;
            symbol.wallId = null;
          }
          drag.previewDegrees = this.normalizeDegrees(symbol.rotation - drag.startRotation);
          this.canvas.style.cursor = 'grabbing';
        }
      }

      if (this.app.state.dragState?.type === 'rotate-text') {
        const drag = this.app.state.dragState;
        const entity = drag.entityType === 'areaLabel' || drag.entityType === 'areaSize'
          ? this.app.state.polygons.find(item => item.id === drag.entityId)
          : this.app.state.labels.find(item => item.id === drag.entityId);
        if (entity) {
          const currentAngle = Math.atan2(
            this.app.state.pointerWorld.y - drag.center.y,
            this.app.state.pointerWorld.x - drag.center.x
          );
          const step = e.ctrlKey ? 1 : 15;
          const pointerDelta = this.normalizeDegrees((currentAngle - drag.startAngle) * 180 / Math.PI);
          const nextRotation = drag.startRotation - Math.round(pointerDelta / step) * step;
          if (drag.moved || Math.abs(nextRotation - drag.startRotation) >= step) {
            if (!drag.moved) { this.app.pushUndo(); drag.moved = true; }
            if (drag.entityType === 'areaLabel') entity.labelRotation = nextRotation;
            else if (drag.entityType === 'areaSize') entity.areaSizeRotation = nextRotation;
            else entity.rotation = nextRotation;
          }
          drag.previewDegrees = this.normalizeDegrees(nextRotation - drag.startRotation);
          this.canvas.style.cursor = 'grabbing';
        }
      }

      if (this.app.state.dragState?.type === 'photo') {
        const drag = this.app.state.dragState;
        const current = this.app.state.pointerWorld;
        this.app.bgImage.x += current.x - drag.lastWorld.x;
        this.app.bgImage.y += current.y - drag.lastWorld.y;
        drag.lastWorld = { ...current };
      }

      if (this.app.state.dragState?.type === 'selection') {
        const drag = this.app.state.dragState;
        const current = this.app.state.pointerWorld;
        let dx = current.x - drag.lastWorld.x;
        let dy = current.y - drag.lastWorld.y;

        // When area grid snap is enabled, snap the selection's absolute
        // position to the selected pitch grid so areas land exactly on the
        // pitch lines, regardless of where the grab point started.
        if (this.app.state.areaGridSnap && drag.startRef) {
          const gridSize = this.app.state.areaGridSnapSize || this.app.state.gridSize;
          const totalDx = current.x - drag.startWorld.x;
          const totalDy = current.y - drag.startWorld.y;
          const snappedRefX = Math.round((drag.startRef.x + totalDx) / gridSize) * gridSize;
          const snappedRefY = Math.round((drag.startRef.y + totalDy) / gridSize) * gridSize;
          const targetTotalX = snappedRefX - drag.startRef.x;
          const targetTotalY = snappedRefY - drag.startRef.y;
          if (!drag.lastSnappedTotal) drag.lastSnappedTotal = { x: 0, y: 0 };
          dx = targetTotalX - drag.lastSnappedTotal.x;
          dy = targetTotalY - drag.lastSnappedTotal.y;
          // Defer committing the snapped total until the move is actually
          // applied below, so a selection that starts off-grid still lands
          // exactly on the pitch grid when the drag activates.
          drag.pendingSnapTotal = { x: targetTotalX, y: targetTotalY };
        } else if (this.app.state.snapEnabled &&
            (this.app.state.selectedLineIds.size > 0 || this.app.state.selectedPolyIds.size > 0)) {
          // Ordinary snap: walls and areas move by whole grid steps, so an
          // on-grid sketch stays on the grid and lengths/areas stay exact
          // (a raw pointer delta left walls at 73.40000610 and areas at
          // 480.00000000000045). Labels and text alone still move freely.
          const step = this.app.state.gridSize || 1;
          const targetTotalX = Math.round((current.x - drag.startWorld.x) / step) * step;
          const targetTotalY = Math.round((current.y - drag.startWorld.y) / step) * step;
          if (!drag.lastSnappedTotal) drag.lastSnappedTotal = { x: 0, y: 0 };
          dx = targetTotalX - drag.lastSnappedTotal.x;
          dy = targetTotalY - drag.lastSnappedTotal.y;
          drag.pendingSnapTotal = { x: targetTotalX, y: targetTotalY };
        }

        if (drag.moved || Math.hypot(current.x - drag.startWorld.x, current.y - drag.startWorld.y) > this.app.getScreenToleranceWorld(2)) {
          if (!drag.moved) { this.app.pushUndo(); drag.moved = true; }
          this.app.moveSelected(dx, dy, false);
          drag.lastWorld = { ...current };
          if (drag.pendingSnapTotal) {
            drag.lastSnappedTotal = drag.pendingSnapTotal;
            drag.pendingSnapTotal = null;
          }
        }
      }

      if (this.app.state.dragState?.type === 'line-endpoint') {
        const drag = this.app.state.dragState;
        const line = this.app.state.lines.find(item => item.id === drag.lineId);
        if (line) {
          const target = this.app.snapLineEndpoint(this.app.state.pointerWorld, line.id, drag.endpoint, drag.excludedRefs);
          if (drag.moved || window.Geometry.distance(target, drag.startPoint) > this.app.getScreenToleranceWorld(1)) {
            if (!drag.moved) { this.app.pushUndo(); drag.moved = true; }
            this.app.moveLineEndpoint(line.id, drag.endpoint, target, false);
            const length = window.Geometry.distance(line.start, line.end);
            const angle = Math.atan2(line.end.y - line.start.y, line.end.x - line.start.x) * 180 / Math.PI;
            this.app.ui.updateHUD(length.toFixed(2), angle.toFixed(1), true);
            this.canvas.style.cursor = 'grabbing';
          }
        }
      }

      if (this.app.state.dragState?.type === 'curve') {
        const drag = this.app.state.dragState;
        const line = this.app.state.lines.find(item => item.id === drag.lineId);
        if (line) {
          const dx = line.end.x - line.start.x;
          const dy = line.end.y - line.start.y;
          const chord = Math.hypot(dx, dy);
          const midpoint = { x: (line.start.x + line.end.x) / 2, y: (line.start.y + line.end.y) / 2 };
          if (chord > 0.01) {
            const pointerNormal = (this.app.state.pointerWorld.x - midpoint.x) * (-dy / chord) +
              (this.app.state.pointerWorld.y - midpoint.y) * (dx / chord);
            const height = drag.startArcHeight + pointerNormal - drag.startPointerNormal;
            if (drag.moved || Math.abs(height - drag.startArcHeight) > this.app.getScreenToleranceWorld(1)) {
              if (!drag.moved) { this.app.pushUndo(); drag.moved = true; }
              const bowAngle = window.Geometry.clampBowAngle(
                4 * Math.atan2(2 * height, chord) * 180 / Math.PI);
              this.app.previewCurveBow(line.id, bowAngle);
              this.canvas.style.cursor = 'grabbing';
            }
          }
        }
      }

      // Dragging a selected dimension chain repositions its offset line.
      if (this.app.state.dragState?.type === 'chain-offset') {
        const drag = this.app.state.dragState;
        const chain = this.app.state.dimensionChains.find(item => item.id === drag.chainId);
        if (chain) {
          if (!drag.moved) { this.app.pushUndo('Move Dimension Chain'); drag.moved = true; }
          const pointer = this.app.state.pointerWorld;
          const signed = (pointer.x - drag.runStart.x) * drag.runNormal.x +
            (pointer.y - drag.runStart.y) * drag.runNormal.y;
          chain.side = signed >= 0 ? 1 : -1;
          chain.offset = Math.max(0.5, Math.abs(signed));
          this.canvas.style.cursor = 'grabbing';
        }
      }

      // Fence dragging
      if ((this.app.state.mode === 'fence' || this.app.state.mode === 'zoom-fence' ||
          this.app.state.dragState?.type === 'selection-marquee') && this.app.state.fenceStart) {
        this.app.state.fenceEnd = { ...this.app.state.pointerWorld };
        if (this.app.state.dragState?.type === 'selection-marquee') {
          const drag = this.app.state.dragState;
          drag.moved = drag.moved || window.Geometry.distance(drag.startWorld, this.app.state.pointerWorld) >
            this.app.getScreenToleranceWorld(2);
        }
      }

      if (this.app.state.mode === 'lasso' && this.app.state.lassoPoints.length > 0) {
        const last = this.app.state.lassoPoints[this.app.state.lassoPoints.length - 1];
        if (window.Geometry.distance(last, this.app.state.pointerWorld) > this.app.getScreenToleranceWorld(3)) {
          this.app.state.lassoPoints.push({ ...this.app.state.pointerWorld });
        }
      }

      const isDrawing = this.app.state.mode === 'draw-exterior' || this.app.state.mode === 'draw-interior';
      if (isDrawing && this.app.state.penDown && this.app.state.currentPolyPoints.length > 0) {
        const target = this.app.getDrawingTarget(this.app.state.pointerWorld);
        const anchor = this.app.state.cursor;
        const dx = target.x - anchor.x;
        const dy = target.y - anchor.y;
        const dist = Math.hypot(dx, dy);
        const mathAngle = Math.atan2(dy, dx) * 180 / Math.PI;
        const apexAngle = (90 - mathAngle + 360) % 360;
        this.app.ui.updateHUD(dist.toFixed(1), Math.round(apexAngle), true);
      }

      // Resolve the measure tool's snap once per move so the cursor marker,
      // the rubber leg, and the next click all agree on the same point.
      if (this.app.state.mode === 'measure') {
        const cursor = this.app.measureTarget(this.app.state.pointerWorld, e.shiftKey);
        this.app.state.measureCursor = cursor;
        const anchor = this.app.lastMeasurePoint();
        if (anchor) {
          this.app.ui.updateHUD(
            window.Geometry.distance(anchor, cursor.point).toFixed(1),
            Math.round(window.Geometry.bearingDegrees(anchor, cursor.point)),
            false
          );
        }
      } else if (this.app.state.measureCursor) {
        this.app.state.measureCursor = null;
      }

      this.render();
    });

    // Click
    this.canvas.addEventListener('click', (e) => {
      if (this.ignoreClick) {
        this.ignoreClick = false;
        return;
      }
      const mode = this.app.state.mode;
      const rect = this.canvas.getBoundingClientRect();
      const screenPt = { x: e.clientX - rect.left, y: e.clientY - rect.top };
      const worldPt = window.Geometry.toWorld(screenPt, this.app.state.pan, this.app.state.zoom);

      if (this.app.handleOrientClick(worldPt)) return;

      // Intercept Set Scale clicks
      if (this.app.bgImage && this.app.bgImage.scaleMode) {
        this.app.handleScaleClick(worldPt);
        return;
      }

      if (mode === 'define-area') {
        this.app.areaWorkflow?.click(worldPt);
      } else if (mode === 'draw-exterior' || mode === 'draw-interior') {
        this.app.handleCanvasClick(worldPt);
      } else if (mode === 'select') {
        this.app.selectLineAt(worldPt, e.ctrlKey);
      } else if (mode === 'place-text') {
        this.app.placeTextAt(worldPt);
      } else if (mode === 'place-symbol') {
        this.app.placeSymbolAt(worldPt);
      } else if (mode === 'insert-vertex') {
        this.app.insertVertexAt(worldPt);
      } else if (mode === 'measure') {
        this.app.handleMeasureClick(worldPt, { ortho: e.shiftKey });
      } else if (mode === 'chain-dim') {
        this.app.handleChainClick(worldPt);
      } else if (mode === 'place-shape') {
        this.app.placeShapeAt(worldPt);
      }
    });

    // The reference workflow uses a continuous chain. Double-click finishes
    // an open chain without adding another segment.
    this.canvas.addEventListener('dblclick', (e) => {
      const mode = this.app.state.mode;
      if ((mode === 'draw-exterior' || mode === 'draw-interior') && this.app.state.penDown) {
        e.preventDefault();
        this.app.penUp();
      }
      // The second click of the double-click lands on the point the first one
      // added and is discarded as a duplicate, so the run ends where it looks.
      if (mode === 'measure' && this.app.state.measureDraft) {
        e.preventDefault();
        this.app.finishMeasureRun(false);
      }
      // Double-clicking a text label edits its wording and
      // double-clicking an area opens its definition.
      if (mode === 'select') {
        const rect = this.canvas.getBoundingClientRect();
        const worldPt = window.Geometry.toWorld({ x: e.clientX - rect.left, y: e.clientY - rect.top }, this.app.state.pan, this.app.state.zoom);
        const hit = this.app.hitTestAt(worldPt);
        if (hit?.type === 'label') {
          e.preventDefault();
          const label = this.app.state.labels.find(item => item.id === hit.id);
          if (!label) return;
          this.app.ui.promptDialog('Text:', label.text).then(text => {
            if (text == null || !String(text).trim() || String(text) === label.text) return;
            if (!this.app.state.labels.includes(label)) return;
            this.app.pushUndo('Edit Text');
            label.text = String(text);
            this.app.render();
          });
        } else if (hit && (hit.type === 'polygon' || hit.type === 'areaLabel' || hit.type === 'areaSize')) {
          const poly = this.app.state.polygons.find(item => item.id === hit.id);
          if (poly && this.app.areaWorkflow?.open) {
            e.preventDefault();
            this.app.areaWorkflow.open(poly, { x: e.clientX, y: e.clientY });
          }
        }
      }
    });

    // Right-click
    this.canvas.addEventListener('contextmenu', (e) => {
      e.preventDefault();
      // Right-click ends the measurement in progress, as it does in CAD. With
      // no run open the canvas menu appears as usual.
      if (this.app.state.mode === 'measure' && this.app.state.measureDraft) {
        this.app.finishMeasureRun(false);
        return;
      }
      if (this.app.state.penDown) this.app.penUp();
      const rect = this.canvas.getBoundingClientRect();
      const worldPt = window.Geometry.toWorld({ x:e.clientX - rect.left, y:e.clientY - rect.top }, this.app.state.pan, this.app.state.zoom);
      const hit = this.app.hitTestAt(worldPt);
      // Right-clicking an unselected item selects it so the menu always
      // acts on what was clicked; empty space or an already-selected item
      // keeps the current selection intact.
      if (hit && !this.app.isHitSelected(hit)) this.app.selectLineAt(worldPt, false);
      this.app.ui.showCanvasContextMenu(e.clientX, e.clientY);
    });

    // Mousedown for pan and fence
    this.canvas.addEventListener('pointerdown', (e) => {
      const rect = this.canvas.getBoundingClientRect();
      const screenPt = { x: e.clientX - rect.left, y: e.clientY - rect.top };
      const worldPt = window.Geometry.toWorld(screenPt, this.app.state.pan, this.app.state.zoom);

      // Hold space + left-drag: temporary pan in any mode. Checked before
      // every tool's own drag handling so the hand tool always wins.
      if (e.button === 0 && this.app.state.spacePan) {
        e.preventDefault();
        this.ignoreClick = true;
        this.canvas.setPointerCapture(e.pointerId);
        this.app.state.isPanning = true;
        this.app.state.spacePanned = true;
        this.app.state.panStart = { x: e.clientX, y: e.clientY };
        this.canvas.style.cursor = 'grabbing';
        return;
      }

      if (e.button === 0 && this.app.bgImage?.editMode) {
        e.preventDefault();
        this.ignoreClick = true;
        this.canvas.setPointerCapture(e.pointerId);
        this.app.state.dragState = { type: 'photo', lastWorld: { ...worldPt } };
        return;
      }

      if (e.button === 0 && this.app.state.mode === 'curve') {
        this.ignoreClick = true;
        const hit = this.app.selectLineAt(worldPt, false);
        if (hit?.type === 'line') {
          const line = this.app.state.lines.find(item => item.id === hit.id);
          if (line) {
            this.canvas.setPointerCapture(e.pointerId);
            this.app.state.dragState = this.createCurveDragState(line, worldPt);
          }
        }
        return;
      }

      if (e.button === 0 && this.app.state.mode === 'select') {
        if (this.app.bgImage?.orientMode || this.app.bgImage?.scaleMode) return;
        this.ignoreClick = true;
        const chainHit = this.getChainHit(screenPt);
        if (chainHit) {
          e.preventDefault();
          if (!e.ctrlKey) this.app.clearSelection(false);
          if (e.ctrlKey && this.app.state.selectedChainIds.has(chainHit.chain.id)) {
            this.app.state.selectedChainIds.delete(chainHit.chain.id);
          } else {
            this.app.state.selectedChainIds.add(chainHit.chain.id);
          }
          const computed = this.app.computeChain(chainHit.chain);
          if (computed && this.app.state.selectedChainIds.has(chainHit.chain.id)) {
            this.canvas.setPointerCapture(e.pointerId);
            this.app.state.dragState = {
              type: 'chain-offset', chainId: chainHit.chain.id,
              runStart: { ...computed.start }, runNormal: { ...computed.normal }, moved: false
            };
            this.canvas.style.cursor = 'grab';
          }
          this.render();
          return;
        }
        const endpointHit = this.getSelectedLineEndpointHit(worldPt);
        if (endpointHit) {
          e.preventDefault();
          this.canvas.setPointerCapture(e.pointerId);
          this.app.state.activeLineEndpoint = { lineId:endpointHit.line.id, endpoint:endpointHit.endpoint };
          this.app.state.dragState = {
            type:'line-endpoint', lineId:endpointHit.line.id, endpoint:endpointHit.endpoint,
            startPoint:{ ...endpointHit.line[endpointHit.endpoint] },
            excludedRefs:this.app.getLineEndpointRefs(endpointHit.line[endpointHit.endpoint]), moved:false
          };
          this.canvas.style.cursor = 'grab';
          this.render();
          return;
        }
        const resizeHandle = this.getSymbolResizeHandleHit(screenPt);
        if (resizeHandle) {
          e.preventDefault();
          this.canvas.setPointerCapture(e.pointerId);
          const symbol = resizeHandle.transform.symbol;
          this.app.state.dragState = {
            type: 'resize-symbol',
            symbolId: symbol.id,
            corner: resizeHandle.corner,
            center: { x: symbol.x, y: symbol.y },
            startDistance: Math.max(1e-6, Math.hypot(worldPt.x - symbol.x, worldPt.y - symbol.y)),
            startScale: Number(symbol.scale) || 1,
            moved: false
          };
          this.canvas.style.cursor = 'nwse-resize';
          return;
        }
        const rotationHandle = this.getRotationHandleHit(screenPt);
        if (rotationHandle) {
          e.preventDefault();
          this.canvas.setPointerCapture(e.pointerId);
          const center = rotationHandle.type === 'area'
            ? rotationHandle.transform.center
            : rotationHandle.type === 'symbol'
            ? { x: rotationHandle.transform.symbol.x, y: rotationHandle.transform.symbol.y }
            : { ...rotationHandle.transform.centerWorld };
          const startAngle = Math.atan2(worldPt.y - center.y, worldPt.x - center.x);
          if (rotationHandle.type === 'area') {
            this.app.state.dragState = { type: 'rotate-area', center: { ...center }, startAngle, previewDegrees: 0, moved: false };
          } else if (rotationHandle.type === 'symbol') {
            this.app.state.dragState = {
              type: 'rotate-symbol',
              symbolId: rotationHandle.transform.symbol.id,
              center: { ...center },
              startAngle,
              startRotation: Number(rotationHandle.transform.symbol.rotation) || 0,
              previewDegrees: 0,
              moved: false
            };
          } else {
            const entity = rotationHandle.transform.entity;
            this.app.state.dragState = {
              type: 'rotate-text', entityType: rotationHandle.type, entityId: entity.id,
              center: { ...center }, startAngle,
              startRotation: Number(rotationHandle.type === 'areaLabel' ? entity.labelRotation
                : rotationHandle.type === 'areaSize' ? (entity.areaSizeRotation ?? entity.labelRotation)
                : entity.rotation) || 0,
              previewDegrees: 0, moved: false
            };
          }
          this.canvas.style.cursor = 'grabbing';
          return;
        }
        const curveHandle = this.app.state.lines.find(line => {
          if (!this.app.state.selectedLineIds.has(line.id)) return false;
          return window.Geometry.distance(worldPt, this.getCurveHandleWorld(line)) <= this.app.getScreenToleranceWorld(9);
        });
        if (curveHandle) {
          this.canvas.setPointerCapture(e.pointerId);
          this.app.state.dragState = this.createCurveDragState(curveHandle, worldPt);
          return;
        }
        // Pressing on something already selected keeps the whole selection,
        // so a fenced group drags together (previously the press collapsed
        // the selection to the item under the pointer and moved it alone).
        // A click that does not drag still narrows the selection on release.
        const preHit = e.ctrlKey ? null : this.app.hitTestAt(worldPt);
        const keepGroup = Boolean(preHit) && (this.app.isHitSelected(preHit) ||
          ((preHit.type === 'areaLabel' || preHit.type === 'areaSize') && this.app.state.selectedPolyIds.has(preHit.id)));
        const hit = keepGroup ? preHit : this.app.selectLineAt(worldPt, e.ctrlKey);
        if (hit && (keepGroup || this.app.isHitSelected(hit))) {
          this.canvas.setPointerCapture(e.pointerId);
          const bounds = this.app.getSelectionBounds?.();
          this.app.state.dragState = {
            type: 'selection',
            collapseOnClick: keepGroup ? { ...worldPt } : null,
            startWorld: { ...worldPt },
            lastWorld: { ...worldPt },
            // Reference point used by area grid snap: snapping the absolute
            // selection position keeps areas aligned to the pitch grid even
            // when the grab point starts between grid lines.
            startRef: bounds ? { x: bounds.x, y: bounds.y } : null,
            moved: false
          };
        } else if (!hit) {
          // The pointer doubles as the ordinary selection-marquee tool when
          // its drag starts on empty canvas. A direct hit still takes the
          // selection/move path above, so line selection remains immediate.
          this.canvas.setPointerCapture(e.pointerId);
          this.app.state.fenceStart = { ...worldPt };
          this.app.state.fenceEnd = { ...worldPt };
          this.app.state.dragState = {
            type: 'selection-marquee',
            startWorld: { ...worldPt },
            additive: e.ctrlKey,
            moved: false
          };
        }
        return;
      }

      if (e.button === 1 || (e.button === 0 && this.app.state.mode === 'pan')) {
        e.preventDefault();
        this.canvas.setPointerCapture(e.pointerId);
        this.app.state.isPanning = true;
        this.app.state.panStart = { x: e.clientX, y: e.clientY };
        this.canvas.style.cursor = 'grabbing';
      }
      // Fence tool: start drag
      if (e.button === 0 && (this.app.state.mode === 'fence' || this.app.state.mode === 'zoom-fence')) {
        this.canvas.setPointerCapture(e.pointerId);
        this.app.state.fenceStart = { ...worldPt };
        this.app.state.fenceEnd = { ...this.app.state.fenceStart };
      }

      if (e.button === 0 && this.app.state.mode === 'lasso') {
        this.canvas.setPointerCapture(e.pointerId);
        this.app.state.lassoPoints = [{ ...worldPt }];
      }
    });

    // Mouseup
    window.addEventListener('pointerup', (e) => {
      if (this.app.state.dragState) {
        const drag = this.app.state.dragState;
        if (drag.type === 'selection-marquee') {
          if (drag.moved && this.app.state.fenceStart && this.app.state.fenceEnd) {
            const fs = this.app.state.fenceStart;
            const fe = this.app.state.fenceEnd;
            this.app.fenceSelect(
              { x: Math.min(fs.x, fe.x), y: Math.min(fs.y, fe.y) },
              { x: Math.max(fs.x, fe.x), y: Math.max(fs.y, fe.y) },
              drag.additive
            );
          }
          this.app.state.fenceStart = null;
          this.app.state.fenceEnd = null;
        }
        if (drag.type === 'rotate-area' && drag.moved && Math.abs(drag.previewDegrees) > 0.001) {
          this.app.rotateSelected(drag.previewDegrees);
        }
        if (drag.type === 'selection' && !drag.moved && drag.collapseOnClick) {
          this.app.selectLineAt(drag.collapseOnClick, false);
        }
        if (drag.type === 'selection' && drag.moved) {
          this.app.resnapSelectedSymbols?.();
          // Walls dropped into a closed arrangement enclose an area just as
          // if the last wall had been drawn there.
          this.app.finalizeSelectionMove();
        }
        if (drag.type === 'line-endpoint') {
          if (drag.moved) this.app.finalizeLineEndpointEdit(drag.lineId);
          this.app.ui.updateHUD('', '', false);
        }
        if (drag.type === 'curve' && drag.moved) {
          const line = this.app.state.lines.find(item => item.id === drag.lineId);
          if (line) this.app.commitCurveBow(line.id, line.bowAngle || 0);
        }
        if (drag.type === 'curve' || drag.type === 'rotate-area' || drag.type === 'rotate-symbol' || drag.type === 'resize-symbol' || drag.type === 'rotate-text' || drag.type === 'line-endpoint') {
          this.canvas.style.cursor = this.app.state.mode === 'curve' ? 'crosshair' : 'default';
        }
        this.ignoreClick = true;
        this.app.state.dragState = null;
        this.render();
        setTimeout(() => { this.ignoreClick = false; }, 0);
      }
      if (this.app.state.isPanning) {
        this.app.state.isPanning = false;
        this.app.state.panStart = null;
        this.setCursor();
      }

      // Fence tool: finish drag and select
      if ((this.app.state.mode === 'fence' || this.app.state.mode === 'zoom-fence') && this.app.state.fenceStart && this.app.state.fenceEnd) {
        const fs = this.app.state.fenceStart;
        const fe = this.app.state.fenceEnd;
        const worldMin = {
          x: Math.min(fs.x, fe.x),
          y: Math.min(fs.y, fe.y)
        };
        const worldMax = {
          x: Math.max(fs.x, fe.x),
          y: Math.max(fs.y, fe.y)
        };
        // Only select if dragged a meaningful distance
        if (Math.abs(fe.x - fs.x) > 0.5 || Math.abs(fe.y - fs.y) > 0.5) {
          if (this.app.state.mode === 'zoom-fence') this.app.zoomToWorldRect(worldMin, worldMax);
          else this.app.fenceSelect(worldMin, worldMax);
        }
        this.app.state.fenceStart = null;
        this.app.state.fenceEnd = null;
        this.render();
      }

      if (this.app.state.mode === 'lasso' && this.app.state.lassoPoints.length > 2) {
        this.app.lassoSelect(this.app.state.lassoPoints, e.ctrlKey);
        this.app.state.lassoPoints = [];
        this.render();
      }

      if (this.canvas.hasPointerCapture && this.canvas.hasPointerCapture(e.pointerId)) {
        this.canvas.releasePointerCapture(e.pointerId);
      }
    });

    // Wheel: zoom OR curve bow
    this.canvas.addEventListener('wheel', (e) => {
      e.preventDefault();
      const isDrawing = this.app.state.mode === 'draw-exterior' || this.app.state.mode === 'draw-interior';

      if (this.app.bgImage?.editMode) {
        this.app.scaleBgImageBy(e.deltaY < 0 ? 1.05 : 0.95);
        return;
      }

      if (this.app.state.mode === 'select') {
        const curveStep = this.app.state.curveMode === 'height'
          ? (e.ctrlKey || e.shiftKey ? 0.1 : 1)
          : (e.ctrlKey ? 1 : (e.shiftKey ? 5 : 15));
        if (this.app.adjustSelectedCurves(e.deltaY < 0 ? curveStep : -curveStep)) return;
        const scaleFactor = e.deltaY < 0 ? 1.08 : 0.92;
        if (this.app.state.selectedAreaLabelIds.size || this.app.state.selectedAreaSizeIds?.size ||
            this.app.state.selectedLabelIds.size || this.app.state.selectedSymbolIds.size) {
          this.app.scaleSelected(scaleFactor);
          return;
        }
      }
      
      // Shift+wheel while drawing = bow the last wall into a curve
      if (e.shiftKey && isDrawing && this.app.state.lines.length > 0) {
        const lastLine = this.app.state.lines[this.app.state.lines.length - 1];
        const increment = this.app.state.curveMode === 'height'
          ? (e.ctrlKey ? 0.1 : 1)
          : (e.ctrlKey ? 1 : 15);
        const delta = e.deltaY < 0 ? increment : -increment;
        this.app.pushUndo();
        if (this.app.state.curveMode === 'height') {
          const chord = window.Geometry.distance(lastLine.start, lastLine.end);
          lastLine.arcHeight = (lastLine.arcHeight || 0) + delta;
          lastLine.bowAngle = window.Geometry.clampBowAngle(
            4 * Math.atan2(2 * lastLine.arcHeight, chord) * 180 / Math.PI);
        } else {
          lastLine.bowAngle = window.Geometry.clampBowAngle((lastLine.bowAngle || 0) + delta);
          const chord = window.Geometry.distance(lastLine.start, lastLine.end);
          lastLine.arcHeight = chord / 2 * Math.tan(lastLine.bowAngle * Math.PI / 720);
        }
        this.render();
        return;
      }

      // Normal zoom
      const zoomFactor = e.deltaY < 0 ? 1.1 : 0.9;
      const rect = this.canvas.getBoundingClientRect();
      const mx = e.clientX - rect.left;
      const my = e.clientY - rect.top;
      this.app.zoomTo(this.app.state.zoom * zoomFactor, mx, my);
    }, { passive: false });
  }

  resize() {
    const parent = this.canvas.parentElement;
    const dpr = Math.max(1, window.devicePixelRatio || 1);
    this.viewWidth = parent.clientWidth;
    this.viewHeight = parent.clientHeight;
    // Backing store is scaled by devicePixelRatio so lines and text stay
    // crisp on high-DPI (4K) displays; all drawing code works in CSS pixels.
    this.canvas.width = Math.round(this.viewWidth * dpr);
    this.canvas.height = Math.round(this.viewHeight * dpr);
    this.canvas.style.width = this.viewWidth + 'px';
    this.canvas.style.height = this.viewHeight + 'px';
    this.render();
  }

  /**
   * Render the current sketch into an arbitrary canvas. Used for exporting
   * at a resolution different from the on-screen canvas.
   */
  renderTo(targetCanvas, options = {}) {
    const originalCanvas = this.canvas;
    const originalCtx = this.ctx;
    this.canvas = targetCanvas;
    this.ctx = targetCanvas.getContext('2d');
    // The target canvas is already sized at the final output resolution, so
    // render 1:1 without the device-pixel-ratio scale used for the on-screen
    // canvas; otherwise the export is zoomed 2x (DPR) and clipped to the
    // top-left corner.
    const previousDpr = this.renderDpr;
    const previousTextZoom = this.exportTextZoom;
    this.renderDpr = 1;
    // Keep text legible: the whole sketch is fitted into the target at a
    // small zoom, which would otherwise shrink labels and dimensions to
    // unreadable dots in the exported image. The clamp scales with the
    // export resolution (2048px base) so text stays proportionally legible
    // at higher resolutions like 3200px and 4800px instead of remaining a
    // fixed pixel size while the drawing grows.
    this.exportTextZoom = 2 * (targetCanvas.width / 2048);
    this.render(options);
    this.renderDpr = previousDpr;
    this.exportTextZoom = previousTextZoom;
    this.canvas = originalCanvas;
    this.ctx = originalCtx;
  }

  // Zoom used for text sizing. During exports the drawing zoom can be very
  // small (whole sketch fitted into the image), so text is clamped to a
  // minimum that stays legible in the output.
  getTextZoom() {
    const zoom = this.app.state.zoom || 1;
    return this.exportTextZoom ? Math.max(zoom, this.exportTextZoom) : zoom;
  }

  // Ghost another page beneath the current one (view-only underlay). Drawn
  // with a fixed alpha before any current-page entities and never hit-tested.
  drawUnderlay() {
    const page = this.app.getUnderlayPage?.();
    if (!page) return;
    const s = this.app.state;
    const ctx = this.ctx;
    ctx.save();
    ctx.strokeStyle = 'rgba(148, 163, 184, 0.35)';
    ctx.lineWidth = 1.5;
    ctx.lineCap = 'round';
    ctx.lineJoin = 'round';
    (page.lines || []).forEach(line => {
      if (!line.start || !line.end) return;
      const a = window.Geometry.toScreen(line.start, s.pan, s.zoom);
      const b = window.Geometry.toScreen(line.end, s.pan, s.zoom);
      ctx.beginPath();
      ctx.moveTo(a.x, a.y);
      ctx.lineTo(b.x, b.y);
      ctx.stroke();
    });
    (page.polygons || []).forEach(poly => {
      const points = poly.points || [];
      if (points.length < 3) return;
      ctx.beginPath();
      points.forEach((point, index) => {
        const p = window.Geometry.toScreen(point, s.pan, s.zoom);
        if (index === 0) ctx.moveTo(p.x, p.y);
        else ctx.lineTo(p.x, p.y);
      });
      ctx.closePath();
      ctx.stroke();
    });
    ctx.restore();
  }

  render(options = {}) {
    const ctx = this.ctx;
    const dpr = this.renderDpr || Math.max(1, window.devicePixelRatio || 1);
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    const clean = options.clean === true;
    if (options.transparent) ctx.clearRect(0, 0, this.viewWidth, this.viewHeight);
    else {
      ctx.fillStyle = this.colors.bg;
      ctx.fillRect(0, 0, this.viewWidth, this.viewHeight);
    }
    if (!clean || this.app.state.options?.saveImageWithSketch) this.drawBgImage();
    if (!clean && this.app.state.gridEnabled) this.drawGrid();
    if (!clean) this.drawUnderlay();
    // Walls covered by a dimension chain suppress their per-wall label for
    // this frame only; the chains themselves repaint after the entities.
    this.chainCoverage = this.app.getChainCoverage?.() || new Set();
    // Paint entities bottom-to-top by layer, so a foreground layer's fill and
    // linework always overlap (and obscure) whatever sits on layers below it,
    // no matter the entity type. Each layer's polygons, walls, labels and
    // symbols are painted together in one pass.
    this.app.flattenLayerOrder().forEach(layer => {
      // Groups normally hold no entities; if one does, they paint at the
      // group's own stack position, and hide with the group like any layer.
      if (!this.app.isLayerVisible(layer.id)) return;
      if (this.app.state.filters.areas !== false) this.drawPolygons(layer.id);
      this.drawLines(layer.id);
      if (this.app.state.filters.labels !== false) this.drawLabels(layer.id);
      if (this.app.state.filters.symbols !== false) this.drawSymbols(layer.id);
    });
    this.drawChains();
    if (!clean) {
      this.drawTransformHandles();
      this.drawLineEndpointHandles();
      this.drawWitnessLines();
      this.drawVertices();
      this.drawRubberBand();
      this.drawKeyboardPreview();
      this.drawKeyboardPointer();
      this.drawSnapPreview();
      this.drawFenceRect();
      this.drawLasso();
      this.drawPlacementPreview();
      this.drawScalePoints();
      this.drawMeasurements();
    }
  }

  drawGrid() {
    const s = this.app.state;
    const pxPerFt = window.Geometry.PIXELS_PER_FOOT * s.zoom;
    const ctx = this.ctx;
    ctx.lineWidth = 1;
    const offsetX = s.pan.x % pxPerFt;
    const offsetY = s.pan.y % pxPerFt;

    ctx.strokeStyle = this.colors.grid10;
    ctx.beginPath();
    for (let x = offsetX; x < this.viewWidth; x += pxPerFt) {
      ctx.moveTo(x, 0); ctx.lineTo(x, this.viewHeight);
    }
    for (let y = offsetY; y < this.viewHeight; y += pxPerFt) {
      ctx.moveTo(0, y); ctx.lineTo(this.viewWidth, y);
    }
    ctx.stroke();

    const majorPx = pxPerFt * 10;
    if (majorPx > 20) {
      const majorOffX = s.pan.x % majorPx;
      const majorOffY = s.pan.y % majorPx;
      ctx.strokeStyle = this.colors.grid50;
      ctx.beginPath();
      for (let x = majorOffX; x < this.viewWidth; x += majorPx) {
        ctx.moveTo(x, 0); ctx.lineTo(x, this.viewHeight);
      }
      for (let y = majorOffY; y < this.viewHeight; y += majorPx) {
        ctx.moveTo(0, y); ctx.lineTo(this.viewWidth, y);
      }
      ctx.stroke();
    }
  }

  pointsMatch(a, b, tolerance = 0.0001) {
    return Math.abs(a.x - b.x) <= tolerance && Math.abs(a.y - b.y) <= tolerance;
  }

  getCurveHandleWorld(line) {
    const midpoint = { x: (line.start.x + line.end.x) / 2, y: (line.start.y + line.end.y) / 2 };
    const dx = line.end.x - line.start.x;
    const dy = line.end.y - line.start.y;
    const chord = Math.hypot(dx, dy);
    if (chord < 0.001) return midpoint;
    const height = Number.isFinite(line.arcHeight)
      ? line.arcHeight
      : chord / 2 * Math.tan((line.bowAngle || 0) * Math.PI / 720);
    return {
      x: midpoint.x + (-dy / chord) * height,
      y: midpoint.y + (dx / chord) * height
    };
  }

  createCurveDragState(line, pointerWorld) {
    const dx = line.end.x - line.start.x;
    const dy = line.end.y - line.start.y;
    const chord = Math.hypot(dx, dy);
    const midpoint = { x: (line.start.x + line.end.x) / 2, y: (line.start.y + line.end.y) / 2 };
    const startArcHeight = Number.isFinite(line.arcHeight)
      ? line.arcHeight
      : chord / 2 * Math.tan((Number(line.bowAngle) || 0) * Math.PI / 720);
    const startPointerNormal = chord > 0.001
      ? (pointerWorld.x - midpoint.x) * (-dy / chord) +
        (pointerWorld.y - midpoint.y) * (dx / chord)
      : 0;
    return {
      type: 'curve',
      lineId: line.id,
      moved: false,
      startArcHeight,
      startPointerNormal
    };
  }

  getCurveMetrics(line) {
    const chord = window.Geometry.distance(line.start, line.end);
    const bowAngle = window.Geometry.clampBowAngle(line.bowAngle);
    const theta = Math.abs(bowAngle) * Math.PI / 180;
    if (chord < 0.001 || theta < 1e-6) return null;
    const radius = (chord / 2) / Math.sin(theta / 2);
    const rise = chord / 2 * Math.tan(theta / 4);
    const dx = line.end.x - line.start.x;
    const dy = line.end.y - line.start.y;
    const normal = { x: -dy / chord, y: dx / chord };
    const sign = Math.sign(bowAngle) || 1;
    const midpoint = { x: (line.start.x + line.end.x) / 2, y: (line.start.y + line.end.y) / 2 };
    const centerDistance = radius * Math.cos(theta / 2);
    return {
      bowAngle,
      theta,
      chord,
      radius,
      rise,
      arcLength: radius * theta,
      midpoint,
      apex: {
        x: midpoint.x + normal.x * rise * sign,
        y: midpoint.y + normal.y * rise * sign
      },
      center: {
        x: midpoint.x - normal.x * centerDistance * sign,
        y: midpoint.y - normal.y * centerDistance * sign
      }
    };
  }

  getPathDimensionSide(points, edgeStart, edgeEnd) {
    if (!points || points.length < 3) {
      return this.app.state.dimensionPlacement === 'in' ? -1 : 1;
    }

    const signedArea = window.Geometry.calculateSignedArea(points);
    let followsPath = true;

    for (let i = 0; i < points.length; i++) {
      const next = points[(i + 1) % points.length];
      if (this.pointsMatch(points[i], edgeStart) && this.pointsMatch(next, edgeEnd)) {
        followsPath = true;
        break;
      }
      if (this.pointsMatch(points[i], edgeEnd) && this.pointsMatch(next, edgeStart)) {
        followsPath = false;
        break;
      }
    }

    // Canvas Y is inverted relative to world Y. For a CCW world polygon the
    // exterior is the positive screen normal; CW polygons use the negative.
    let outsideSide = signedArea >= 0 ? 1 : -1;
    if (!followsPath) outsideSide *= -1;
    return this.app.state.dimensionPlacement === 'in' ? -outsideSide : outsideSide;
  }

  getLineDimensionSide(line) {
    const polygon = this.app.state.polygons.find(poly => {
      for (let i = 0; i < poly.points.length; i++) {
        const next = poly.points[(i + 1) % poly.points.length];
        if ((this.pointsMatch(poly.points[i], line.start) && this.pointsMatch(next, line.end)) ||
            (this.pointsMatch(poly.points[i], line.end) && this.pointsMatch(next, line.start))) {
          return true;
        }
      }
      return false;
    });

    if (polygon) return this.getPathDimensionSide(polygon.points, line.start, line.end);

    if (this.app.state.penDown && this.app.state.currentLineIds.includes(line.id)) {
      return this.getPathDimensionSide(this.app.state.currentPolyPoints, line.start, line.end);
    }

    return this.app.state.dimensionPlacement === 'in' ? -1 : 1;
  }

  // Line color from the area definition of an adjoining typed polygon, so an
  // area's Line color setting tints its boundary walls even when the walls are
  // drawn or loaded after the setting was saved. Lines get a draw color
  // stamped at creation, so for boundary walls the area setting must win;
  // walls outside typed areas keep their own color.
  buildAreaLineColors() {
    const colors = new Map();
    (this.app.state.polygons || []).forEach(poly => {
      if (!poly.type || poly.type === 'UND') return;
      const definition = this.app.areaCodes?.find(item => item.code === poly.type);
      const color = poly.lineColor || definition?.color;
      if (!color) return;
      (poly.edges || []).forEach(edge => {
        if (edge.lineId != null && !colors.has(edge.lineId)) colors.set(edge.lineId, color);
      });
    });
    return colors;
  }

  // Filled wall solids for geometric wall thickness. Interior walls fill
  // first so exterior color wins where walls overlap at junctions.
  drawWallSolids(layerId, areaLineColors, solidByLine) {
    const s = this.app.state;
    const ctx = this.ctx;
    const toScreen = point => window.Geometry.toScreen(point, s.pan, s.zoom);

    const fillSolid = (line, solid) => {
      const isSelected = s.selectedLineIds.has(line.id);
      const isCurveTarget = isSelected &&
        (s.mode === 'curve' || (s.dragState?.type === 'curve' && s.dragState.lineId === line.id));
      const isExt = line.type === 'exterior';
      const color = isCurveTarget
        ? this.colors.wallPreview
        : isSelected
        ? this.colors.selectLine
        : (areaLineColors.get(line.id) || line.color || (isExt ? this.colors.wallExt : this.colors.wallInt));
      ctx.save();
      ctx.fillStyle = color;
      solid.pieces.forEach(piece => {
        ctx.beginPath();
        piece.forEach((point, index) => {
          const screen = toScreen(point);
          if (index === 0) ctx.moveTo(screen.x, screen.y);
          else ctx.lineTo(screen.x, screen.y);
        });
        ctx.closePath();
        ctx.fill();
      });
      if (isSelected) {
        // Outline keeps the selection readable over area fills.
        ctx.strokeStyle = this.colors.selectLine;
        ctx.lineWidth = 1.5;
        ctx.setLineDash([]);
        solid.pieces.forEach(piece => {
          ctx.beginPath();
          piece.forEach((point, index) => {
            const screen = toScreen(point);
            if (index === 0) ctx.moveTo(screen.x, screen.y);
            else ctx.lineTo(screen.x, screen.y);
          });
          ctx.closePath();
          ctx.stroke();
        });
      }
      ctx.restore();
    };

    ['interior', 'exterior'].forEach(passType => {
      s.lines.forEach(line => {
        if (line.type !== passType) return;
        if (layerId != null && this.app.getEntityLayerId(line) !== layerId) return;
        if (!this.app.isEntityVisible(line)) return;
        const isSelected = s.selectedLineIds.has(line.id);
        if (s.filters[passType === 'exterior' ? 'exterior' : 'interior'] === false && !isSelected) return;
        if (s.penDown && s.currentLineIds.includes(line.id)) return; // legacy preview while drawing
        const solid = solidByLine.get(line.id);
        if (solid) fillSolid(line, solid);
        else if (line.bowAngle && Math.abs(line.bowAngle) > 0.5) {
          this.drawCurvedWallSolid(line, isSelected);
        }
      });
    });
  }

  // Bowed wall solid approximated by offsetting the centerline arc by ±t/2
  // (annular sector). Hosted-opening gaps are not cut in bowed walls (v1).
  drawCurvedWallSolid(line, isSelected) {
    const s = this.app.state;
    const ctx = this.ctx;
    const metrics = this.getCurveMetrics(line);
    if (!metrics) return;
    const thickness = this.app.wallThicknessOf(line);
    if (!(thickness > 0)) return;
    const half = thickness / 2;
    const center = metrics.center;
    const mod = v => ((v % (2 * Math.PI)) + 2 * Math.PI) % (2 * Math.PI);
    const aStart = Math.atan2(line.start.y - center.y, line.start.x - center.x);
    const aEnd = Math.atan2(line.end.y - center.y, line.end.x - center.x);
    const aApex = Math.atan2(metrics.apex.y - center.y, metrics.apex.x - center.x);
    // Travel direction whose sweep passes through the apex.
    const ccw = mod(aApex - aStart) <= mod(aEnd - aStart);
    const sweep = ccw ? mod(aEnd - aStart) : -mod(aStart - aEnd);
    const steps = Math.max(8, Math.ceil(Math.abs(sweep) / (Math.PI / 24)));
    const outerRadius = metrics.radius + half;
    const innerRadius = Math.max(0.01, metrics.radius - half);
    const at = (radius, angle) => window.Geometry.toScreen(
      { x: center.x + Math.cos(angle) * radius, y: center.y + Math.sin(angle) * radius }, s.pan, s.zoom);

    const isCurveTarget = isSelected &&
      (s.mode === 'curve' || (s.dragState?.type === 'curve' && s.dragState.lineId === line.id));
    const color = isCurveTarget
      ? this.colors.wallPreview
      : isSelected
      ? this.colors.selectLine
      : (line.color || (line.type === 'exterior' ? this.colors.wallExt : this.colors.wallInt));

    ctx.save();
    ctx.fillStyle = color;
    ctx.beginPath();
    for (let i = 0; i <= steps; i++) {
      const point = at(outerRadius, aStart + sweep * i / steps);
      if (i === 0) ctx.moveTo(point.x, point.y);
      else ctx.lineTo(point.x, point.y);
    }
    for (let i = steps; i >= 0; i--) {
      const point = at(innerRadius, aStart + sweep * i / steps);
      ctx.lineTo(point.x, point.y);
    }
    ctx.closePath();
    ctx.fill();
    ctx.restore();
  }

  drawLines(layerId = null) {
    const s = this.app.state;
    const ctx = this.ctx;
    const areaLineColors = this.buildAreaLineColors();
    // Geometric wall thickness: filled wall solids replace the single-line
    // strokes. The active (pen-down) walls keep the legacy stroke preview.
    const solids = this.app.getWallSolids();
    const solidByLine = solids ? new Map(solids.solids.map(solid => [solid.lineId, solid])) : null;
    if (solids) this.drawWallSolids(layerId, areaLineColors, solidByLine);

    s.lines.forEach(line => {
      if (layerId != null && this.app.getEntityLayerId(line) !== layerId) return;
      if (!this.app.isEntityVisible(line)) return;
      const isSelected = s.selectedLineIds.has(line.id);
      const isExt = line.type === 'exterior';
      if (s.filters[isExt ? 'exterior' : 'interior'] === false && !isSelected) return;
      const isActive = s.penDown && s.currentLineIds.includes(line.id);
      const isBowed = Boolean(line.bowAngle && Math.abs(line.bowAngle) > 0.5);
      const solidHandled = Boolean(solids) && !isActive &&
        (solidByLine.has(line.id) || (isBowed && this.app.wallThicknessOf(line) > 0));
      const isCurveTarget = isSelected &&
        (s.mode === 'curve' || (s.dragState?.type === 'curve' && s.dragState.lineId === line.id));
      const isCurveDrag = s.dragState?.type === 'curve' && s.dragState.lineId === line.id && s.dragState.moved;

      const wallColor = isCurveTarget
        ? this.colors.wallPreview
        : isSelected
        ? this.colors.selectLine
        : (isActive ? this.colors.wallActive : (areaLineColors.get(line.id) || line.color || (isExt ? this.colors.wallExt : this.colors.wallInt)));
      ctx.strokeStyle = wallColor;
      const baseWidth = Number(line.lineThickness) || this.wallWidths[isExt ? 'exterior' : 'interior'];
      ctx.lineWidth = isSelected ? Math.max(5, baseWidth + 1) : baseWidth;
      ctx.setLineDash(line.lineStyle === 'dashed' ? [9, 6] : line.lineStyle === 'dotted' ? [2, 5] : []);
      ctx.lineCap = 'round';
      ctx.lineJoin = 'round';
      
      const start = window.Geometry.toScreen(line.start, s.pan, s.zoom);
      const end = window.Geometry.toScreen(line.end, s.pan, s.zoom);

      if (!solidHandled && isBowed) {
        // Draw as an arc
        this.drawArc(ctx, start, end, line.bowAngle);
      } else if (!solidHandled) {
        const visible = window.WallSolids.openingRuns(line, s.symbols);

        ctx.beginPath();
        visible.forEach(segment => {
          const startRatio = segment.start;
          const endRatio = segment.end;
          const segmentStart = window.Geometry.toScreen({
            x: line.start.x + (line.end.x - line.start.x) * startRatio,
            y: line.start.y + (line.end.y - line.start.y) * startRatio
          }, s.pan, s.zoom);
          const segmentEnd = window.Geometry.toScreen({
            x: line.start.x + (line.end.x - line.start.x) * endRatio,
            y: line.start.y + (line.end.y - line.start.y) * endRatio
          }, s.pan, s.zoom);
          ctx.moveTo(segmentStart.x, segmentStart.y);
          ctx.lineTo(segmentEnd.x, segmentEnd.y);
        });
        ctx.stroke();
      }

      if (isSelected) {
        const midpoint = window.Geometry.toScreen(this.getCurveHandleWorld(line), s.pan, s.zoom);
        ctx.save();
        ctx.fillStyle = '#22C55E';
        ctx.strokeStyle = '#ECFDF5';
        ctx.lineWidth = 1;
        ctx.fillRect(midpoint.x - 4, midpoint.y - 4, 8, 8);
        ctx.strokeRect(midpoint.x - 4, midpoint.y - 4, 8, 8);
        ctx.restore();
      }
      
      if (!isCurveDrag && (s.dimensionsEnabled || isActive) && s.filters.dimensions !== false && !line.dimensionHidden && !this.chainCoverage?.has(line.id)) {
        ctx.setLineDash([]);
        this.drawDimension(
          line.start,
          line.end,
          start,
          end,
          wallColor,
          this.getLineDimensionSide(line),
          window.Geometry.pinnedDimensionPoint(line),
          s.selectedDimensionIds.has(line.id)
        );
      }
      if (isCurveDrag) this.drawCurveGuides(line);
      ctx.setLineDash([]);
    });
  }

  traceArc(ctx, start, end, bowAngleDeg) {
    const bowRad = bowAngleDeg * Math.PI / 180;
    const mx = (start.x + end.x) / 2;
    const my = (start.y + end.y) / 2;
    const dx = end.x - start.x;
    const dy = end.y - start.y;
    const chordLen = Math.hypot(dx, dy);
    if (chordLen < 1) { ctx.lineTo(end.x, end.y); return; }

    // Radius from chord length and arc angle
    const halfAngle = Math.abs(bowRad) / 2;
    if (halfAngle < 0.01) { ctx.lineTo(end.x, end.y); return; }
    const radius = (chordLen / 2) / Math.sin(halfAngle);

    // Normal to chord
    const nx = -dy / chordLen;
    const ny = dx / chordLen;

    // Distance from chord midpoint to arc center
    const h = radius * Math.cos(halfAngle);
    const sign = bowRad > 0 ? 1 : -1;

    const cx = mx + nx * h * sign;
    const cy = my + ny * h * sign;

    // Start and end angles relative to center
    const startAngle = Math.atan2(start.y - cy, start.x - cx);
    // Use the explicit signed sweep so Canvas follows major arcs rather than
    // silently choosing the shorter path between the same endpoints.
    ctx.arc(cx, cy, Math.abs(radius), startAngle, startAngle + bowRad, bowRad < 0);
  }

  // Draw an arc between two screen points with a given bow angle.
  drawArc(ctx, start, end, bowAngleDeg) {
    ctx.beginPath();
    ctx.moveTo(start.x, start.y);
    this.traceArc(ctx, start, end, bowAngleDeg);
    ctx.stroke();
  }

  drawCurveGuides(line) {
    const metrics = this.getCurveMetrics(line);
    if (!metrics) return;
    const s = this.app.state;
    const ctx = this.ctx;
    const start = window.Geometry.toScreen(line.start, s.pan, s.zoom);
    const end = window.Geometry.toScreen(line.end, s.pan, s.zoom);
    const midpoint = window.Geometry.toScreen(metrics.midpoint, s.pan, s.zoom);
    const apex = window.Geometry.toScreen(metrics.apex, s.pan, s.zoom);
    const center = window.Geometry.toScreen(metrics.center, s.pan, s.zoom);
    const guideColor = '#22C55E';
    const chordColor = 'rgba(239, 68, 68, 0.62)';

    ctx.save();
    ctx.lineWidth = 1.25;
    ctx.strokeStyle = chordColor;
    ctx.setLineDash([8, 6]);
    ctx.beginPath();
    ctx.moveTo(start.x, start.y);
    ctx.lineTo(end.x, end.y);
    ctx.stroke();

    ctx.strokeStyle = guideColor;
    ctx.setLineDash([4, 4]);
    ctx.beginPath();
    ctx.moveTo(midpoint.x, midpoint.y);
    ctx.lineTo(apex.x, apex.y);
    ctx.stroke();

    ctx.setLineDash([]);
    ctx.globalAlpha = 0.72;
    ctx.beginPath();
    ctx.moveTo(center.x, center.y);
    ctx.lineTo(start.x, start.y);
    ctx.moveTo(center.x, center.y);
    ctx.lineTo(end.x, end.y);
    ctx.stroke();

    const tickX = apex.x - midpoint.x;
    const tickY = apex.y - midpoint.y;
    const tickLength = Math.hypot(tickX, tickY);
    if (tickLength > 1) {
      const tx = -tickY / tickLength * 5;
      const ty = tickX / tickLength * 5;
      ctx.beginPath();
      ctx.moveTo(midpoint.x - tx, midpoint.y - ty);
      ctx.lineTo(midpoint.x + tx, midpoint.y + ty);
      ctx.moveTo(apex.x - tx, apex.y - ty);
      ctx.lineTo(apex.x + tx, apex.y + ty);
      ctx.stroke();
    }
    ctx.restore();

    this.drawGuideLabel(`${metrics.arcLength.toFixed(1)}'`, apex.x, apex.y - 15,
      guideColor, 'bold 12px monospace');
    this.drawGuideLabel(`${metrics.chord.toFixed(1)}'`, midpoint.x, midpoint.y - 11,
      this.colors.wallPreview, 'bold 11px monospace');

    const riseMid = { x: (midpoint.x + apex.x) / 2, y: (midpoint.y + apex.y) / 2 };
    const riseDx = apex.x - midpoint.x;
    const riseDy = apex.y - midpoint.y;
    const riseScreenLength = Math.hypot(riseDx, riseDy) || 1;
    this.drawGuideLabel(`${metrics.rise.toFixed(1)}'`,
      riseMid.x + (-riseDy / riseScreenLength) * 18,
      riseMid.y + (riseDx / riseScreenLength) * 18,
      guideColor);

    const radiusLabel = { x: (center.x + end.x) / 2, y: (center.y + end.y) / 2 };
    this.drawGuideLabel(`${metrics.radius.toFixed(1)}'`, radiusLabel.x, radiusLabel.y,
      guideColor);
    this.drawGuideLabel(`${Math.round(Math.abs(metrics.bowAngle))}\u00B0`, center.x, center.y - 14,
      guideColor, 'bold 11px monospace');
  }

  // Screen anchor (position and rotation) of a wall's measurement label.
  // A label the user has moved keeps its stored world position; otherwise
  // it sits at the default offset from the wall midpoint.
  getDimensionAnchor(line) {
    const s = this.app.state;
    const start = window.Geometry.toScreen(line.start, s.pan, s.zoom);
    const end = window.Geometry.toScreen(line.end, s.pan, s.zoom);
    const dx = end.x - start.x;
    const dy = end.y - start.y;
    const len = Math.hypot(dx, dy);
    if (len < 1) return null;
    let angle = Math.atan2(dy, dx);
    if (angle > Math.PI / 2) angle -= Math.PI;
    if (angle < -Math.PI / 2) angle += Math.PI;
    const override = window.Geometry.pinnedDimensionPoint(line);
    if (Number.isFinite(override?.x) && Number.isFinite(override?.y)) {
      const screen = window.Geometry.toScreen(override, s.pan, s.zoom);
      return { x: screen.x, y: screen.y, angle };
    }
    const nx = -dy / len;
    const ny = dx / len;
    const offset = Math.max(9, 16 * this.getTextZoom());
    const side = this.getLineDimensionSide(line);
    return {
      x: (start.x + end.x) / 2 + nx * offset * side,
      y: (start.y + end.y) / 2 + ny * offset * side,
      angle
    };
  }

  // World position of a wall's measurement label (moved position or default).
  getDimensionLabelWorld(line) {
    const anchor = this.getDimensionAnchor(line);
    if (!anchor) return null;
    return window.Geometry.toWorld({ x: anchor.x, y: anchor.y }, this.app.state.pan, this.app.state.zoom);
  }

  // Find the measurement label under a world point, if any.
  hitDimensionAt(worldPt) {
    const s = this.app.state;
    if (s.filters.dimensions === false) return null;
    const screenPt = window.Geometry.toScreen(worldPt, s.pan, s.zoom);
    const options = s.options || {};
    const fontSize = Math.max(9, (Number(options.dimensionFontSize) || 11) * this.getTextZoom());
    let best = null;
    let bestDist = Infinity;
    this.ctx.save();
    this.ctx.font = `${options.dimensionItalic ? 'italic ' : ''}${options.dimensionBold ? 'bold ' : ''}${fontSize}px "${options.dimensionFont || 'Arial'}", monospace`;
    s.lines.forEach(line => {
      if (!this.app.isDimensionVisible(line)) return;
      const anchor = this.getDimensionAnchor(line);
      if (!anchor) return;
      const text = this.app.formatLength?.(window.Geometry.distance(line.start, line.end)) || '';
      const halfWidth = this.ctx.measureText(text).width / 2 + 6;
      const halfHeight = fontSize / 2 + 6;
      const dx = screenPt.x - anchor.x;
      const dy = screenPt.y - anchor.y;
      const localX = Math.cos(-anchor.angle) * dx - Math.sin(-anchor.angle) * dy;
      const localY = Math.sin(-anchor.angle) * dx + Math.cos(-anchor.angle) * dy;
      if (Math.abs(localX) <= halfWidth && Math.abs(localY) <= halfHeight) {
        const d = Math.hypot(localX, localY);
        if (d < bestDist) { bestDist = d; best = line.id; }
      }
    });
    this.ctx.restore();
    return best;
  }

  drawDimension(worldStart, worldEnd, screenStart, screenEnd, color, side = 1, overrideWorld = null, selected = false) {
    const dist = window.Geometry.distance(worldStart, worldEnd);
    if (dist < 0.1) return;
    const options = this.app.state.options || {};
    if (options.minimumDimensionEnabled && dist < (Number(options.minimumDimension) || 0)) return;
    const ctx = this.ctx;
    const dx = screenEnd.x - screenStart.x;
    const dy = screenEnd.y - screenStart.y;
    const len = Math.hypot(dx, dy);
    if (len < 1) return;
    let angle = Math.atan2(dy, dx);
    if (angle > Math.PI / 2) angle -= Math.PI;
    if (angle < -Math.PI / 2) angle += Math.PI;
    let labelX, labelY;
    if (Number.isFinite(overrideWorld?.x) && Number.isFinite(overrideWorld?.y)) {
      const overrideScreen = window.Geometry.toScreen(overrideWorld, this.app.state.pan, this.app.state.zoom);
      labelX = overrideScreen.x;
      labelY = overrideScreen.y;
    } else {
      const nx = -dy / len;
      const ny = dx / len;
      const offset = Math.max(7, 10 * this.getTextZoom());
      labelX = (screenStart.x + screenEnd.x) / 2 + nx * offset * side;
      labelY = (screenStart.y + screenEnd.y) / 2 + ny * offset * side;
    }
    const text = this.app.formatLength?.(dist) || (dist.toFixed(1) + "'");
    const fontSize = Math.max(9, (Number(options.dimensionFontSize) || 11) * this.getTextZoom());
    ctx.save();
    ctx.font = `${options.dimensionItalic ? 'italic ' : ''}${options.dimensionBold ? 'bold ' : ''}${fontSize}px "${options.dimensionFont || 'Arial'}", monospace`;
    if (selected) {
      // Leader line back to the wall plus a highlight box around the label
      ctx.save();
      ctx.setLineDash([3, 3]);
      ctx.strokeStyle = this.colors.selectLine;
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo((screenStart.x + screenEnd.x) / 2, (screenStart.y + screenEnd.y) / 2);
      ctx.lineTo(labelX, labelY);
      ctx.stroke();
      ctx.restore();
    }
    ctx.translate(labelX, labelY);
    ctx.rotate(angle);
    if (selected) {
      const boxWidth = ctx.measureText(text).width + 8;
      const boxHeight = fontSize + 8;
      ctx.fillStyle = this.colors.selectPoly;
      ctx.strokeStyle = this.colors.selectLine;
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.rect(-boxWidth / 2, -boxHeight / 2, boxWidth, boxHeight);
      ctx.fill();
      ctx.stroke();
    }
    ctx.fillStyle = selected
      ? this.colors.selectLine
      : (options.sameColorAsLine ? (color || this.colors.dimLabel) : this.colors.dimLabel);
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillText(text, 0, 0);
    ctx.restore();
  }

  // Screen-space geometry of a chain's dimension tier(s), shared by the
  // renderer and the hit-tester.
  getChainGeometry(chain) {
    const computed = this.app.computeChain(chain);
    if (!computed || computed.refs.length < 2) return null;
    const s = this.app.state;
    const signedOffset = Math.max(0.5, Number(chain.offset) || 3) * (chain.side || 1);
    const pointAt = (t, extraOffset = 0) => window.Geometry.toScreen({
      x: computed.start.x + computed.dir.x * t + computed.normal.x * (signedOffset + extraOffset),
      y: computed.start.y + computed.dir.y * t + computed.normal.y * (signedOffset + extraOffset)
    }, s.pan, s.zoom);
    const wallAt = t => window.Geometry.toScreen({
      x: computed.start.x + computed.dir.x * t,
      y: computed.start.y + computed.dir.y * t
    }, s.pan, s.zoom);
    return { computed, signedOffset, pointAt, wallAt };
  }

  // Hit-test a screen point against every chain's dimension lines.
  getChainHit(screenPt) {
    const tolerance = 8;
    const segDist = (p, a, b) => {
      const dx = b.x - a.x, dy = b.y - a.y;
      const lenSq = dx * dx + dy * dy;
      if (lenSq < 1e-6) return Math.hypot(p.x - a.x, p.y - a.y);
      const t = Math.max(0, Math.min(1, ((p.x - a.x) * dx + (p.y - a.y) * dy) / lenSq));
      return Math.hypot(p.x - (a.x + t * dx), p.y - (a.y + t * dy));
    };
    for (const chain of this.app.state.dimensionChains) {
      const geometry = this.getChainGeometry(chain);
      if (!geometry) continue;
      const { computed, pointAt, signedOffset } = geometry;
      const tier1 = segDist(screenPt, pointAt(0), pointAt(computed.length));
      if (tier1 <= tolerance) return { chain };
      if (chain.showOverall) {
        const extra = Math.sign(signedOffset || 1) * Math.max(1.5, Math.abs(signedOffset) * 0.5);
        const tier2 = segDist(screenPt, pointAt(0, extra), pointAt(computed.length, extra));
        if (tier2 <= tolerance) return { chain };
      }
    }
    return null;
  }

  // Chained dimension strings: one shared dimension line with per-span
  // labels, plus an overall dimension on an outer tier.
  drawChains() {
    const s = this.app.state;
    if (!s.dimensionChains.length || s.filters.dimensions === false) return;
    const ctx = this.ctx;
    const options = s.options || {};
    const fontSize = Math.max(9, (Number(options.dimensionFontSize) || 11) * this.getTextZoom());
    const font = `${options.dimensionItalic ? 'italic ' : ''}${options.dimensionBold ? 'bold ' : ''}${fontSize}px "${options.dimensionFont || 'Arial'}", monospace`;
    const minimum = options.minimumDimensionEnabled ? (Number(options.minimumDimension) || 0) : 0;

    s.dimensionChains.forEach(chain => {
      const geometry = this.getChainGeometry(chain);
      if (!geometry) return;
      const { computed, pointAt, wallAt, signedOffset } = geometry;
      const selected = s.selectedChainIds.has(chain.id);
      const lineColor = selected ? this.colors.selectLine : (this.colors.dimLabel || '#94a3b8');
      const refs = computed.refs;
      const length = computed.length;

      // Screen-space axis of the dimension line, for label rotation.
      const a0 = pointAt(0);
      const aEnd = pointAt(length);
      let angle = Math.atan2(aEnd.y - a0.y, aEnd.x - a0.x);
      if (angle > Math.PI / 2) angle -= Math.PI;
      if (angle < -Math.PI / 2) angle += Math.PI;
      // Screen-space outward normal of the dimension line (points away from
      // the wall run regardless of chain side).
      const axisLen = Math.hypot(aEnd.x - a0.x, aEnd.y - a0.y) || 1;
      const rawNx = -(aEnd.y - a0.y) / axisLen;
      const rawNy = (aEnd.x - a0.x) / axisLen;
      const outward = rawNx * (wallAt(0).x - a0.x) + rawNy * (wallAt(0).y - a0.y) < 0 ? 1 : -1;
      const nx = rawNx * outward;
      const ny = rawNy * outward;

      ctx.save();
      ctx.strokeStyle = lineColor;
      ctx.lineWidth = selected ? 1.5 : 1;
      ctx.setLineDash([]);

      // Witness lines from the wall to just past the dimension line.
      refs.forEach(t => {
        const wall = wallAt(t);
        const dim = pointAt(t);
        ctx.beginPath();
        ctx.moveTo(wall.x + nx * 3, wall.y + ny * 3);
        ctx.lineTo(dim.x + nx * 4, dim.y + ny * 4);
        ctx.stroke();
      });

      // Shared dimension line with architectural tick marks at each ref.
      ctx.beginPath();
      ctx.moveTo(a0.x, a0.y);
      ctx.lineTo(aEnd.x, aEnd.y);
      ctx.stroke();
      refs.forEach(t => {
        const p = pointAt(t);
        ctx.beginPath();
        ctx.moveTo(p.x - 4, p.y + 4);
        ctx.lineTo(p.x + 4, p.y - 4);
        ctx.stroke();
      });

      // Per-span labels.
      ctx.font = font;
      ctx.textAlign = 'center';
      ctx.textBaseline = 'middle';
      for (let i = 0; i < refs.length - 1; i++) {
        const span = refs[i + 1] - refs[i];
        if (chain.hiddenSpans?.includes(i)) continue;
        if (span < minimum || span < 0.05) continue;
        const mid = pointAt((refs[i] + refs[i + 1]) / 2);
        const text = this.app.formatLength?.(span) || (span.toFixed(1) + "'");
        ctx.save();
        ctx.translate(mid.x + nx * (fontSize * 0.9), mid.y + ny * (fontSize * 0.9));
        ctx.rotate(angle);
        ctx.fillStyle = lineColor;
        ctx.fillText(text, 0, 0);
        ctx.restore();
      }

      // Overall dimension on a second, outer tier.
      if (chain.showOverall) {
        const extra = Math.sign(signedOffset || 1) * Math.max(1.5, Math.abs(signedOffset) * 0.5);
        const b0 = pointAt(0, extra);
        const bEnd = pointAt(length, extra);
        [0, length].forEach(t => {
          const inner = pointAt(t);
          const outer = pointAt(t, extra);
          ctx.beginPath();
          ctx.moveTo(inner.x + nx * 2, inner.y + ny * 2);
          ctx.lineTo(outer.x + nx * 4, outer.y + ny * 4);
          ctx.stroke();
        });
        ctx.beginPath();
        ctx.moveTo(b0.x, b0.y);
        ctx.lineTo(bEnd.x, bEnd.y);
        ctx.stroke();
        [b0, bEnd].forEach(p => {
          ctx.beginPath();
          ctx.moveTo(p.x - 4, p.y + 4);
          ctx.lineTo(p.x + 4, p.y - 4);
          ctx.stroke();
        });
        const overallText = this.app.formatLength?.(length) || (length.toFixed(1) + "'");
        const mid = { x: (b0.x + bEnd.x) / 2 + nx * (fontSize * 0.9), y: (b0.y + bEnd.y) / 2 + ny * (fontSize * 0.9) };
        ctx.save();
        ctx.translate(mid.x, mid.y);
        ctx.rotate(angle);
        ctx.font = `bold ${font.replace(/^bold /, '')}`;
        ctx.fillStyle = lineColor;
        ctx.fillText(overallText, 0, 0);
        ctx.restore();
      }

      if (selected) {
        // Offset grab handle at the middle of the dimension line.
        const mid = pointAt(length / 2);
        ctx.fillStyle = this.colors.selectLine;
        ctx.beginPath();
        ctx.arc(mid.x, mid.y, 4, 0, Math.PI * 2);
        ctx.fill();
      }
      ctx.restore();
    });
  }

  drawGuideLabel(text, x, y, color, font = '11px monospace') {
    const ctx = this.ctx;
    ctx.save();
    ctx.font = font;
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillStyle = color;
    ctx.fillText(text, x, y);
    ctx.restore();
  }

  drawLiveAngle(previousWorld, anchorWorld, targetWorld) {
    const s = this.app.state;
    const previous = window.Geometry.toScreen(previousWorld, s.pan, s.zoom);
    const anchor = window.Geometry.toScreen(anchorWorld, s.pan, s.zoom);
    const target = window.Geometry.toScreen(targetWorld, s.pan, s.zoom);
    const v1 = { x: previous.x - anchor.x, y: previous.y - anchor.y };
    const v2 = { x: target.x - anchor.x, y: target.y - anchor.y };
    const len1 = Math.hypot(v1.x, v1.y);
    const len2 = Math.hypot(v2.x, v2.y);
    if (len1 < 4 || len2 < 4) return;

    const cosine = Math.max(-1, Math.min(1, (v1.x * v2.x + v1.y * v2.y) / (len1 * len2)));
    const angleDeg = Math.acos(cosine) * 180 / Math.PI;
    const a1 = Math.atan2(v1.y, v1.x);
    const a2 = Math.atan2(v2.y, v2.x);
    let delta = a2 - a1;
    while (delta > Math.PI) delta -= Math.PI * 2;
    while (delta < -Math.PI) delta += Math.PI * 2;

    const radius = 28;
    const ctx = this.ctx;
    ctx.save();
    ctx.strokeStyle = this.colors.wallPreview;
    ctx.lineWidth = 1.5;
    ctx.beginPath();
    ctx.arc(anchor.x, anchor.y, radius, a1, a1 + delta, delta < 0);
    ctx.stroke();
    ctx.restore();

    const midAngle = a1 + delta / 2;
    this.drawGuideLabel(
      Math.round(angleDeg) + '°',
      anchor.x + Math.cos(midAngle) * (radius + 14),
      anchor.y + Math.sin(midAngle) * (radius + 14),
      this.colors.wallPreview,
      'bold 11px monospace'
    );
  }

  drawRiseRun(anchorWorld, targetWorld) {
    const dxWorld = targetWorld.x - anchorWorld.x;
    const dyWorld = targetWorld.y - anchorWorld.y;
    if (Math.abs(dxWorld) < 0.1 || Math.abs(dyWorld) < 0.1) return;

    const s = this.app.state;
    const anchor = window.Geometry.toScreen(anchorWorld, s.pan, s.zoom);
    const target = window.Geometry.toScreen(targetWorld, s.pan, s.zoom);
    const corner = { x: target.x, y: anchor.y };
    const ctx = this.ctx;

    ctx.save();
    ctx.setLineDash([4, 4]);
    ctx.strokeStyle = 'rgba(239, 68, 68, 0.65)';
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(anchor.x, anchor.y);
    ctx.lineTo(corner.x, corner.y);
    ctx.lineTo(target.x, target.y);
    ctx.stroke();
    ctx.restore();

    this.drawGuideLabel(
      Math.abs(dxWorld).toFixed(1) + "'",
      (anchor.x + corner.x) / 2,
      anchor.y - 11,
      this.colors.wallPreview
    );
    this.drawGuideLabel(
      Math.abs(dyWorld).toFixed(1) + "'",
      target.x + 15,
      (corner.y + target.y) / 2,
      this.colors.wallPreview
    );
  }

  drawReturnGuide(startWorld, targetWorld) {
    const distance = window.Geometry.distance(startWorld, targetWorld);
    if (distance < 0.5) return;

    const s = this.app.state;
    const start = window.Geometry.toScreen(startWorld, s.pan, s.zoom);
    const target = window.Geometry.toScreen(targetWorld, s.pan, s.zoom);
    const ctx = this.ctx;
    ctx.save();
    ctx.setLineDash([3, 5]);
    ctx.strokeStyle = this.colors.witnessLine;
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(start.x, start.y);
    ctx.lineTo(target.x, target.y);
    ctx.stroke();
    ctx.restore();

    const midX = (start.x + target.x) / 2;
    const midY = (start.y + target.y) / 2;
    this.drawGuideLabel(distance.toFixed(1) + "'", midX, midY - 10, this.colors.dimLabel);
  }

  drawWitnessLines() {
    const s = this.app.state;
    if (s.options?.alignmentLines === false) return;
    if (!s.penDown || s.currentPolyPoints.length === 0) return;
    if (s.mode !== 'draw-exterior' && s.mode !== 'draw-interior') return;

    const ctx = this.ctx;
    const cursorWorld = s.pointerWorld;
    const threshold = this.app.getScreenToleranceWorld(8);
    const allPoints = [];
    s.lines.forEach(l => { allPoints.push(l.start, l.end); });

    ctx.save();
    ctx.setLineDash([4, 6]);
    ctx.strokeStyle = this.colors.witnessLine;
    ctx.lineWidth = 1;
    allPoints.forEach(pt => {
      if (Math.abs(pt.x - cursorWorld.x) < threshold) {
        const sp = window.Geometry.toScreen(pt, s.pan, s.zoom);
        ctx.beginPath(); ctx.moveTo(sp.x, 0); ctx.lineTo(sp.x, this.viewHeight); ctx.stroke();
      }
      if (Math.abs(pt.y - cursorWorld.y) < threshold) {
        const sp = window.Geometry.toScreen(pt, s.pan, s.zoom);
        ctx.beginPath(); ctx.moveTo(0, sp.y); ctx.lineTo(this.viewWidth, sp.y); ctx.stroke();
      }
    });
    ctx.restore();
  }

  drawSnapPreview() {
    const s = this.app.state;
    if (s.options?.alignmentPoints === false) return;
    if (!s.penDown) return;
    if (s.mode !== 'draw-exterior' && s.mode !== 'draw-interior') return;
    const snapped = this.app.getDrawingTarget(s.pointerWorld);
    const ss = window.Geometry.toScreen(snapped, s.pan, s.zoom);
    const ctx = this.ctx;
    ctx.save();
    ctx.strokeStyle = this.colors.snapDot;
    ctx.lineWidth = 1.5;
    ctx.beginPath();
    ctx.moveTo(ss.x - 6, ss.y); ctx.lineTo(ss.x + 6, ss.y);
    ctx.moveTo(ss.x, ss.y - 6); ctx.lineTo(ss.x, ss.y + 6);
    ctx.stroke();
    ctx.restore();
  }

  drawVertices() {
    const s = this.app.state;
    if (s.options?.alignmentPoints === false) return;
    const ctx = this.ctx;
    const points = s.currentPolyPoints;
    if (!points || points.length === 0) return;
    points.forEach((pt, index) => {
      const sp = window.Geometry.toScreen(pt, s.pan, s.zoom);
      ctx.beginPath();
      ctx.arc(sp.x, sp.y, index === 0 ? 6 : 4, 0, Math.PI * 2);
      ctx.fillStyle = index === 0 ? '#FFFFFF' : this.colors.vertex;
      ctx.fill();
      ctx.strokeStyle = this.colors.vertexStroke;
      ctx.lineWidth = index === 0 ? 2.5 : 1.5;
      ctx.stroke();
    });
  }

  drawRubberBand() {
    const s = this.app.state;
    if (!s.penDown || s.currentPolyPoints.length === 0) return;
    if (s.mode !== 'draw-exterior' && s.mode !== 'draw-interior') return;

    const lastPt = s.currentPolyPoints[s.currentPolyPoints.length - 1];
    // When the keyboard cursor has moved away from the last committed point
    // (keyboard input), skip the rubber band so it doesn't draw a solid line
    // to the mouse position. The keyboard preview (dashed line) handles this.
    if (lastPt.x !== s.cursor.x || lastPt.y !== s.cursor.y) return;

    const ctx = this.ctx;
    const lastScreen = window.Geometry.toScreen(lastPt, s.pan, s.zoom);
    const snapped = this.app.getDrawingTarget(s.pointerWorld);
    const targetScreen = window.Geometry.toScreen(snapped, s.pan, s.zoom);
    const isExt = s.mode === 'draw-exterior';

    ctx.save();
    ctx.strokeStyle = this.colors.wallPreview;
    ctx.lineWidth = this.wallWidths[isExt ? 'exterior' : 'interior'];
    ctx.lineCap = 'round';
    ctx.beginPath();
    ctx.moveTo(lastScreen.x, lastScreen.y);
    ctx.lineTo(targetScreen.x, targetScreen.y);
    ctx.stroke();
    ctx.restore();

    const dist = window.Geometry.distance(lastPt, snapped);
    if (dist > 0.3) {
      const previewPoints = [...s.currentPolyPoints, snapped];
      const dimensionSide = this.getPathDimensionSide(previewPoints, lastPt, snapped);
      this.drawDimension(
        lastPt,
        snapped,
        lastScreen,
        targetScreen,
        this.colors.wallPreview,
        dimensionSide
      );
      this.drawRiseRun(lastPt, snapped);

      if (s.currentPolyPoints.length >= 2) {
        const previousPt = s.currentPolyPoints[s.currentPolyPoints.length - 2];
        this.drawLiveAngle(previousPt, lastPt, snapped);
        this.drawReturnGuide(s.currentPolyPoints[0], snapped);
      }
    }
  }

  drawKeyboardPreview() {
    const s = this.app.state;
    if (!s.penDown || s.currentPolyPoints.length === 0) return;
    if (s.mode !== 'draw-exterior' && s.mode !== 'draw-interior') return;

    const lastPt = s.currentPolyPoints[s.currentPolyPoints.length - 1];
    const cursor = s.cursor;
    // Only draw if cursor has moved past the last committed point
    if (lastPt.x === cursor.x && lastPt.y === cursor.y) return;

    const ctx = this.ctx;
    const lastScreen = window.Geometry.toScreen(lastPt, s.pan, s.zoom);
    const cursorScreen = window.Geometry.toScreen(cursor, s.pan, s.zoom);
    const isExt = s.mode === 'draw-exterior';

    ctx.save();
    ctx.strokeStyle = this.colors.wallPreview;
    ctx.lineWidth = this.wallWidths[isExt ? 'exterior' : 'interior'];
    ctx.lineCap = 'round';
    ctx.setLineDash([8, 5]);
    ctx.beginPath();
    ctx.moveTo(lastScreen.x, lastScreen.y);
    ctx.lineTo(cursorScreen.x, cursorScreen.y);
    ctx.stroke();
    ctx.restore();

    // Dimension label for the preview line
    const dist = Math.hypot(cursor.x - lastPt.x, cursor.y - lastPt.y);
    if (dist > 0.3) {
      const previewPoints = [...s.currentPolyPoints, cursor];
      const dimensionSide = this.getPathDimensionSide(previewPoints, lastPt, cursor);
      this.drawDimension(
        lastPt,
        cursor,
        lastScreen,
        cursorScreen,
        this.colors.wallPreview,
        dimensionSide
      );
      this.drawRiseRun(lastPt, cursor);

      if (s.currentPolyPoints.length >= 2) {
        const previousPt = s.currentPolyPoints[s.currentPolyPoints.length - 2];
        this.drawLiveAngle(previousPt, lastPt, cursor);
        this.drawReturnGuide(s.currentPolyPoints[0], cursor);
      }
    }
  }

  // During keyboard drawing the OS cursor cannot be repositioned by script,
  // so while keyboard-pointer mode is active the OS cursor is hidden and the
  // pointer image is painted at the keyboard cursor (end of the line).
  drawKeyboardPointer() {
    const s = this.app.state;
    if (!s.keyboardPointer || !s.penDown) return;
    if (s.mode !== 'draw-exterior' && s.mode !== 'draw-interior') return;
    const { img, size } = this.getDrawingCursorImage();
    if (!img || !img.complete || !img.naturalWidth) return;
    const sp = window.Geometry.toScreen(s.cursor, s.pan, s.zoom);
    const half = size / 2;
    this.ctx.drawImage(img, sp.x - half, sp.y - half, size, size);
  }

  // Returns { img, size } of the pointer variant to paint on-canvas, matching
  // the size option and theme used for the OS cursor.
  getDrawingCursorImage() {
    const size = Number(this.app.state.options?.pointerSize);
    const valid = [32, 64, 128].includes(size) ? size : 128;
    const invert = this.app.state.canvasTheme === 'light';
    const key = `${valid}|${invert}`;
    let entry = this.cursorDrawCache.get(key);
    if (!entry) {
      const img = new Image();
      entry = { img, size: valid, failed: false };
      img.onload = () => this.render();
      img.onerror = () => { entry.failed = true; this.render(); };
      img.src = __cursorUrl(`cursor-${valid}${invert ? '-light' : ''}.png`);
      this.cursorDrawCache.set(key, entry);
    }
    if (entry.failed || (entry.img.complete && !entry.img.naturalWidth)) {
      // Variant file not built; fall back to the original pointer image.
      return { img: this.cursorImg, size: this.cursorImg.naturalWidth || 128 };
    }
    return { img: entry.img, size: entry.size };
  }

  drawPolygons(layerId = null) {
    const s = this.app.state;
    const ctx = this.ctx;
    s.polygons.forEach(poly => {
      if (poly.points.length < 3) return;
      if (layerId != null && this.app.getEntityLayerId(poly) !== layerId) return;
      if (!this.app.isEntityVisible(poly)) return;
      const isSelected = s.selectedPolyIds.has(poly.id);
      const areaDefinition = this.app.areaCodes?.find(item => item.code === poly.type);
      const fillPattern = poly.fillPattern || areaDefinition?.fillStyle || 'solid';
      const fillColor = poly.fillColor || areaDefinition?.fillColor || this.colors.fillGLA;
      const fillOpacity = poly.fillOpacity ?? areaDefinition?.fillOpacity;
      const outlineColor = poly.lineColor || areaDefinition?.color || this.colors.polyOutline;

      ctx.beginPath();
      const start = window.Geometry.toScreen(poly.points[0], s.pan, s.zoom);
      ctx.moveTo(start.x, start.y);
      for (let i = 0; i < poly.points.length; i++) {
        const nextIndex = (i + 1) % poly.points.length;
        const edgeStart = window.Geometry.toScreen(poly.points[i], s.pan, s.zoom);
        const edgeEnd = window.Geometry.toScreen(poly.points[nextIndex], s.pan, s.zoom);
        const bowAngle = Number(poly.edges?.[i]?.bowAngle) || 0;
        if (Math.abs(bowAngle) > 0.5) this.traceArc(ctx, edgeStart, edgeEnd, bowAngle);
        else ctx.lineTo(edgeEnd.x, edgeEnd.y);
      }
      ctx.closePath();
      
      // Draw-first geometry stays visually as clean wall linework until an
      // area definition is applied, matching the reference workflow.
      if (isSelected || poly.type !== 'UND') {
        if (poly.type !== 'UND' && fillPattern !== 'none') {
          ctx.fillStyle = fillOpacity == null ? fillColor : this.colorWithAlpha(fillColor, Number(fillOpacity) / 100);
          ctx.fill();
          if (fillPattern !== 'solid') {
            ctx.save();
            ctx.clip();
            ctx.strokeStyle = outlineColor;
            ctx.fillStyle = outlineColor;
            ctx.globalAlpha = 0.42;
            ctx.lineWidth = 1;
            const spacing = 10;
            if (fillPattern === 'diagonal' || fillPattern === 'crosshatch') {
              for (let x = -this.viewHeight; x < this.viewWidth + this.viewHeight; x += spacing) {
                ctx.beginPath(); ctx.moveTo(x, 0); ctx.lineTo(x - this.viewHeight, this.viewHeight); ctx.stroke();
              }
            }
            if (fillPattern === 'crosshatch') {
              for (let x = 0; x < this.viewWidth + this.viewHeight; x += spacing) {
                ctx.beginPath(); ctx.moveTo(x, 0); ctx.lineTo(x + this.viewHeight, this.viewHeight); ctx.stroke();
              }
            }
            if (fillPattern === 'dots') {
              for (let x = 4; x < this.viewWidth; x += spacing) for (let y = 4; y < this.viewHeight; y += spacing) {
                ctx.beginPath(); ctx.arc(x, y, 1.2, 0, Math.PI * 2); ctx.fill();
              }
            }
            ctx.restore();
          }
        }
        if (isSelected) { ctx.fillStyle = this.colors.selectPoly; ctx.fill(); }
        ctx.strokeStyle = isSelected ? this.colors.selectPolyOutline : outlineColor;
        ctx.lineWidth = isSelected ? Math.max(2, Number(poly.lineThickness) || 1) : (Number(poly.lineThickness) || 1);
        ctx.setLineDash(poly.lineStyle === 'dashed' ? [9, 6] : poly.lineStyle === 'dotted' ? [2, 5] : []);
        ctx.stroke();
        ctx.setLineDash([]);
      }

      if (poly.type === 'UND') return;

      // Area label and calculated size are separate text objects. They share
      // typography by default, but each has its own position and transform.
      const groups = this.getAreaLabelGroups(poly);
      const drawAreaText = (type, lines) => {
        if (!lines.length) return;
        const isSize = type === 'areaSize';
        const position = isSize
          ? (this.app.getAreaSizePosition?.(poly) || this.app.getAreaLabelPosition(poly))
          : this.app.getAreaLabelPosition(poly);
        const center = window.Geometry.toScreen(position, s.pan, s.zoom);
        const scale = Number(isSize ? (poly.areaSizeScale ?? poly.labelScale) : poly.labelScale) || 1;
        const rotation = Number(isSize ? (poly.areaSizeRotation ?? poly.labelRotation) : poly.labelRotation) || 0;
        ctx.save();
        ctx.translate(center.x, center.y);
        ctx.rotate(rotation * Math.PI / 180);
        ctx.fillStyle = poly.labelColor || this.colors.dimLabel;
        ctx.textAlign = 'center';
        lines.forEach((text, index) => {
          const options = s.options || {};
          const isFirstAutoPostLine = !isSize ? index === 0 : !groups.label.length && index === 0;
          const weight = ((poly.labelBold ?? options.autoPostBold) || isFirstAutoPostLine) ? 'bold ' : '';
          const fontSize = Math.max(9, (Number(poly.labelFontSize) || Number(options.autoPostFontSize) || (index === 0 ? 12 : 11)) * scale * this.getTextZoom());
          ctx.font = `${(poly.labelItalic ?? options.autoPostItalic) ? 'italic ' : ''}${weight}${fontSize}px "${poly.labelFontFamily || options.autoPostFont || 'Inter'}", sans-serif`;
          const y = (index - (lines.length - 1) / 2) * Math.max(9, 15 * scale * this.getTextZoom());
          ctx.fillText(text, 0, y);
          if (poly.labelUnderline) {
            const width = ctx.measureText(text).width;
            ctx.beginPath();
            ctx.moveTo(-width / 2, y + fontSize * 0.43);
            ctx.lineTo(width / 2, y + fontSize * 0.43);
            ctx.strokeStyle = ctx.fillStyle;
            ctx.lineWidth = Math.max(1, fontSize / 14);
            ctx.stroke();
          }
        });
        ctx.restore();
      };
      drawAreaText('areaLabel', groups.label);
      drawAreaText('areaSize', groups.size);
    });
  }

  // --- Fence Selection Rectangle ---
  drawFenceRect() {
    const s = this.app.state;
    const pointerMarquee = s.mode === 'select' && s.dragState?.type === 'selection-marquee';
    if ((s.mode !== 'fence' && s.mode !== 'zoom-fence' && !pointerMarquee) || !s.fenceStart || !s.fenceEnd) return;

    const ctx = this.ctx;
    const a = window.Geometry.toScreen(s.fenceStart, s.pan, s.zoom);
    const b = window.Geometry.toScreen(s.fenceEnd, s.pan, s.zoom);

    const x = Math.min(a.x, b.x);
    const y = Math.min(a.y, b.y);
    const w = Math.abs(b.x - a.x);
    const h = Math.abs(b.y - a.y);

    ctx.save();
    ctx.fillStyle = this.colors.fenceFill;
    ctx.fillRect(x, y, w, h);
    ctx.setLineDash([4, 4]);
    ctx.strokeStyle = this.colors.fenceStroke;
    ctx.lineWidth = 1;
    ctx.strokeRect(x, y, w, h);
    ctx.setLineDash([]);
    ctx.restore();
  }

  drawLasso() {
    const s = this.app.state;
    if (s.mode !== 'lasso' || s.lassoPoints.length < 2) return;
    const ctx = this.ctx;
    ctx.save();
    ctx.setLineDash([4, 4]);
    ctx.strokeStyle = this.colors.fenceStroke;
    ctx.fillStyle = this.colors.fenceFill;
    ctx.beginPath();
    s.lassoPoints.forEach((point, index) => {
      const screen = window.Geometry.toScreen(point, s.pan, s.zoom);
      if (index === 0) ctx.moveTo(screen.x, screen.y);
      else ctx.lineTo(screen.x, screen.y);
    });
    ctx.closePath();
    ctx.fill();
    ctx.stroke();
    ctx.restore();
  }

  drawLabels(layerId = null) {
    const s = this.app.state;
    const ctx = this.ctx;

    s.labels.forEach(label => {
      if (layerId != null && this.app.getEntityLayerId(label) !== layerId) return;
      if (!this.app.isEntityVisible(label)) return;
      const screen = window.Geometry.toScreen({ x: label.x, y: label.y }, s.pan, s.zoom);
      const isSelected = s.selectedLabelIds.has(label.id);
      const fontSize = Math.max(9, (label.fontSize || 12) * this.getTextZoom());

      ctx.save();
      ctx.translate(screen.x, screen.y);
      ctx.rotate((label.rotation || 0) * Math.PI / 180);
      const fontFamily = label.fontFamily || 'Arial';
      ctx.font = `${label.italic ? 'italic ' : ''}${label.bold ? 'bold ' : ''}${fontSize}px "${fontFamily}", sans-serif`;
      ctx.textAlign = 'center';
      ctx.textBaseline = 'middle';

      const metrics = ctx.measureText(label.text);
      const pad = 4;
      const bgW = metrics.width + pad * 2;
      const bgH = fontSize + pad * 2;
      const backgroundOpacity = Math.max(0, Math.min(100, Number(s.options?.textBackgroundOpacity) || 0)) / 100;

      if (isSelected || backgroundOpacity > 0) {
        ctx.fillStyle = isSelected ? 'rgba(99, 179, 237, 0.25)' :
          this.colorWithAlpha(s.options?.textBackground || '#111111', backgroundOpacity);
        ctx.beginPath();
        ctx.roundRect(-bgW / 2, -bgH / 2, bgW, bgH, 3);
        ctx.fill();
      }

      if (isSelected) {
        ctx.strokeStyle = '#63B3ED';
        ctx.lineWidth = 1.5;
        ctx.beginPath();
        ctx.roundRect(-bgW / 2, -bgH / 2, bgW, bgH, 3);
        ctx.stroke();
      }

      // Labels saved with the legacy automatic colour follow the canvas theme.
      const ink = label.color && String(label.color).toLowerCase() !== '#e2e8f0' ? label.color : null;
      ctx.fillStyle = ink || (isSelected ? '#63B3ED' : (s.canvasTheme === 'light' ? '#1f2937' : '#E2E8F0'));
      ctx.fillText(label.text, 0, 0);
      if (label.underline) {
        ctx.strokeStyle = ctx.fillStyle;
        ctx.lineWidth = Math.max(1, fontSize / 14);
        ctx.beginPath();
        ctx.moveTo(-metrics.width / 2, fontSize * 0.43);
        ctx.lineTo(metrics.width / 2, fontSize * 0.43);
        ctx.stroke();
      }
      ctx.restore();
    });
  }

  // Symbol rendering definitions
  static SYMBOL_DEFS = {
    // Doors
    'door-single':   { icon: 'ph-door-open', label: 'Door' },
    'door-double':   { icon: 'ph-door-open', label: 'Double Door' },
    'door-sliding':  { icon: 'ph-arrows-horizontal', label: 'Sliding Door' },
    'door-pocket':   { icon: 'ph-arrow-line-right', label: 'Pocket Door' },
    'door-bi-fold':  { icon: 'ph-arrows-in-simple', label: 'Bi-Fold' },
    'door-garage':   { icon: 'ph-garage', label: 'Garage Door' },
    // Windows
    'window-single': { icon: 'ph-browser', label: 'Window' },
    'window-double': { icon: 'ph-browsers', label: 'Double Window' },
    'window-bay':    { icon: 'ph-rectangle', label: 'Bay Window' },
    // Stairs
    'stairs-straight': { icon: 'ph-stairs', label: 'Stairs' },
    'stairs-l':      { icon: 'ph-stairs', label: 'L-Stairs' },
    'stairs-spiral': { icon: 'ph-spiral', label: 'Spiral Stairs' },
    // Fixtures
    'fixture-toilet':  { icon: 'ph-toilet', label: 'Toilet' },
    'fixture-sink':    { icon: 'ph-drop', label: 'Sink' },
    'fixture-tub':     { icon: 'ph-bathtub', label: 'Tub' },
    'fixture-shower':  { icon: 'ph-shower', label: 'Shower' },
    'fixture-washer':  { icon: 'ph-washing-machine', label: 'Washer' },
    'fixture-dryer':   { icon: 'ph-fan', label: 'Dryer' },
    // Misc
    'misc-fireplace':  { icon: 'ph-fire', label: 'Fireplace' },
    'misc-closet':     { icon: 'ph-wardrobe', label: 'Closet' },
    'misc-counter':    { icon: 'ph-minus', label: 'Counter' },
    'misc-island':     { icon: 'ph-square', label: 'Island' },
    'misc-hvac':       { icon: 'ph-thermometer', label: 'HVAC' },
    'misc-water-heater': { icon: 'ph-flame', label: 'Water Heater' }
  };

  getSymbolDefinition(type) {
    return window.SKETCH_SYMBOL_LIBRARY?.symbols?.[type]
      || window.SKETCH_SYMBOL_DEFS?.[type]?.definition
      || null;
  }

  traceSymbolPolyline(ctx, vertices, closed) {
    if (!vertices?.length) return;
    ctx.moveTo(vertices[0].x, vertices[0].y);
    const count = closed ? vertices.length : vertices.length - 1;
    for (let index = 0; index < count; index++) {
      const start = vertices[index];
      const end = vertices[(index + 1) % vertices.length];
      const bulge = Number(start.b) || 0;
      const chord = Math.hypot(end.x - start.x, end.y - start.y);
      if (Math.abs(bulge) < 1e-7 || chord < 1e-7) {
        ctx.lineTo(end.x, end.y);
        continue;
      }
      const normalX = -(end.y - start.y) / chord;
      const normalY = (end.x - start.x) / chord;
      const centerOffset = chord * (1 - bulge * bulge) / (4 * bulge);
      const centerX = (start.x + end.x) / 2 + normalX * centerOffset;
      const centerY = (start.y + end.y) / 2 + normalY * centerOffset;
      const radius = chord * (1 + bulge * bulge) / (4 * Math.abs(bulge));
      const startAngle = Math.atan2(start.y - centerY, start.x - centerX);
      const sweep = 4 * Math.atan(bulge);
      ctx.arc(centerX, centerY, radius, startAngle, startAngle + sweep, sweep < 0);
    }
  }

  renderSymbolGeometry(ctx, definition, pixelScale, options = {}) {
    if (!definition?.geometry?.length || !Number.isFinite(pixelScale) || pixelScale <= 0) return;
    const stroke = options.stroke || (this.app.state.canvasTheme === 'light' ? '#334155' : '#E2E8F0');
    const background = options.background || this.colors.bg;
    const selected = Boolean(options.selected);

    ctx.save();
    ctx.scale(pixelScale, -pixelScale);
    ctx.lineCap = 'round';
    ctx.lineJoin = 'round';
    ctx.lineWidth = (selected ? 2 : 1.25) / pixelScale;
    ctx.strokeStyle = selected ? '#63B3ED' : stroke;

    definition.geometry.forEach(entity => {
      ctx.save();
      const shouldStroke = entity.strokeAlpha == null || Number(entity.strokeAlpha) > 0;
      ctx.beginPath();
      if (entity.entityType === 'LINE') {
        ctx.moveTo(entity.startPoint.x, entity.startPoint.y);
        ctx.lineTo(entity.endPoint.x, entity.endPoint.y);
      } else if (entity.entityType === 'POLYLINE') {
        this.traceSymbolPolyline(ctx, entity.vertices, Boolean(entity.closed));
        if (entity.closed) {
          ctx.closePath();
          ctx.fillStyle = Number(entity.fillColor) === 0 ? (selected ? '#63B3ED' : stroke) : background;
          ctx.fill();
        }
      } else if (entity.entityType === 'ELLIPSE' || entity.entityType === 'ARC') {
        const center = entity.center;
        const axis = entity.axisPoint;
        const dx = axis.x - center.x;
        const dy = axis.y - center.y;
        const radiusX = Math.hypot(dx, dy);
        const radiusY = radiusX * Math.abs(Number(entity.ratio) || 1);
        const rotation = Math.atan2(dy, dx);
        const start = entity.entityType === 'ARC' ? Number(entity.startAngle || 0) * Math.PI / 180 : 0;
        const end = entity.entityType === 'ARC' ? Number(entity.endAngle || 0) * Math.PI / 180 : Math.PI * 2;
        ctx.ellipse(center.x, center.y, radiusX, radiusY, rotation, start, end, false);
        if (entity.entityType === 'ELLIPSE' && entity.fillColor != null) {
          ctx.fillStyle = Number(entity.fillColor) === 0 ? (selected ? '#63B3ED' : stroke) : background;
          ctx.fill();
        }
      } else if (entity.entityType === 'TEXT') {
        const point = entity.insertionPoint || entity.insertPoint || { x: 0, y: 0 };
        const alignment = String(entity.alignment || '').toUpperCase();
        ctx.translate(point.x, point.y);
        ctx.rotate((Number(entity.rotation) || 0) * Math.PI / 180);
        ctx.scale(1, -1);
        ctx.font = `${Math.max(1, Number(entity.fontSize || entity.FontSize) || 10)}px Arial, sans-serif`;
        ctx.textAlign = alignment.includes('RIGHT') ? 'right' : alignment.includes('CENTER') ? 'center' : 'left';
        ctx.textBaseline = alignment.includes('TOP') ? 'top' : alignment.includes('BOTTOM') ? 'bottom' : 'middle';
        ctx.fillStyle = selected ? '#63B3ED' : stroke;
        ctx.fillText(String(entity.textString || ''), 0, 0);
        ctx.restore();
        return;
      }
      if (shouldStroke) ctx.stroke();
      ctx.restore();
    });
    ctx.restore();
  }

  renderSymbolPreview(canvas, type) {
    if (!canvas) return;
    const definition = this.getSymbolDefinition(type);
    if (!definition) return;
    const ctx = canvas.getContext('2d');
    const bounds = definition.bounds || { minX: -6, minY: -6, maxX: 6, maxY: 6 };
    const width = Math.max(1, bounds.maxX - bounds.minX);
    const height = Math.max(1, bounds.maxY - bounds.minY);
    const scale = Math.min((canvas.width - 14) / width, (canvas.height - 14) / height);
    const centerX = (bounds.minX + bounds.maxX) / 2;
    const centerY = (bounds.minY + bounds.maxY) / 2;
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.save();
    ctx.translate(canvas.width / 2 - centerX * scale, canvas.height / 2 + centerY * scale);
    this.renderSymbolGeometry(ctx, definition, scale, {
      stroke: this.app.state.canvasTheme === 'light' ? '#334155' : '#E2E8F0',
      background: this.colors.bg
    });
    ctx.restore();
  }

  drawSymbols(layerId = null) {
    const s = this.app.state;
    const ctx = this.ctx;

    s.symbols.forEach(sym => {
      if (layerId != null && this.app.getEntityLayerId(sym) !== layerId) return;
      if (!this.app.isEntityVisible(sym)) return;
      const screen = window.Geometry.toScreen({ x: sym.x, y: sym.y }, s.pan, s.zoom);
      const isSelected = s.selectedSymbolIds.has(sym.id);
      const definition = this.getSymbolDefinition(sym.symbolType);
      const def = CanvasEngine.SYMBOL_DEFS[sym.symbolType] || window.SKETCH_SYMBOL_DEFS?.[sym.symbolType];
      const label = def ? def.label : sym.symbolType;
      const size = Math.max(12, 24 * s.zoom * (sym.scale || 1));
      const halfSize = size / 2;

      ctx.save();
      ctx.translate(screen.x, screen.y);
      if (sym.rotation) ctx.rotate(sym.rotation * Math.PI / 180);
      if (sym.flipH) ctx.scale(-1, 1);

      if (definition) {
        const pixelScale = window.Geometry.PIXELS_PER_FOOT * s.zoom * (sym.scale || 1) / 12;
        this.renderSymbolGeometry(ctx, definition, pixelScale, { selected: isSelected });
        ctx.restore();
        return;
      }

      // Draw a symbol box
      ctx.fillStyle = isSelected ? 'rgba(99, 179, 237, 0.3)' : 'rgba(40, 40, 55, 0.85)';
      ctx.strokeStyle = isSelected ? '#63B3ED' : '#718096';
      ctx.lineWidth = isSelected ? 2 : 1;
      ctx.beginPath();
      ctx.roundRect(-halfSize, -halfSize, size, size, 3);
      ctx.fill();
      ctx.stroke();

      // Draw label below
      const fontSize = Math.max(7, 9 * s.zoom);
      ctx.font = `${fontSize}px Arial, sans-serif`;
      ctx.textAlign = 'center';
      ctx.textBaseline = 'top';
      ctx.fillStyle = isSelected ? '#63B3ED' : '#A0AEC0';
      ctx.fillText(label, 0, halfSize + 2);

      // Draw a simple icon inside the box
      ctx.font = `${Math.max(10, 16 * s.zoom)}px "Phosphor"`;
      ctx.textBaseline = 'middle';
      ctx.fillStyle = isSelected ? '#63B3ED' : '#E2E8F0';
      // Use a simple text character as fallback
      const charMap = {
        'door': '⬚', 'window': '▬', 'stairs': '≡',
        'toilet': '◯', 'sink': '◉', 'tub': '▭', 'shower': '◈',
        'fireplace': '△', 'closet': '▣', 'counter': '▬', 'island': '□'
      };
      const parts = sym.symbolType.split('-');
      const shortType = parts[parts.length - 1];
      const family = parts[0];
      ctx.fillText(charMap[shortType] || charMap[family] || '⬜', 0, 0);

      ctx.restore();
    });
  }

  drawRotationAngle(center, reference, current, degrees) {
    const ctx = this.ctx;
    const radius = Math.max(30, Math.min(72, Math.hypot(reference.x - center.x, reference.y - center.y)));
    const start = Math.atan2(reference.y - center.y, reference.x - center.x);
    const end = Math.atan2(current.y - center.y, current.x - center.x);
    ctx.save();
    ctx.fillStyle = 'rgba(99, 102, 241, 0.16)';
    ctx.strokeStyle = 'rgba(99, 102, 241, 0.62)';
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(center.x, center.y);
    ctx.arc(center.x, center.y, radius, start, end, degrees < 0);
    ctx.closePath();
    ctx.fill();
    ctx.stroke();
    ctx.fillStyle = '#6366F1';
    ctx.font = 'bold 16px Inter, sans-serif';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillText(`${Math.abs(Math.round(degrees))}°`, center.x, center.y);
    ctx.restore();
  }

  drawTransformHandles() {
    if (this.app.state.mode !== 'select') return;
    const ctx = this.ctx;
    const drag = this.app.state.dragState;
    const green = '#22C55E';
    const greenLight = '#DCFCE7';

    const textTransforms = [
      ...this.app.state.polygons
        .filter(poly => this.app.state.selectedAreaLabelIds.has(poly.id))
        .map(poly => this.getAreaLabelTransform(poly)),
      ...this.app.state.polygons
        .filter(poly => this.app.state.selectedAreaSizeIds?.has(poly.id))
        .map(poly => this.getAreaSizeTransform(poly)),
      ...this.app.state.labels
        .filter(label => this.app.state.selectedLabelIds.has(label.id))
        .map(label => this.getLabelTransform(label))
    ].filter(Boolean);
    textTransforms.forEach(transform => {
      ctx.save();
      ctx.strokeStyle = green;
      ctx.fillStyle = green;
      ctx.lineWidth = 1.1;
      ctx.setLineDash([4, 3]);
      ctx.beginPath();
      ctx.moveTo(transform.corners[0].x, transform.corners[0].y);
      transform.corners.slice(1).forEach(point => ctx.lineTo(point.x, point.y));
      ctx.closePath();
      ctx.stroke();
      ctx.beginPath();
      ctx.moveTo(transform.topAnchor.x, transform.topAnchor.y);
      ctx.lineTo(transform.handleScreen.x, transform.handleScreen.y);
      ctx.stroke();
      ctx.setLineDash([]);
      transform.corners.forEach(point => {
        ctx.fillRect(point.x - 4, point.y - 4, 8, 8);
        ctx.strokeStyle = greenLight;
        ctx.strokeRect(point.x - 4, point.y - 4, 8, 8);
        ctx.strokeStyle = green;
      });
      ctx.beginPath();
      ctx.arc(transform.handleScreen.x, transform.handleScreen.y, 8, 0, Math.PI * 2);
      ctx.fill();
      ctx.strokeStyle = greenLight;
      ctx.stroke();
      if (drag?.type === 'rotate-text' && drag.entityType === transform.type && drag.entityId === transform.entity.id && drag.moved) {
        const delta = drag.previewDegrees || 0;
        const deltaRadians = -delta * Math.PI / 180;
        const currentVector = {
          x: transform.handleScreen.x - transform.centerScreen.x,
          y: transform.handleScreen.y - transform.centerScreen.y
        };
        const reference = {
          x: transform.centerScreen.x + currentVector.x * Math.cos(deltaRadians) - currentVector.y * Math.sin(deltaRadians),
          y: transform.centerScreen.y + currentVector.x * Math.sin(deltaRadians) + currentVector.y * Math.cos(deltaRadians)
        };
        this.drawRotationAngle(transform.centerScreen, reference, transform.handleScreen, delta);
      }
      ctx.restore();
    });

    const area = this.getSelectedAreaTransform();
    if (area) {
      ctx.save();
      ctx.strokeStyle = green;
      ctx.fillStyle = green;
      ctx.lineWidth = 1.25;
      ctx.setLineDash([4, 4]);

      if (drag?.type === 'rotate-area') {
        const radians = drag.previewDegrees * Math.PI / 180;
        const rotateWorld = point => {
          const dx = point.x - area.center.x;
          const dy = point.y - area.center.y;
          return {
            x: area.center.x + dx * Math.cos(radians) - dy * Math.sin(radians),
            y: area.center.y + dx * Math.sin(radians) + dy * Math.cos(radians)
          };
        };
        this.app.state.polygons
          .filter(poly => this.app.state.selectedPolyIds.has(poly.id))
          .forEach(poly => {
            const points = poly.points.map(point => window.Geometry.toScreen(rotateWorld(point), this.app.state.pan, this.app.state.zoom));
            if (!points.length) return;
            ctx.beginPath();
            ctx.moveTo(points[0].x, points[0].y);
            points.slice(1).forEach(point => ctx.lineTo(point.x, point.y));
            ctx.closePath();
            ctx.stroke();
          });
        const previewTop = window.Geometry.toScreen(rotateWorld({ x: area.center.x, y: area.bounds.maxY }), this.app.state.pan, this.app.state.zoom);
        const previewHandle = window.Geometry.toScreen(rotateWorld(area.handleWorld), this.app.state.pan, this.app.state.zoom);
        ctx.beginPath();
        ctx.moveTo(previewTop.x, previewTop.y);
        ctx.lineTo(previewHandle.x, previewHandle.y);
        ctx.stroke();
        ctx.setLineDash([]);
        ctx.fillStyle = green;
        ctx.beginPath();
        ctx.arc(previewHandle.x, previewHandle.y, 8, 0, Math.PI * 2);
        ctx.fill();
        ctx.strokeStyle = greenLight;
        ctx.stroke();
        this.drawRotationAngle(area.centerScreen, area.handleScreen, previewHandle, drag.previewDegrees);
      } else {
        ctx.beginPath();
        ctx.moveTo(area.topScreen.x, area.topScreen.y);
        ctx.lineTo(area.handleScreen.x, area.handleScreen.y);
        ctx.stroke();
        ctx.setLineDash([]);
        ctx.beginPath();
        ctx.arc(area.handleScreen.x, area.handleScreen.y, 8, 0, Math.PI * 2);
        ctx.fill();
        ctx.strokeStyle = greenLight;
        ctx.stroke();
      }
      ctx.restore();
    }

    this.app.state.symbols
      .filter(symbol => this.app.state.selectedSymbolIds.has(symbol.id))
      .forEach(symbol => {
        const transform = this.getSymbolTransform(symbol);
        if (!transform) return;
        ctx.save();
        ctx.strokeStyle = green;
        ctx.fillStyle = green;
        ctx.lineWidth = 1.1;
        ctx.setLineDash([4, 3]);
        ctx.beginPath();
        ctx.moveTo(transform.corners[0].x, transform.corners[0].y);
        transform.corners.slice(1).forEach(point => ctx.lineTo(point.x, point.y));
        ctx.closePath();
        ctx.stroke();
        ctx.beginPath();
        ctx.moveTo(transform.topAnchor.x, transform.topAnchor.y);
        ctx.lineTo(transform.handleScreen.x, transform.handleScreen.y);
        ctx.stroke();
        ctx.setLineDash([]);
        transform.corners.forEach(point => {
          ctx.fillRect(point.x - 4, point.y - 4, 8, 8);
          ctx.strokeStyle = greenLight;
          ctx.strokeRect(point.x - 4, point.y - 4, 8, 8);
          ctx.strokeStyle = green;
        });
        ctx.beginPath();
        ctx.arc(transform.handleScreen.x, transform.handleScreen.y, 8, 0, Math.PI * 2);
        ctx.fill();
        ctx.strokeStyle = greenLight;
        ctx.stroke();

        if (drag?.type === 'resize-symbol' && drag.symbolId === symbol.id && drag.moved) {
          const corner = transform.corners[drag.corner] || transform.corners[2];
          ctx.font = '600 12px sans-serif';
          ctx.textAlign = 'left';
          ctx.textBaseline = 'bottom';
          ctx.fillStyle = green;
          ctx.fillText(`${Math.round((Number(symbol.scale) || 1) * 100)}%`, corner.x + 8, corner.y - 6);
        }

        if (drag?.type === 'rotate-symbol' && drag.symbolId === symbol.id && drag.moved) {
          const delta = drag.previewDegrees || 0;
          const deltaRadians = -delta * Math.PI / 180;
          const currentVector = {
            x: transform.handleScreen.x - transform.centerScreen.x,
            y: transform.handleScreen.y - transform.centerScreen.y
          };
          const reference = {
            x: transform.centerScreen.x + currentVector.x * Math.cos(deltaRadians) - currentVector.y * Math.sin(deltaRadians),
            y: transform.centerScreen.y + currentVector.x * Math.sin(deltaRadians) + currentVector.y * Math.cos(deltaRadians)
          };
          this.drawRotationAngle(transform.centerScreen, reference, transform.handleScreen, delta);
        }
        ctx.restore();
      });
  }

  drawPlacementPreview() {
    const s = this.app.state;
    const draggingSymbol = Boolean(s.dragSymbol);
    if (!draggingSymbol && s.mode !== 'place-text' && s.mode !== 'place-symbol') return;
    if (!this.mouseScreen) return;

    const ctx = this.ctx;
    const mx = this.mouseScreen.x;
    const my = this.mouseScreen.y;

    // The preview is drawn fully opaque (no globalAlpha ghosting) so the
    // symbol/text follows the cursor as a clean preview instead of leaving
    // a translucent gradient trail during fast mouse movement.
    ctx.save();

    if (s.mode === 'place-text' && s.placementText) {
      const fontSize = Math.max(9, 12 * s.zoom);
      ctx.font = `${fontSize}px Arial, sans-serif`;
      ctx.textAlign = 'center';
      ctx.textBaseline = 'middle';
      ctx.fillStyle = '#63B3ED';
      ctx.fillText(s.placementText, mx, my);
    } else if (draggingSymbol || (s.mode === 'place-symbol' && s.placementSymbol)) {
      const previewType = draggingSymbol ? s.dragSymbol : s.placementSymbol;
      const definition = this.getSymbolDefinition(previewType);
      if (definition) {
        // While dragging the pointer world position is not refreshed, so
        // derive it fresh from the dragover-fed mouse coordinates.
        const worldPt = draggingSymbol
          ? window.Geometry.toWorld(this.mouseScreen, s.pan, s.zoom)
          : (s.pointerWorld || window.Geometry.toWorld(this.mouseScreen, s.pan, s.zoom));
        const placement = this.app.resolveSymbolPlacement(worldPt, previewType);
        const preview = window.Geometry.toScreen(placement, s.pan, s.zoom);
        ctx.translate(preview.x, preview.y);
        if (placement.rotation) ctx.rotate(placement.rotation * Math.PI / 180);
        this.renderSymbolGeometry(ctx, definition, window.Geometry.PIXELS_PER_FOOT * s.zoom / 12, {
          selected: true,
          stroke: '#63B3ED'
        });
        ctx.restore();
        return;
      }
      const def = CanvasEngine.SYMBOL_DEFS[previewType] || window.SKETCH_SYMBOL_DEFS?.[previewType];
      const label = def ? def.label : previewType;
      const size = Math.max(16, 24 * s.zoom);
      ctx.fillStyle = 'rgba(99, 179, 237, 0.3)';
      ctx.strokeStyle = '#63B3ED';
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.roundRect(mx - size / 2, my - size / 2, size, size, 3);
      ctx.fill();
      ctx.stroke();

      const fontSize = Math.max(7, 9 * s.zoom);
      ctx.font = `${fontSize}px Arial, sans-serif`;
      ctx.textAlign = 'center';
      ctx.fillStyle = '#63B3ED';
      ctx.fillText(label, mx, my + size / 2 + 10);
    } else if (s.mode === 'place-shape' && s.pendingShape) {
      const pts = this.app.getShapePoints(s.pendingShape);
      const theta = (Number(s.pendingShape.rotation) || 0) * Math.PI / 180;
      const cos = Math.cos(theta), sin = Math.sin(theta);
      const rotated = pts.map(p => ({ x: p.x * cos - p.y * sin, y: p.x * sin + p.y * cos }));
      const minX = Math.min(...rotated.map(p => p.x));
      const minY = Math.min(...rotated.map(p => p.y));
      const anchor = s.pointerWorld || window.Geometry.toWorld(this.mouseScreen, s.pan, s.zoom);
      const screen = rotated.map(p => window.Geometry.toScreen(
        { x: anchor.x + (p.x - minX), y: anchor.y + (p.y - minY) }, s.pan, s.zoom));
      ctx.fillStyle = 'rgba(99, 179, 237, 0.15)';
      ctx.strokeStyle = '#63B3ED';
      ctx.lineWidth = 1;
      ctx.beginPath();
      screen.forEach((p, i) => { if (i === 0) ctx.moveTo(p.x, p.y); else ctx.lineTo(p.x, p.y); });
      ctx.closePath();
      ctx.fill();
      ctx.stroke();
    }

    ctx.restore();
  }

  drawBgImage() {
    const bg = this.app.bgImage;
    if (!bg || !bg.img || !bg.visible) return;

    const ctx = this.ctx;
    const s = this.app.state;
    const PPF = window.Geometry.PIXELS_PER_FOOT;

    // Convert image center (world coords) to screen coords
    const screen = window.Geometry.toScreen({ x: bg.x, y: bg.y }, s.pan, s.zoom);
    const imgW = bg.img.width * bg.scale * s.zoom;
    const imgH = bg.img.height * bg.scale * s.zoom;

    ctx.save();
    ctx.globalAlpha = bg.opacity;
    ctx.translate(screen.x, screen.y);

    // Rotation
    if (bg.rotation) {
      ctx.rotate(bg.rotation * Math.PI / 180);
    }

    // Flip
    const sx = bg.flipH ? -1 : 1;
    const sy = bg.flipV ? -1 : 1;
    ctx.scale(sx, sy);

    ctx.drawImage(bg.img, -imgW / 2, -imgH / 2, imgW, imgH);
    ctx.restore();
  }

  drawScalePoints() {
    const bg = this.app.bgImage;
    if (!bg || (!bg.scaleMode && !bg.orientMode)) return;

    const ctx = this.ctx;
    const s = this.app.state;

    const drawMarker = (worldPt, label) => {
      const screen = window.Geometry.toScreen(worldPt, s.pan, s.zoom);
      ctx.save();
      ctx.fillStyle = '#F6AD55';
      ctx.strokeStyle = '#F6AD55';
      ctx.lineWidth = 2;

      // Crosshair
      ctx.beginPath();
      ctx.moveTo(screen.x - 10, screen.y);
      ctx.lineTo(screen.x + 10, screen.y);
      ctx.moveTo(screen.x, screen.y - 10);
      ctx.lineTo(screen.x, screen.y + 10);
      ctx.stroke();

      // Circle
      ctx.beginPath();
      ctx.arc(screen.x, screen.y, 6, 0, Math.PI * 2);
      ctx.stroke();

      // Label
      ctx.font = '11px Arial, sans-serif';
      ctx.textAlign = 'left';
      ctx.fillText(label, screen.x + 12, screen.y - 4);
      ctx.restore();
    };

    const p1 = bg.scaleMode ? bg.scaleP1 : bg.orientP1;
    const p2 = bg.scaleMode ? bg.scaleP2 : bg.orientP2;
    if (p1) drawMarker(p1, 'Point 1');
    if (p2) drawMarker(p2, 'Point 2');

    // Draw line between points
    if (p1 && p2) {
      const s1 = window.Geometry.toScreen(p1, s.pan, s.zoom);
      const s2 = window.Geometry.toScreen(p2, s.pan, s.zoom);
      ctx.save();
      ctx.setLineDash([4, 4]);
      ctx.strokeStyle = '#F6AD55';
      ctx.lineWidth = 1.5;
      ctx.beginPath();
      ctx.moveTo(s1.x, s1.y);
      ctx.lineTo(s2.x, s2.y);
      ctx.stroke();
      ctx.setLineDash([]);
      ctx.restore();
    }
  }

  // Transient measure-tool annotations. View-only; never persisted or exported.
  drawMeasurements() {
    const s = this.app.state;
    if (s.mode !== 'measure') return;
    const ctx = this.ctx;
    const options = s.options || {};
    const fontSize = Math.max(10, (Number(options.dimensionFontSize) || 11) * this.getTextZoom());
    const font = `bold ${fontSize}px "${options.dimensionFont || 'Arial'}", monospace`;
    const toScreen = pt => window.Geometry.toScreen(pt, s.pan, s.zoom);

    const chip = (text, x, y, accent = false) => {
      ctx.save();
      ctx.font = font;
      const pad = 4;
      const width = ctx.measureText(text).width + pad * 2;
      const height = fontSize + pad * 2;
      ctx.fillStyle = accent ? this.measureColors.accentFill : this.measureColors.chipFill;
      ctx.fillRect(x - width / 2, y - height / 2, width, height);
      ctx.strokeStyle = this.measureColors.line;
      ctx.lineWidth = 1;
      ctx.strokeRect(x - width / 2, y - height / 2, width, height);
      ctx.fillStyle = accent ? this.measureColors.accentText : this.measureColors.line;
      ctx.textAlign = 'center';
      ctx.textBaseline = 'middle';
      ctx.fillText(text, x, y);
      ctx.restore();
    };

    const drawRun = (points, closed, live) => {
      if (!points.length) return;
      const screen = points.map(toScreen);
      const ring = closed && points.length >= 3;
      ctx.save();
      const trace = () => {
        ctx.beginPath();
        screen.forEach((pt, index) => (index ? ctx.lineTo(pt.x, pt.y) : ctx.moveTo(pt.x, pt.y)));
        if (ring) ctx.closePath();
      };
      if (ring) {
        trace();
        ctx.fillStyle = this.measureColors.areaFill;
        ctx.fill();
      }
      ctx.strokeStyle = this.measureColors.line;
      ctx.lineWidth = live ? 1.25 : 1.75;
      ctx.setLineDash(live ? [5, 4] : [7, 4]);
      trace();
      ctx.stroke();
      ctx.setLineDash([]);
      screen.forEach((pt, index) => {
        ctx.beginPath();
        ctx.arc(pt.x, pt.y, index === 0 ? 4 : 3, 0, Math.PI * 2);
        ctx.fillStyle = index === 0 ? '#FFFFFF' : this.measureColors.line;
        ctx.fill();
        ctx.strokeStyle = this.measureColors.line;
        ctx.lineWidth = 1.5;
        ctx.stroke();
      });
      ctx.restore();

      const metrics = window.Geometry.polylineMetrics(points, ring);
      metrics.segments.forEach(segment => {
        const a = toScreen(segment.start);
        const b = toScreen(segment.end);
        const run = Math.hypot(b.x - a.x, b.y - a.y);
        if (run < 24) return; // no room for a legible label
        // Ride the label off to one side so it never sits on the line itself.
        const offset = 12;
        chip(this.app.formatLength(segment.length),
          (a.x + b.x) / 2 - (b.y - a.y) / run * offset,
          (a.y + b.y) / 2 + (b.x - a.x) / run * offset);
      });

      if (ring) {
        const centroid = toScreen(window.Geometry.polygonCentroid(points));
        chip(this.app.formatArea(metrics.area), centroid.x, centroid.y - fontSize * 0.85, true);
        chip(`Perimeter ${this.app.formatLength(metrics.total)}`, centroid.x, centroid.y + fontSize * 0.85);
      } else if (!live && metrics.segments.length > 1) {
        // A run in progress reports its total in the cursor readout instead.
        const end = screen[screen.length - 1];
        chip(`Total ${this.app.formatLength(metrics.total)}`, end.x, end.y - fontSize - 14, true);
      }
    };

    s.measurements.forEach(run => drawRun(run.points, run.closed, false));

    const draft = s.measureDraft;
    if (draft) {
      const cursor = s.measureCursor?.point || s.pointerWorld;
      drawRun([...draft.points, cursor], false, true);
    }
    this.drawMeasureCursor(fontSize, font);
  }

  // What the measure cursor has caught, and what the next click would add:
  // leg length, its bearing, the corner it turns, the running total, and the
  // area the run would enclose if it were closed here.
  getMeasureReadout() {
    const s = this.app.state;
    const cursor = s.measureCursor;
    if (!cursor) return [];
    const snapNames = {
      corner: 'Corner snap', midpoint: 'Midpoint snap', edge: 'On wall',
      grid: 'Grid', ortho: '45\u00b0 lock', free: 'Free'
    };
    const draft = s.measureDraft;
    if (!draft) return [{ text: snapNames[cursor.type] || 'Free', muted: true }, { text: 'Click to start a run', muted: true }];

    const points = [...draft.points, cursor.point];
    const metrics = window.Geometry.polylineMetrics(points, false);
    const leg = metrics.segments[metrics.segments.length - 1];
    const lines = [];
    if (leg) {
      lines.push({ text: this.app.formatLengthDual(leg.length), accent: true });
      lines.push({ text: `Bearing ${leg.bearing.toFixed(1)}\u00b0   \u0394 ${this.app.formatLength(Math.abs(leg.dx))} / ${this.app.formatLength(Math.abs(leg.dy))}` });
    }
    if (points.length >= 3) {
      const corner = window.Geometry.cornerAngle(points[points.length - 3], points[points.length - 2], points[points.length - 1]);
      if (corner != null) lines.push({ text: `Corner ${corner.toFixed(1)}\u00b0` });
    }
    if (metrics.segments.length > 1) lines.push({ text: `Total ${this.app.formatLengthDual(metrics.total)}` });
    if (points.length >= 3) {
      const closedMetrics = window.Geometry.polylineMetrics(points, true);
      lines.push({ text: `Closes at ${this.app.formatArea(closedMetrics.area)}`, muted: true });
    }
    lines.push({ text: snapNames[cursor.type] || 'Free', muted: true });
    lines.push({ text: 'Enter finish \u00b7 Shift+Enter close \u00b7 Backspace undo', muted: true });
    return lines;
  }

  drawMeasureCursor(fontSize, font) {
    const s = this.app.state;
    const cursor = s.measureCursor;
    if (!cursor) return;
    const ctx = this.ctx;
    const colors = this.measureColors;
    const pt = window.Geometry.toScreen(cursor.point, s.pan, s.zoom);
    const radius = 6;

    ctx.save();
    ctx.strokeStyle = cursor.type === 'free' || cursor.type === 'grid' ? colors.line : colors.snap;
    ctx.lineWidth = 1.75;
    ctx.beginPath();
    if (cursor.type === 'corner') {
      ctx.rect(pt.x - radius, pt.y - radius, radius * 2, radius * 2);
    } else if (cursor.type === 'midpoint') {
      ctx.moveTo(pt.x, pt.y - radius);
      ctx.lineTo(pt.x + radius, pt.y + radius);
      ctx.lineTo(pt.x - radius, pt.y + radius);
      ctx.closePath();
    } else if (cursor.type === 'edge') {
      ctx.moveTo(pt.x - radius, pt.y - radius); ctx.lineTo(pt.x + radius, pt.y + radius);
      ctx.moveTo(pt.x + radius, pt.y - radius); ctx.lineTo(pt.x - radius, pt.y + radius);
    } else if (cursor.type === 'ortho') {
      ctx.moveTo(pt.x - radius, pt.y); ctx.lineTo(pt.x + radius, pt.y);
      ctx.moveTo(pt.x, pt.y - radius); ctx.lineTo(pt.x, pt.y + radius);
    } else {
      ctx.arc(pt.x, pt.y, 3.5, 0, Math.PI * 2);
    }
    ctx.stroke();
    ctx.restore();

    const lines = this.getMeasureReadout();
    if (!lines.length) return;
    ctx.save();
    ctx.font = font;
    const pad = 8;
    const lineHeight = fontSize + 5;
    const width = Math.max(...lines.map(line => ctx.measureText(line.text).width)) + pad * 2;
    const height = lines.length * lineHeight + pad * 2 - 5;
    let x = pt.x + 18;
    let y = pt.y + 18;
    if (x + width > this.viewWidth - 8) x = pt.x - 18 - width;
    if (y + height > this.viewHeight - 8) y = pt.y - 18 - height;
    ctx.fillStyle = colors.panelFill;
    ctx.fillRect(x, y, width, height);
    ctx.strokeStyle = colors.line;
    ctx.lineWidth = 1;
    ctx.strokeRect(x, y, width, height);
    ctx.textAlign = 'left';
    ctx.textBaseline = 'top';
    lines.forEach((line, index) => {
      ctx.fillStyle = line.accent ? colors.line : line.muted ? colors.muted : colors.text;
      ctx.fillText(line.text, x + pad, y + pad + index * lineHeight);
    });
    ctx.restore();
  }
}

window.CanvasEngine = CanvasEngine;

;
/* Fixed appraisal-report renderer. All print-facing output is generated here. */
(function (root) {
  'use strict';

  const PAGE_WIDTH = 612;
  const PAGE_HEIGHT = 1008;
  const SKETCH_WIDTH = 540;
  const SKETCH_HEIGHT = 392;
  const SKETCH_PAD = 20;
  const SKETCH_HEIGHTS = { letter: 392, legal: 608, a4: 442 };
  const LOWER_MARGIN = 37;
  const PAGE_SIZES = Object.freeze({
    legal: { width: 612, height: 1008, css: 'legal portrait', label: 'Legal' },
    letter: { width: 612, height: 792, css: 'letter portrait', label: 'Letter' },
    a4: { width: 595.28, height: 841.89, css: 'A4 portrait', label: 'A4' }
  });
  const escapeHTML = value => String(value == null ? '' : value)
    .replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;').replace(/'/g, '&#39;');
  const number = value => Number.isFinite(Number(value)) ? Number(value) : 0;
  const decimal = value => number(value).toFixed(1);
  const pointsEqual = (a, b, tolerance = 1e-4) => a && b &&
    Math.abs(number(a.x) - number(b.x)) <= tolerance && Math.abs(number(a.y) - number(b.y)) <= tolerance;

  // Same default anchor the canvas uses, so an unmoved label prints where it
  // was drawn. The bounding-box centre remains only for isolated contexts
  // that load this renderer without the geometry engine.
  function defaultLabelPosition(poly) {
    const geometry = root.Geometry;
    if (geometry && typeof geometry.areaLabelAnchor === 'function') {
      return geometry.areaLabelAnchor(geometry.flattenRing ? geometry.flattenRing(poly) : poly.points);
    }
    const xs = poly.points.map(point => number(point.x));
    const ys = poly.points.map(point => number(point.y));
    return { x: (Math.min(...xs) + Math.max(...xs)) / 2, y: (Math.min(...ys) + Math.max(...ys)) / 2 };
  }

  function normalizeLayout(layout) {
    const paperSize = PAGE_SIZES[layout?.paperSize] ? layout.paperSize : 'legal';
    return {
      paperSize,
      currentForm: layout?.currentForm || `standard-${paperSize}`,
      showSketchComments: layout?.showSketchComments !== false,
      showCommentTables: layout?.showCommentTables !== false,
      showPageNumbers: layout?.showPageNumbers !== false,
      calculationAddendum: Boolean(layout?.calculationAddendum),
      deedCallsAddendum: Boolean(layout?.deedCallsAddendum),
      captions: {
        sketchPageTitle: layout?.captions?.sketchPageTitle || 'SKETCH/AREA TABLE ADDENDUM',
        subjectLabel: layout?.captions?.subjectLabel || 'SUBJECT INFO',
        sketchLabel: layout?.captions?.sketchLabel || 'SKETCH',
        summaryLabel: layout?.captions?.summaryLabel || 'AREA CALCULATIONS SUMMARY',
        commentTable1Label: layout?.captions?.commentTable1Label || 'COMMENT TABLE 1',
        commentTable2Label: layout?.captions?.commentTable2Label || 'COMMENT TABLE 2',
        commentTable3Label: layout?.captions?.commentTable3Label || 'COMMENT TABLE 3'
      }
    };
  }

  function hexWithOpacity(color, opacity) {
    const value = String(color || '#ffffff').trim();
    if (/^#[0-9a-f]{6}$/i.test(value)) {
      const alpha = Math.round(Math.max(0, Math.min(100, number(opacity))) * 2.55).toString(16).padStart(2, '0');
      return value + alpha;
    }
    return value;
  }

  function flattenPolygon(poly) {
    if (root.Geometry && typeof root.Geometry.flattenRing === 'function') {
      try { return root.Geometry.flattenRing(poly, undefined, 0.02); } catch (_) { /* use vertices */ }
    }
    return (poly.points || []).map(point => ({ ...point }));
  }

  function signedArea(points) {
    return points.reduce((sum, point, index) => {
      const next = points[(index + 1) % points.length];
      return sum + number(point.x) * number(next.y) - number(next.x) * number(point.y);
    }, 0) / 2;
  }

  // Wall thickness of a line in feet, mirroring app.wallThicknessOf but driven
  // by the export model's options (per-line `thickness` override in inches wins).
  function wallThicknessFeet(line, options) {
    if (number(line?.thickness) > 0) return number(line.thickness) / 12;
    const opts = options || {};
    const inches = line?.type === 'interior'
      ? number(opts.interiorThickness)
      : number(opts.exteriorThickness);
    return inches / 12;
  }

  function wallSolidsEnabled(options) {
    const opts = options || {};
    return Boolean(opts.wallSolids) &&
      (number(opts.exteriorThickness) > 0 || number(opts.interiorThickness) > 0) &&
      Boolean(root.WallSolids);
  }

  // Chain dimension-line offsets (feet) beyond the wall run, including the
  // outer overall tier when shown.
  function chainOffsets(chain) {
    const signedOffset = Math.max(0.5, number(chain.offset) || 3) * (chain.side || 1);
    const offsets = [signedOffset];
    if (chain.showOverall) {
      offsets.push(signedOffset + Math.sign(signedOffset || 1) * Math.max(1.5, Math.abs(signedOffset) * 0.5));
    }
    return offsets;
  }

  function getBounds(page, options) {
    const points = [];
    (page.lines || []).forEach(line => points.push(line.start, line.end));
    (page.polygons || []).forEach(poly => points.push(...flattenPolygon(poly)));
    // Wall solids and dimension chains stretch past the wall centerlines; the
    // fixed sketch box would otherwise clip them at the edges.
    if (wallSolidsEnabled(options)) {
      const solids = root.WallSolids.computeWallSolids(page.lines || [], line => wallThicknessFeet(line, options), page.symbols || []);
      solids.solids.forEach(solid => solid.pieces.forEach(piece => piece.forEach(point => points.push(point))));
    }
    if (root.Geometry && typeof root.Geometry.computeDimensionChain === 'function') {
      (page.dimensionChains || []).forEach(chain => {
        const computed = root.Geometry.computeDimensionChain(page.lines || [], page.symbols || [], chain.seedLineId, 0.25, options);
        if (!computed || computed.refs.length < 2) return;
        chainOffsets(chain).forEach(offset => [0, computed.length].forEach(t => points.push({
          x: number(computed.start.x) + computed.dir.x * t + computed.normal.x * offset,
          y: number(computed.start.y) + computed.dir.y * t + computed.normal.y * offset
        })));
      });
    }
    // Labels and symbols are intentionally excluded from the fit bounds:
    // a stray text label or north arrow far from the drawing was shrinking
    // the sketch and leaving a large empty margin inside the sketch box.
    const usable = points.filter(point => point && Number.isFinite(Number(point.x)) && Number.isFinite(Number(point.y)));
    if (!usable.length) return { minX: 0, maxX: 40, minY: 0, maxY: 28 };
    let minX = Math.min(...usable.map(point => number(point.x)));
    let maxX = Math.max(...usable.map(point => number(point.x)));
    let minY = Math.min(...usable.map(point => number(point.y)));
    let maxY = Math.max(...usable.map(point => number(point.y)));
    if (maxX - minX < 1) { minX -= .5; maxX += .5; }
    if (maxY - minY < 1) { minY -= .5; maxY += .5; }
    return { minX, maxX, minY, maxY };
  }

  function makeTransform(page, sketchHeight, options) {
    const bounds = getBounds(page, options);
    const padX = SKETCH_PAD;
    const padY = SKETCH_PAD;
    const scale = Math.min((SKETCH_WIDTH - padX * 2) / (bounds.maxX - bounds.minX),
      (sketchHeight - padY * 2) / (bounds.maxY - bounds.minY));
    const drawnWidth = (bounds.maxX - bounds.minX) * scale;
    const drawnHeight = (bounds.maxY - bounds.minY) * scale;
    const offsetX = (SKETCH_WIDTH - drawnWidth) / 2;
    const offsetY = (sketchHeight - drawnHeight) / 2;
    const point = world => ({
      x: offsetX + (number(world.x) - bounds.minX) * scale,
      y: offsetY + (bounds.maxY - number(world.y)) * scale
    });
    return { point, scale, bounds };
  }

  function polygonPath(poly, transform) {
    const points = flattenPolygon(poly).map(transform.point);
    return points.map((point, index) => `${index ? 'L' : 'M'}${point.x.toFixed(2)},${point.y.toFixed(2)}`).join(' ') + ' Z';
  }

  function linePath(line, transform, symbols) {
    if (!line.bowAngle || Math.abs(number(line.bowAngle)) < .5) {
      const runs = root.WallSolids && typeof root.WallSolids.openingRuns === 'function'
        ? root.WallSolids.openingRuns(line, symbols) : [{ start: 0, end: 1 }];
      const at = t => transform.point({
        x: number(line.start.x) + (number(line.end.x) - number(line.start.x)) * t,
        y: number(line.start.y) + (number(line.end.y) - number(line.start.y)) * t
      });
      return runs.map(run => {
        const start = at(run.start);
        const end = at(run.end);
        return `M${start.x.toFixed(2)},${start.y.toFixed(2)} L${end.x.toFixed(2)},${end.y.toFixed(2)}`;
      }).join(' ');
    }
    const faux = { points: [line.start, line.end], edges: [{ bowAngle: line.bowAngle }, { bowAngle: 0 }] };
    const points = flattenPolygon(faux).map(transform.point);
    return points.map((point, index) => `${index ? 'L' : 'M'}${point.x.toFixed(2)},${point.y.toFixed(2)}`).join(' ');
  }

  function polylinePath(vertices, closed) {
    if (!vertices || !vertices.length) return '';
    let path = `M${number(vertices[0].x).toFixed(2)},${(-number(vertices[0].y)).toFixed(2)}`;
    const count = closed ? vertices.length : vertices.length - 1;
    for (let index = 0; index < count; index++) {
      const start = vertices[index];
      const end = vertices[(index + 1) % vertices.length];
      const bulge = number(start.b);
      const dx = number(end.x) - number(start.x);
      const dy = number(end.y) - number(start.y);
      const chord = Math.hypot(dx, dy);
      if (Math.abs(bulge) < 1e-7 || chord < 1e-7) {
        path += ` L${number(end.x).toFixed(2)},${(-number(end.y)).toFixed(2)}`;
      } else {
        const radius = chord * (1 + bulge * bulge) / (4 * Math.abs(bulge));
        const sweep = Math.abs(4 * Math.atan(bulge));
        path += ` A${radius.toFixed(2)},${radius.toFixed(2)} 0 ${sweep > Math.PI ? 1 : 0},${bulge > 0 ? 0 : 1} ${number(end.x).toFixed(2)},${(-number(end.y)).toFixed(2)}`;
      }
    }
    return path + (closed ? ' Z' : '');
  }

  function renderNorthArrow(page) {
    const angle = ((number(page.northAngle) % 360) + 360) % 360;
    return `<g class="report-north-arrow" transform="translate(516 24) rotate(${angle.toFixed(2)})"><path d="M0 -12 L4.2 6 L0 2.4 L-4.2 6 Z" fill="#111111"/><circle r="14" fill="none" stroke="#111111" stroke-width=".6"/></g><text class="report-north-label" x="516" y="46" text-anchor="middle" font-family="Arial,sans-serif" font-size="6.5" fill="#111111">N${angle ? ` (${Math.round(angle)}\u00B0)` : ''}</text>`;
  }

  function renderSymbolGeometry(symbol, transform) {
    const definition = root.SKETCH_SYMBOL_LIBRARY?.symbols?.[symbol.symbolType];
    if (!definition?.geometry?.length) return '';
    const point = transform.point(symbol);
    const scale = transform.scale * Math.max(.001, number(symbol.scale) || 1) / 12;
    const stroke = escapeHTML(symbol.color || '#111111');
    const entities = definition.geometry.map(entity => {
      if (entity.entityType === 'LINE') return `<path d="M${number(entity.startPoint?.x)},${-number(entity.startPoint?.y)} L${number(entity.endPoint?.x)},${-number(entity.endPoint?.y)}"/>`;
      if (entity.entityType === 'POLYLINE') return `<path d="${polylinePath(entity.vertices, Boolean(entity.closed))}" ${entity.closed && Number(entity.fillColor) === 0 ? 'fill="currentColor"' : 'fill="none"'}/>`;
      if (entity.entityType === 'ELLIPSE' || entity.entityType === 'ARC') {
        const center = entity.center || { x: 0, y: 0 };
        const axis = entity.axisPoint || center;
        const rx = Math.hypot(number(axis.x) - number(center.x), number(axis.y) - number(center.y));
        const ry = rx * Math.abs(number(entity.ratio) || 1);
        if (entity.entityType === 'ELLIPSE') return `<ellipse cx="${number(center.x)}" cy="${-number(center.y)}" rx="${rx}" ry="${ry}" fill="none"/>`;
        const startAngle = number(entity.startAngle) * Math.PI / 180;
        const endAngle = number(entity.endAngle) * Math.PI / 180;
        const start = { x: number(center.x) + rx * Math.cos(startAngle), y: -number(center.y) - ry * Math.sin(startAngle) };
        const end = { x: number(center.x) + rx * Math.cos(endAngle), y: -number(center.y) - ry * Math.sin(endAngle) };
        const sweep = ((number(entity.endAngle) - number(entity.startAngle)) % 360 + 360) % 360;
        return `<path d="M${start.x},${start.y} A${rx},${ry} 0 ${sweep > 180 ? 1 : 0},0 ${end.x},${end.y}" fill="none"/>`;
      }
      if (entity.entityType === 'TEXT') {
        const insert = entity.insertionPoint || entity.insertPoint || { x: 0, y: 0 };
        return `<text x="${number(insert.x)}" y="${-number(insert.y)}" font-size="${Math.max(1, number(entity.fontSize || entity.FontSize) || 10)}" stroke="none" fill="currentColor">${escapeHTML(entity.textString || '')}</text>`;
      }
      return '';
    }).join('');
    return `<g transform="translate(${point.x.toFixed(2)} ${point.y.toFixed(2)}) rotate(${-number(symbol.rotation)}) scale(${scale.toFixed(4)})" color="${stroke}" stroke="currentColor" stroke-width="${Math.max(.55 / scale, .15).toFixed(3)}" stroke-linecap="round" stroke-linejoin="round">${entities}</g>`;
  }

  function dimensionSide(page, line) {
    const polygon = (page.polygons || []).find(poly => (poly.points || []).some((point, index) => {
      const next = poly.points[(index + 1) % poly.points.length];
      return (pointsEqual(point, line.start) && pointsEqual(next, line.end)) || (pointsEqual(point, line.end) && pointsEqual(next, line.start));
    }));
    if (!polygon) return 1;
    const forward = polygon.points.some((point, index) => pointsEqual(point, line.start) && pointsEqual(polygon.points[(index + 1) % polygon.points.length], line.end));
    let side = signedArea(polygon.points) >= 0 ? 1 : -1;
    if (!forward) side *= -1;
    return side;
  }

  function renderDimension(page, line, transform, color) {
    if (line.dimensionHidden) return '';
    const distance = Math.hypot(number(line.end?.x) - number(line.start?.x), number(line.end?.y) - number(line.start?.y));
    if (distance < .1) return '';
    const start = transform.point(line.start);
    const end = transform.point(line.end);
    const dx = end.x - start.x;
    const dy = end.y - start.y;
    const length = Math.hypot(dx, dy) || 1;
    const side = dimensionSide(page, line);
    const moved = root.Geometry && typeof root.Geometry.pinnedDimensionPoint === 'function'
      ? root.Geometry.pinnedDimensionPoint(line) : line.dimensionPosition;
    let x, y;
    if (Number.isFinite(moved?.x) && Number.isFinite(moved?.y)) {
      const point = transform.point(moved);
      x = point.x;
      y = point.y;
    } else {
      x = (start.x + end.x) / 2 + (-dy / length) * 10 * side;
      y = (start.y + end.y) / 2 + (dx / length) * 10 * side;
    }
    let angle = Math.atan2(dy, dx) * 180 / Math.PI;
    if (angle > 90) angle -= 180;
    if (angle < -90) angle += 180;
    // Dimension text is drawn in the wall's effective color (area Line color
    // override first), like the canvas output.
    return `<text class="report-dimension" x="${x.toFixed(2)}" y="${y.toFixed(2)}" fill="${escapeHTML(color || '#111111')}" transform="rotate(${angle.toFixed(2)} ${x.toFixed(2)} ${y.toFixed(2)})">${decimal(distance)}'</text>`;
  }

  // Filled wall-solid polygons (geometric thickness). Straight walls with a
  // positive thickness are handled here; everything else keeps its stroke.
  function renderWallSolids(page, transform, areaLineColors, options) {
    const opts = options || {};
    const byId = new Map((page.lines || []).map(line => [line.id, line]));
    const solids = root.WallSolids.computeWallSolids(page.lines || [], line => wallThicknessFeet(line, opts), page.symbols || []);
    return solids.solids.map(solid => {
      const line = byId.get(solid.lineId);
      if (!line) return '';
      const color = escapeHTML(areaLineColors.get(line.id) || line.color ||
        (line.type === 'exterior' ? (opts.exteriorColor || '#2B6CB0') : (opts.interiorColor || '#63B3ED')));
      return solid.pieces.map(piece => {
        const d = piece.map((point, index) => {
          const p = transform.point(point);
          return `${index ? 'L' : 'M'}${p.x.toFixed(2)},${p.y.toFixed(2)}`;
        }).join(' ') + ' Z';
        return `<path d="${d}" fill="${color}"/>`;
      }).join('');
    }).join('');
  }

  // Chained dimension strings, mirroring canvas.drawChains in report space:
  // witness lines, shared dimension line with ticks, per-span labels and an
  // optional overall tier.
  function renderChains(page, transform, options) {
    if (!root.Geometry || typeof root.Geometry.computeDimensionChain !== 'function') return '';
    const opts = options || {};
    const minimum = opts.minimumDimensionEnabled ? number(opts.minimumDimension) : 0;
    const color = '#111111';
    const stroke = `stroke="${color}" stroke-width="0.6" fill="none"`;
    const tick = p => `<path d="M${(p.x - 3).toFixed(2)},${(p.y + 3).toFixed(2)} L${(p.x + 3).toFixed(2)},${(p.y - 3).toFixed(2)}" ${stroke}/>`;
    return (page.dimensionChains || []).map(chain => {
      const computed = root.Geometry.computeDimensionChain(page.lines || [], page.symbols || [], chain.seedLineId, 0.25, opts);
      if (!computed || computed.refs.length < 2) return '';
      const signedOffset = Math.max(0.5, number(chain.offset) || 3) * (chain.side || 1);
      const pointAt = (t, extraOffset = 0) => transform.point({
        x: computed.start.x + computed.dir.x * t + computed.normal.x * (signedOffset + extraOffset),
        y: computed.start.y + computed.dir.y * t + computed.normal.y * (signedOffset + extraOffset)
      });
      const wallAt = t => transform.point({
        x: computed.start.x + computed.dir.x * t,
        y: computed.start.y + computed.dir.y * t
      });
      const refs = computed.refs;
      const length = computed.length;
      const a0 = pointAt(0);
      const aEnd = pointAt(length);
      let angle = Math.atan2(aEnd.y - a0.y, aEnd.x - a0.x) * 180 / Math.PI;
      if (angle > 90) angle -= 180;
      if (angle < -90) angle += 180;
      const axisLen = Math.hypot(aEnd.x - a0.x, aEnd.y - a0.y) || 1;
      const rawNx = -(aEnd.y - a0.y) / axisLen;
      const rawNy = (aEnd.x - a0.x) / axisLen;
      const outward = rawNx * (wallAt(0).x - a0.x) + rawNy * (wallAt(0).y - a0.y) < 0 ? 1 : -1;
      const nx = rawNx * outward;
      const ny = rawNy * outward;
      let svg = '';
      refs.forEach(t => {
        const wall = wallAt(t);
        const dim = pointAt(t);
        svg += `<path d="M${(wall.x + nx * 2).toFixed(2)},${(wall.y + ny * 2).toFixed(2)} L${(dim.x + nx * 3).toFixed(2)},${(dim.y + ny * 3).toFixed(2)}" ${stroke}/>`;
      });
      svg += `<path d="M${a0.x.toFixed(2)},${a0.y.toFixed(2)} L${aEnd.x.toFixed(2)},${aEnd.y.toFixed(2)}" ${stroke}/>`;
      refs.forEach(t => { svg += tick(pointAt(t)); });
      for (let index = 0; index < refs.length - 1; index++) {
        const span = refs[index + 1] - refs[index];
        if ((chain.hiddenSpans || []).includes(index)) continue;
        if (span < minimum || span < 0.05) continue;
        const mid = pointAt((refs[index] + refs[index + 1]) / 2);
        const x = mid.x + nx * 7;
        const y = mid.y + ny * 7;
        svg += `<text class="report-dimension" x="${x.toFixed(2)}" y="${y.toFixed(2)}" fill="${color}" transform="rotate(${angle.toFixed(2)} ${x.toFixed(2)} ${y.toFixed(2)})">${decimal(span)}'</text>`;
      }
      if (chain.showOverall) {
        const extra = Math.sign(signedOffset || 1) * Math.max(1.5, Math.abs(signedOffset) * 0.5);
        const b0 = pointAt(0, extra);
        const bEnd = pointAt(length, extra);
        [0, length].forEach(t => {
          const inner = pointAt(t);
          const outer = pointAt(t, extra);
          svg += `<path d="M${(inner.x + nx * 1.5).toFixed(2)},${(inner.y + ny * 1.5).toFixed(2)} L${(outer.x + nx * 3).toFixed(2)},${(outer.y + ny * 3).toFixed(2)}" ${stroke}/>`;
        });
        svg += `<path d="M${b0.x.toFixed(2)},${b0.y.toFixed(2)} L${bEnd.x.toFixed(2)},${bEnd.y.toFixed(2)}" ${stroke}/>`;
        svg += tick(b0) + tick(bEnd);
        const x = (b0.x + bEnd.x) / 2 + nx * 7;
        const y = (b0.y + bEnd.y) / 2 + ny * 7;
        svg += `<text class="report-dimension" x="${x.toFixed(2)}" y="${y.toFixed(2)}" fill="${color}" font-weight="bold" transform="rotate(${angle.toFixed(2)} ${x.toFixed(2)} ${y.toFixed(2)})">${decimal(length)}'</text>`;
      }
      return svg;
    }).join('');
  }

  function renderSketchSvg(page, areaCodes, sketchHeight, options) {
    const opts = options || {};
    const transform = makeTransform(page, sketchHeight, opts);
    const codeMap = new Map((areaCodes || []).map(code => [code.code, code]));
    // An area's Line color setting tints its boundary walls even when the
    // walls were drawn or loaded before the setting was saved; the canvas
    // honors this via buildAreaLineColors, so the printed output must apply
    // the same override instead of the color stamped on the line at creation.
    const areaLineColors = new Map();
    (page.polygons || []).forEach(poly => {
      if (!poly.type || poly.type === 'UND') return;
      const color = poly.lineColor || codeMap.get(poly.type)?.color;
      if (!color) return;
      (poly.edges || []).forEach(edge => {
        if (edge.lineId != null && !areaLineColors.has(edge.lineId)) areaLineColors.set(edge.lineId, color);
      });
    });
    const patterns = `<defs><pattern id="diag" width="7" height="7" patternUnits="userSpaceOnUse" patternTransform="rotate(45)"><path d="M0 0V7" stroke="currentColor" stroke-width=".45"/></pattern><pattern id="cross" width="7" height="7" patternUnits="userSpaceOnUse"><path d="M0 0L7 7M7 0L0 7" stroke="currentColor" stroke-width=".4"/></pattern><pattern id="dots" width="7" height="7" patternUnits="userSpaceOnUse"><circle cx="2" cy="2" r=".6" fill="currentColor"/></pattern></defs>`;
    const polygons = (page.polygons || []).filter(poly => poly.points?.length >= 3).map(poly => {
      const definition = codeMap.get(poly.type) || {};
      const outline = escapeHTML(poly.lineColor || definition.color || '#111111');
      const fillColor = poly.fillColor || definition.fillColor || definition.color || '#ffffff';
      const opacity = poly.fillOpacity ?? definition.fillOpacity ?? 10;
      const style = poly.fillPattern || definition.fillStyle || 'solid';
      let fill = style === 'none' || poly.type === 'UND' ? 'none' : hexWithOpacity(fillColor, opacity);
      if (style === 'diagonal') fill = 'url(#diag)';
      if (style === 'crosshatch') fill = 'url(#cross)';
      if (style === 'dots') fill = 'url(#dots)';
      return `<path d="${polygonPath(poly, transform)}" fill="${fill}" color="${outline}" stroke="${outline}" stroke-width="${Math.max(.65, number(poly.lineThickness) || .8)}"/>`;
    }).join('');
    const lines = (page.lines || []).map(line => {
      // Straight walls with geometric thickness are drawn as filled solids
      // below; bowed walls keep their stroked arc (v1 approximation).
      const solidHandled = wallSolidsEnabled(opts) &&
        root.WallSolids.isStraight(line) && wallThicknessFeet(line, opts) > 0;
      if (solidHandled) return '';
      const color = escapeHTML(areaLineColors.get(line.id) || line.color || '#111111');
      const width = line.type === 'exterior' ? Math.max(1.3, number(line.lineThickness) || 2) : Math.max(.65, number(line.lineThickness) || 1);
      const dash = line.lineStyle === 'dashed' ? '8 5' : line.lineStyle === 'dotted' ? '1.5 3.5' : '';
      const d = linePath(line, transform, page.symbols);
      if (!d) return '';
      return `<path d="${d}" fill="none" stroke="${color}" stroke-width="${width}" ${dash ? `stroke-dasharray="${dash}"` : ''} stroke-linecap="round" stroke-linejoin="round"/>`;
    }).join('');
    const wallSolids = wallSolidsEnabled(opts) ? renderWallSolids(page, transform, areaLineColors, opts) : '';
    // Per-wall dimensions are render-suppressed for walls covered by a chain,
    // matching the canvas coexistence rule (never mutates dimensionHidden).
    const chainCovered = new Set();
    if (root.Geometry && typeof root.Geometry.computeDimensionChain === 'function') {
      (page.dimensionChains || []).forEach(chain => {
        const computed = root.Geometry.computeDimensionChain(page.lines || [], page.symbols || [], chain.seedLineId, 0.25, opts);
        (computed?.lineIds || []).forEach(id => chainCovered.add(id));
      });
    }
    const dimensions = (page.lines || []).map(line => chainCovered.has(line.id) ? '' : renderDimension(page, line, transform, areaLineColors.get(line.id) || line.color)).join('');
    const chains = renderChains(page, transform, opts);
    const areaLabels = (page.polygons || []).filter(poly => poly.type && poly.type !== 'UND' && poly.points?.length >= 3).map(poly => {
      const labelWorld = poly.labelPosition || defaultLabelPosition(poly);
      const labelCenter = transform.point(labelWorld);
      const name = poly.name || codeMap.get(poly.type)?.name || poly.type;
      const labelScale = Math.max(.5, Math.min(4, number(poly.labelScale) || 1));
      const sizeScale = Math.max(.5, Math.min(4, number(poly.areaSizeScale ?? labelScale) || 1));
      const sizeWorld = poly.areaSizePosition || { x: labelWorld.x, y: labelWorld.y - .75 * sizeScale };
      const sizeCenter = transform.point(sizeWorld);
      const labelRotation = -number(poly.labelRotation);
      const sizeRotation = -number(poly.areaSizeRotation ?? poly.labelRotation);
      const style = `${poly.labelBold === false ? 'font-weight:normal;' : ''}${poly.labelItalic ? 'font-style:italic;' : ''}`;
      const common = `fill="${escapeHTML(poly.labelColor || '#111111')}" font-family="${escapeHTML(poly.labelFontFamily || 'Arial')}" style="${style}"`;
      return `<text class="report-area-label" x="${labelCenter.x.toFixed(2)}" y="${labelCenter.y.toFixed(2)}" transform="rotate(${labelRotation.toFixed(2)} ${labelCenter.x.toFixed(2)} ${labelCenter.y.toFixed(2)})" ${common} font-size="${(9.72 * labelScale).toFixed(2)}">${escapeHTML(name)}</text><text class="report-area-label report-area-size" x="${sizeCenter.x.toFixed(2)}" y="${sizeCenter.y.toFixed(2)}" transform="rotate(${sizeRotation.toFixed(2)} ${sizeCenter.x.toFixed(2)} ${sizeCenter.y.toFixed(2)})" ${common} font-size="${(9.72 * sizeScale).toFixed(2)}">${decimal(poly.net ?? poly.area ?? poly.gross)} sf</text>`;
    }).join('');
    const labels = (page.labels || []).map(label => {
      const point = transform.point(label);
      const size = Math.max(7, Math.min(16, number(label.fontSize) * .78 || 9));
      return `<text x="${point.x.toFixed(2)}" y="${point.y.toFixed(2)}" transform="rotate(${-number(label.rotation)} ${point.x.toFixed(2)} ${point.y.toFixed(2)})" text-anchor="middle" dominant-baseline="middle" fill="${escapeHTML(label.color && String(label.color).toLowerCase() !== '#e2e8f0' ? label.color : '#111111')}" font-family="${escapeHTML(label.fontFamily || 'Arial')}" font-size="${size}" style="${label.bold ? 'font-weight:bold;' : ''}${label.italic ? 'font-style:italic;' : ''}">${escapeHTML(label.text || '')}</text>`;
    }).join('');
    const symbols = (page.symbols || []).map(symbol => renderSymbolGeometry(symbol, transform)).join('');
    return `<svg class="report-sketch-svg" viewBox="0 0 ${SKETCH_WIDTH} ${sketchHeight}" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="Sketch page" style="height:${sketchHeight.toFixed(2)}pt">${patterns}${polygons}${wallSolids}${lines}${dimensions}${chains}${areaLabels}${labels}${symbols}${renderNorthArrow(page)}</svg>`;
  }

  function calculationRows(page, areaCodes) {
    const definitions = new Map((areaCodes || []).map(code => [code.code, code]));
    const groups = new Map();
    (page.polygons || []).filter(poly => poly.type && poly.type !== 'UND' && poly.points?.length >= 3).forEach(poly => {
      const definition = definitions.get(poly.type) || {};
      const key = definition.group || poly.type;
      if (!groups.has(key)) groups.set(key, []);
      groups.get(key).push({ poly, definition });
    });
    const rows = [];
    groups.forEach((items, group) => {
      const total = items.reduce((sum, item) => sum + number(item.poly.net), 0);
      items.forEach((item, index) => rows.push({ code: index === 0 ? group : '', description: item.poly.name || item.definition.name || item.poly.type,
        factor: number(item.poly.effectiveMultiplier ?? item.definition.multiplier ?? 1), gross: number(item.poly.gross ?? item.poly.area), perimeter: number(item.poly.perimeter), total: index === items.length - 1 ? total : null }));
    });
    return rows;
  }

  function renderCalculations(page, areaCodes, layout, lowerHeight) {
    const rows = calculationRows(page, areaCodes);
    const definitions = new Map((areaCodes || []).map(code => [code.code, code]));
    const livable = (page.polygons || []).filter(poly => definitions.get(poly.type)?.includeInGLA);
    const livableTotal = livable.reduce((sum, poly) => sum + number(poly.net), 0);
    const body = rows.map(row => `<tr><td>${escapeHTML(row.code)}</td><td>${escapeHTML(row.description)}</td><td>${row.factor.toFixed(2)}</td><td>${decimal(row.gross)}</td><td>${decimal(row.perimeter)}</td><td>${row.total == null ? '' : decimal(row.total)}</td></tr>`).join('');
    return `<section class="report-calculations"><div class="report-section-title">${escapeHTML(layout.captions.summaryLabel)}</div><table><thead><tr><th>Code</th><th>Description</th><th>Factor</th><th>Net Size</th><th>Perimeter</th><th>Net Totals</th></tr></thead><tbody>${body}</tbody></table>${livable.length ? `<div class="report-net-total" style="top:${Math.max(30, lowerHeight - 50).toFixed(2)}pt"><span>Net LIVABLE</span><span>cnt</span><span>${livable.length}</span><span>(rounded)</span><span>${Math.round(livableTotal).toLocaleString('en-US')}</span></div>` : ''}</section>`;
  }

  function renderComments(commentTables, layout, lowerHeight) {
    const labels = [layout.captions.commentTable1Label, layout.captions.commentTable2Label, layout.captions.commentTable3Label];
    const table = index => `<section class="report-comment report-comment-${index + 1}"><div class="report-section-title">${escapeHTML(labels[index])}</div><div class="report-comment-lines">${Array.from({ length: 10 }, (_, row) => `<div>${escapeHTML(commentTables?.[index]?.[row] || '')}</div>`).join('')}</div></section>`;
    return `<div class="report-comments" style="grid-template-rows:${(lowerHeight / 2).toFixed(2)}pt 1fr">${table(0)}${table(1)}${table(2)}</div>`;
  }

  function renderSubject(info, layout) {
    const field = key => escapeHTML(info?.[key] || (key === 'propertyAddress' ? info?.address : '') || '');
    return `<section class="report-subject"><div class="report-section-title">${escapeHTML(layout.captions.subjectLabel)}</div><div class="report-subject-row two"><span><b>File No.:</b>${field('fileNo')}</span><span><b>Parcel No.:</b>${field('parcelNo')}</span></div><div class="report-subject-row"><span><b>Property Address:</b>${field('propertyAddress')}</span></div><div class="report-subject-row four"><span><b>City:</b>${field('city')}</span><span><b>County:</b>${field('county')}</span><span><b>State:</b>${field('state')}</span><span><b>ZipCode:</b>${field('zipCode')}</span></div><div class="report-subject-row"><span><b>Owner:</b>${field('owner')}</span></div><div class="report-subject-row client"><span><b>Client:</b>${field('client')}</span><span><b>Client Address:</b>${field('clientAddress')}</span></div><div class="report-subject-row appraiser"><span><b>Appraiser Name:</b>${field('appraiserName')}</span><span><b>Inspection Date:</b>${field('inspectionDate')}</span></div></section>`;
  }

  const REPORT_CSS = `*{box-sizing:border-box}html,body{margin:0;padding:0;background:#fff;color:#000}body{font-family:Verdana,Arial,sans-serif}.report-page{position:relative;width:612pt;height:792pt;overflow:hidden;background:#fff;color:#000;page-break-after:always;break-after:page}.report-page:last-child{page-break-after:auto;break-after:auto}.report-title{position:absolute;left:36pt;top:16.4pt;width:540pt;height:17pt;text-align:center;font-size:12pt;font-weight:bold;line-height:17pt}.report-section-title{height:12pt;background:#d3d3d3;border-bottom:.45pt solid #555;text-align:center;font-size:8pt;font-weight:bold;line-height:12pt}.report-subject{position:absolute;left:36pt;top:38.2pt;width:540pt;height:72.2pt;border:.45pt solid #555;font-size:7.02pt;overflow:hidden}.report-subject>.report-section-title{height:12pt}.report-subject-row{height:9.9pt;border-bottom:.35pt solid #777;display:grid;grid-template-columns:1fr;line-height:9.4pt;padding:0 4.6pt}.report-subject-row:last-child{border-bottom:0}.report-subject-row.two{grid-template-columns:105pt 1fr}.report-subject-row.four{grid-template-columns:59pt 103pt 97pt 1fr}.report-subject-row.client{grid-template-columns:98pt 1fr}.report-subject-row.appraiser{grid-template-columns:136pt 1fr}.report-subject-row span{white-space:nowrap;overflow:hidden}.report-subject-row b{margin-right:4pt;font-weight:bold}.report-sketch-heading{position:absolute;left:36pt;top:110.4pt;width:540pt;height:12pt;border-left:.45pt solid #555;border-right:.45pt solid #555}.report-sketch-body{position:absolute;left:36pt;top:122.4pt;width:540pt;height:391.8pt;border:.45pt solid #555;border-top:0;overflow:hidden}.report-sketch-svg{display:block;width:540pt;height:392pt;background:#fff}.report-dimension{font:7.98px Arial,sans-serif;text-anchor:middle;dominant-baseline:middle}.report-area-label{font-family:Arial,sans-serif;text-anchor:middle}.report-lower{position:absolute;left:36pt;top:514.2pt;width:540pt;height:240.7pt;border:.45pt solid #555;display:grid;grid-template-columns:324pt 216pt}.report-calculations{position:relative;border-right:.45pt solid #555;font-size:7.98pt;overflow:hidden}.report-calculations table{width:100%;border-collapse:collapse;table-layout:fixed}.report-calculations th{height:12pt;border-bottom:.35pt solid #777;font-size:7.02pt;font-weight:normal;text-align:left;padding:1pt 2pt}.report-calculations td{height:10.74pt;padding:0 2pt;white-space:nowrap;overflow:hidden}.report-calculations th:nth-child(1),.report-calculations td:nth-child(1){width:30pt}.report-calculations th:nth-child(2),.report-calculations td:nth-child(2){width:86pt}.report-calculations th:nth-child(3),.report-calculations td:nth-child(3){width:38pt;text-align:right}.report-calculations th:nth-child(4),.report-calculations td:nth-child(4){width:49pt;text-align:right}.report-calculations th:nth-child(5),.report-calculations td:nth-child(5){width:52pt;text-align:right}.report-calculations th:nth-child(6),.report-calculations td:nth-child(6){text-align:right}.report-net-total{position:absolute;left:0;right:0;bottom:3pt;text-align:right;padding-right:5pt;font-size:7.98pt;font-weight:normal}.report-comments{position:relative;display:grid;grid-template-columns:108pt 108pt;grid-template-rows:120pt 1fr}.report-comment{overflow:hidden}.report-comment-1{grid-column:1/3;border-bottom:.45pt solid #555}.report-comment-2{border-right:.45pt solid #555}.report-comment-lines>div{height:10.45pt;border-bottom:.25pt solid #aaa;padding:1.2pt 2pt;font-size:6pt;white-space:nowrap;overflow:hidden}.report-comment-lines>div:last-child{border-bottom:0}.report-page-number{position:absolute;right:36pt;top:770.5pt;font-size:6pt}@page{size:letter portrait;margin:0}@media print{.report-page{box-shadow:none;margin:0}.report-preview-only{display:none!important}}`;

  const PRECISE_REPORT_CSS = REPORT_CSS + `.report-subject-row b{font-weight:normal}.report-calculations th{background:#d3d3d3}.report-calculations th:nth-child(1),.report-calculations td:nth-child(1){width:41pt}.report-calculations th:nth-child(2),.report-calculations td:nth-child(2){width:88pt}.report-calculations th:nth-child(3),.report-calculations td:nth-child(3){width:42pt}.report-calculations th:nth-child(4),.report-calculations td:nth-child(4){width:51pt}.report-calculations th:nth-child(5),.report-calculations td:nth-child(5){width:54pt}.report-calculations th:nth-child(6),.report-calculations td:nth-child(6){width:48pt}.report-comment-lines>div{border-bottom:0;height:10.45pt}.report-net-total{top:190pt;bottom:auto;left:0;right:0;height:10pt;padding:0;font-size:7.98pt;font-weight:normal;text-align:left}.report-net-total span{position:absolute;white-space:nowrap}.report-net-total span:nth-child(1){left:46.5pt}.report-net-total span:nth-child(2){left:147pt}.report-net-total span:nth-child(3){left:204pt}.report-net-total span:nth-child(4){left:227pt}.report-net-total span:nth-child(5){right:3pt}`;

  function renderPage(model, page, pageIndex, areaCodes, layout) {
    const size = PAGE_SIZES[layout.paperSize];
    const sketchHeight = SKETCH_HEIGHTS[layout.paperSize] || SKETCH_HEIGHT;
    const lowerTop = 122.4 + sketchHeight;
    const lowerHeight = Math.max(120, size.height - lowerTop - LOWER_MARGIN);
    const horizontalShift = (size.width - 612) / 2;
    const comments = layout.showCommentTables ? renderComments(model.commentTables || [], layout, lowerHeight) : '';
    const lowerColumns = layout.showCommentTables ? '324pt 216pt' : '540pt';
    const sketchComments = layout.showSketchComments && model.subjectInfo?.generalComments
      ? `<div style="position:absolute;left:5pt;right:5pt;bottom:4pt;font:6pt Verdana,Arial,sans-serif;color:#333">${escapeHTML(model.subjectInfo.generalComments)}</div>` : '';
    return `<article class="report-page" data-report-page="${pageIndex + 1}" data-paper-size="${layout.paperSize}" data-current-form="${escapeHTML(layout.currentForm)}" style="width:${size.width}pt;height:${size.height}pt"><div style="position:absolute;inset:0;transform:translateX(${horizontalShift.toFixed(2)}pt)"><div class="report-title">${escapeHTML(layout.captions.sketchPageTitle)}</div>${renderSubject(model.subjectInfo || {}, layout)}<section class="report-sketch-heading"><div class="report-section-title">${escapeHTML(layout.captions.sketchLabel)}</div></section><section class="report-sketch-body" style="height:${sketchHeight.toFixed(2)}pt">${renderSketchSvg(page, areaCodes, sketchHeight, model.options)}${sketchComments}</section><div class="report-lower" style="top:${lowerTop.toFixed(2)}pt;height:${lowerHeight.toFixed(2)}pt;grid-template-columns:${lowerColumns}">${renderCalculations(page, areaCodes, layout, lowerHeight)}${comments}</div></div>${layout.showPageNumbers ? `<div class="report-page-number" style="top:${(size.height - 21.5).toFixed(2)}pt;right:${((size.width - 540) / 2).toFixed(2)}pt">Page ${pageIndex + 1}</div>` : ''}</article>`;
  }
  function render(model, areaCodes) {
    const pages = model?.pages?.length ? model.pages : [{ index: 0, lines: [], polygons: [], labels: [], symbols: [] }];
    const layout = normalizeLayout(model?.layout);
    return pages.map((page, index) => renderPage(model || {}, page, index, areaCodes || root.AREA_CODES || [], layout)).join('');
  }
  function documentHTML(model, areaCodes, autoPrint = false) {
    const title = escapeHTML(String(model?.name || 'Sketch'));
    const layout = normalizeLayout(model?.layout);
    const size = PAGE_SIZES[layout.paperSize];
    return `<!doctype html><html><head><meta charset="utf-8"><title>${title}</title><style>${PRECISE_REPORT_CSS}@page{size:${size.css};margin:0}</style></head><body>${render(model, areaCodes)}${autoPrint ? '<script>window.addEventListener("load",()=>setTimeout(()=>window.print(),60))<\/script>' : ''}</body></html>`;
  }
  root.SketchReport = Object.freeze({ PAGE_WIDTH, PAGE_HEIGHT, PAGE_SIZES, REPORT_CSS: PRECISE_REPORT_CSS, normalizeLayout, render, documentHTML, renderSketchSvg, calculationRows });
})(typeof globalThis !== 'undefined' ? globalThis : window);

;
/*
 * Export - SVG and DXF output.
 * Exports the current page geometry in true scale (1 foot = 20 SVG units).
 * SVG mirrors the canvas appearance (wall colors/widths, fills, labels,
 * symbols). DXF targets simple R12 ASCII (LINE/ARC/POLYLINE/TEXT) for
 * interoperability, not round-trip editing.
 */

