// P8 ref extract: bgimage-drag-3 — lines 7555-7600 of 00) sketch standalone.html

7555:       if (e.button === 0 && this.app.state.spacePan) {
7556:         e.preventDefault();
7557:         this.ignoreClick = true;
7558:         this.canvas.setPointerCapture(e.pointerId);
7559:         this.app.state.isPanning = true;
7560:         this.app.state.spacePanned = true;
7561:         this.app.state.panStart = { x: e.clientX, y: e.clientY };
7562:         this.canvas.style.cursor = 'grabbing';
7563:         return;
7564:       }
7565: 
7566:       if (e.button === 0 && this.app.bgImage?.editMode) {
7567:         e.preventDefault();
7568:         this.ignoreClick = true;
7569:         this.canvas.setPointerCapture(e.pointerId);
7570:         this.app.state.dragState = { type: 'photo', lastWorld: { ...worldPt } };
7571:         return;
7572:       }
7573: 
7574:       if (e.button === 0 && this.app.state.mode === 'curve') {
7575:         this.ignoreClick = true;
7576:         const hit = this.app.selectLineAt(worldPt, false);
7577:         if (hit?.type === 'line') {
7578:           const line = this.app.state.lines.find(item => item.id === hit.id);
7579:           if (line) {
7580:             this.canvas.setPointerCapture(e.pointerId);
7581:             this.app.state.dragState = this.createCurveDragState(line, worldPt);
7582:           }
7583:         }
7584:         return;
7585:       }
7586: 
7587:       if (e.button === 0 && this.app.state.mode === 'select') {
7588:         if (this.app.bgImage?.orientMode || this.app.bgImage?.scaleMode) return;
7589:         this.ignoreClick = true;
7590:         const chainHit = this.getChainHit(screenPt);
7591:         if (chainHit) {
7592:           e.preventDefault();
7593:           if (!e.ctrlKey) this.app.clearSelection(false);
7594:           if (e.ctrlKey && this.app.state.selectedChainIds.has(chainHit.chain.id)) {
7595:             this.app.state.selectedChainIds.delete(chainHit.chain.id);
7596:           } else {
7597:             this.app.state.selectedChainIds.add(chainHit.chain.id);
7598:           }
7599:           const computed = this.app.computeChain(chainHit.chain);
7600:           if (computed && this.app.state.selectedChainIds.has(chainHit.chain.id)) {
