// P8 ref extract: keyboard-pointer-mode — lines 6580-6700 of 00) sketch standalone.html

6580:       if (e.key === 'ArrowUp') {
6581:         dx = parsed.run; dy = parsed.rise;
6582:       } else if (e.key === 'ArrowDown') {
6583:         dx = parsed.run; dy = -parsed.rise;
6584:       } else if (e.key === 'ArrowLeft') {
6585:         dx = -parsed.run; dy = parsed.rise;
6586:       } else if (e.key === 'ArrowRight') {
6587:         dx = parsed.run; dy = parsed.rise;
6588:       } else return;
6589: 
6590:       e.preventDefault();
6591:       const newPos = { x: this.app.state.cursor.x + dx, y: this.app.state.cursor.y + dy };
6592:       this.app.state.cursor = { x: this.app.cleanCoord(newPos.x), y: this.app.cleanCoord(newPos.y) };
6593:       this.buffer = '';
6594:       this.bowAngle = 0;
6595:       this.enterKeyboardPointerMode();
6596:       this.updateKeyboardHUD();
6597:       this.app.render();
6598:       return;
6599:     }
6600: 
6601:     // Plain distance or no buffer
6602:     if (parsed && parsed.type === 'distance') {
6603:       arrowDist = parsed.distance;
6604:     } else {
6605:       const unit = this.app.state.options?.stepUnit || this.app.state.stepUnit || 'whole';
6606:       arrowDist = e.shiftKey ? 0.1 : (unit === 'tenths' ? 0.1 : unit === 'half' ? 0.5 : 1.0);
6607:     }
6608: 
6609:     // PgUp/Home/End/PgDn walk 45-degree diagonals (up-right, up-left,
6610:     // down-left, down-right); the distance is measured along the
6611:     // diagonal, so "10" PgUp moves 7.07' over and 7.07' up.
6612:     const diagonal = { PageUp: [1, 1], Home: [-1, 1], End: [-1, -1], PageDown: [1, -1] }[e.key];
6613:     if (e.key === 'ArrowUp') { dy = arrowDist; }
6614:     else if (e.key === 'ArrowDown') { dy = -arrowDist; }
6615:     else if (e.key === 'ArrowLeft') { dx = -arrowDist; }
6616:     else if (e.key === 'ArrowRight') { dx = arrowDist; }
6617:     else if (diagonal) { dx = diagonal[0] * arrowDist * Math.SQRT1_2; dy = diagonal[1] * arrowDist * Math.SQRT1_2; }
6618:     else return;
6619: 
6620:     e.preventDefault();
6621: 
6622:     let newPos = { x: this.app.state.cursor.x + dx, y: this.app.state.cursor.y + dy };
6623: 
6624:     // Ctrl+arrow for alignment snapping
6625:     if (e.ctrlKey && !diagonal) {
6626:       let snapDir = -1;
6627:       if (e.key === 'ArrowUp') snapDir = 0;
6628:       else if (e.key === 'ArrowDown') snapDir = 180;
6629:       else if (e.key === 'ArrowLeft') snapDir = 270;
6630:       else if (e.key === 'ArrowRight') snapDir = 90;
6631: 
6632:       if (e.shiftKey) {
6633:         const snapPos = window.Geometry.findIntersectingAlignment(
6634:           this.app.state.cursor, this.app.state.lines, snapDir
6635:         );
6636:         if (snapPos) { newPos = snapPos; }
6637:       } else {
6638:         const snapPos = window.Geometry.findAlignment(
6639:           this.app.state.cursor, this.app.state.lines, snapDir
6640:         );
6641:         if (snapPos) { newPos = snapPos; }
6642:       }
6643:     }
6644: 
6645:     this.app.state.cursor = { x: this.app.cleanCoord(newPos.x), y: this.app.cleanCoord(newPos.y) };
6646:     this.buffer = '';
6647:     this.bowAngle = 0;
6648:     this.enterKeyboardPointerMode();
6649:     this.updateKeyboardHUD();
6650:     this.app.render();
6651:   }
6652: 
6653:   // While the arrow keys are driving the cursor, park the visible pointer
6654:   // at the keyboard cursor (end of the line) instead of leaving it at the
6655:   // mouse position. Moving the mouse more than a few pixels restores the
6656:   // normal OS cursor (see CanvasEngine.pointermove).
6657:   enterKeyboardPointerMode() {
6658:     if (!this.app.state.penDown) return;
6659:     this.app.state.keyboardPointer = true;
6660:     this.app.state.keyboardPointerAnchor = {
6661:       x: this.app.canvas.mouseScreen?.x ?? 0,
6662:       y: this.app.canvas.mouseScreen?.y ?? 0
6663:     };
6664:     this.app.canvas.setCursor();
6665:   }
6666: 
6667:   // Update HUD to show the distance and angle from the last committed point
6668:   // to the current cursor position (keyboard preview line).
6669:   updateKeyboardHUD() {
6670:     const pts = this.app.state.currentPolyPoints;
6671:     if (pts.length > 0) {
6672:       const lastPt = pts[pts.length - 1];
6673:       const cursor = this.app.state.cursor;
6674:       const dx = cursor.x - lastPt.x;
6675:       const dy = cursor.y - lastPt.y;
6676:       const dist = Math.hypot(dx, dy);
6677:       const angle = Math.round(Math.atan2(dy, dx) * 180 / Math.PI);
6678:       this.app.ui.updateHUD(dist.toFixed(1), angle, this.app.state.penDown);
6679:     }
6680:   }
6681: 
6682:   // Space released: restore the normal cursor. A quick tap (no pan drag)
6683:   // keeps the legacy dimension-label toggle.
6684:   handleKeyUp(e) {
6685:     if (e.code !== 'Space') return;
6686:     const target = e.target;
6687:     const tag = target.tagName;
6688:     const isTextEntry = tag === 'TEXTAREA' ||
6689:       (tag === 'INPUT' && /^(text|number|search|email|tel|password|url)$/i.test(target.type || 'text'));
6690:     if (isTextEntry) return;
6691: 
6692:     this.app.state.spacePan = false;
6693:     if (!this.app.state.spacePanned) {
6694:       const isDrawing = this.app.state.mode === 'draw-exterior' || this.app.state.mode === 'draw-interior';
6695:       const hasSelection = this.app.state.selectedLineIds.size > 0 || this.app.state.selectedPolyIds.size > 0 ||
6696:         this.app.state.selectedAreaLabelIds.size > 0 || this.app.state.selectedAreaSizeIds?.size > 0 ||
6697:         this.app.state.selectedLabelIds.size > 0 || this.app.state.selectedSymbolIds.size > 0 ||
6698:         this.app.state.selectedDimensionIds.size > 0 || this.app.state.selectedChainIds.size > 0;
6699:       if (isDrawing) this.app.toggleLastDimension();
6700:       else if (hasSelection) this.app.toggleSelectedDimensions();
