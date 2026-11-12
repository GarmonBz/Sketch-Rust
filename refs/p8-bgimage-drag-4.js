// P8 ref extract: bgimage-drag-4 — lines 7825-7850 of 00) sketch standalone.html

7825:       if (this.canvas.hasPointerCapture && this.canvas.hasPointerCapture(e.pointerId)) {
7826:         this.canvas.releasePointerCapture(e.pointerId);
7827:       }
7828:     });
7829: 
7830:     // Wheel: zoom OR curve bow
7831:     this.canvas.addEventListener('wheel', (e) => {
7832:       e.preventDefault();
7833:       const isDrawing = this.app.state.mode === 'draw-exterior' || this.app.state.mode === 'draw-interior';
7834: 
7835:       if (this.app.bgImage?.editMode) {
7836:         this.app.scaleBgImageBy(e.deltaY < 0 ? 1.05 : 0.95);
7837:         return;
7838:       }
7839: 
7840:       if (this.app.state.mode === 'select') {
7841:         const curveStep = this.app.state.curveMode === 'height'
7842:           ? (e.ctrlKey || e.shiftKey ? 0.1 : 1)
7843:           : (e.ctrlKey ? 1 : (e.shiftKey ? 5 : 15));
7844:         if (this.app.adjustSelectedCurves(e.deltaY < 0 ? curveStep : -curveStep)) return;
7845:         const scaleFactor = e.deltaY < 0 ? 1.08 : 0.92;
7846:         if (this.app.state.selectedAreaLabelIds.size || this.app.state.selectedAreaSizeIds?.size ||
7847:             this.app.state.selectedLabelIds.size || this.app.state.selectedSymbolIds.size) {
7848:           this.app.scaleSelected(scaleFactor);
7849:           return;
7850:         }
