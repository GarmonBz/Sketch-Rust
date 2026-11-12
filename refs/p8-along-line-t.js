// P8 ref extract: along-line-t — lines 18030-18070 of 00) sketch standalone.html

18030:     // The intersection must lie on BOTH segments. Without this, an extended
18031:     // (extrapolated) crossing registers as a junction, and the wall healer
18032:     // fabricates phantom walls / truncates real ones mid-draw.
18033:     const EPS = 1e-6;
18034:     if (t < -EPS || t > 1 + EPS || u < -EPS || u > 1 + EPS) return null;
18035:     // Two segments that only share a corner are not a crossing.
18036:     if ((t <= EPS || t >= 1 - EPS) && (u <= EPS || u >= 1 - EPS)) return null;
18037:     return { x: a1.x + t * r1x, y: a1.y + t * r1y };
18038:   }
18039: 
18040:   alongLineT(line, point) {
18041:     const dx = line.end.x - line.start.x, dy = line.end.y - line.start.y;
18042:     return ((point.x - line.start.x) * dx + (point.y - line.start.y) * dy) / (dx * dx + dy * dy);
18043:   }
18044: 
18045:   // Walk a closed loop through the wall graph starting along firstId from
18046:   // startKey. At junction nodes the straightest unused continuation is tried
18047:   // first, and a wall may be passed straight through where a junction sits
18048:   // on its body, so T-junctions and healed crossings traverse correctly.
18049:   linePassesThrough(point, line) {
18050:     return this.pointKey(point) !== this.pointKey(line.start) &&
18051:       this.pointKey(point) !== this.pointKey(line.end);
18052:   }
18053: 
18054:   exitKeyFrom(nodePoint, fromPoint, line) {
18055:     const pointKey = point => this.pointKey(point);
18056:     const dirX = nodePoint.x - fromPoint.x, dirY = nodePoint.y - fromPoint.y;
18057:     const dotStart = dirX * (line.start.x - nodePoint.x) + dirY * (line.start.y - nodePoint.y);
18058:     const dotEnd = dirX * (line.end.x - nodePoint.x) + dirY * (line.end.y - nodePoint.y);
18059:     return dotStart > dotEnd ? pointKey(line.start) : pointKey(line.end);
18060:   }
18061: 
18062:   walkClosedLoop(nodes, byId, startKey, firstId) {
18063:     const pointKey = point => this.pointKey(point);
18064:     const deviation = (fromPoint, nodePoint, line) => {
18065:       const inX = nodePoint.x - fromPoint.x, inY = nodePoint.y - fromPoint.y;
18066:       const outKey = this.exitKeyFrom(nodePoint, fromPoint, line);
18067:       const out = pointKey(line.start) === outKey ? line.start : line.end;
18068:       const outX = out.x - nodePoint.x, outY = out.y - nodePoint.y;
18069:       const inLen = Math.hypot(inX, inY) || 1, outLen = Math.hypot(outX, outY) || 1;
18070:       return Math.abs((inX / inLen) * (outY / outLen) - (inY / inLen) * (outX / outLen));
