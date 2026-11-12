// P8 ref extract: drawing-scale-factor — lines 18950-18990 of 00) sketch standalone.html

18950:   zoomTo(newZoom, centerX, centerY) {
18951:     const oldZoom = this.state.zoom;
18952:     newZoom = Math.max(0.02, Math.min(10, newZoom));
18953:     this.state.pan.x = centerX - (centerX - this.state.pan.x) * (newZoom / oldZoom);
18954:     this.state.pan.y = centerY - (centerY - this.state.pan.y) * (newZoom / oldZoom);
18955:     this.state.zoom = newZoom;
18956:     this.ui.updateZoomDisplay();
18957:     this.render();
18958:   }
18959: 
18960:   // Zoom that 100% means at the current 1:N drawing scale (1:10 is 1.0).
18961:   drawingScaleFactor() {
18962:     const scale = Number(this.state.drawingScale) || 10;
18963:     return 10 / scale;
18964:   }
18965: 
18966:   setDrawingScale(scale) {
18967:     let next = Number(scale);
18968:     if (!Number.isFinite(next) || next <= 0) { this.ui.updateZoomDisplay(); return; }
18969:     // 1:1 through 1:100 covers residential sketches and keeps every scale
18970:     // inside the zoom range, so the percentage readout stays truthful.
18971:     next = Math.max(1, Math.min(100, next));
18972:     const previous = this.drawingScaleFactor();
18973:     this.state.drawingScale = next;
18974:     const cx = this.canvas.viewWidth / 2;
18975:     const cy = this.canvas.viewHeight / 2;
18976:     this.zoomTo(this.state.zoom * this.drawingScaleFactor() / previous, cx, cy);
18977:   }
18978: 
18979:   zoomIn() {
18980:     const cx = this.canvas.viewWidth / 2;
18981:     const cy = this.canvas.viewHeight / 2;
18982:     this.zoomTo(this.state.zoom * 1.2, cx, cy);
18983:   }
18984: 
18985:   zoomOut() {
18986:     const cx = this.canvas.viewWidth / 2;
18987:     const cy = this.canvas.viewHeight / 2;
18988:     this.zoomTo(this.state.zoom / 1.2, cx, cy);
18989:   }
18990: 
