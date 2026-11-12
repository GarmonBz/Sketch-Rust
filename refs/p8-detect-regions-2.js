// P8 ref extract: detect-regions-2 — lines 17640-17800 of 00) sketch standalone.html

17640:     if (areaLayer && /^Area( \d+)?$/.test(areaLayer.name)) {
17641:       areaLayer.name = this.uniqueLayerName(name || 'Area');
17642:       this.ui.updateLayersPanel?.();
17643:     }
17644:     this.recalcAutoSubtract();
17645:     this.ui.updateCalcsPanel();
17646:     this.render();
17647:   }
17648: 
17649:   // Auto-detect enclosed regions formed by the walls and create an undefined
17650:   // (UND) area for each one not already covered by an existing polygon. Runs
17651:   // face traversal over the wall graph; a single undo covers every created area.
17652:   detectRegions() {
17653:     const MIN_REGION_AREA = 4; // sq ft; drop sliver faces
17654:     const faces = window.Geometry.findEnclosedFaces(this.state.lines, MIN_REGION_AREA);
17655:     if (!faces.length) { this.ui.notify('No enclosed regions found.'); return; }
17656:     const existing = this.state.polygons;
17657:     const toCreate = faces.filter(face => {
17658:       const centroid = window.Geometry.polygonCentroid(face.points);
17659:       return !existing.some(poly =>
17660:         window.Geometry.pointInPolygon(centroid, window.Geometry.flattenRing(poly)));
17661:     });
17662:     if (!toCreate.length) { this.ui.notify('All enclosed regions are already defined.'); return; }
17663:     this.pushUndo('Detect Regions');
17664:     toCreate.forEach(face => {
17665:       const poly = {
17666:         points: face.points.map(point => ({ x: point.x, y: point.y })),
17667:         edges: (face.edges || []).map(edge => ({ lineId: edge.lineId, bowAngle: 0 })),
17668:         type: 'UND',
17669:         name: 'Undefined Area',
17670:         gross: face.area,
17671:         area: face.area,
17672:         factorOverride: null,
17673:         subtractMode: this.state.options?.autoSubtractOverlapped === false ? 'never' : 'auto',
17674:         subtractTargetId: null,
17675:         layerId: this.state.activeLayerId,
17676:         id: this.genId()
17677:       };
17678:       if (poly.edges.length !== poly.points.length) {
17679:         poly.edges = this.buildPolygonEdges(poly, this.state.lines);
17680:       }
17681:       this.state.polygons.push(poly);
17682:       this.assignAreaToDedicatedLayer(poly);
17683:     });
17684:     this.recalcAutoSubtract();
17685:     this.ui.updateCalcsPanel();
17686:     this.render();
17687:     this.ui.notify(`Detected ${toCreate.length} enclosed region${toCreate.length === 1 ? '' : 's'}.`);
17688:   }
17689: 
17690:   // Parametric room templates. Returns the unrotated outline points for the
17691:   // given shape config (anchored at the local origin, bottom-left).
17692:   getShapePoints(shape) {
17693:     const w = Math.max(0.1, Number(shape.width) || 0);
17694:     const d = Math.max(0.1, Number(shape.depth) || 0);
17695:     if (shape.kind === 'rectangle') {
17696:       return [{ x: 0, y: 0 }, { x: w, y: 0 }, { x: w, y: d }, { x: 0, y: d }];
17697:     }
17698:     // L-shape: rectangle with a corner notch cut from the top-right.
17699:     const cw = Math.min(w, Math.max(0, Number(shape.cutWidth) || 0));
17700:     const cd = Math.min(d, Math.max(0, Number(shape.cutDepth) || 0));
17701:     return [
17702:       { x: 0, y: 0 }, { x: w, y: 0 }, { x: w, y: d - cd },
17703:       { x: w - cw, y: d - cd }, { x: w - cw, y: d }, { x: 0, y: d }
17704:     ];
17705:   }
17706: 
17707:   // Anchor the pending room template at a world point, creating a closed
17708:   // exterior-wall loop with shared endpoints and (optionally) an area polygon.
17709:   placeShapeAt(worldPt) {
17710:     const shape = this.state.pendingShape;
17711:     if (!shape) return;
17712:     const base = this.getShapePoints(shape);
17713:     const theta = (Number(shape.rotation) || 0) * Math.PI / 180;
17714:     const cos = Math.cos(theta), sin = Math.sin(theta);
17715:     let pts = base.map(p => ({ x: p.x * cos - p.y * sin, y: p.x * sin + p.y * cos }));
17716:     const minX = Math.min(...pts.map(p => p.x));
17717:     const minY = Math.min(...pts.map(p => p.y));
17718:     pts = pts.map(p => ({ x: worldPt.x + (p.x - minX), y: worldPt.y + (p.y - minY) }));
17719: 
17720:     this.pushUndo('Place Shape');
17721:     const style = this.getDefaultLineStyle('exterior');
17722:     const created = [];
17723:     for (let i = 0; i < pts.length; i++) {
17724:       const a = pts[i], b = pts[(i + 1) % pts.length];
17725:       const line = {
17726:         start: { ...a }, end: { ...b }, type: 'exterior', bowAngle: 0,
17727:         color: style.color, lineThickness: style.lineThickness,
17728:         layerId: this.state.activeLayerId, id: this.genId()
17729:       };
17730:       this.state.lines.push(line);
17731:       created.push(line);
17732:     }
17733:     if (shape.defineAsArea) {
17734:       const edges = created.map(line => ({ lineId: line.id, bowAngle: 0 }));
17735:       const area = window.Geometry.calculateArea(pts, edges);
17736:       const poly = {
17737:         points: pts.map(p => ({ ...p })), edges,
17738:         type: 'UND', name: 'Undefined Area', gross: area, area,
17739:         factorOverride: null,
17740:         subtractMode: this.state.options?.autoSubtractOverlapped === false ? 'never' : 'auto',
17741:         subtractTargetId: null, layerId: this.state.activeLayerId, id: this.genId()
17742:       };
17743:       this.state.polygons.push(poly);
17744:       this.assignAreaToDedicatedLayer(poly);
17745:     }
17746:     this.recalcAutoSubtract();
17747:     this.ui.updateCalcsPanel();
17748:     this.render();
17749:   }
17750: 
17751:   reopenSelectedArea() {
17752:     const poly = this.state.polygons.find(item => this.state.selectedPolyIds.has(item.id));
17753:     if (!poly || poly.points.length < 3) { this.ui.notify('Select an area to reopen.'); return; }
17754:     this.pushUndo('Reopen Area');
17755:     const openStart = { ...poly.points[0] };
17756:     const pathStart = { ...poly.points[1] };
17757:     const boundaryId = poly.edges && poly.edges[0] && poly.edges[0].lineId;
17758:     const edgeMatches = line => line.id === boundaryId ||
17759:       ((this.pointsEqual(line.start, openStart) && this.pointsEqual(line.end, pathStart)) ||
17760:        (this.pointsEqual(line.start, pathStart) && this.pointsEqual(line.end, openStart)));
17761:     const boundary = this.state.lines.find(edgeMatches);
17762:     if (boundary) this.state.lines = this.state.lines.filter(line => line.id !== boundary.id);
17763:     this.state.polygons = this.state.polygons.filter(item => item.id !== poly.id);
17764:     // Keep editing on the area's own layer so the re-closed area reuses it
17765:     // instead of stacking a fresh one on top.
17766:     const polyLayerId = this.getEntityLayerId(poly);
17767:     if (this.getLayerById(polyLayerId)) this.state.activeLayerId = polyLayerId;
17768:     this.recalculate();
17769:     this.clearSelection(false);
17770:     this.setMode('draw-exterior');
17771:     this.state.penDown = true;
17772:     this.state.currentPolyPoints = [
17773:       ...poly.points.slice(1).map(point => ({ ...point })),
17774:       { ...openStart }
17775:     ];
17776:     this.state.currentLineIds = (poly.edges || []).slice(1).map(edge => edge.lineId).filter(id => id != null);
17777:     this.state.cursor = { ...openStart };
17778:     this.state.pointerWorld = { ...openStart };
17779:     this.ui.updateHUD('', '', true);
17780:     this.ui.notify('Area reopened. Continue drawing from the open end, then close back to the starting point.');
17781:     this.ui.updateCalcsPanel();
17782:     this.render();
17783:   }
17784: 
17785:   getSelectedGeometryRotationModel() {
17786:     const key = point => `${point.x.toFixed(5)},${point.y.toFixed(5)}`;
17787:     const selectedKeys = new Set();
17788:     this.state.lines.forEach(line => {
17789:       if (this.state.selectedLineIds.has(line.id)) {
17790:         selectedKeys.add(key(line.start));
17791:         selectedKeys.add(key(line.end));
17792:       }
17793:     });
17794:     this.state.polygons.forEach(poly => {
17795:       if (this.state.selectedPolyIds.has(poly.id)) poly.points.forEach(point => selectedKeys.add(key(point)));
17796:     });
17797:     if (selectedKeys.size === 0) return null;
17798: 
17799:     const uniquePoints = new Map();
17800:     this.state.lines.forEach(line => [line.start, line.end].forEach(point => {
