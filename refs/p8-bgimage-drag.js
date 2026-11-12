// P8 ref extract: bgimage-drag — lines 7270-7300 of 00) sketch standalone.html

7270:           const step = e.ctrlKey ? 1 : 15;
7271:           const pointerDelta = this.normalizeDegrees((currentAngle - drag.startAngle) * 180 / Math.PI);
7272:           const nextRotation = drag.startRotation - Math.round(pointerDelta / step) * step;
7273:           if (drag.moved || Math.abs(nextRotation - drag.startRotation) >= step) {
7274:             if (!drag.moved) { this.app.pushUndo(); drag.moved = true; }
7275:             if (drag.entityType === 'areaLabel') entity.labelRotation = nextRotation;
7276:             else if (drag.entityType === 'areaSize') entity.areaSizeRotation = nextRotation;
7277:             else entity.rotation = nextRotation;
7278:           }
7279:           drag.previewDegrees = this.normalizeDegrees(nextRotation - drag.startRotation);
7280:           this.canvas.style.cursor = 'grabbing';
7281:         }
7282:       }
7283: 
7284:       if (this.app.state.dragState?.type === 'photo') {
7285:         const drag = this.app.state.dragState;
7286:         const current = this.app.state.pointerWorld;
7287:         this.app.bgImage.x += current.x - drag.lastWorld.x;
7288:         this.app.bgImage.y += current.y - drag.lastWorld.y;
7289:         drag.lastWorld = { ...current };
7290:       }
7291: 
7292:       if (this.app.state.dragState?.type === 'selection') {
7293:         const drag = this.app.state.dragState;
7294:         const current = this.app.state.pointerWorld;
7295:         let dx = current.x - drag.lastWorld.x;
7296:         let dy = current.y - drag.lastWorld.y;
7297: 
7298:         // When area grid snap is enabled, snap the selection's absolute
7299:         // position to the selected pitch grid so areas land exactly on the
7300:         // pitch lines, regardless of where the grab point started.
