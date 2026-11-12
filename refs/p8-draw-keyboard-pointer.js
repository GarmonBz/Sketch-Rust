// P8 ref extract: draw-keyboard-pointer — lines 9030-9165 of 00) sketch standalone.html

9030:         this.drawLiveAngle(previousPt, lastPt, snapped);
9031:         this.drawReturnGuide(s.currentPolyPoints[0], snapped);
9032:       }
9033:     }
9034:   }
9035: 
9036:   drawKeyboardPreview() {
9037:     const s = this.app.state;
9038:     if (!s.penDown || s.currentPolyPoints.length === 0) return;
9039:     if (s.mode !== 'draw-exterior' && s.mode !== 'draw-interior') return;
9040: 
9041:     const lastPt = s.currentPolyPoints[s.currentPolyPoints.length - 1];
9042:     const cursor = s.cursor;
9043:     // Only draw if cursor has moved past the last committed point
9044:     if (lastPt.x === cursor.x && lastPt.y === cursor.y) return;
9045: 
9046:     const ctx = this.ctx;
9047:     const lastScreen = window.Geometry.toScreen(lastPt, s.pan, s.zoom);
9048:     const cursorScreen = window.Geometry.toScreen(cursor, s.pan, s.zoom);
9049:     const isExt = s.mode === 'draw-exterior';
9050: 
9051:     ctx.save();
9052:     ctx.strokeStyle = this.colors.wallPreview;
9053:     ctx.lineWidth = this.wallWidths[isExt ? 'exterior' : 'interior'];
9054:     ctx.lineCap = 'round';
9055:     ctx.setLineDash([8, 5]);
9056:     ctx.beginPath();
9057:     ctx.moveTo(lastScreen.x, lastScreen.y);
9058:     ctx.lineTo(cursorScreen.x, cursorScreen.y);
9059:     ctx.stroke();
9060:     ctx.restore();
9061: 
9062:     // Dimension label for the preview line
9063:     const dist = Math.hypot(cursor.x - lastPt.x, cursor.y - lastPt.y);
9064:     if (dist > 0.3) {
9065:       const previewPoints = [...s.currentPolyPoints, cursor];
9066:       const dimensionSide = this.getPathDimensionSide(previewPoints, lastPt, cursor);
9067:       this.drawDimension(
9068:         lastPt,
9069:         cursor,
9070:         lastScreen,
9071:         cursorScreen,
9072:         this.colors.wallPreview,
9073:         dimensionSide
9074:       );
9075:       this.drawRiseRun(lastPt, cursor);
9076: 
9077:       if (s.currentPolyPoints.length >= 2) {
9078:         const previousPt = s.currentPolyPoints[s.currentPolyPoints.length - 2];
9079:         this.drawLiveAngle(previousPt, lastPt, cursor);
9080:         this.drawReturnGuide(s.currentPolyPoints[0], cursor);
9081:       }
9082:     }
9083:   }
9084: 
9085:   // During keyboard drawing the OS cursor cannot be repositioned by script,
9086:   // so while keyboard-pointer mode is active the OS cursor is hidden and the
9087:   // pointer image is painted at the keyboard cursor (end of the line).
9088:   drawKeyboardPointer() {
9089:     const s = this.app.state;
9090:     if (!s.keyboardPointer || !s.penDown) return;
9091:     if (s.mode !== 'draw-exterior' && s.mode !== 'draw-interior') return;
9092:     const { img, size } = this.getDrawingCursorImage();
9093:     if (!img || !img.complete || !img.naturalWidth) return;
9094:     const sp = window.Geometry.toScreen(s.cursor, s.pan, s.zoom);
9095:     const half = size / 2;
9096:     this.ctx.drawImage(img, sp.x - half, sp.y - half, size, size);
9097:   }
9098: 
9099:   // Returns { img, size } of the pointer variant to paint on-canvas, matching
9100:   // the size option and theme used for the OS cursor.
9101:   getDrawingCursorImage() {
9102:     const size = Number(this.app.state.options?.pointerSize);
9103:     const valid = [32, 64, 128].includes(size) ? size : 128;
9104:     const invert = this.app.state.canvasTheme === 'light';
9105:     const key = `${valid}|${invert}`;
9106:     let entry = this.cursorDrawCache.get(key);
9107:     if (!entry) {
9108:       const img = new Image();
9109:       entry = { img, size: valid, failed: false };
9110:       img.onload = () => this.render();
9111:       img.onerror = () => { entry.failed = true; this.render(); };
9112:       img.src = __cursorUrl(`cursor-${valid}${invert ? '-light' : ''}.png`);
9113:       this.cursorDrawCache.set(key, entry);
9114:     }
9115:     if (entry.failed || (entry.img.complete && !entry.img.naturalWidth)) {
9116:       // Variant file not built; fall back to the original pointer image.
9117:       return { img: this.cursorImg, size: this.cursorImg.naturalWidth || 128 };
9118:     }
9119:     return { img: entry.img, size: entry.size };
9120:   }
9121: 
9122:   drawPolygons(layerId = null) {
9123:     const s = this.app.state;
9124:     const ctx = this.ctx;
9125:     s.polygons.forEach(poly => {
9126:       if (poly.points.length < 3) return;
9127:       if (layerId != null && this.app.getEntityLayerId(poly) !== layerId) return;
9128:       if (!this.app.isEntityVisible(poly)) return;
9129:       const isSelected = s.selectedPolyIds.has(poly.id);
9130:       const areaDefinition = this.app.areaCodes?.find(item => item.code === poly.type);
9131:       const fillPattern = poly.fillPattern || areaDefinition?.fillStyle || 'solid';
9132:       const fillColor = poly.fillColor || areaDefinition?.fillColor || this.colors.fillGLA;
9133:       const fillOpacity = poly.fillOpacity ?? areaDefinition?.fillOpacity;
9134:       const outlineColor = poly.lineColor || areaDefinition?.color || this.colors.polyOutline;
9135: 
9136:       ctx.beginPath();
9137:       const start = window.Geometry.toScreen(poly.points[0], s.pan, s.zoom);
9138:       ctx.moveTo(start.x, start.y);
9139:       for (let i = 0; i < poly.points.length; i++) {
9140:         const nextIndex = (i + 1) % poly.points.length;
9141:         const edgeStart = window.Geometry.toScreen(poly.points[i], s.pan, s.zoom);
9142:         const edgeEnd = window.Geometry.toScreen(poly.points[nextIndex], s.pan, s.zoom);
9143:         const bowAngle = Number(poly.edges?.[i]?.bowAngle) || 0;
9144:         if (Math.abs(bowAngle) > 0.5) this.traceArc(ctx, edgeStart, edgeEnd, bowAngle);
9145:         else ctx.lineTo(edgeEnd.x, edgeEnd.y);
9146:       }
9147:       ctx.closePath();
9148:       
9149:       // Draw-first geometry stays visually as clean wall linework until an
9150:       // area definition is applied, matching the reference workflow.
9151:       if (isSelected || poly.type !== 'UND') {
9152:         if (poly.type !== 'UND' && fillPattern !== 'none') {
9153:           ctx.fillStyle = fillOpacity == null ? fillColor : this.colorWithAlpha(fillColor, Number(fillOpacity) / 100);
9154:           ctx.fill();
9155:           if (fillPattern !== 'solid') {
9156:             ctx.save();
9157:             ctx.clip();
9158:             ctx.strokeStyle = outlineColor;
9159:             ctx.fillStyle = outlineColor;
9160:             ctx.globalAlpha = 0.42;
9161:             ctx.lineWidth = 1;
9162:             const spacing = 10;
9163:             if (fillPattern === 'diagonal' || fillPattern === 'crosshatch') {
9164:               for (let x = -this.viewHeight; x < this.viewWidth + this.viewHeight; x += spacing) {
9165:                 ctx.beginPath(); ctx.moveTo(x, 0); ctx.lineTo(x - this.viewHeight, this.viewHeight); ctx.stroke();
