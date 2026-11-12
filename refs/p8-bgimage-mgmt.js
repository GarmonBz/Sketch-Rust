// P8 ref extract: bgimage-mgmt — lines 19140-19345 of 00) sketch standalone.html

19140:     this.clearSelection();
19141:     this.recalculate();
19142:     this.ui.updateUndoRedoControls();
19143:     this.ui.updateNorthDisplay?.();
19144:     this.ui.updateHUD('', '', this.state.penDown);
19145:     this.ui.updateCalcsPanel();
19146:     this.render();
19147:   }
19148: 
19149:   // Rewind to the snapshot at the given undo-stack index (undo history panel).
19150:   // Each snapshot records the state BEFORE its action, so applying entry i
19151:   // means undoing until entry i itself is popped and restored.
19152:   undoTo(index) {
19153:     const target = Math.max(0, Math.min(index, this.state.undoStack.length - 1));
19154:     let guard = this.state.undoStack.length + 1;
19155:     while (this.state.undoStack.length > target && guard-- > 0) this.undo();
19156:   }
19157: 
19158:   // --- Reference image ---
19159: 
19160:   loadBgImage(file) {
19161:     const reader = new FileReader();
19162:     reader.onload = (e) => {
19163:       const img = new Image();
19164:       img.onload = () => {
19165:         this.bgImage.img = img;
19166:         this.bgImage.visible = true;
19167:         this.bgImage.x = 0;
19168:         this.bgImage.y = 0;
19169:         // Auto-scale: fit image to ~60 feet wide by default
19170:         const PPF = window.Geometry.PIXELS_PER_FOOT;
19171:         this.bgImage.scale = (60 * PPF) / img.width;
19172:         this.bgImage.rotation = 0;
19173:         this.bgImage.flipH = false;
19174:         this.bgImage.flipV = false;
19175:         this.bgImage.opacity = 0.5;
19176:         const slider = document.getElementById('photo-intensity');
19177:         if (slider) slider.value = 50;
19178:         this.render();
19179:       };
19180:       img.src = e.target.result;
19181:     };
19182:     reader.readAsDataURL(file);
19183:   }
19184: 
19185:   async showBgImage() {
19186:     if (!this.bgImage.img) { await this.ui.alertDialog('No image loaded. Use Load Image first.'); return; }
19187:     this.bgImage.visible = true;
19188:     this.render();
19189:   }
19190: 
19191:   hideBgImage() {
19192:     this.bgImage.visible = false;
19193:     this.render();
19194:   }
19195: 
19196:   async removeBgImage() {
19197:     if (!this.bgImage.img) return;
19198:     if (!(await this.ui.confirmDialog('Remove the background image?'))) return;
19199:     this.bgImage.img = null;
19200:     this.bgImage.visible = false;
19201:     this.bgImage.scaleMode = false;
19202:     this.bgImage.scaleP1 = null;
19203:     this.bgImage.scaleP2 = null;
19204:     this.render();
19205:   }
19206: 
19207:   rotateBgImage(degrees) {
19208:     if (!this.bgImage.img) return;
19209:     this.bgImage.rotation = (this.bgImage.rotation + degrees) % 360;
19210:     this.render();
19211:   }
19212: 
19213:   flipBgImageH() {
19214:     if (!this.bgImage.img) return;
19215:     this.bgImage.flipH = !this.bgImage.flipH;
19216:     this.render();
19217:   }
19218: 
19219:   flipBgImageV() {
19220:     if (!this.bgImage.img) return;
19221:     this.bgImage.flipV = !this.bgImage.flipV;
19222:     this.render();
19223:   }
19224: 
19225:   scaleBgImageBy(factor) {
19226:     if (!this.bgImage.img) return;
19227:     this.bgImage.scale *= factor;
19228:     this.render();
19229:   }
19230: 
19231:   setBgIntensity(value) {
19232:     this.bgImage.opacity = value / 100;
19233:     this.render();
19234:   }
19235: 
19236:   async startSetScale() {
19237:     if (!this.bgImage.img) { await this.ui.alertDialog('No image loaded.'); return; }
19238:     this.bgImage.scaleMode = true;
19239:     this.bgImage.scaleP1 = null;
19240:     this.bgImage.scaleP2 = null;
19241:     this.setMode('select');
19242:     await this.ui.alertDialog('Click two points on the image that represent a known distance, then enter the measurement.');
19243:   }
19244: 
19245:   togglePhotoEditMode() {
19246:     if (!this.bgImage.img) {
19247:       this.ui.notify('Load an image before entering Image Edit Mode.');
19248:       return;
19249:     }
19250:     this.bgImage.editMode = !this.bgImage.editMode;
19251:     this.bgImage.orientMode = false;
19252:     document.getElementById('btn-photo-edit-mode')?.classList.toggle('active', this.bgImage.editMode);
19253:     this.ui.notify(this.bgImage.editMode
19254:       ? 'Image Edit Mode: drag the image to reposition it; use the wheel to resize.'
19255:       : 'Image Edit Mode off.');
19256:     this.render();
19257:   }
19258: 
19259:   startPhotoOrient() {
19260:     if (!this.bgImage.img) {
19261:       this.ui.notify('Load an image before using Orient Guide.');
19262:       return;
19263:     }
19264:     this.bgImage.editMode = false;
19265:     this.bgImage.orientMode = true;
19266:     this.bgImage.orientP1 = null;
19267:     this.bgImage.orientP2 = null;
19268:     document.getElementById('btn-photo-edit-mode')?.classList.remove('active');
19269:     this.setMode('select');
19270:     this.ui.notify('Orient Guide: click two points along a line in the image.');
19271:     this.render();
19272:   }
19273: 
19274:   handleOrientClick(worldPt) {
19275:     if (!this.bgImage.orientMode) return false;
19276:     if (!this.bgImage.orientP1) {
19277:       this.bgImage.orientP1 = { ...worldPt };
19278:       this.render();
19279:       return true;
19280:     }
19281: 
19282:     this.bgImage.orientP2 = { ...worldPt };
19283:     const dx = this.bgImage.orientP2.x - this.bgImage.orientP1.x;
19284:     const dy = this.bgImage.orientP2.y - this.bgImage.orientP1.y;
19285:     const currentAngle = Math.atan2(dy, dx) * 180 / Math.PI;
19286:     this.bgImage.orientMode = false;
19287:     this.bgImage.orientP1 = null;
19288:     this.bgImage.orientP2 = null;
19289:     this.ui.promptDialog('Orient this guide Horizontal or Vertical?', 'Horizontal').then(orientation => {
19290:       const value = (orientation || '').trim().toLowerCase();
19291:       if (value.startsWith('v')) this.bgImage.rotation += 90 - currentAngle;
19292:       else this.bgImage.rotation -= currentAngle;
19293:       this.ui.notify('Image orientation applied.');
19294:       this.render();
19295:     });
19296:     return true;
19297:   }
19298: 
19299:   async handleScaleClick(worldPt) {
19300:     if (!this.bgImage.scaleMode) return false;
19301: 
19302:     if (!this.bgImage.scaleP1) {
19303:       this.bgImage.scaleP1 = { ...worldPt };
19304:       this.render();
19305:       return true;
19306:     }
19307: 
19308:     this.bgImage.scaleP2 = { ...worldPt };
19309:     this.render();
19310: 
19311:     // Calculate pixel distance on image between the two points
19312:     const dx = this.bgImage.scaleP2.x - this.bgImage.scaleP1.x;
19313:     const dy = this.bgImage.scaleP2.y - this.bgImage.scaleP1.y;
19314:     const worldDist = Math.sqrt(dx * dx + dy * dy);
19315: 
19316:     const measurement = await this.ui.promptDialog('Enter the known distance between the two points (in feet):');
19317:     if (!measurement || isNaN(parseFloat(measurement)) || parseFloat(measurement) <= 0) {
19318:       this.bgImage.scaleMode = false;
19319:       this.bgImage.scaleP1 = null;
19320:       this.bgImage.scaleP2 = null;
19321:       this.render();
19322:       return true;
19323:     }
19324: 
19325:     const knownFeet = parseFloat(measurement);
19326:     // worldDist is in world units (feet). We need to adjust image scale so the
19327:     // distance on the image matches knownFeet.
19328:     // Current ratio: worldDist feet on screen = actual knownFeet
19329:     // Scale factor adjustment:
19330:     const scaleFactor = knownFeet / worldDist;
19331:     this.bgImage.scale *= scaleFactor;
19332: 
19333:     this.bgImage.scaleMode = false;
19334:     this.bgImage.scaleP1 = null;
19335:     this.bgImage.scaleP2 = null;
19336:     this.render();
19337:     return true;
19338:   }
19339: 
19340:   // --- Page Management ---
19341: 
19342:   getPageData() {
19343:     return {
19344:       lines: JSON.parse(JSON.stringify(this.state.lines)),
19345:       polygons: JSON.parse(JSON.stringify(this.state.polygons)),
