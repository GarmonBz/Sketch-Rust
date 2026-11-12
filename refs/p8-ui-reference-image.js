// P8 ref extract: ui-reference-image — lines 13700-13790 of 00) sketch standalone.html

13700:               <fieldset><legend>Ink Options</legend><label><input type="checkbox" name="inkConfirmation" ${checked('inkConfirmation')}> Ink Confirmation</label><label>Ink Recognition <select name="inkRecognition"><option>0.1</option><option>0.25</option><option>0.5</option></select></label></fieldset>
13701:               <fieldset><legend>Unit of Measure</legend><label><input type="radio" name="unitSystem" value="imperial" ${radio('unitSystem','imperial')}> Imperial</label><label><input type="radio" name="unitSystem" value="metric" ${radio('unitSystem','metric')}> Metric</label></fieldset>
13702:               <div class="option-split"><fieldset><legend>Decimal Precision</legend>${[1,.1,.01,.001,.0001].map(value => `<label><input type="radio" name="decimalPrecision" value="${value}" ${radio('decimalPrecision',value)}> ${value}</label>`).join('')}</fieldset><fieldset><legend>Step Unit</legend>${['tenths','half','whole'].map(value => `<label><input type="radio" name="stepUnit" value="${value}" ${radio('stepUnit',value)}> ${value[0].toUpperCase()+value.slice(1)}</label>`).join('')}</fieldset></div>
13703:               <fieldset><legend>Dimension Options</legend><label>Dimension <input type="number" name="dimensionOption" step="0.1" value="${o.dimensionOption}"></label><label>Ortho <input type="number" name="orthoAngle" min="1" max="90" value="${o.orthoAngle}">°</label><label><input type="checkbox" name="autoPlacement" ${checked('autoPlacement')}> Auto Placement</label><label><input type="checkbox" name="stripTrailingZeroes" ${checked('stripTrailingZeroes')}> Strip Trailing Zeroes</label><label><input type="checkbox" name="sameColorAsLine" ${checked('sameColorAsLine')}> Same Color As Line</label><label>Area Label Suffix <input name="areaLabelSuffix" value="${this.escapeHTML(o.areaLabelSuffix)}"></label></fieldset>
13704:             </section>
13705:             <section class="option-panel" data-option-panel="display">
13706:               <fieldset><legend>Alignment Indicators</legend><label><input type="checkbox" name="alignmentLines" ${checked('alignmentLines')}> Lines Visible</label><label><input type="checkbox" name="alignmentPoints" ${checked('alignmentPoints')}> Points Visible</label><label><input type="checkbox" name="testCloseIndicators" ${checked('testCloseIndicators')}> Test Close Indicators</label><label>Color <input type="color" name="alignmentColor" value="${o.alignmentColor}"></label><label>Style <select name="alignmentStyle"><option value="dash" ${selected('alignmentStyle','dash')}>Dash</option><option value="solid" ${selected('alignmentStyle','solid')}>Solid</option></select></label></fieldset>
13707:               <fieldset><legend>Grid</legend><label><input type="checkbox" name="gridEnabled" ${checked('gridEnabled')}> Enabled</label></fieldset>
13708:               <fieldset><legend>Minimum Dimension Properties</legend><label><input type="checkbox" name="minimumDimensionEnabled" ${checked('minimumDimensionEnabled')}> Enabled</label><label>Minimum length <input type="number" name="minimumDimension" min="0" step="0.1" value="${o.minimumDimension}"></label></fieldset>
13709:               <fieldset><legend>Auto-Shade</legend><label><input type="checkbox" name="autoShade" ${checked('autoShade')}> Enabled</label><label>Percent <input type="range" name="autoShadePercent" min="0" max="100" value="${o.autoShadePercent}"></label><label><input type="checkbox" name="selectionHighlight" ${checked('selectionHighlight')}> Selection Highlight</label></fieldset>
13710:             </section>
13711:             <section class="option-panel" data-option-panel="hardware">
13712:               <fieldset><legend>DISTO Discovery</legend><label><input type="radio" name="distoDiscovery" value="automatic" ${radio('distoDiscovery','automatic')}> Automatic</label><label><input type="radio" name="distoDiscovery" value="manual" ${radio('distoDiscovery','manual')}> Manual</label><label>COM Port <input name="distoComPort" value="${this.escapeHTML(o.distoComPort || '')}"></label></fieldset>
13713:               <fieldset><legend>Configuration</legend><label>Rounding <select name="distoRounding"><option value="none">None</option><option value="nearestHalf" ${selected('distoRounding','nearestHalf')}>Nearest Half</option><option value="nearestHundredth" ${selected('distoRounding','nearestHundredth')}>Nearest Hundredth</option><option value="nearestTenth" ${selected('distoRounding','nearestTenth')}>Nearest Tenth</option><option value="nearestWhole" ${selected('distoRounding','nearestWhole')}>Nearest Whole</option></select></label><label>Drawing Mode <select name="distoDrawingMode"><option value="standard">Standard</option><option value="continuous" ${selected('distoDrawingMode','continuous')}>Continuous</option></select></label></fieldset>
13714:             </section>
13715:             <section class="option-panel" data-option-panel="system">
13716:               <fieldset><legend>Auto-Save</legend><label><input type="checkbox" name="autoSaveEnabled" ${checked('autoSaveEnabled')}> Enabled</label><label>Interval (minutes) <input type="number" name="autoSaveInterval" min="1" max="120" value="${o.autoSaveInterval}"></label><small>Auto-save data is kept in this browser.</small></fieldset>
13717:               <fieldset><legend>Image Files</legend><label>Size <select name="imageSize"><option ${selected('imageSize','4800x4800')}>4800x4800</option><option ${selected('imageSize','3200x3200')}>3200x3200</option><option ${selected('imageSize','2048x1536')}>2048x1536</option><option ${selected('imageSize','1920x1080')}>1920x1080</option><option ${selected('imageSize','1280x720')}>1280x720</option></select></label><label><input type="radio" name="imageDepth" value="8" ${radio('imageDepth',8)}> 8-Bit</label><label><input type="radio" name="imageDepth" value="24" ${radio('imageDepth',24)}> 24-Bit</label><label><input type="checkbox" name="appendPageNumber" ${checked('appendPageNumber')}> Append Page Number</label><label><input type="checkbox" name="formatPNG" ${(o.imageFormats||[]).includes('png')?'checked':''}> PNG</label><label><input type="checkbox" name="formatJPG" ${(o.imageFormats||[]).includes('jpg')?'checked':''}> JPG</label><label><input type="checkbox" name="formatBMP" ${(o.imageFormats||[]).includes('bmp')?'checked':''}> BMP</label></fieldset>
13718:               <fieldset><legend>Application</legend><label><input type="checkbox" name="disableVideoTutorial" ${checked('disableVideoTutorial')}> Disable Video Tutorial</label><label><input type="checkbox" name="autoHealFiles" ${checked('autoHealFiles')}> Auto Heal Files</label></fieldset>
13719:             </section>
13720:             <section class="option-panel" data-option-panel="fonts">
13721:               ${[['dimension','Dimension Properties'],['text','Text Label Properties'],['autoPost','Automatic labels Properties']].map(([prefix,title]) => `<fieldset><legend>${title}</legend><label>Font <select name="${prefix}Font">${fontOptions(`${prefix}Font`)}</select></label><label>Size <select name="${prefix}FontSize">${sizeOptions(`${prefix}FontSize`)}</select></label><label><input type="checkbox" name="${prefix}Bold" ${checked(`${prefix}Bold`)}> Bold</label><label><input type="checkbox" name="${prefix}Italic" ${checked(`${prefix}Italic`)}> Italic</label><label><input type="checkbox" name="${prefix}Underline" ${checked(`${prefix}Underline`)}> Underline</label>${prefix === 'text' ? `<label>Color <input type="color" name="textColor" value="${o.textColor}"></label>` : ''}${prefix === 'autoPost' ? `<label><input type="checkbox" name="circleAreaCalcs" ${checked('circleAreaCalcs')}> Circle Area Calcs</label>` : ''}</fieldset>`).join('')}
13722:               <fieldset><legend>Text Background</legend><label>Color <input type="color" name="textBackground" value="${o.textBackground}"></label><label>Opacity <input type="number" name="textBackgroundOpacity" min="0" max="100" value="${o.textBackgroundOpacity}"></label></fieldset>
13723:             </section>
13724:             <section class="option-panel" data-option-panel="drawing">
13725:               <div class="option-split"><fieldset><legend>Area Line Properties</legend><label>Drawing Color <input type="color" name="exteriorColor" value="${o.exteriorColor}"></label><label>Thickness <input type="number" name="exteriorWidth" min="2" max="12" value="${o.exteriorWidth}"></label></fieldset><fieldset><legend>Interior Line Properties</legend><label>Drawing Color <input type="color" name="interiorColor" value="${o.interiorColor}"></label><label>Thickness <input type="number" name="interiorWidth" min="1" max="6" value="${o.interiorWidth}"></label></fieldset></div>
13726:               <fieldset><legend>Wall Solids</legend><label><input type="checkbox" name="wallSolids" ${checked('wallSolids')}> Enable wall solids</label><label>Exterior thickness (in) <input type="number" name="exteriorThickness" min="0" step="0.5" value="${o.exteriorThickness ?? 6}"></label><label>Interior thickness (in) <input type="number" name="interiorThickness" min="0" step="0.5" value="${o.interiorThickness ?? 4}"></label><small>When disabled, walls render as legacy single-line strokes.</small></fieldset>
13727:               <fieldset><legend>Witness Line Properties</legend><label>Base <input type="color" name="witnessBaseColor" value="${o.witnessBaseColor}"></label><label>Interior <input type="color" name="witnessInteriorColor" value="${o.witnessInteriorColor}"></label><label>Curve <input type="color" name="witnessCurveColor" value="${o.witnessCurveColor}"></label><label>Label Opacity <input type="number" name="labelOpacity" min="0" max="100" value="${o.labelOpacity}"></label></fieldset>
13728:               <fieldset><legend>Define Area Properties</legend><label><input type="checkbox" name="displayOnClose" ${checked('displayOnClose')}> Display on Close</label><label><input type="checkbox" name="defineFirst" ${checked('defineFirst')}> Define First</label><label><input type="checkbox" name="autoSubtractOverlapped" ${checked('autoSubtractOverlapped')}> Auto-Subtract Overlapped Areas</label><label><input type="checkbox" name="quickShapeDefine" ${checked('quickShapeDefine')}> Quick-Shape Define</label><label><input type="checkbox" name="redefine" ${checked('redefine')}> Re-Define</label><label><input type="checkbox" name="splitWallsAtJunctions" ${checked('splitWallsAtJunctions')}> Split wall dimensions at area junctions</label></fieldset>
13729:               <fieldset><legend>Keyboard Modifiers</legend><label><input type="checkbox" name="panZoomDelete" ${checked('panZoomDelete')}> Pan-Zoom / Delete</label></fieldset>
13730:               <fieldset><legend>Drawing Pointer</legend><label>Size <select name="pointerSize"><option value="128" ${selected('pointerSize',128)}>128x128</option><option value="64" ${selected('pointerSize',64)}>64x64</option><option value="32" ${selected('pointerSize',32)}>32x32</option></select></label></fieldset>
13731:               <fieldset><legend>Reference image</legend><label><input type="checkbox" name="saveImageWithSketch" ${checked('saveImageWithSketch')}> Save Image With Sketch</label><label>Image Quality <input type="number" name="imageQuality" min="1" max="100" value="${o.imageQuality}"></label></fieldset>
13732:             </section>
13733:           </div>
13734:         </div>
13735:         <div class="options-actions"><button type="button" data-defaults>Restore Defaults</button><button type="button" data-save-defaults>Save as Default</button><span></span><button type="button" data-close>Cancel</button><button type="submit">OK</button></div>
13736:       </form>`;
13737:     document.body.appendChild(overlay);
13738:     overlay.querySelectorAll('[data-close]').forEach(button => button.addEventListener('click', () => overlay.remove()));
13739:     overlay.querySelectorAll('[data-option-tab]').forEach(button => button.addEventListener('click', () => {
13740:       overlay.querySelectorAll('[data-option-tab]').forEach(tab => tab.classList.toggle('active', tab === button));
13741:       overlay.querySelectorAll('[data-option-panel]').forEach(panel => panel.classList.toggle('active', panel.dataset.optionPanel === button.dataset.optionTab));
13742:     }));
13743:     overlay.querySelector('[data-defaults]').addEventListener('click', () => {
13744:       this.app.applyOptions(JSON.parse(JSON.stringify(this.defaultOptions)));
13745:       overlay.remove();
13746:       this.notify('Options restored to defaults.');
13747:     });
13748:     overlay.querySelector('[data-save-defaults]').addEventListener('click', () => {
13749:       const next = collectOptions(overlay.querySelector('form'));
13750:       this.app.applyOptions(next);
13751:       this.defaultOptions = JSON.parse(JSON.stringify(next));
13752:       this.storedDefaultOptions = next;
13753:       try {
13754:         window.localStorage?.setItem('sketch.defaultOptions', JSON.stringify(next));
13755:         this.notify('Current options saved as defaults.');
13756:       } catch (_) {
13757:         this.notify('Current options applied, but could not be saved as defaults.');
13758:       }
13759:     });
13760:     overlay.querySelector('form').addEventListener('submit', event => {
13761:       event.preventDefault();
13762:       this.app.applyOptions(collectOptions(event.currentTarget));
13763:       overlay.remove();
13764:       this.notify('Options applied.');
13765:     });
13766:   }
13767: 
13768:   setPhotometricsModule(enabled) {
13769:     const tab = document.querySelector('[data-target="tab-photometrics"]');
13770:     if (tab) tab.hidden = !enabled;
13771:     if (!enabled && tab?.classList.contains('active')) {
13772:       document.querySelector('[data-target="tab-home"]')?.click();
13773:     }
13774:     this.notify(`Reference image module ${enabled ? 'activated' : 'deactivated'}.`);
13775:   }
13776: 
13777:   showPageLayout() {
13778:     document.getElementById('page-layout-overlay')?.remove();
13779:     const current = this.app.normalizePageLayout(this.app.state.pageLayout);
13780:     const formLabels = {
13781:       standard: size => `Standard ${size}`,
13782:       'full-page': size => `Full Page ${size}`,
13783:       'sketch-only': size => `${size} Size Sketch Only`,
13784:       'with-calculations': size => `${size} W/ Calculations`
13785:     };
13786:     const paperLabel = paper => paper === 'a4' ? 'A4' : `${paper[0].toUpperCase()}${paper.slice(1)}`;
13787:     const formOptions = (paper, selected) => Object.entries(formLabels).map(([key, label]) => {
13788:       const value = `${key}-${paper}`;
13789:       return `<option value="${value}" ${selected === value ? 'selected' : ''}>${label(paperLabel(paper))}</option>`;
13790:     }).join('');
