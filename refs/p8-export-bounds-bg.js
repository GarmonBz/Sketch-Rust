// P8 ref extract: export-bounds-bg — lines 19890-19980 of 00) sketch standalone.html

19890:       // empty temporary sets while preserving the user's live selection.
19891:       this.state.selectedLineIds = new Set();
19892:       this.state.selectedPolyIds = new Set();
19893:       this.state.selectedAreaLabelIds = new Set();
19894:       this.state.selectedAreaSizeIds = new Set();
19895:       this.state.selectedLabelIds = new Set();
19896:       this.state.selectedSymbolIds = new Set();
19897:       this.state.activeLineEndpoint = null;
19898:       this.state.penDown = false;
19899:       this.state.dragState = null;
19900: 
19901:       // Parse the selected export resolution and create an off-screen canvas
19902:       // at that exact size so the PNG matches the option chosen by the user.
19903:       const sizeStr = this.state.options.imageSize || '2048x1536';
19904:       const [targetWidth, targetHeight] = sizeStr.split('x').map(Number);
19905:       const exportCanvas = document.createElement('canvas');
19906:       exportCanvas.width = targetWidth;
19907:       exportCanvas.height = targetHeight;
19908: 
19909:       // Fit all sketch content into the export canvas while preserving aspect ratio.
19910:       const hasContent = this.state.lines.length || this.state.polygons.length ||
19911:         this.state.labels.length || this.state.symbols.length ||
19912:         (this.state.options.saveImageWithSketch && this.bgImage.img);
19913:       if (hasContent) {
19914:         const bounds = this.getExportBounds();
19915:         const worldW = bounds.maxX - bounds.minX;
19916:         const worldH = bounds.maxY - bounds.minY;
19917:         const margin = 60;
19918:         const zoomX = (targetWidth - margin * 2) / (Math.max(worldW, 1) * window.Geometry.PIXELS_PER_FOOT);
19919:         const zoomY = (targetHeight - margin * 2) / (Math.max(worldH, 1) * window.Geometry.PIXELS_PER_FOOT);
19920:         this.state.zoom = Math.max(0.1, Math.min(10, Math.min(zoomX, zoomY)));
19921:         const cx = (bounds.minX + bounds.maxX) / 2;
19922:         const cy = (bounds.minY + bounds.maxY) / 2;
19923:         this.state.pan.x = targetWidth / 2 - cx * window.Geometry.PIXELS_PER_FOOT * this.state.zoom;
19924:         this.state.pan.y = targetHeight / 2 + cy * window.Geometry.PIXELS_PER_FOOT * this.state.zoom;
19925:       } else {
19926:         // Empty sketch: center the origin in the exported image.
19927:         this.state.zoom = 1;
19928:         this.state.pan.x = targetWidth / 2;
19929:         this.state.pan.y = targetHeight / 2;
19930:       }
19931: 
19932:       this.canvas.renderTo(exportCanvas, { transparent: true, clean: true });
19933: 
19934:       const link = document.createElement('a');
19935:       link.download = `${this.fileName}.png`;
19936:       link.href = exportCanvas.toDataURL('image/png');
19937:       link.click();
19938:     } finally {
19939:       Object.assign(this.state, transientState);
19940:       this.state.pan = transientState.pan;
19941:       this.render();
19942:       document.getElementById('file-menu').style.display = 'none';
19943:       const submenu = document.getElementById('file-recent-submenu');
19944:       if (submenu) submenu.style.display = 'none';
19945:     }
19946:   }
19947: 
19948:   getExportBounds() {
19949:     let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;
19950:     const expand = (x, y) => {
19951:       minX = Math.min(minX, x); maxX = Math.max(maxX, x);
19952:       minY = Math.min(minY, y); maxY = Math.max(maxY, y);
19953:     };
19954:     this.state.lines.forEach(line => {
19955:       expand(line.start.x, line.start.y);
19956:       expand(line.end.x, line.end.y);
19957:     });
19958:     this.state.polygons.forEach(poly => poly.points.forEach(p => expand(p.x, p.y)));
19959:     this.state.labels.forEach(label => expand(label.x, label.y));
19960:     this.state.symbols.forEach(symbol => expand(symbol.x, symbol.y));
19961:     if (this.state.options.saveImageWithSketch && this.bgImage?.img && this.bgImage.visible) {
19962:       const halfW = (this.bgImage.img.width / this.bgImage.scale) / 2;
19963:       const halfH = (this.bgImage.img.height / this.bgImage.scale) / 2;
19964:       const halfDiag = Math.hypot(halfW, halfH);
19965:       expand(this.bgImage.x - halfDiag, this.bgImage.y - halfDiag);
19966:       expand(this.bgImage.x + halfDiag, this.bgImage.y + halfDiag);
19967:     }
19968:     if (!isFinite(minX)) {
19969:       minX = maxX = minY = maxY = 0;
19970:     }
19971:     return { minX, maxX, minY, maxY };
19972:   }
19973: 
19974:   exportPDF() {
19975:     const model = this.buildExportModel();
19976:     const printWindow = window.open('', '_blank');
19977:     if (!printWindow) {
19978:       this.ui.notify('Allow pop-ups to open the appraisal report.');
19979:       return;
19980:     }
