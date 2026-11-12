class InputController {
  constructor(app) {
    this.app = app;
    this.buffer = '';
    // Curve bowing state: when drawing, mouse wheel bows the rubber-band
    this.bowAngle = 0; // degrees of arc bow, 0 = straight
    
    this.handleKeyDown = this.handleKeyDown.bind(this);
    this.handleKeyUp = this.handleKeyUp.bind(this);
    window.addEventListener('keydown', this.handleKeyDown);
    window.addEventListener('keyup', this.handleKeyUp);
  }

  // Parse the buffer for rise×run or distance<angle
  parseBuffer() {
    const buf = this.buffer.trim();
    if (!buf) return null;

    // Rise × Run: "10x5" or "10X5"
    const rxMatch = buf.match(/^([0-9.]+)[xX×]([0-9.]+)$/);
    if (rxMatch) {
      return {
        type: 'riserun',
        rise: parseFloat(rxMatch[1]),
        run: parseFloat(rxMatch[2])
      };
    }

    // Distance < Angle: "10<45"
    const angMatch = buf.match(/^([0-9.]+)<([0-9.]+)$/);
    if (angMatch) {
      return {
        type: 'angle',
        distance: parseFloat(angMatch[1]),
        angle: parseFloat(angMatch[2])
      };
    }

    // Direct turn: "10l45" or "10r45". Omitting the angle defaults to 90 degrees.
    const turnMatch = buf.match(/^([0-9.]+)([lLrR])([0-9.]*)$/);
    if (turnMatch) {
      return {
        type: 'turn',
        distance: parseFloat(turnMatch[1]),
        direction: turnMatch[2].toLowerCase(),
        angle: turnMatch[3] ? parseFloat(turnMatch[3]) : 90
      };
    }

    // Plain distance
    const val = parseFloat(buf);
    if (!isNaN(val)) {
      return { type: 'distance', distance: val };
    }

    return null;
  }

  handleKeyDown(e) {
    // Keyboard drawing must keep working after the user touches a toolbar
    // button, slider, combobox, or checkbox. Only real text-entry fields
    // keep capturing keystrokes; every other focused control releases
    // focus so the key still reaches the drawing commands below.
    const target = e.target;
    const tag = target.tagName;
    if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT' || tag === 'BUTTON') {
      const isTextEntry = tag === 'TEXTAREA' ||
        (tag === 'INPUT' && /^(text|number|search|email|tel|password|url)$/i.test(target.type || 'text'));
      // A dropdown needs its arrows, Enter and type-ahead to choose an
      // option; blurring it here leaked Enter to the canvas and switched the
      // tool to Draw. Clicking the drawing releases it as usual.
      if (isTextEntry || tag === 'SELECT') {
        if (e.key === 'Escape') { target.blur(); return; }
        return;
      }
      // Sliders, checkboxes, color pickers, selects, and buttons: drop
      // focus so Enter/Space/arrows drive drawing instead of silently
      // dying on the control (or re-triggering it).
      target.blur();
    }

    const isDrawing = this.app.state.mode === 'draw-exterior' || this.app.state.mode === 'draw-interior';
    const hasSelection = this.app.state.selectedLineIds.size > 0 || this.app.state.selectedPolyIds.size > 0 ||
      this.app.state.selectedAreaLabelIds.size > 0 || this.app.state.selectedAreaSizeIds?.size > 0 ||
      this.app.state.selectedLabelIds.size > 0 || this.app.state.selectedSymbolIds.size > 0 ||
      this.app.state.selectedDimensionIds.size > 0 || this.app.state.selectedChainIds.size > 0;

    // --- Global shortcuts ---

    if (e.ctrlKey && e.key.toLowerCase() === 'z') {
      e.preventDefault();
      if (e.shiftKey) this.app.redo();
      else this.app.undo();
      return;
    }
    if (e.ctrlKey && e.key.toLowerCase() === 'y') {
      e.preventDefault(); this.app.redo(); return;
    }
    if (e.ctrlKey && e.key.toLowerCase() === 'a') {
      e.preventDefault(); this.app.selectAll(); return;
    }
    if (e.ctrlKey && e.key.toLowerCase() === 'x') {
      e.preventDefault(); this.app.cutSelected(); return;
    }
    // The measure tool binds Ctrl+C to its own readings; see the measure block.
    if (e.ctrlKey && e.key.toLowerCase() === 'c' && hasSelection && this.app.state.mode !== 'measure') {
      e.preventDefault(); this.app.copySelected(); return;
    }
    if (e.ctrlKey && e.key.toLowerCase() === 'v') {
      e.preventDefault(); this.app.paste(); return;
    }
    if (e.ctrlKey && e.key.toLowerCase() === 's') {
      e.preventDefault(); this.app.saveToFile(); return;
    }
    if (e.ctrlKey && e.key.toLowerCase() === 'o') {
      e.preventDefault();
      document.getElementById('file-open-input')?.click();
      return;
    }
    if (e.ctrlKey && e.key.toLowerCase() === 'n') {
      e.preventDefault(); this.app.newSketch(); return;
    }
    if (e.ctrlKey && e.key === 'F1') {
      e.preventDefault(); this.app.emailSupport(); return;
    }
    if (e.key === 'F1') {
      e.preventDefault(); this.app.ui.showShortcuts(); return;
    }
    if (e.ctrlKey && e.key === 'F3') {
      e.preventDefault(); this.app.autoClose(); return;
    }
    if (e.ctrlKey && e.key === 'F12') {
      e.preventDefault(); this.app.saveAs(); return;
    }

    if (e.key === 'F2') {
      e.preventDefault(); this.app.saveToFile(); return;
    }
    if (e.key === 'F3') {
      e.preventDefault(); document.getElementById('file-open-input')?.click(); return;
    }
    if (e.key === 'F4') {
      e.preventDefault(); this.app.ui.showPanel('panel-define-area'); return;
    }
    if (e.key === 'F5') {
      e.preventDefault(); this.app.setMode('draw-interior'); return;
    }
    if (e.key === 'F12') {
      e.preventDefault(); this.app.saveAs(); return;
    }

    // --- Measure tool ---
    // Claimed before the shared Escape/Enter/Delete handling so a measuring
    // session keeps its own meaning for those keys.
    if (this.app.state.mode === 'measure') {
      if (e.key === 'Enter') {
        e.preventDefault();
        if (!this.app.finishMeasureRun(e.shiftKey)) this.app.ui.notify('Measure: click at least two points first.');
        return;
      }
      if (e.key === 'Backspace') {
        e.preventDefault();
        this.app.undoMeasurePoint();
        return;
      }
      if (e.key === 'Delete') {
        e.preventDefault();
        this.app.clearMeasurements();
        return;
      }
      if (e.ctrlKey && e.key.toLowerCase() === 'c') {
        e.preventDefault();
        this.app.copyMeasurements();
        return;
      }
      if (e.key === 'Escape') {
        e.preventDefault();
        // Back out one level at a time: the open run, then the readings left
        // on screen, then the tool itself.
        if (!this.app.cancelMeasureRun() && !this.app.clearMeasurements()) this.app.setMode('select');
        this.buffer = '';
        return;
      }
    }

    // Escape
    if (e.key === 'Escape') {
      if (this.app.state.penDown) {
        this.app.penUp();
      } else if (hasSelection) {
        this.app.clearSelection();
      } else {
        this.app.setMode('select');
      }
      this.buffer = '';
      this.bowAngle = 0;
      return;
    }

    // F6: Toggle Calculations panel
    if (e.key === 'F6') {
      e.preventDefault();
      this.app.ui.toggleCalcsPanel();
      return;
    }

    if (e.key === 'Insert') {
      e.preventDefault();
      this.app.setMode('insert-vertex');
      return;
    }

    // Delete / Backspace
    if (e.key === 'Delete' || e.key === 'Backspace') {
      if (hasSelection) {
        e.preventDefault();
        this.app.deleteSelected();
        return;
      }
      // In drawing mode, backspace removes last char from buffer
      if (isDrawing && this.buffer.length > 0) {
        e.preventDefault();
        this.buffer = this.buffer.slice(0, -1);
        this.app.ui.updateHUD(this.buffer || '', null, this.app.state.penDown);
        return;
      }
    }

    // G: Toggle Grid
    if (e.key.toLowerCase() === 'g' && !e.ctrlKey) {
      this.app.toggleGrid(); return;
    }

    if (!e.ctrlKey && !e.altKey && e.key.toLowerCase() === 'e') {
      this.app.setMode('draw-exterior'); return;
    }
    if (!e.ctrlKey && !e.altKey && e.key.toLowerCase() === 'i') {
      this.app.setMode('draw-interior'); return;
    }
    if (!e.ctrlKey && !e.altKey && e.key.toLowerCase() === 'p' && !isDrawing) {
      this.app.setMode('select'); return;
    }
    if (!e.ctrlKey && !e.altKey && e.key.toLowerCase() === 's' && !isDrawing) {
      this.app.ui.showPanel('panel-symbol-library'); return;
    }
    if (!e.ctrlKey && !e.altKey && e.key.toLowerCase() === 't' && !isDrawing) {
      this.app.ui.showPanel('panel-text-library'); return;
    }

    if (hasSelection && e.shiftKey && e.ctrlKey && e.key.toLowerCase() === 'r') {
      e.preventDefault(); this.app.rotateSelected(1); return;
    }
    if (hasSelection && e.shiftKey && !e.ctrlKey && e.key.toLowerCase() === 'r') {
      e.preventDefault(); this.app.rotateSelected(5); return;
    }
    if (hasSelection && !e.ctrlKey && !e.altKey && e.key.toLowerCase() === 'h') {
      e.preventDefault(); this.app.orientSelected('horizontal'); return;
    }
    if (hasSelection && !e.ctrlKey && !e.altKey && e.key.toLowerCase() === 'v') {
      e.preventDefault(); this.app.orientSelected('vertical'); return;
    }
    if (hasSelection && (e.code === 'NumpadAdd' || e.key === '+')) {
      e.preventDefault(); this.app.scaleSelected(1.1); return;
    }
    if (hasSelection && (e.code === 'NumpadSubtract' || e.key === '-')) {
      e.preventDefault(); this.app.scaleSelected(0.9); return;
    }

    if (!isDrawing && !e.ctrlKey && !e.altKey && e.key.toLowerCase() === 'f') {
      this.app.flipSketch('horizontal'); return;
    }
    if (!isDrawing && !e.ctrlKey && !e.altKey && e.key.toLowerCase() === 'm') {
      this.app.flipSketch('vertical'); return;
    }
    if (!isDrawing && !e.ctrlKey && !e.altKey && e.key.toLowerCase() === 'l') {
      this.app.rotateSketch(-90); return;
    }
    if (!isDrawing && !e.ctrlKey && !e.altKey && e.key.toLowerCase() === 'r') {
      this.app.rotateSketch(90); return;
    }

    if (!e.ctrlKey && !e.altKey && e.key.toLowerCase() === 'd' && !this.buffer) {
      this.app.state.curveMode = this.app.state.curveMode === 'degree' ? 'height' : 'degree';
      this.app.ui.notify(`Curve mode: ${this.app.state.curveMode === 'degree' ? 'Degree' : 'Height'}`);
      return;
    }

    // C: Center / Shift+C: Center & Fit (when not drawing)
    if (e.key.toLowerCase() === 'c' && !e.ctrlKey && !isDrawing) {
      if (e.shiftKey) this.app.centerAndFit();
      else this.app.centerSketch();
      return;
    }

    // Space: hold to pan (temporary hand tool). The old quick-tap behavior
    // (toggle dimension labels) is preserved in handleKeyUp, so it only fires
    // when the key is released without having dragged.
    if (e.code === 'Space' && !e.repeat) {
      e.preventDefault();
      this.app.state.spacePan = true;
      this.app.state.spacePanned = false;
      this.app.canvas.setCursor();
      return;
    }

    // Move the selection with the arrows in any non-drawing mode, so a line
    // selected with the pointer, fence, or lasso can always be nudged.
    if (!isDrawing && hasSelection) {
      const step = e.shiftKey ? 0.1 : 1.0;
      let dx = 0, dy = 0;
      if (e.key === 'ArrowUp') { dy = step; }
      else if (e.key === 'ArrowDown') { dy = -step; }
      else if (e.key === 'ArrowLeft') { dx = -step; }
      else if (e.key === 'ArrowRight') { dx = step; }
      if (dx !== 0 || dy !== 0) {
        e.preventDefault();
        this.app.moveSelected(dx, dy);
        this.app.finalizeSelectionMove();
        return;
      }
    }

    // If not drawing, Enter → draw-exterior
    if (!isDrawing) {
      if (e.key === 'Enter') this.app.setMode('draw-exterior');
      return;
    }

    // ============================
    // Drawing mode shortcuts below
    // ============================

    // Buffer characters: digits, dot, 'x', '<', and direct-turn L/R
    if (/^[0-9.]$/.test(e.key)) {
      this.buffer += e.key;
      this.app.ui.updateHUD(this.buffer, null, this.app.state.penDown);
      return;
    }
    if (e.key.toLowerCase() === 'x' && this.buffer.length > 0) {
      // Only add 'x' if buffer has a number before it (rise×run)
      if (/[0-9]$/.test(this.buffer)) {
        this.buffer += 'x';
        this.app.ui.updateHUD(this.buffer, null, this.app.state.penDown);
        return;
      }
    }
    if ((e.key.toLowerCase() === 'l' || e.key.toLowerCase() === 'r') && /^[0-9.]+$/.test(this.buffer)) {
      this.buffer += e.key.toLowerCase();
      this.app.ui.updateHUD(this.buffer, 90, this.app.state.penDown);
      return;
    }
    if (e.key === '<' || e.key === ',') {
      // '<' for angle entry (Shift+, on US keyboard = <)
      if (this.buffer.length > 0 && /[0-9]$/.test(this.buffer)) {
        this.buffer += '<';
        this.app.ui.updateHUD(this.buffer, null, this.app.state.penDown);
        return;
      }
    }

    // Tab = decimal
    if (e.key === 'Tab') {
      e.preventDefault();
      this.buffer += '.';
      this.app.ui.updateHUD(this.buffer, null, this.app.state.penDown);
      return;
    }

    // Enter: always commits the line from the last committed point to the
    // current cursor position. The buffer is NOT used by Enter — it belongs
    // to the arrow key handler. The only exception: if the cursor hasn't
    // moved (no arrow key was pressed) and the buffer has a distance, we
    // move the cursor in the direction of the last segment as a convenience
    // (legacy behavior).
    if (e.key === 'Enter') {
      if (!this.app.state.penDown) {
        this.app.penDown();
      } else {
        let cursor = this.app.state.cursor;
        const lastPt = this.app.state.currentPolyPoints.length > 0
          ? this.app.state.currentPolyPoints[this.app.state.currentPolyPoints.length - 1]
          : null;

        // If the cursor hasn't moved from the last committed point (no arrow
        // preview is active) and the buffer holds a wall command, apply it to
        // compute the target point. This covers direct entry: "20"<Enter> keeps
        // the current heading, "10<45"<Enter> draws at an absolute angle,
        // "10x5"<Enter> draws a rise/run diagonal, and "10r"<Enter> turns
        // relative to the last wall.
        if (lastPt && cursor.x === lastPt.x && cursor.y === lastPt.y) {
          const parsed = this.parseBuffer();
          if (parsed) {
            const lastId = this.app.state.currentLineIds[this.app.state.currentLineIds.length - 1];
            const lastLine = this.app.state.lines.find(line => line.id === lastId);
            const heading = lastLine
              ? Math.atan2(lastLine.end.y - lastLine.start.y, lastLine.end.x - lastLine.start.x)
              : Math.PI / 2; // first wall defaults to pointing up
            if (parsed.type === 'distance') {
              cursor = {
                x: cursor.x + parsed.distance * Math.cos(heading),
                y: cursor.y + parsed.distance * Math.sin(heading)
              };
            } else if (parsed.type === 'angle') {
              const rad = parsed.angle * Math.PI / 180;
              cursor = {
                x: cursor.x + parsed.distance * Math.cos(rad),
                y: cursor.y + parsed.distance * Math.sin(rad)
              };
            } else if (parsed.type === 'riserun') {
              cursor = { x: cursor.x + parsed.run, y: cursor.y + parsed.rise };
            } else if (parsed.type === 'turn') {
              const turn = parsed.direction === 'l' ? parsed.angle : -parsed.angle;
              const rad = heading + turn * Math.PI / 180;
              cursor = {
                x: cursor.x + parsed.distance * Math.cos(rad),
                y: cursor.y + parsed.distance * Math.sin(rad)
              };
            }
            cursor = { x: this.app.cleanCoord(cursor.x), y: this.app.cleanCoord(cursor.y) };
          }
        }

        // A second Enter with nothing typed and the cursor still on the last
        // point ends the wall chain (pen up).
        if (lastPt && !this.buffer && cursor.x === lastPt.x && cursor.y === lastPt.y &&
            this.app.state.currentLineIds.length > 0) {
          this.app.penUp();
        } else
        // Commit the line from the last committed point to the cursor.
        if (lastPt && (lastPt.x !== cursor.x || lastPt.y !== cursor.y)) {
          const dist = Math.hypot(cursor.x - lastPt.x, cursor.y - lastPt.y);
          const angle = Math.round(Math.atan2(cursor.y - lastPt.y, cursor.x - lastPt.x) * 180 / Math.PI);
          // Temporarily set cursor to the last committed point so drawLine
          // creates a line from there to the actual endpoint.
          this.app.state.cursor = { ...lastPt };
          this.app.drawLine(cursor, dist, angle);
        }
      }
      this.buffer = '';
      this.bowAngle = 0;
      this.app.ui.updateHUD('', null, this.app.state.penDown);
      return;
    }

    // A: Auto Close
    if (e.key.toLowerCase() === 'a' && this.app.state.penDown) {
      this.app.autoClose();
      this.buffer = '';
      this.bowAngle = 0;
      return;
    }

    // B: Bay Window mirror
    if (e.key.toLowerCase() === 'b' && this.app.state.penDown) {
      this.app.bayWindowMirror();
      this.buffer = '';
      return;
    }

    // C: Center (in drawing mode)
    if (e.key.toLowerCase() === 'c' && !e.ctrlKey) {
      if (e.shiftKey) this.app.centerAndFit();
      else this.app.centerSketch();
      return;
    }

    // J: Jump to nearest endpoint
    if (e.key.toLowerCase() === 'j') {
      const nearest = window.Geometry.findNearestEndpoint(
        this.app.state.cursor, this.app.state.lines, Infinity
      );
      if (nearest) {
        this.app.state.cursor = nearest;
        this.app.render();
      }
      return;
    }

    // Arrow keys: move cursor in direction (do NOT commit a wall yet)
    let arrowDist = 0, dx = 0, dy = 0;
    const parsed = this.parseBuffer();

    if (parsed && parsed.type === 'riserun') {
      // Rise×Run + Arrow: direction determines sign
      if (e.key === 'ArrowUp') {
        dx = parsed.run; dy = parsed.rise;
      } else if (e.key === 'ArrowDown') {
        dx = parsed.run; dy = -parsed.rise;
      } else if (e.key === 'ArrowLeft') {
        dx = -parsed.run; dy = parsed.rise;
      } else if (e.key === 'ArrowRight') {
        dx = parsed.run; dy = parsed.rise;
      } else return;

      e.preventDefault();
      const newPos = { x: this.app.state.cursor.x + dx, y: this.app.state.cursor.y + dy };
      this.app.state.cursor = { x: this.app.cleanCoord(newPos.x), y: this.app.cleanCoord(newPos.y) };
      this.buffer = '';
      this.bowAngle = 0;
      this.enterKeyboardPointerMode();
      this.updateKeyboardHUD();
      this.app.render();
      return;
    }

    // Plain distance or no buffer
    if (parsed && parsed.type === 'distance') {
      arrowDist = parsed.distance;
    } else {
      const unit = this.app.state.options?.stepUnit || this.app.state.stepUnit || 'whole';
      arrowDist = e.shiftKey ? 0.1 : (unit === 'tenths' ? 0.1 : unit === 'half' ? 0.5 : 1.0);
    }

    // PgUp/Home/End/PgDn walk 45-degree diagonals (up-right, up-left,
    // down-left, down-right); the distance is measured along the
    // diagonal, so "10" PgUp moves 7.07' over and 7.07' up.
    const diagonal = { PageUp: [1, 1], Home: [-1, 1], End: [-1, -1], PageDown: [1, -1] }[e.key];
    if (e.key === 'ArrowUp') { dy = arrowDist; }
    else if (e.key === 'ArrowDown') { dy = -arrowDist; }
    else if (e.key === 'ArrowLeft') { dx = -arrowDist; }
    else if (e.key === 'ArrowRight') { dx = arrowDist; }
    else if (diagonal) { dx = diagonal[0] * arrowDist * Math.SQRT1_2; dy = diagonal[1] * arrowDist * Math.SQRT1_2; }
    else return;

    e.preventDefault();

    let newPos = { x: this.app.state.cursor.x + dx, y: this.app.state.cursor.y + dy };

    // Ctrl+arrow for alignment snapping
    if (e.ctrlKey && !diagonal) {
      let snapDir = -1;
      if (e.key === 'ArrowUp') snapDir = 0;
      else if (e.key === 'ArrowDown') snapDir = 180;
      else if (e.key === 'ArrowLeft') snapDir = 270;
      else if (e.key === 'ArrowRight') snapDir = 90;

      if (e.shiftKey) {
        const snapPos = window.Geometry.findIntersectingAlignment(
          this.app.state.cursor, this.app.state.lines, snapDir
        );
        if (snapPos) { newPos = snapPos; }
      } else {
        const snapPos = window.Geometry.findAlignment(
          this.app.state.cursor, this.app.state.lines, snapDir
        );
        if (snapPos) { newPos = snapPos; }
      }
    }

    this.app.state.cursor = { x: this.app.cleanCoord(newPos.x), y: this.app.cleanCoord(newPos.y) };
    this.buffer = '';
    this.bowAngle = 0;
    this.enterKeyboardPointerMode();
    this.updateKeyboardHUD();
    this.app.render();
  }

  // While the arrow keys are driving the cursor, park the visible pointer
  // at the keyboard cursor (end of the line) instead of leaving it at the
  // mouse position. Moving the mouse more than a few pixels restores the
  // normal OS cursor (see CanvasEngine.pointermove).
  enterKeyboardPointerMode() {
    if (!this.app.state.penDown) return;
    this.app.state.keyboardPointer = true;
    this.app.state.keyboardPointerAnchor = {
      x: this.app.canvas.mouseScreen?.x ?? 0,
      y: this.app.canvas.mouseScreen?.y ?? 0
    };
    this.app.canvas.setCursor();
  }

  // Update HUD to show the distance and angle from the last committed point
  // to the current cursor position (keyboard preview line).
  updateKeyboardHUD() {
    const pts = this.app.state.currentPolyPoints;
    if (pts.length > 0) {
      const lastPt = pts[pts.length - 1];
      const cursor = this.app.state.cursor;
      const dx = cursor.x - lastPt.x;
      const dy = cursor.y - lastPt.y;
      const dist = Math.hypot(dx, dy);
      const angle = Math.round(Math.atan2(dy, dx) * 180 / Math.PI);
      this.app.ui.updateHUD(dist.toFixed(1), angle, this.app.state.penDown);
    }
  }

  // Space released: restore the normal cursor. A quick tap (no pan drag)
  // keeps the legacy dimension-label toggle.
  handleKeyUp(e) {
    if (e.code !== 'Space') return;
    const target = e.target;
    const tag = target.tagName;
    const isTextEntry = tag === 'TEXTAREA' ||
      (tag === 'INPUT' && /^(text|number|search|email|tel|password|url)$/i.test(target.type || 'text'));
    if (isTextEntry) return;

    this.app.state.spacePan = false;
    if (!this.app.state.spacePanned) {
      const isDrawing = this.app.state.mode === 'draw-exterior' || this.app.state.mode === 'draw-interior';
      const hasSelection = this.app.state.selectedLineIds.size > 0 || this.app.state.selectedPolyIds.size > 0 ||
        this.app.state.selectedAreaLabelIds.size > 0 || this.app.state.selectedAreaSizeIds?.size > 0 ||
        this.app.state.selectedLabelIds.size > 0 || this.app.state.selectedSymbolIds.size > 0 ||
        this.app.state.selectedDimensionIds.size > 0 || this.app.state.selectedChainIds.size > 0;
      if (isDrawing) this.app.toggleLastDimension();
      else if (hasSelection) this.app.toggleSelectedDimensions();
    }
    this.app.canvas.setCursor();
  }
}

window.InputController = InputController;

;
/* Generated by tools/build-symbol-library.js. Do not edit directly. */
window.