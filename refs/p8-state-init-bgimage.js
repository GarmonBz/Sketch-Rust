// P8 ref extract: state-init-bgimage — lines 14480-14545 of 00) sketch standalone.html

14480:       dimensionChains: [],       // chained dimension strings (persisted per page)
14481:       selectedChainIds: new Set(),
14482:       dragState: null,
14483:       zoomFenceStart: null,
14484:       zoomFenceEnd: null,
14485:       curveMode: 'degree',
14486:       options: {
14487:         inputMode: 'pen-ink', promptOnDeletion: false, inkConfirmation: true,
14488:         unitSystem: 'imperial', decimalPrecision: 0.1, stepUnit: 'whole', dimensionOption: 8.5,
14489:         orthoAngle: 5, autoPlacement: true, stripTrailingZeroes: false, sameColorAsLine: false,
14490:         areaLabelSuffix: 'sf', alignmentLines: true, alignmentPoints: true, testCloseIndicators: true,
14491:         alignmentColor: '#808080', alignmentStyle: 'dash', gridBackground: '#FFFFFF',
14492:         majorGridColor: '#334155', minorGridColor: '#1f2937', minimumDimensionEnabled: false,
14493:         minimumDimension: 12, autoShade: false, autoShadePercent: 50, selectionHighlight: true,
14494:         distoDiscovery: 'automatic', distoRounding: 'none', distoDrawingMode: 'standard',
14495:         autoSaveEnabled: true, autoSaveInterval: 5, imageSize: '2048x1536', imageDepth: 24,
14496:         appendPageNumber: true, imageFormats: ['png'], disableVideoTutorial: false, autoHealFiles: true,
14497:         dimensionFont: 'Arial', dimensionFontSize: 12, dimensionBold: false, dimensionItalic: false, dimensionUnderline: false,
14498:         textFont: 'Arial', textFontSize: 12, textBold: false, textItalic: false, textUnderline: false, textColor: '#e2e8f0',
14499:         autoPostFont: 'Arial', autoPostFontSize: 16, autoPostBold: false, autoPostItalic: false, autoPostUnderline: false,
14500:         circleAreaCalcs: false, textBackground: '#111111', textBackgroundOpacity: 0,
14501:         exteriorColor: '#2B6CB0', interiorColor: '#63B3ED', exteriorWidth: 4, interiorWidth: 2,
14502:         wallSolids: false, // master switch for geometric wall solids; thicknesses below are inches and only apply when enabled
14503:         exteriorThickness: 6, interiorThickness: 4, // geometric wall thickness in inches; 0 = legacy single-line render
14504:         witnessBaseColor: '#ef4444', witnessInteriorColor: '#2563eb', witnessCurveColor: '#22c55e', labelOpacity: 50,
14505:         displayOnClose: false, defineFirst: false, autoSubtractOverlapped: true, quickShapeDefine: false, redefine: false,
14506:         panZoomDelete: true, saveImageWithSketch: false, imageQuality: 85, suppressMinimizeConfirmation: false,
14507:         pointerSize: 128
14508:       }
14509:     };
14510: 
14511:     // Entity ids are sketch-global and monotonic. They are deliberately not
14512:     // part of page data or undo snapshots.
14513:     this.nextId = 1;
14514:     this.areaCodes = window.AreaCodes
14515:       ? window.AreaCodes.cloneAreaCodes(window.AREA_CODES)
14516:       : JSON.parse(JSON.stringify(window.AREA_CODES || []));
14517:     this.areaTotals = { totalsByCode: {}, totalsByGroup: {}, totalGLA: 0, totalAllAreas: 0 };
14518:     this.breakdown = new window.AreaBreakdown({ codeTable: this.areaCodes });
14519:     this.debugAreaCalculations = false;
14520: 
14521:     // Multi-page support
14522:     this.pages = [this.getPageData()];
14523:     this.currentPage = 0;
14524:     // View-only underlay: another page ghosted beneath the current one.
14525:     // Never persisted, never part of undo snapshots.
14526:     this.underlayPageIndex = null;
14527:     this.fileName = 'Untitled';
14528:     this.ensureLayers();
14529: 
14530:     // Background image (photometrics)
14531:     this.bgImage = {
14532:       img: null,           // HTMLImageElement
14533:       visible: false,
14534:       x: 0, y: 0,         // world position (center)
14535:       scale: 1.0,          // image scale (px per world unit)
14536:       rotation: 0,         // degrees
14537:       opacity: 0.5,        // 0-1
14538:       flipH: false,
14539:       flipV: false,
14540:       scaleMode: false,    // true when in Set Scale mode
14541:       scaleP1: null,
14542:       scaleP2: null,
14543:       editMode: false,
14544:       orientMode: false,
14545:       orientP1: null,
