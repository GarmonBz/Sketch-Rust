// P8 ref extract: draw-loop-keyboard — lines 7970-8030 of 00) sketch standalone.html

7970:   }
7971: 
7972:   render(options = {}) {
7973:     const ctx = this.ctx;
7974:     const dpr = this.renderDpr || Math.max(1, window.devicePixelRatio || 1);
7975:     ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
7976:     const clean = options.clean === true;
7977:     if (options.transparent) ctx.clearRect(0, 0, this.viewWidth, this.viewHeight);
7978:     else {
7979:       ctx.fillStyle = this.colors.bg;
7980:       ctx.fillRect(0, 0, this.viewWidth, this.viewHeight);
7981:     }
7982:     if (!clean || this.app.state.options?.saveImageWithSketch) this.drawBgImage();
7983:     if (!clean && this.app.state.gridEnabled) this.drawGrid();
7984:     if (!clean) this.drawUnderlay();
7985:     // Walls covered by a dimension chain suppress their per-wall label for
7986:     // this frame only; the chains themselves repaint after the entities.
7987:     this.chainCoverage = this.app.getChainCoverage?.() || new Set();
7988:     // Paint entities bottom-to-top by layer, so a foreground layer's fill and
7989:     // linework always overlap (and obscure) whatever sits on layers below it,
7990:     // no matter the entity type. Each layer's polygons, walls, labels and
7991:     // symbols are painted together in one pass.
7992:     this.app.flattenLayerOrder().forEach(layer => {
7993:       // Groups normally hold no entities; if one does, they paint at the
7994:       // group's own stack position, and hide with the group like any layer.
7995:       if (!this.app.isLayerVisible(layer.id)) return;
7996:       if (this.app.state.filters.areas !== false) this.drawPolygons(layer.id);
7997:       this.drawLines(layer.id);
7998:       if (this.app.state.filters.labels !== false) this.drawLabels(layer.id);
7999:       if (this.app.state.filters.symbols !== false) this.drawSymbols(layer.id);
8000:     });
8001:     this.drawChains();
8002:     if (!clean) {
8003:       this.drawTransformHandles();
8004:       this.drawLineEndpointHandles();
8005:       this.drawWitnessLines();
8006:       this.drawVertices();
8007:       this.drawRubberBand();
8008:       this.drawKeyboardPreview();
8009:       this.drawKeyboardPointer();
8010:       this.drawSnapPreview();
8011:       this.drawFenceRect();
8012:       this.drawLasso();
8013:       this.drawPlacementPreview();
8014:       this.drawScalePoints();
8015:       this.drawMeasurements();
8016:     }
8017:   }
8018: 
8019:   drawGrid() {
8020:     const s = this.app.state;
8021:     const pxPerFt = window.Geometry.PIXELS_PER_FOOT * s.zoom;
8022:     const ctx = this.ctx;
8023:     ctx.lineWidth = 1;
8024:     const offsetX = s.pan.x % pxPerFt;
8025:     const offsetY = s.pan.y % pxPerFt;
8026: 
8027:     ctx.strokeStyle = this.colors.grid10;
8028:     ctx.beginPath();
8029:     for (let x = offsetX; x < this.viewWidth; x += pxPerFt) {
8030:       ctx.moveTo(x, 0); ctx.lineTo(x, this.viewHeight);
