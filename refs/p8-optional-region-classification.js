// P8 ref extract: optional region classification script — lines 22488 onward

22488: /* Optional region classification. No drawing completion event opens this UI. */
22489: (() => {
22490:   'use strict';
22491:   function boot() {
22492:     const app = window.app;
22493:     if (!app || app.areaWorkflow) return;
22494:     const panel = document.querySelector('#panel-define-area > .panel-body');
22495:     const canvas = document.getElementById('sketchCanvas');
22496:     if (!panel || !canvas) return;
22497:     const esc = value => app.ui.escapeHTML(value);
22498:     // The code library and the legacy area operations move into a sheet over the
22499:     // panel instead of sitting inline under the list: the canvas chooser is the
22500:     // definition surface now. The controls keep their IDs, handlers, state and
22501:     // their #panel-define-area ancestry; only their place in the flow changes.
22502:     const advanced = document.createElement('div');
22503:     advanced.id = 'area-advanced-tools';
22504:     advanced.hidden = true;
22505:     advanced.setAttribute('role', 'dialog');
22506:     advanced.setAttribute('aria-modal', 'true');
22507:     advanced.setAttribute('aria-label', 'Edit areas');
22508:     const advancedHead = document.createElement('div');
22509:     advancedHead.className = 'area-advanced-head';
22510:     advancedHead.innerHTML = '<strong>Edit areas</strong>';
22511:     const advancedClose = document.createElement('button');
22512:     advancedClose.type = 'button';
22513:     advancedClose.className = 'area-advanced-close';
22514:     advancedClose.setAttribute('aria-label', 'Close the area editor');
22515:     advancedClose.innerHTML = '&times;';
22516:     advancedHead.append(advancedClose);
22517:     const advancedBody = document.createElement('div');
22518:     advancedBody.className = 'area-advanced-body';
22519:     [...panel.children].forEach(node => advancedBody.append(node));
22520:     advanced.append(advancedHead, advancedBody);
22521:     buildEditSheet(advancedBody);
22522:     const surface = document.createElement('section');
22523:     surface.className = 'area-workflow';
22524:     surface.innerHTML = `<div class="area-workflow-note" id="area-workflow-status" role="status"></div>
22525:       <div class="area-palette" id="area-palette" role="listbox" aria-label="Area types"></div>
22526:       <div class="area-workflow-foot">
22527:       <div class="area-workflow-links"><button id="area-new" type="button">New area</button><span aria-hidden="true">·</span><button id="area-find-undefined" type="button" aria-pressed="false">Find undefined</button><span aria-hidden="true">·</span><button id="area-open-library" type="button">Edit areas…</button></div></div>`;
22528:     panel.append(surface, advanced);
22529:     document.querySelector('[data-sidebar-target="panel-define-area"] span').textContent = 'Areas';
22530:     document.getElementById('tool-define-area').title = 'Define Areas: click inside an outline';
22531:     const overlay = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
22532:     overlay.classList.add('area-workflow-overlay');
22533:     overlay.setAttribute('aria-hidden', 'true');
22534:     canvas.parentElement.append(overlay);
22535:     const labels = { GLA:'Living space', GBA:'Building area', GAR:'Garage / carport', BSMT:'Basement', 'P/P':'Outdoor', OTH:'Storage', LAND:'Land', SITE:'Site', NCA:'Non-calculated', UND:'Undefined' };
22536:     let chooser = null, target = null, targetPage = null, origin = null;
22537:     let remembered = null, repeat = false, findUndefined = false, previousDraw = 'draw-exterior';
22538:     let priorSelection = null, refreshQueued = false;
22539:     let armedCode = null, dragCode = null, dragGhost = null, hoverPoly = null, provisional = null, hoverRegion = null;
22540:     let paletteSignature = '';
22541:     const collapsedGroups = new Set();
22542:     const query = selector => surface.querySelector(selector);
22543:     function notify(message) { query('#area-workflow-status').textContent = message; }
22544:     let advancedOrigin = null;
22545:     function openAdvanced(trigger) {
22546:       close();
22547:       advancedOrigin = trigger || null;
22548:       advanced.hidden = false;
22549:       advancedClose.focus();
22550:     }
22551:     function closeAdvanced(restoreFocus = false) {
22552:       if (advanced.hidden) return;
22553:       advanced.hidden = true;
22554:       if (restoreFocus && advancedOrigin?.isConnected) advancedOrigin.focus();
22555:       advancedOrigin = null;
22556:     }
22557:     /* Re-nest the moved controls into three plain sections. Anything that
22558:        cannot be found is skipped, so a future bundle that drops a control
22559:        degrades instead of throwing. */
22560:     function buildEditSheet(host) {
22561:       const grab = selector => host.querySelector(selector);
22562:       const section = (title, hint) => {
22563:         const el = document.createElement('section');
22564:         el.className = 'area-edit-group';
22565:         el.innerHTML = '<h3 class="area-edit-title">' + title + '</h3>' +
22566:           (hint ? '<p class="area-edit-hint">' + hint + '</p>' : '');
22567:         return el;
22568:       };
22569:       const ops = section('Area operations', 'Acts on the areas selected on the canvas. Redefine uses the type armed in the palette.');
22570:       const opRow = document.createElement('div');
22571:       opRow.className = 'area-edit-actions';
22572:       for (const id of ['btn-redefine-area', 'btn-clone-area', 'btn-reopen-area', 'btn-detect-regions']) {
22573:         const btn = grab('#' + id);
22574:         if (btn) opRow.append(btn);
22575:       }
22576:       ops.append(opRow);
22577: 
22578:       const labels = section('Label defaults', 'Applied to areas you define from now on.');
22579:       const post = grab('.auto-post');
22580:       if (post) {
22581:         post.querySelector('.auto-post-title')?.remove();
22582:         labels.append(post);
22583:       }
22584: 
22585:       // The code library is retired: the palette browses types, and its
22586:       // right-click menu covers New, Edit, Delete and assigning to a selection.
22587:       host.replaceChildren(ops, labels);
22588:     }
22589: 
22590:     function editable(poly) { return app.isEntityVisible(poly) && !app.isEntityLocked(poly); }
22591: 
22592:     /* ---- Area-type palette -------------------------------------------------
22593:        Every record in app.areaCodes carries the colour it paints on canvas, so
22594:        the palette is that colour list. Picking and applying are one gesture:
22595:        drag a swatch onto an outline, or arm a swatch and click outlines. */
22596:     function codeColor(entry) { return entry?.color || entry?.fillColor || '#858585'; }
22597:     function groupLabel(group) { return labels[group] || group; }
22598:     function usageCount(code) { return app.state.polygons.filter(p => p.type === code).length; }
22599:     function paletteGroups() {
22600:       const seen = [];
22601:       for (const entry of app.areaCodes) if (entry.group !== 'UND' && !seen.includes(entry.group)) seen.push(entry.group);
22602:       return seen;
22603:     }
22604:     function assignTo(poly, code) {
22605:       if (!poly || !app.state.polygons.includes(poly)) return false;
22606:       if (!editable(poly)) { notify('Show and unlock this area before changing its definition.'); return false; }
22607:       const entry = app.areaCodes.find(c => c.code === code);
22608:       // Same transaction the Apply path uses: one undo entry, labels left to defaults.
22609:       app.defineAreaTarget(poly, code, entry?.name || code);
22610:       return true;
22611:     }
22612:     function setArmed(code) {
22613:       const next = armedCode === code ? null : code;
22614:       armedCode = next;
22615:       // This is the state the code tree used to own. Define First reads it when
22616:       // a new polygon is drawn, and Redefine reads it for the selection.
22617:       const entry = next ? app.areaCodes.find(c => c.code === next) : null;
22618:       app.state.selectedAreaType = next;
22619:       app.state.selectedAreaName = entry ? (entry.name || next) : null;
22620:       app.ui.updateAreaActionStates?.();
22621:       if (next) activate();
22622:       notify(next ? 'Click an outline to assign ' + (app.areaCodes.find(c => c.code === next)?.name || next) + '. Escape stops.' : '');
22623:       paletteSignature = '';
22624:       schedule();
22625:     }
22626:     /* Context menu for a type. Options first, editing last, so the common
22627:        cases do not require opening a dialog at all. */
22628:     function showTypeMenu(entry, clientX, clientY) {
22629:       const selectedIds = [...app.state.selectedPolyIds];
22630:       const usedBy = app.state.polygons.filter(p => p.type === entry.code);
22631:       const label = entry.name || entry.code;
22632:       const items = [];
22633:       if (selectedIds.length) items.push({
22634:         action: 'assign',
22635:         icon: entry.code === 'UND' ? 'ph-eraser' : 'ph-paint-bucket',
22636:         label: (entry.code === 'UND' ? 'Clear ' : 'Assign to ') + selectedIds.length +
22637:           (selectedIds.length === 1 ? ' selected area' : ' selected areas'),
22638:         run: () => {
22639:           // Re-resolve each polygon: defineAreaTarget can swap state.polygons
22640:           // for a fresh array, which would strand a captured object list.
22641:           let done = 0;
22642:           for (const id of selectedIds) {
22643:             const poly = app.state.polygons.find(p => p.id === id);
22644:             if (poly && assignTo(poly, entry.code)) done++;
22645:           }
22646:           notify(done + (done === 1 ? ' area set to ' : ' areas set to ') + label + '.');
22647:           schedule();
22648:         }
22649:       });
22650:       items.push({
22651:         action: 'arm', icon: 'ph-cursor-click',
22652:         label: armedCode === entry.code ? 'Stop assigning' : 'Assign by clicking outlines',
22653:         run: () => setArmed(entry.code)
22654:       });
22655:       if (usedBy.length) items.push({
22656:         action: 'select', icon: 'ph-selection-all',
22657:         label: 'Select the ' + usedBy.length + (usedBy.length === 1 ? ' area' : ' areas') + ' using it',
22658:         run: () => {
22659:           app.state.selectedPolyIds = new Set(usedBy.map(p => p.id));
22660:           app.ui.updateAreaActionStates();
22661:           app.render();
22662:           notify('Selected ' + usedBy.length + (usedBy.length === 1 ? ' area' : ' areas') + ' using ' + label + '.');
22663:         }
22664:       });
22665:       items.push({ separator: true });
22666:       items.push({ action: 'edit', icon: 'ph-pencil-simple', label: 'Edit type...',
22667:         run: () => app.ui.showAreaDefinitionEditor(entry.code) });
22668:       items.push({ action: 'new', icon: 'ph-plus', label: 'New type...',
22669:         run: () => app.ui.showAddAreaCode() });
22670:       items.push({ action: 'delete', icon: 'ph-trash', label: 'Delete type...', run: async () => {
22671:         // UND is the sentinel the app uses for an unclassified outline; removing
22672:         // it would leave nothing to fall back to.
22673:         if (entry.code === 'UND') { notify('Undefined is how unclassified outlines are marked. It cannot be deleted.'); return; }
22674:         const inUse = app.state.polygons.filter(p => p.type === entry.code).map(p => p.id);
22675:         const ok = await app.ui.confirmDialog(inUse.length
22676:           ? 'Delete ' + label + '? ' + inUse.length + (inUse.length === 1 ? ' area using it becomes unclassified.' : ' areas using it become unclassified.')
22677:           : 'Delete ' + label + ' from the code library?');
22678:         if (!ok) return;
22679:         for (const id of inUse) {
22680:           const poly = app.state.polygons.find(p => p.id === id);
22681:           if (poly) app.defineAreaTarget(poly, 'UND', 'Undefined Area');
22682:         }
22683:         if (armedCode === entry.code) setArmed(entry.code);
22684:         app.setAreaCodes(app.areaCodes.filter(c => c.code !== entry.code));
22685:         try { localStorage.setItem('sketch.areaCodes', JSON.stringify(app.areaCodes)); } catch (_) { /* unavailable */ }
22686:         app.ui.populateAreaCodes(document.getElementById('area-code-search')?.value || '');
22687:         paletteSignature = '';
22688:         notify(label + ' deleted from the library.');
22689:         app.render();
22690:         schedule();
22691:       } });
22692:       const anchor = { getBoundingClientRect: () => ({ left: clientX, bottom: clientY }) };
22693:       app.ui.showCommandPopup(anchor, items, 'sketch-context-menu');
22694:     }
22695: 
22696:     function makeTile(entry, isClear) {
22697:       const tile = document.createElement('button');
22698:       tile.type = 'button';
22699:       tile.className = 'area-tile';
22700:       tile.dataset.code = entry.code;
22701:       tile.setAttribute('role', 'option');
22702:       tile.setAttribute('aria-selected', String(armedCode === entry.code));
22703:       const count = usageCount(entry.code);
22704:       tile.innerHTML = `<span class="area-tile-swatch" style="background:${esc(codeColor(entry))}"></span>` +
22705:         `<span class="area-tile-name">${esc(isClear ? 'Undefined · clear' : (entry.name || entry.code))}</span>` +
22706:         (count && !isClear ? `<span class="area-tile-used">${count}</span>` : '') +
22707:         `<span class="area-tile-code">${esc(entry.code)}</span>`;
22708:       tile.title = isClear
22709:         ? 'Drag onto an area to clear its classification. Right-click for options.'
22710:         : 'Drag onto an outline, or click to arm. Right-click for options.';
22711:       tile.addEventListener('pointerdown', startTileDrag);
22712:       tile.addEventListener('click', () => {
22713:         if (tile.dataset.pointerHandled) { delete tile.dataset.pointerHandled; return; }
22714:         setArmed(entry.code);
22715:       });
22716:       /* Right-click a type to edit the code itself - name, colours, factor -
22717:          without drilling into the edit sheet. */
22718:       tile.addEventListener('contextmenu', event => {
22719:         event.preventDefault();
22720:         event.stopPropagation();
22721:         showTypeMenu(entry, event.clientX, event.clientY);
22722:       });
22723:       tile.addEventListener('keydown', event => {
22724:         if (event.key !== 'Enter' && event.key !== ' ') return;
22725:         event.preventDefault();
22726:         setArmed(entry.code);
22727:       });
22728:       return tile;
22729:     }
22730:     function paintPalette() {
22731:       const host = query('#area-palette');
22732:       if (!host) return;
22733:       // A rebuild mid-drag would detach the tile holding the pointer capture.
22734:       if (dragCode) return;
22735:       const matches = app.areaCodes.filter(entry => entry.code !== 'UND');
22736:       const undefinedEntry = app.areaCodes.find(c => c.code === 'UND') || { code: 'UND', name: 'Undefined Area', color: '#E53E3E' };
22737:       const signature = JSON.stringify([[...collapsedGroups], armedCode,
22738:         matches.map(c => [c.code, c.name, codeColor(c), usageCount(c.code)])]);
22739:       if (signature === paletteSignature) return;
22740:       paletteSignature = signature;
22741:       const scroll = host.scrollTop;
22742:       host.replaceChildren();
22743:       // areaCodes is not group-contiguous (OTH sits either side of LAND/SITE),
22744:       // so collect each group rather than heading on adjacency.
22745:       for (const group of paletteGroups()) {
22746:         const inGroup = matches.filter(entry => entry.group === group);
22747:         if (!inGroup.length) continue;
22748:         const collapsed = collapsedGroups.has(group);
22749:         const head = document.createElement('button');
22750:         head.type = 'button';
22751:         head.className = 'area-tile-group';
22752:         head.dataset.group = group;
22753:         head.setAttribute('aria-expanded', String(!collapsed));
22754:         head.innerHTML = '<i class="ph ' + (collapsed ? 'ph-caret-right' : 'ph-caret-down') + '" aria-hidden="true"></i>' +
22755:           '<span class="area-group-name">' + esc(groupLabel(group)) + '</span>' +
22756:           '<span class="area-group-count">' + inGroup.length + '</span>';
22757:         head.onclick = () => {
22758:           if (collapsed) collapsedGroups.delete(group); else collapsedGroups.add(group);
22759:           paletteSignature = '';
22760:           paintPalette();
22761:         };
22762:         host.append(head);
22763:         if (!collapsed) for (const entry of inGroup) host.append(makeTile(entry, false));
22764:       }
22765:       if (!matches.length) {
22766:         const empty = document.createElement('div');
22767:         empty.className = 'area-palette-empty';
22768:         empty.textContent = 'No area types in the library.';
22769:         host.append(empty);
22770:       }
22771:       host.append(makeTile(undefinedEntry, true));
22772:       host.scrollTop = scroll;
22773:     }
22774: 
22775:     /* Drag targets: the canvas resolves through the same point-in-polygon test
22776:        the click path uses, so drag and click always agree about the outline. */
22777:     function polyAtClient(clientX, clientY) {
22778:       const rect = canvas.getBoundingClientRect();
22779:       if (clientX < rect.left || clientX > rect.right || clientY < rect.top || clientY > rect.bottom) return null;
22780:       const world = window.Geometry.toWorld({ x: clientX - rect.left, y: clientY - rect.top }, app.state.pan, app.state.zoom);
22781:       return app.state.polygons.filter(poly => editable(poly) && window.Geometry.pointInPolygon(world, window.Geometry.flattenRing(poly)))
22782:         .sort((a, b) => Math.abs(a.area ?? a.gross ?? 0) - Math.abs(b.area ?? b.gross ?? 0))[0] || null;
22783:     }
22784:     /* Drag targets resolve like a define click: an existing area, or the
22785:        enclosed wall region (a three-sided porch against the house) that a
22786:        drop materializes. */
22787:     function regionAtClient(clientX, clientY) {
22788:       const rect = canvas.getBoundingClientRect();
22789:       if (clientX < rect.left || clientX > rect.right || clientY < rect.top || clientY > rect.bottom) return null;
22790:       const world = window.Geometry.toWorld({ x: clientX - rect.left, y: clientY - rect.top }, app.state.pan, app.state.zoom);
22791:       const region = app.resolveRegionAt(world);
22792:       if (region.poly && !editable(region.poly)) return null;
22793:       return region.poly || region.face ? { ...region, world } : null;
22794:     }
22795:     function startTileDrag(event) {
22796:       if (event.button != null && event.button !== 0) return;
22797:       const tile = event.currentTarget, code = tile.dataset.code;
22798:       const startX = event.clientX, startY = event.clientY;
22799:       let moved = false;
22800:       try { tile.setPointerCapture(event.pointerId); } catch (_) { /* pointer may be gone */ }
22801:       const move = moveEvent => {
22802:         if (!moved && Math.abs(moveEvent.clientX - startX) + Math.abs(moveEvent.clientY - startY) < 5) return;
22803:         if (!moved) {
22804:           moved = true; dragCode = code;
22805:           const entry = app.areaCodes.find(c => c.code === code) || { code, name: code };
22806:           dragGhost = document.createElement('div');
22807:           dragGhost.className = 'area-drag-ghost';
22808:           dragGhost.innerHTML = `<i style="background:${esc(codeColor(entry))}"></i>${esc(entry.name || code)} <small>${esc(code)}</small>`;
22809:           document.body.append(dragGhost);
22810:           document.body.classList.add('area-dragging');
22811:         }
22812:         dragGhost.style.left = moveEvent.clientX + 'px';
22813:         dragGhost.style.top = moveEvent.clientY + 'px';
22814:         const over = regionAtClient(moveEvent.clientX, moveEvent.clientY);
22815:         const overPoly = over ? (over.poly || over.face.points) : null;
22816:         if (overPoly !== hoverPoly) { hoverPoly = overPoly; paintOverlay(); }
22817:       };
22818:       const finish = upEvent => {
22819:         try { tile.releasePointerCapture(event.pointerId); } catch (_) { /* already released */ }
22820:         tile.removeEventListener('pointermove', move);
22821:         tile.removeEventListener('pointerup', finish);
22822:         tile.removeEventListener('pointercancel', finish);
22823:         document.body.classList.remove('area-dragging');
22824:         if (dragGhost) { dragGhost.remove(); dragGhost = null; }
22825:         const wasDragging = moved;
22826:         dragCode = null; hoverPoly = null;
22827:         if (!wasDragging) { tile.dataset.pointerHandled = '1'; setArmed(code); return; }
22828:         const over = regionAtClient(upEvent.clientX, upEvent.clientY);
22829:         const target = over ? (over.poly || app.regionAreaAt(over.world).poly) : null;
22830:         if (target && assignTo(target, code)) {
22831:           notify(code === 'UND' ? 'Classification cleared.' : 'Area defined. Drag another type, or click one to keep assigning.');
22832:         }
22833:         paintOverlay();
22834:         schedule();
22835:       };
22836:       tile.addEventListener('pointermove', move);
22837:       tile.addEventListener('pointerup', finish);
22838:       tile.addEventListener('pointercancel', finish);
22839:     }
22840: 
22841:     /* Canvas overlay: undefined-region highlights plus the live drop preview. */
22842:     function paintOverlay() {
22843:       overlay.replaceChildren();
22844:       overlay.setAttribute('viewBox', `0 0 ${canvas.clientWidth} ${canvas.clientHeight}`);
22845:       const ring = (poly, stroke, fill, dash) => {
22846:         const shape = document.createElementNS('http://www.w3.org/2000/svg', 'polygon');
22847:         shape.setAttribute('points', window.Geometry.flattenRing(poly).map(pt => {
22848:           const screen = window.Geometry.toScreen(pt, app.state.pan, app.state.zoom);
22849:           return `${screen.x},${screen.y}`;
22850:         }).join(' '));
22851:         shape.setAttribute('fill', fill);
22852:         shape.setAttribute('stroke', stroke);
22853:         shape.setAttribute('stroke-width', '2');
22854:         if (dash) shape.setAttribute('stroke-dasharray', dash);
22855:         overlay.append(shape);
22856:       };
22857:       if (findUndefined) for (const poly of app.state.polygons.filter(p => p.type === 'UND' && app.isEntityVisible(p))) {
22858:         ring(poly, 'var(--bg-active)', 'rgba(0,122,204,.08)', '6 4');
22859:       }
22860:       if (dragCode && hoverPoly) {
22861:         const color = codeColor(app.areaCodes.find(c => c.code === dragCode));
22862:         ring(hoverPoly, color, color + '33');
22863:       }
22864:       /* Define-mode hover: shade the region a click would define
22865:          and show its measured area at the label anchor. */
22866:       if (hoverRegion && !dragCode && !chooser && app.state.mode === 'define-area') {
22867:         const points = hoverRegion.poly ? window.Geometry.flattenRing(hoverRegion.poly) : hoverRegion.face.points;
22868:         const area = hoverRegion.poly ? Math.abs(hoverRegion.poly.gross ?? hoverRegion.poly.area ?? 0) : hoverRegion.face.area;
22869:         ring(points, '#6366F1', 'rgba(99,102,241,.16)');
22870:         const anchor = window.Geometry.toScreen(window.Geometry.areaLabelAnchor(points), app.state.pan, app.state.zoom);
22871:         const text = document.createElementNS('http://www.w3.org/2000/svg', 'text');
22872:         text.setAttribute('x', String(anchor.x));
22873:         text.setAttribute('y', String(anchor.y));
22874:         text.setAttribute('text-anchor', 'middle');
22875:         text.setAttribute('dominant-baseline', 'middle');
22876:         text.setAttribute('fill', '#4F46E5');
22877:         text.setAttribute('font-size', '15');
22878:         text.setAttribute('font-weight', '600');
22879:         text.textContent = area.toFixed(1);
22880:         overlay.append(text);
22881:       }
22882:     }
22883:     function trackHover(event) {
22884:       let next = null;
22885:       if (app.state.mode === 'define-area' && !chooser && !dragCode) {
22886:         const rect = canvas.getBoundingClientRect();
22887:         const world = window.Geometry.toWorld({ x: event.clientX - rect.left, y: event.clientY - rect.top }, app.state.pan, app.state.zoom);
22888:         const region = app.resolveRegionAt(world);
22889:         if (region.face || (region.poly && editable(region.poly))) next = region;
22890:       }
22891:       const same = next === hoverRegion || (next && hoverRegion && next.poly === hoverRegion.poly && next.face === hoverRegion.face);
22892:       if (same) return;
22893:       hoverRegion = next;
22894:       schedule();
22895:     }
22896:     canvas.addEventListener('pointermove', trackHover);
22897:     canvas.addEventListener('pointerleave', () => { if (hoverRegion) { hoverRegion = null; schedule(); } });
22898:     function close(restoreFocus = false) {
22899:       if (!chooser) return;
22900:       // A region materialized only to open this chooser is removed again when
22901:       // the chooser is dismissed without a definition.
22902:       const discard = provisional && provisional === target && app.state.polygons.includes(provisional) &&
22903:         provisional.type === 'UND';
22904:       provisional = null;
22905:       chooser.remove(); chooser = null; target = null; targetPage = null;
22906:       if (priorSelection) app.state.selectedPolyIds = new Set(priorSelection.filter(id => app.state.polygons.some(p => p.id === id)));
22907:       priorSelection = null;
22908:       app.ui.updateAreaActionStates(); app.render();
22909:       if (discard) app.undo();
22910:       if (restoreFocus && origin?.isConnected) origin.focus();
22911:     }
22912:     function activate() {
22913:       close(); closeAdvanced();
22914:       if (app.state.mode === 'draw-exterior' || app.state.mode === 'draw-interior') previousDraw = app.state.mode;
22915:       app.setMode('define-area');
22916:       app.ui.showPanel('panel-define-area');
22917:       notify('Click inside a closed outline. Escape returns to drawing.');
22918:     }
22919:     function beginDrawing() { close(); closeAdvanced(); disarm(); app.setMode(previousDraw); notify('Drawing mode. Area definition remains optional.'); }
22920:     function disarm() {
22921:       if (!armedCode) return;
22922:       armedCode = null;
22923:       paletteSignature = '';
22924:       schedule();
22925:     }
22926:     function open(poly, point, trigger) {
22927:       close();
22928:       if (!editable(poly)) { notify('Show and unlock this area before editing its definition.'); return; }
22929:       target = poly; targetPage = app.state.polygons; origin = trigger || null;
22930:       priorSelection = [...app.state.selectedPolyIds]; app.state.selectedPolyIds = new Set([poly.id]);
22931:       app.ui.updateAreaActionStates(); app.render();
22932:       const post = { ...app.state.autoPost, dims:app.state.dimensionsEnabled, ...poly.autoPost };
22933:       const known = app.areaCodes.find(c => c.code === poly.type);
22934:       let code = poly.type !== 'UND' ? poly.type : (repeat ? remembered : null);
22935:       let group = app.areaCodes.find(c => c.code === code)?.group || 'GLA';
22936:       chooser = document.createElement('form'); chooser.className = 'area-chooser'; chooser.id = 'area-definition-chooser';
22937:       chooser.setAttribute('role','dialog'); chooser.setAttribute('aria-label','Define selected area');
22938:       chooser.innerHTML = `<div class="area-chooser-head"><strong>${poly.type === 'UND' ? 'Define this area' : 'Edit definition'}</strong><span>${Number((poly.type === 'UND' ? poly.gross ?? poly.area : poly.net ?? poly.area) ?? 0).toFixed(1)} sf</span></div>
22939:         <label class="area-field"><span>What is this space?</span><input id="area-definition-search" type="search" placeholder="Find any type or custom code..."></label>
22940:         <div class="area-chooser-types"></div><label class="area-field"><span>Classification</span><select id="area-definition-code" required></select></label>
22941:         <label class="area-field"><span>Name · optional</span><input id="area-definition-name" value="${esc(poly.type === 'UND' ? '' : poly.name || '')}" placeholder="Custom area name"></label>
22942:         <details><summary>Label settings</summary><div class="area-chooser-checks">${[['code','Code'],['name','Name'],['dims','Dimensions'],['calcs','Area'],['base','Base'],['factor','Factor']].map(([key,name]) => `<label><input type="checkbox" name="post-${key}" ${post[key] ? 'checked' : ''}>${name}</label>`).join('')}</div>
22943:           <label class="area-field"><span>Area suffix</span><select name="suffix"></select></label></details>
22944:         <label class="area-chooser-repeat"><input type="checkbox" name="repeat" ${repeat ? 'checked' : ''}>Keep assigning this type</label>
22945:         <div class="area-chooser-actions"><button type="button" data-cancel>Cancel</button><button class="area-primary" type="submit">${poly.type === 'UND' ? 'Define area' : 'Save changes'}</button></div>`;
22946:       const form = chooser;
22947:       const search = form.querySelector('#area-definition-search');
22948:       const select = form.querySelector('#area-definition-code');
22949:       const suffix = form.elements.namedItem('suffix');
22950:       for (const option of document.getElementById('autopost-suffix').options) suffix.add(new Option(option.textContent,option.value));
22951:       suffix.value = post.suffix || 'sf';
22952:       const codes = () => {
22953:         const values = app.areaCodes.slice();
22954:         if (poly.type !== 'UND' && !known) values.push({code:poly.type,name:poly.name || poly.type,group:'Custom',description:'Existing classification'});
22955:         return values;
22956:       };
22957:       function drawChoices() {
22958:         const term = search.value.trim().toLowerCase();
22959:         const values = codes();
22960:         const groups = [...new Set(values.map(c=>c.group))];
22961:         const host = form.querySelector('.area-chooser-types'); host.replaceChildren();
22962:         for (const key of groups) {
22963:           if (term && !values.some(c=>c.group===key && `${c.code} ${c.name} ${c.description} ${labels[key] || key}`.toLowerCase().includes(term))) continue;
22964:           const button = document.createElement('button'); button.type='button'; button.textContent=labels[key] || key;
22965:           button.setAttribute('aria-pressed',String(key===group));
22966:           button.onclick=()=>{group=key;search.value='';code=null;drawChoices();}; host.append(button);
22967:         }
22968:         const matches=values.filter(c=>term ? `${c.code} ${c.name} ${c.description} ${labels[c.group] || c.group}`.toLowerCase().includes(term) : c.group===group);
22969:         select.replaceChildren(new Option('Choose a classification...', ''));
22970:         for(const item of matches) select.add(new Option(`${item.name} (${item.code})`,item.code));
22971:         if(matches.some(c=>c.code===code)) select.value=code; else select.value='';
22972:         form.querySelector('[type=submit]').disabled=!select.value;
22973:       }
22974:       search.oninput=drawChoices;
22975:       select.onchange=()=>{code=select.value;form.querySelector('[type=submit]').disabled=!code;};
22976:       form.querySelector('[data-cancel]').onclick=()=>{close(true);notify('Canceled. Drawing unchanged.');};
22977:       form.onsubmit=event=>{
22978:         event.preventDefault();
22979:         if (targetPage!==app.state.polygons || !app.state.polygons.includes(poly) || !editable(poly)) {close(true);notify('Area changed. Select it again.');return;}
22980:         const definition=codes().find(c=>c.code===select.value);
22981:         if(!definition)return;
22982:         const autoPost={suffix:suffix.value};
22983:         for(const key of ['code','name','dims','calcs','base','factor'])autoPost[key]=form.elements.namedItem(`post-${key}`).checked;
22984:         const name=form.querySelector('#area-definition-name').value.trim() || definition.name;
22985:         repeat=form.elements.namedItem('repeat').checked;remembered=repeat ? definition.code : null;
22986:         // Reuse the core classification transaction; one undo includes labels.
22987:         app.defineAreaTarget(poly,definition.code,name,autoPost);
22988:         provisional=null;
22989:         close(true);notify('Area defined. Click another outline, or choose Draw.');
22990:       };
22991:       document.body.append(form);drawChoices();
22992:       const bounds=canvas.getBoundingClientRect();
22993:       form.style.left=Math.max(10,Math.min(window.innerWidth-310,(point?.x ?? bounds.right-310)+16))+'px';
22994:       form.style.top=Math.max(10,Math.min(window.innerHeight-form.offsetHeight-10,point?.y ?? bounds.top+20))+'px';
22995:       search.focus();
22996:     }
22997:     function click(worldPt) {
22998:       const region=app.regionAreaAt(worldPt);
22999:       if(!region.poly){notify('No closed area here. Use Detect Regions in advanced tools for enclosed wall networks.');return;}
23000:       if(armedCode){if(assignTo(region.poly,armedCode))notify('Assigned. Click another outline, or Escape to stop.');schedule();return;}
23001:       const screen=window.Geometry.toScreen(worldPt,app.state.pan,app.state.zoom),bounds=canvas.getBoundingClientRect();
23002:       open(region.poly,{x:bounds.left+screen.x,y:bounds.top+screen.y});
23003:       if(region.created && target===region.poly)provisional=region.poly;
23004:     }
23005:     function refresh() {
23006:       if(chooser && (targetPage!==app.state.polygons || !app.state.polygons.includes(target) || !editable(target)))close();
23007:       paintOverlay();
23008:       paintPalette();
23009:     }
23010:     function schedule(){if(refreshQueued)return;refreshQueued=true;requestAnimationFrame(()=>{refreshQueued=false;refresh();});}
23011:     app.areaWorkflow={activate,beginDrawing,click,close,open,polyAt:polyAtClient,refresh:schedule};
23012:     const render=app.render.bind(app);app.render=function(...args){const result=render(...args);schedule();return result;};
23013:     const setMode=app.setMode.bind(app);app.setMode=function(mode){if(mode!==app.state.mode)close();if(mode!=='define-area')disarm();return setMode(mode);};
23014:     query('#area-new').onclick = () => {
23015:       const undefinedOutlines = app.state.polygons.filter(p => p.type === 'UND' && app.isEntityVisible(p));
23016:       if (undefinedOutlines.length) {
23017:         activate();
23018:         findUndefined = true;
23019:         query('#area-find-undefined').setAttribute('aria-pressed', 'true');
23020:         notify(undefinedOutlines.length + (undefinedOutlines.length === 1 ? ' outline is' : ' outlines are') +
23021:           ' waiting. Click one to define it, or drag a type onto it.');
23022:       } else {
23023:         beginDrawing();
23024:         notify('Draw a closed outline, then drag a type onto it.');
23025:       }
23026:       schedule();
23027:     };
23028:     query('#area-open-library').onclick=event=>openAdvanced(event.currentTarget);
23029:     /* The Calcs tree is the areas list; right-click a row to edit that area. */
23030:     document.addEventListener('contextmenu', event => {
23031:       const node = event.target.closest?.('#calcs-tree [data-poly-id]');
23032:       if (!node) return;
23033:       const poly = app.state.polygons.find(p => String(p.id) === node.dataset.polyId);
23034:       if (!poly) return;
23035:       event.preventDefault();
23036:       open(poly, { x: event.clientX, y: event.clientY }, node);
23037:     });
23038:     const calcsTree = document.getElementById('calcs-tree');
23039:     if (calcsTree && !document.getElementById('area-calcs-hint')) {
23040:       const hint = document.createElement('div');
23041:       hint.id = 'area-calcs-hint';
23042:       hint.className = 'area-calcs-hint';
23043:       hint.textContent = 'Right-click an area here or on the drawing to edit its definition.';
23044:       calcsTree.insertAdjacentElement('afterend', hint);
23045:     }
23046:     advancedClose.onclick=()=>closeAdvanced(true);
23047:     query('#area-find-undefined').onclick=()=>{findUndefined=!findUndefined;query('#area-find-undefined').setAttribute('aria-pressed',String(findUndefined));refresh();notify(findUndefined?'Undefined outlines highlighted. Classification is optional.':'Undefined highlights hidden.');};
23048:     // Dialog input must not trigger drawing shortcuts. Escape exits the optional tool.
23049:     document.addEventListener('keydown',event=>{
23050:       if(event.key==='Escape' && !advanced.hidden){event.preventDefault();event.stopImmediatePropagation();closeAdvanced(true);return;}
23051:       if(event.key==='Escape' && (chooser || app.state.mode==='define-area')){event.preventDefault();event.stopImmediatePropagation();beginDrawing();return;}
23052:       if(chooser?.contains(event.target)){event.stopImmediatePropagation();if(event.key==='Tab'){const controls=[...chooser.querySelectorAll('button,input,select,summary')].filter(e=>!e.disabled && e.getClientRects().length);const first=controls[0],last=controls.at(-1);if(event.shiftKey && document.activeElement===first){event.preventDefault();last.focus();}else if(!event.shiftKey && document.activeElement===last){event.preventDefault();first.focus();}}}
23053:     },true);
23054:     new ResizeObserver(()=>{if(chooser)close();schedule();}).observe(canvas);
23055:     window.addEventListener('resize',()=>close());
23056:     refresh();
23057:   }
23058:   if(window.app)boot();else window.addEventListener('load',boot);
23059: })();
23060: 
23061: </script>
23062: <!-- /AREA DEFINITION WORKFLOW -->
23063: </body>
23064: </html>
