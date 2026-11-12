// P8 ref extract: bgimage-drag-2 — lines 7450-7475 of 00) sketch standalone.html

7450:         return;
7451:       }
7452:       const mode = this.app.state.mode;
7453:       const rect = this.canvas.getBoundingClientRect();
7454:       const screenPt = { x: e.clientX - rect.left, y: e.clientY - rect.top };
7455:       const worldPt = window.Geometry.toWorld(screenPt, this.app.state.pan, this.app.state.zoom);
7456: 
7457:       if (this.app.handleOrientClick(worldPt)) return;
7458: 
7459:       // Intercept Set Scale clicks
7460:       if (this.app.bgImage && this.app.bgImage.scaleMode) {
7461:         this.app.handleScaleClick(worldPt);
7462:         return;
7463:       }
7464: 
7465:       if (mode === 'define-area') {
7466:         this.app.areaWorkflow?.click(worldPt);
7467:       } else if (mode === 'draw-exterior' || mode === 'draw-interior') {
7468:         this.app.handleCanvasClick(worldPt);
7469:       } else if (mode === 'select') {
7470:         this.app.selectLineAt(worldPt, e.ctrlKey);
7471:       } else if (mode === 'place-text') {
7472:         this.app.placeTextAt(worldPt);
7473:       } else if (mode === 'place-symbol') {
7474:         this.app.placeSymbolAt(worldPt);
7475:       } else if (mode === 'insert-vertex') {
