// P8 ref extract: draw-bgimage-scalepoints — lines 9820-9910 of 00) sketch standalone.html

9820:       ctx.fill();
9821:       ctx.stroke();
9822:     }
9823: 
9824:     ctx.restore();
9825:   }
9826: 
9827:   drawBgImage() {
9828:     const bg = this.app.bgImage;
9829:     if (!bg || !bg.img || !bg.visible) return;
9830: 
9831:     const ctx = this.ctx;
9832:     const s = this.app.state;
9833:     const PPF = window.Geometry.PIXELS_PER_FOOT;
9834: 
9835:     // Convert image center (world coords) to screen coords
9836:     const screen = window.Geometry.toScreen({ x: bg.x, y: bg.y }, s.pan, s.zoom);
9837:     const imgW = bg.img.width * bg.scale * s.zoom;
9838:     const imgH = bg.img.height * bg.scale * s.zoom;
9839: 
9840:     ctx.save();
9841:     ctx.globalAlpha = bg.opacity;
9842:     ctx.translate(screen.x, screen.y);
9843: 
9844:     // Rotation
9845:     if (bg.rotation) {
9846:       ctx.rotate(bg.rotation * Math.PI / 180);
9847:     }
9848: 
9849:     // Flip
9850:     const sx = bg.flipH ? -1 : 1;
9851:     const sy = bg.flipV ? -1 : 1;
9852:     ctx.scale(sx, sy);
9853: 
9854:     ctx.drawImage(bg.img, -imgW / 2, -imgH / 2, imgW, imgH);
9855:     ctx.restore();
9856:   }
9857: 
9858:   drawScalePoints() {
9859:     const bg = this.app.bgImage;
9860:     if (!bg || (!bg.scaleMode && !bg.orientMode)) return;
9861: 
9862:     const ctx = this.ctx;
9863:     const s = this.app.state;
9864: 
9865:     const drawMarker = (worldPt, label) => {
9866:       const screen = window.Geometry.toScreen(worldPt, s.pan, s.zoom);
9867:       ctx.save();
9868:       ctx.fillStyle = '#F6AD55';
9869:       ctx.strokeStyle = '#F6AD55';
9870:       ctx.lineWidth = 2;
9871: 
9872:       // Crosshair
9873:       ctx.beginPath();
9874:       ctx.moveTo(screen.x - 10, screen.y);
9875:       ctx.lineTo(screen.x + 10, screen.y);
9876:       ctx.moveTo(screen.x, screen.y - 10);
9877:       ctx.lineTo(screen.x, screen.y + 10);
9878:       ctx.stroke();
9879: 
9880:       // Circle
9881:       ctx.beginPath();
9882:       ctx.arc(screen.x, screen.y, 6, 0, Math.PI * 2);
9883:       ctx.stroke();
9884: 
9885:       // Label
9886:       ctx.font = '11px Arial, sans-serif';
9887:       ctx.textAlign = 'left';
9888:       ctx.fillText(label, screen.x + 12, screen.y - 4);
9889:       ctx.restore();
9890:     };
9891: 
9892:     const p1 = bg.scaleMode ? bg.scaleP1 : bg.orientP1;
9893:     const p2 = bg.scaleMode ? bg.scaleP2 : bg.orientP2;
9894:     if (p1) drawMarker(p1, 'Point 1');
9895:     if (p2) drawMarker(p2, 'Point 2');
9896: 
9897:     // Draw line between points
9898:     if (p1 && p2) {
9899:       const s1 = window.Geometry.toScreen(p1, s.pan, s.zoom);
9900:       const s2 = window.Geometry.toScreen(p2, s.pan, s.zoom);
9901:       ctx.save();
9902:       ctx.setLineDash([4, 4]);
9903:       ctx.strokeStyle = '#F6AD55';
9904:       ctx.lineWidth = 1.5;
9905:       ctx.beginPath();
9906:       ctx.moveTo(s1.x, s1.y);
9907:       ctx.lineTo(s2.x, s2.y);
9908:       ctx.stroke();
9909:       ctx.setLineDash([]);
9910:       ctx.restore();
