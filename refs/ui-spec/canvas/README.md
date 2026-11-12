# Canvas renders of the original (forced)

The original paints its canvas on requestAnimationFrame, which never fires in
the hidden browser tab the references are captured from, so most gallery
captures showed a blank canvas. These PNGs are the original's own canvas
(`toDataURL`) after `app.canvas.resize(); app.render()` at 1400x860:
default (1047x703), collapsed dock (1329x703), dock at 400px (959x703); pan
stays at its initial (523.5, 351.5). `../gallery/*.png` had their canvas
regions composited from these wherever the canvas showed through (white or
stale-grid pixels; overlays untouched).
