// P8 ref extract: keyboard-pointer-ctx — lines 6840-6880 of 00) sketch standalone.html

6840:   }
6841: 
6842:   normalizeDegrees(value) {
6843:     let result = value % 360;
6844:     if (result > 180) result -= 360;
6845:     if (result < -180) result += 360;
6846:     return result;
6847:   }
6848: 
6849:   // Central cursor policy: an active drag wins, then holding space
6850:   // (temporary hand tool), then the active mode's cursor.
6851:   setCursor() {
6852:     const s = this.app.state;
6853:     if (s.isPanning || s.dragState) { this.canvas.style.cursor = 'grabbing'; return; }
6854:     if (s.spacePan || s.mode === 'pan') { this.canvas.style.cursor = 'grab'; return; }
6855:     if (s.mode === 'draw-exterior' || s.mode === 'draw-interior') {
6856:       // Keyboard-pointer mode hides the OS cursor; the pointer is painted at
6857:       // the keyboard cursor instead (see drawKeyboardPointer).
6858:       if (s.keyboardPointer) { this.canvas.style.cursor = 'none'; return; }
6859:       this.canvas.style.cursor = this.getDrawingCursor() || `url('${__cursorUrl('cursor.png')}') 64 64, crosshair`;
6860:       return;
6861:     }
6862:     if (s.mode === 'fence' || s.mode === 'lasso' || s.mode === 'insert-vertex' || s.mode === 'zoom-fence' || s.mode === 'curve' || s.mode === 'measure' || s.mode === 'chain-dim' || s.mode === 'place-shape') {
6863:       this.canvas.style.cursor = 'crosshair';
6864:       return;
6865:     }
6866:     if (s.mode === 'place-text' || s.mode === 'place-symbol') { this.canvas.style.cursor = 'copy'; return; }
6867:     this.canvas.style.cursor = 'default';
6868:   }
6869: 
6870:   // Resolves the drawing pointer for the current size option and background
6871:   // theme. Prefers the pre-built static variant; if that is missing it falls
6872:   // back to a runtime data URL, and finally to the original pointer.
6873:   getDrawingCursor() {
6874:     const size = Number(this.app.state.options?.pointerSize);
6875:     const valid = [32, 64, 128].includes(size) ? size : 128;
6876:     const invert = this.app.state.canvasTheme === 'light';
6877:     const key = `${valid}|${invert}`;
6878:     const src = __cursorUrl(`cursor-${valid}${invert ? '-light' : ''}.png`);
6879:     let entry = this.cursorCache.get(key);
6880:     if (!entry) {
