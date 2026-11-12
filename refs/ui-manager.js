class UIManager {
  constructor(app) {
    this.app = app;
    this.defaultOptions = JSON.parse(JSON.stringify(app.state.options || {}));
    this.storedDefaultOptions = {};
    try {
      const parsedDefaults = JSON.parse(window.localStorage?.getItem('sketch.defaultOptions') || 'null');
      if (parsedDefaults && typeof parsedDefaults === 'object' && !Array.isArray(parsedDefaults)) {
        this.storedDefaultOptions = parsedDefaults;
        this.defaultOptions = { ...this.defaultOptions, ...parsedDefaults };
      }
    } catch (_) { /* Keep bundled defaults when stored defaults are invalid. */ }
    this.applyInterfaceTheme();
    try {
      const storedAreaCodes = JSON.parse(window.localStorage?.getItem('sketch.areaCodes') || 'null');
      if (Array.isArray(storedAreaCodes) && window.AreaCodes) {
        this.app.areaCodes = window.AreaCodes.validateAreaCodes(storedAreaCodes);
        this.app.breakdown.codeTable = this.app.areaCodes;
      }
    } catch (_) { /* Keep bundled defaults when stored data is invalid. */ }

    this.defaultTextLibraryLabels = [
      '½ Bath', 'Atrium', 'Bath', 'Bed', 'Breakfast', 'Cabin', 'Carport',
      'Den', 'Dining', 'Dining Room', 'Entrance', 'Family', 'Foyer', 'Garage',
      'Kitchen', 'Laundry', 'Living Room', 'Master', 'Office', 'Patio',
      'Play Cabin', 'Porch', 'Suite', 'Utilities', 'Wood Deck'
    ];
    this.textLibraryLabels = this.readStoredList('sketch.textLibraryLabels', this.app.state.textLibraryLabels)
      || this.defaultTextLibraryLabels.slice();
    this.app.state.textLibraryLabels = this.textLibraryLabels.slice();
    this.activeTextRange = 'all';

    const library = window.SKETCH_SYMBOL_LIBRARY || { categories: [], symbols: {}, defaultFavorites: [] };
    const categoryIcons = {
      arrows: 'ph-arrow-up-right', bath: 'ph-bathtub', cameras: 'ph-camera', circles: 'ph-circle',
      doors: 'ph-door-open', fire: 'ph-fire-extinguisher', fireplaces: 'ph-fire', insurance: 'ph-warning',
      kitchen: 'ph-oven', miscellaneous: 'ph-shapes', pools: 'ph-swimming-pool', shapes: 'ph-square',
      stairs: 'ph-stairs', utility: 'ph-washing-machine', windows: 'ph-browser', favorites: 'ph-star'
    };
    const toItem = definition => ({
      type: definition.id,
      label: definition.label,
      icon: categoryIcons[definition.category] || 'ph-square',
      definition
    });
    this.symbolIndex = new Map(Object.values(library.symbols || {}).map(definition => [definition.id, toItem(definition)]));
    this.symbolCategories = Object.fromEntries((library.categories || [])
      .filter(category => category.key !== 'favorites')
      .map(category => [category.key, {
        name: category.name,
        items: category.symbolIds.map(id => this.symbolIndex.get(id)).filter(Boolean)
      }]));
    this.defaultSymbolFavorites = (library.defaultFavorites || []).filter(type => this.symbolIndex.has(type));
    const storedFavorites = this.readStoredList('sketch.symbolFavorites', this.app.state.symbolFavorites);
    this.symbolFavorites = (storedFavorites || []).filter(type => this.symbolIndex.has(type));
    if (!this.symbolFavorites.length) this.symbolFavorites = this.defaultSymbolFavorites.slice();
    this.app.state.symbolFavorites = this.symbolFavorites.slice();
    this.activeSymbolCategory = 'favorites';
    window.SKETCH_SYMBOL_DEFS = Object.fromEntries([...this.symbolIndex].map(([type, item]) => [type, { label: item.label, icon: item.icon, definition: item.definition }]));

    this.populateAreaCodes();
    this.bindEvents();
    this.populateTextLibrary();
    this.populateSymbolCategorySelect();
    this.populateSymbolLibrary('favorites');
    this.syncSettingsControls();
    this.restoreSidebarState();
    this.updateContextualTools(false);
  }

  readStoredList(key, fallback) {
    if (Array.isArray(fallback) && fallback.length) return fallback.slice();
    try {
      const parsed = JSON.parse(window.localStorage?.getItem(key) || 'null');
      return Array.isArray(parsed) && parsed.length ? parsed : null;
    } catch (_) { return null; }
  }

  persistLibraryState() {
    this.app.state.textLibraryLabels = this.textLibraryLabels.slice();
    this.app.state.symbolFavorites = this.symbolFavorites.slice();
    try {
      window.localStorage?.setItem('sketch.textLibraryLabels', JSON.stringify(this.textLibraryLabels));
      window.localStorage?.setItem('sketch.symbolFavorites', JSON.stringify(this.symbolFavorites));
    } catch (_) { /* Storage may be unavailable in embedded contexts. */ }
  }

  escapeHTML(value) {
    return String(value == null ? '' : value)
      .replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
      .replace(/\"/g, '&quot;').replace(/'/g, '&#39;');
  }

  populateAreaCodes(filter = '') {
    const tree = document.querySelector('.area-tree');
    if (!tree) return;
    const query = String(filter || '').trim().toLowerCase();
    const groups = new Map();
    (this.app.areaCodes || []).forEach(code => {
      const searchable = `${code.code} ${code.name} ${code.description} ${code.group}`.toLowerCase();
      if (query && !searchable.includes(query)) return;
      if (!groups.has(code.group)) groups.set(code.group, []);
      groups.get(code.group).push(code);
    });
    const groupNames = {
      GLA: 'Gross Living Area', GBA: 'Gross Building Area', BSMT: 'Basement',
      'P/P': 'Porch, Patio, Etc.', GAR: 'Garage, Carport, Etc.', LAND: 'Land',
      SITE: 'Subject Site', OTH: 'Storage', NCA: 'Non-Calculated Area', UND: 'Undefined Area'
    };
    tree.innerHTML = [...groups.entries()].map(([group, codes]) => {
      const expanded = Boolean(query);
      const direct = codes.length === 1 && codes[0].code === group && ['LAND', 'SITE', 'OTH', 'NCA', 'UND'].includes(group);
      if (direct) {
        const code = codes[0];
        return `<button class="area-btn area-btn-root" draggable="true" data-code="${this.escapeHTML(code.code)}"
          data-name="${this.escapeHTML(code.name)}">${this.escapeHTML(code.code)} - ${this.escapeHTML(groupNames[group] || code.name)}</button>`;
      }
      return `
      <div class="tree-node ${expanded ? 'expanded' : ''}" data-area-group="${this.escapeHTML(group)}">
        <div class="tree-label" role="treeitem" aria-expanded="${expanded}"><span class="tree-expander" aria-hidden="true">${expanded ? '&#9662;' : '&#9656;'}</span>
          ${this.escapeHTML(group)} - ${this.escapeHTML(groupNames[group] || group)}</div>
        <div class="tree-children">
          ${codes.map(code => `<button class="area-btn" draggable="true" data-code="${this.escapeHTML(code.code)}"
            data-name="${this.escapeHTML(code.name)}">${this.escapeHTML(code.code)} - ${this.escapeHTML(code.name)}</button>`).join('')}
        </div>
      </div>`;
    }).join('') || '<div class="library-empty">No matching area codes.</div>';
    this.bindAreaTreeEvents();
    this.updateAreaActionStates();
  }

  bindAreaTreeEvents() {
    document.querySelectorAll('.tree-label').forEach(label => {
      label.onclick = event => {
        const node = event.target.closest('.tree-node');
        if (!node) return;
        node.classList.toggle('expanded');
        const expander = node.querySelector('.tree-expander');
        if (expander) expander.textContent = node.classList.contains('expanded') ? '\u25BE' : '\u25B8';
        event.currentTarget.setAttribute('aria-expanded', String(node.classList.contains('expanded')));
      };
      label.addEventListener('contextmenu', event => {
        event.preventDefault();
        const node = event.target.closest('.tree-node');
        const group = node?.dataset?.areaGroup;
        if (!group) return;
        const anchor = { getBoundingClientRect: () => ({ left: event.clientX, bottom: event.clientY }) };
        this.showCommandPopup(anchor, [{
          action: 'add-code', label: `Add Code to ${group}`, icon: 'ph-plus-circle',
          run: () => { this.showAddAreaCode(group); }
        }]);
      });
    });
    document.querySelectorAll('.area-btn').forEach(button => {
      button.draggable = true;
      button.ondragstart = event => event.dataTransfer.setData('application/x-sketch-item', JSON.stringify({
        kind: 'area', code: button.dataset.code, name: button.dataset.name
      }));
      button.onclick = () => {
        document.querySelectorAll('.area-btn').forEach(item => item.classList.remove('selected'));
        button.classList.add('selected');
        this.app.state.selectedAreaType = button.dataset.code;
        this.app.state.selectedAreaName = button.dataset.name;
        const nameInput = document.getElementById('area-name-input');
        if (nameInput) nameInput.value = button.dataset.name;
        this.updateAreaActionStates();
      };
    });
  }

  updateAreaActionStates() {
    const hasCode = Boolean(this.app.state.selectedAreaType);
    const hasArea = this.app.state.selectedPolyIds && this.app.state.selectedPolyIds.size > 0;
    const hasUndefined = this.app.state.polygons && this.app.state.polygons.some(poly => poly.type === 'UND');
    const apply = document.getElementById('btn-apply-area');
    const edit = document.getElementById('btn-edit-area');
    if (apply) apply.disabled = !(hasCode && (hasArea || hasUndefined));
    if (edit) edit.disabled = !(hasCode || hasArea);
  }

  activateRibbonTab(targetId) {
    document.querySelectorAll('.tab').forEach(tab => tab.classList.toggle('active', tab.dataset.target === targetId));
    document.querySelectorAll('.ribbon-content').forEach(content => {
      content.style.display = content.id === targetId ? 'flex' : 'none';
    });
    const fileMenu = document.getElementById('file-menu');
    if (fileMenu) fileMenu.style.display = 'none';
  }

  updateContextualTools(autoActivate = true) {
    const hasArea = this.app.state.selectedPolyIds?.size > 0;
    const hasText = this.app.state.selectedAreaLabelIds?.size > 0 || this.app.state.selectedAreaSizeIds?.size > 0 ||
      this.app.state.selectedLabelIds?.size > 0;
    const hasLine = this.app.state.selectedLineIds?.size > 0;
    const hasSymbol = this.app.state.selectedSymbolIds?.size > 0;
    const hasHostedOpening = this.app.state.symbols?.some(sym =>
      this.app.state.selectedSymbolIds?.has(sym.id) && sym.wallId);
    const hasContext = hasArea || hasText || hasLine || hasSymbol;
    const tab = document.getElementById('edit-tools-tab');
    const ribbon = document.getElementById('tab-edit-tools');
    if (!tab || !ribbon) return;
    const wasHidden = tab.hidden;
    tab.hidden = !hasContext;
    document.querySelectorAll('.contextual-area-tools').forEach(group => group.hidden = !hasArea);
    document.querySelectorAll('.contextual-text-tools').forEach(group => group.hidden = !hasText);
    document.querySelectorAll('.contextual-line-tools').forEach(group => group.hidden = !hasLine);
    document.querySelectorAll('.contextual-symbol-tools').forEach(group => group.hidden = !hasHostedOpening);

    if (!hasContext) {
      if (tab.classList.contains('active')) this.activateRibbonTab('tab-home');
      return;
    }
    this.syncContextualControls();
    if (autoActivate && wasHidden) this.activateRibbonTab('tab-edit-tools');
  }

  syncContextualControls() {
    const area = this.app.state.polygons.find(poly => this.app.state.selectedPolyIds.has(poly.id));
    const areaLabel = this.app.state.polygons.find(poly =>
      this.app.state.selectedAreaLabelIds.has(poly.id) || this.app.state.selectedAreaSizeIds?.has(poly.id));
    const label = this.app.state.labels.find(item => this.app.state.selectedLabelIds.has(item.id));
    const line = this.app.state.lines.find(item => this.app.state.selectedLineIds.has(item.id));
    const setValue = (id, value) => { const input = document.getElementById(id); if (input && value != null) input.value = value; };
    if (area) {
      setValue('area-fill-pattern', area.fillPattern || 'solid');
      setValue('area-fill-color', area.fillColor || '#4299e1');
      setValue('area-line-style', area.lineStyle || 'solid');
      setValue('area-line-thickness', String(area.lineThickness || 1));
      setValue('area-line-color', area.lineColor || '#3182ce');
    }
    if (label) {
      setValue('text-font-family', label.fontFamily || 'Arial');
      setValue('text-font-size', String(label.fontSize || 12));
      setValue('text-color', label.color || '#e2e8f0');
      setValue('selected-text-value', label.text || '');
      [['text-bold', label.bold], ['text-italic', label.italic], ['text-underline', label.underline]].forEach(([command, active]) => {
        document.querySelector(`[data-command="${command}"]`)?.classList.toggle('active', Boolean(active));
      });
    }
    if (areaLabel && !label) {
      setValue('text-font-family', areaLabel.labelFontFamily || this.app.state.options?.autoPostFont || 'Arial');
      setValue('text-font-size', String(areaLabel.labelFontSize || this.app.state.options?.autoPostFontSize || 16));
      setValue('text-color', areaLabel.labelColor || '#475569');
      const areaSizeSelected = this.app.state.selectedAreaSizeIds?.has(areaLabel.id) &&
        !this.app.state.selectedAreaLabelIds.has(areaLabel.id);
      setValue('selected-text-value', areaSizeSelected
        ? `${(areaLabel.netArea ?? areaLabel.area).toFixed(1)} ${areaLabel.autoPost?.suffix || this.app.state.autoPost?.suffix || 'sf'}`
        : areaLabel.name || areaLabel.type || '');
      [['text-bold', areaLabel.labelBold], ['text-italic', areaLabel.labelItalic], ['text-underline', areaLabel.labelUnderline]].forEach(([command, active]) => {
        document.querySelector(`[data-command="${command}"]`)?.classList.toggle('active', Boolean(active));
      });
    }
    if (line) {
      setValue('selected-line-length', window.Geometry.distance(line.start, line.end).toFixed(2));
      setValue('selected-line-style', line.lineStyle || 'solid');
      setValue('selected-line-color', line.color || '#3182ce');
      const endpointLabel = document.getElementById('selected-line-endpoint-label');
      const endpoint = this.app.state.activeLineEndpoint?.lineId === line.id
        ? this.app.state.activeLineEndpoint.endpoint : 'end';
      if (endpointLabel) endpointLabel.textContent = `Length · ${endpoint === 'start' ? 'Start' : 'End'} handle`;
    }
  }

  populateTextLibrary(filter = '') {
    const list = document.getElementById('text-lib-list');
    if (!list) return;

    const q = String(filter || '').trim().toLowerCase();
    const letters = this.activeTextRange === 'all' ? '' : this.activeTextRange;
    const filtered = this.textLibraryLabels
      .filter(label => !q || label.toLowerCase().includes(q))
      .filter(label => !letters || letters.includes(label.trim().charAt(0).toLowerCase()))
      .sort((a, b) => a.localeCompare(b));

    list.innerHTML = filtered.map(label =>
      `<button class="lib-item" type="button" role="option" draggable="true" data-text="${this.escapeHTML(label)}">${this.escapeHTML(label)}</button>`
    ).join('') || '<div class="library-empty">No matching labels.</div>';

    list.querySelectorAll('.lib-item').forEach(btn => {
      btn.addEventListener('dragstart', event => {
        event.dataTransfer.setData('application/x-sketch-item', JSON.stringify({ kind: 'text', text: btn.dataset.text }));
      });
      btn.addEventListener('click', () => {
        // Highlight active
        list.querySelectorAll('.lib-item').forEach(b => b.classList.remove('active'));
        btn.classList.add('active');
        // Start placement
        this.app.startTextPlacement(btn.dataset.text);
      });
    });
  }

  populateSymbolCategorySelect() {
    const select = document.getElementById('symbol-category-select');
    if (!select) return;
    const options = [{ key: 'favorites', name: 'Favorites' }, ...Object.entries(this.symbolCategories).map(([key, category]) => ({ key, name: category.name }))];
    select.innerHTML = options.map(option => `<option value="${this.escapeHTML(option.key)}">${this.escapeHTML(option.name)}</option>`).join('');
    select.value = this.activeSymbolCategory;
  }

  getSymbolItems(category = this.activeSymbolCategory) {
    if (category === 'favorites') return this.symbolFavorites.map(type => this.symbolIndex.get(type)).filter(Boolean);
    return this.symbolCategories[category]?.items || [];
  }

  populateSymbolLibrary(category = this.activeSymbolCategory, filter = '') {
    const list = document.getElementById('symbol-lib-list');
    if (!list) return;
    this.activeSymbolCategory = category;
    const q = String(filter || '').trim().toLowerCase();
    const items = this.getSymbolItems(category).filter(item => !q || item.label.toLowerCase().includes(q));
    const select = document.getElementById('symbol-category-select');
    if (select && select.value !== category) select.value = category;

    list.innerHTML = items.map(item => {
      return `<button class="lib-symbol-item" type="button" role="option" draggable="true" data-type="${this.escapeHTML(item.type)}">
        <i class="ph ${this.escapeHTML(item.icon || 'ph-square')}"></i>
        <span>${this.escapeHTML(item.label)}</span>
      </button>`;
    }).join('') || '<div class="library-empty">No matching symbols.</div>';

    list.querySelectorAll('.lib-symbol-item').forEach(btn => {
      btn.addEventListener('dragstart', event => {
        event.dataTransfer.setData('application/x-sketch-item', JSON.stringify({ kind: 'symbol', symbolType: btn.dataset.type }));
        // Track the dragged symbol so the canvas can show a live snap-aware
        // preview while it is over the drawing.
        this.app.state.dragSymbol = btn.dataset.type;
        this.app.render();
      });
      btn.addEventListener('dragend', () => {
        if (this.app.state.dragSymbol) {
          this.app.state.dragSymbol = null;
          this.app.render();
        }
      });
      btn.addEventListener('click', () => {
        list.querySelectorAll('.lib-symbol-item').forEach(b => b.classList.remove('active'));
        btn.classList.add('active');
        this.updateSymbolPreview(btn.dataset.type);
        this.app.startSymbolPlacement(btn.dataset.type);
      });
    });
  }

  updateSymbolPreview(type) {
    const preview = document.getElementById('symbol-preview');
    if (!preview) return;
    const item = this.symbolIndex.get(type);
    preview.innerHTML = item
      ? `<canvas width="92" height="68" aria-label="${this.escapeHTML(item.label)} preview"></canvas><div><strong>${this.escapeHTML(item.label)}</strong><small>${this.escapeHTML(item.definition?.wallSnap && item.definition.wallSnap !== 'NONE' ? `Wall snap: ${item.definition.wallSnap.toLowerCase()}` : 'Click the drawing to place')}</small></div>`
      : '<span>Select a symbol</span>';
    if (item) this.app.canvas?.renderSymbolPreview?.(preview.querySelector('canvas'), type);
  }

  showTextLibraryManager() {
    document.getElementById('text-library-manager')?.remove();
    const overlay = document.createElement('div');
    overlay.id = 'text-library-manager';
    overlay.className = 'sidebar-manager-overlay';
    overlay.innerHTML = `
      <div class="sidebar-manager-dialog" role="dialog" aria-modal="true" aria-labelledby="text-manager-title">
        <div class="sidebar-manager-title"><strong id="text-manager-title">Manage Text Library</strong><button type="button" data-close aria-label="Close">&times;</button></div>
        <div class="sidebar-manager-grid">
          <div class="manager-main">
            <select class="manager-list" size="14" aria-label="Text library labels"></select>
            <input class="manager-input" type="text" placeholder="Label text" aria-label="Label text">
          </div>
          <div class="manager-alpha">
            ${['All','ABC','DEF','GHI','JKL','MNO','PQR','STU','VWX','YZ'].map((label, index) => `<button type="button" data-range="${index ? label.toLowerCase() : 'all'}" class="${index ? '' : 'active'}">${label}</button>`).join('')}
          </div>
        </div>
        <div class="sidebar-manager-actions"><button type="button" data-add>Add</button><button type="button" data-remove>Remove</button><button type="button" data-replace>Replace</button></div>
      </div>`;
    document.body.appendChild(overlay);
    const list = overlay.querySelector('.manager-list');
    const input = overlay.querySelector('.manager-input');
    let range = 'all';
    const refresh = selected => {
      const items = this.textLibraryLabels.filter(label => range === 'all' || range.includes(label.trim().charAt(0).toLowerCase())).sort((a, b) => a.localeCompare(b));
      list.innerHTML = items.map(label => `<option value="${this.escapeHTML(label)}">${this.escapeHTML(label)}</option>`).join('');
      if (selected && items.includes(selected)) list.value = selected;
      input.value = list.value || input.value;
    };
    refresh();
    list.addEventListener('change', () => { input.value = list.value; });
    overlay.querySelectorAll('[data-range]').forEach(button => button.addEventListener('click', () => {
      overlay.querySelectorAll('[data-range]').forEach(item => item.classList.remove('active'));
      button.classList.add('active');
      range = button.dataset.range;
      refresh();
    }));
    const save = selected => {
      this.textLibraryLabels = [...new Set(this.textLibraryLabels)].sort((a, b) => a.localeCompare(b));
      this.persistLibraryState();
      this.populateTextLibrary(document.getElementById('text-lib-search')?.value || '');
      refresh(selected);
    };
    overlay.querySelector('[data-add]').onclick = () => {
      const value = input.value.trim();
      if (!value) { this.notify('Enter a text label first.'); return; }
      if (this.textLibraryLabels.some(label => label.toLowerCase() === value.toLowerCase())) { this.notify('That label already exists.'); return; }
      this.textLibraryLabels.push(value); save(value);
    };
    overlay.querySelector('[data-remove]').onclick = () => {
      const value = list.value;
      if (!value) { this.notify('Select a label to remove.'); return; }
      this.textLibraryLabels = this.textLibraryLabels.filter(label => label !== value); input.value = ''; save();
    };
    overlay.querySelector('[data-replace]').onclick = () => {
      const oldValue = list.value;
      const value = input.value.trim();
      if (!oldValue || !value) { this.notify('Select a label and enter its replacement.'); return; }
      if (this.textLibraryLabels.some(label => label !== oldValue && label.toLowerCase() === value.toLowerCase())) { this.notify('That label already exists.'); return; }
      this.textLibraryLabels = this.textLibraryLabels.map(label => label === oldValue ? value : label); save(value);
    };
    overlay.querySelector('[data-close]').onclick = () => overlay.remove();
    overlay.addEventListener('click', event => { if (event.target === overlay) overlay.remove(); });
    input.focus();
  }

  showSymbolOrganizer() {
    document.getElementById('symbol-organizer')?.remove();
    const overlay = document.createElement('div');
    overlay.id = 'symbol-organizer';
    overlay.className = 'sidebar-manager-overlay';
    const items = [...this.symbolIndex.values()].sort((a, b) => a.label.localeCompare(b.label));
    overlay.innerHTML = `
      <div class="sidebar-manager-dialog symbol-organizer-dialog" role="dialog" aria-modal="true" aria-labelledby="symbol-organizer-title">
        <div class="sidebar-manager-title"><strong id="symbol-organizer-title">Organize Favorite Symbols</strong><button type="button" data-close aria-label="Close">&times;</button></div>
        <input class="manager-input" type="search" placeholder="Filter symbols..." aria-label="Filter symbols">
        <div class="symbol-organizer-list">${items.map(item => `<label data-search="${this.escapeHTML(item.label.toLowerCase())}"><input type="checkbox" value="${this.escapeHTML(item.type)}" ${this.symbolFavorites.includes(item.type) ? 'checked' : ''}><i class="ph ${this.escapeHTML(item.icon)}"></i><span>${this.escapeHTML(item.label)}</span></label>`).join('')}</div>
        <div class="sidebar-manager-actions"><button type="button" data-defaults>Restore Defaults</button><button type="button" data-save>Save Favorites</button></div>
      </div>`;
    document.body.appendChild(overlay);
    const filter = overlay.querySelector('.manager-input');
    filter.addEventListener('input', () => overlay.querySelectorAll('.symbol-organizer-list label').forEach(label => {
      label.hidden = !label.dataset.search.includes(filter.value.trim().toLowerCase());
    }));
    overlay.querySelector('[data-defaults]').onclick = () => overlay.querySelectorAll('input[type="checkbox"]').forEach(box => { box.checked = this.defaultSymbolFavorites.includes(box.value); });
    overlay.querySelector('[data-save]').onclick = () => {
      this.symbolFavorites = [...overlay.querySelectorAll('input[type="checkbox"]:checked')].map(box => box.value);
      this.persistLibraryState();
      this.populateSymbolLibrary('favorites', document.getElementById('symbol-lib-search')?.value || '');
      overlay.remove();
      this.notify('Favorite symbols updated.');
    };
    overlay.querySelector('[data-close]').onclick = () => overlay.remove();
    overlay.addEventListener('click', event => { if (event.target === overlay) overlay.remove(); });
    filter.focus();
  }

  showPanel(panelId, options = {}) {
    const panelIds = ['panel-define-area', 'panel-calcs', 'panel-text-library', 'panel-symbol-library', 'panel-layers'];
    if (!panelIds.includes(panelId)) return;
    panelIds.forEach(id => {
      const el = document.getElementById(id);
      if (el) el.hidden = id !== panelId;
    });
    document.querySelectorAll('.sidebar-tab').forEach(tab => {
      const active = tab.dataset.sidebarTarget === panelId;
      tab.classList.toggle('active', active);
      tab.setAttribute('aria-selected', String(active));
    });
    const key = document.getElementById(panelId)?.dataset.sidebarKey || 'define';
    this.app.state.sidebar.active = key;
    if (options.open !== false) this.setSidebarOpen(true);
    if (key === 'calcs') this.updateCalcsPanel();
    if (key === 'layers') this.updateLayersPanel();
  }

  setSidebarOpen(open) {
    const dock = document.getElementById('sidebar-dock');
    if (!dock) return;
    dock.classList.toggle('open', Boolean(open));
    this.app.state.sidebar.open = Boolean(open);
    window.setTimeout(() => this.app.canvas?.resize?.(), 160);
  }

  restoreSidebarState() {
    const state = this.app.state.sidebar || { active: 'define', open: true, pinned: true };
    const map = { define: 'panel-define-area', text: 'panel-text-library', symbols: 'panel-symbol-library', calcs: 'panel-calcs', layers: 'panel-layers' };
    this.showPanel(map[state.active] || map.define, { open: false });
    this.setSidebarOpen(state.open !== false);
    document.querySelectorAll('.sidebar-pin').forEach(pin => {
      const pinned = state.pinned !== false;
      pin.classList.toggle('active', pinned);
      pin.setAttribute('aria-pressed', String(pinned));
      pin.title = pinned ? 'Allow sidebar to auto-hide' : 'Keep sidebar open';
    });
  }

  loadLibraryState() {
    if (Array.isArray(this.app.state.textLibraryLabels) && this.app.state.textLibraryLabels.length) {
      this.textLibraryLabels = [...new Set(this.app.state.textLibraryLabels.map(String).map(value => value.trim()).filter(Boolean))];
    }
    if (Array.isArray(this.app.state.symbolFavorites)) {
      this.symbolFavorites = [...new Set(this.app.state.symbolFavorites)].filter(type => this.symbolIndex.has(type));
    }
    this.persistLibraryState();
    this.populateTextLibrary(document.getElementById('text-lib-search')?.value || '');
    this.populateSymbolLibrary(this.activeSymbolCategory, document.getElementById('symbol-lib-search')?.value || '');
  }

  bindEvents() {
    // --- Shared command routing (QAT, Ribbon, File menu, Status bar) ---
    document.querySelectorAll('[data-command]').forEach(btn => {
      btn.addEventListener('click', event => this.executeCommand(btn.dataset.command, event));
    });

    // --- Docked sidebar shell ---
    document.querySelectorAll('.sidebar-tab').forEach(tab => {
      tab.addEventListener('click', () => {
        /* Sidebar redesign: clicking any tab activates/retains its view.
           Collapse is the dedicated chevron (#sidebar-collapse) only. */
        if (!tab.classList.contains('active')) this.showPanel(tab.dataset.sidebarTarget);
      });
    });
    document.querySelectorAll('.sidebar-pin').forEach(pin => {
      pin.addEventListener('click', () => {
        const pinned = !(this.app.state.sidebar.pinned !== false);
        this.app.state.sidebar.pinned = pinned;
        document.querySelectorAll('.sidebar-pin').forEach(item => {
          item.classList.toggle('active', pinned);
          item.setAttribute('aria-pressed', String(pinned));
          item.title = pinned ? 'Allow sidebar to auto-hide' : 'Keep sidebar open';
        });
        this.notify(pinned ? 'Sidebar pinned open.' : 'Sidebar will hide after placement.');
      });
    });

    // --- File menu dismisses on an outside press or Escape ---
    document.addEventListener('pointerdown', event => {
      const menu = document.getElementById('file-menu');
      if (!menu || menu.style.display === 'none') return;
      if (event.target.closest('#file-menu, #file-recent-submenu, .tab.file-tab')) return;
      this.app.closeFileMenu();
    }, true);
    document.addEventListener('keydown', event => {
      if (event.key !== 'Escape') return;
      const menu = document.getElementById('file-menu');
      if (!menu || menu.style.display === 'none') return;
      event.stopPropagation();
      this.app.closeFileMenu();
    }, true);

    const areaCodeSearch = document.getElementById('area-code-search');
    if (areaCodeSearch) areaCodeSearch.addEventListener('input', event => this.populateAreaCodes(event.target.value));

    // --- Dimension posting ---
    const dimsCheckbox = document.getElementById('autopost-dims');
    if (dimsCheckbox) {
      dimsCheckbox.addEventListener('change', (e) => {
        this.app.state.dimensionsEnabled = e.target.checked;
        this.app.render();
      });
    }

    const dimensionPlacement = document.getElementById('dimension-placement');
    if (dimensionPlacement) {
      dimensionPlacement.addEventListener('change', (e) => {
        this.app.state.dimensionPlacement = e.target.value === 'in' ? 'in' : 'out';
        this.app.render();
      });
    }

    const drawColorInput = document.getElementById('draw-color-input');
    if (drawColorInput) {
      drawColorInput.addEventListener('input', (e) => {
        this.app.state.drawColor = e.target.value;
        this.notify(`Drawing color set to ${e.target.value}`);
        this.app.render();
      });
    }

    const bindAreaStyle = (id, property, numeric = false) => {
      const control = document.getElementById(id);
      if (control) control.addEventListener('change', event => {
        this.app.setSelectedAreaStyle(property, numeric ? Number(event.target.value) : event.target.value);
      });
    };
    bindAreaStyle('area-fill-pattern', 'fillPattern');
    bindAreaStyle('area-fill-color', 'fillColor');
    bindAreaStyle('area-line-style', 'lineStyle');
    bindAreaStyle('area-line-thickness', 'lineThickness', true);
    bindAreaStyle('area-line-color', 'lineColor');

    const bindTextStyle = (id, property, numeric = false) => {
      const control = document.getElementById(id);
      if (control) control.addEventListener('change', event => {
        this.app.setSelectedTextStyle(property, numeric ? Number(event.target.value) : event.target.value);
      });
    };
    bindTextStyle('text-font-family', 'fontFamily');
    bindTextStyle('text-font-size', 'fontSize', true);
    bindTextStyle('text-color', 'color');

    const bindColorInput = (inputId) => {
      const input = document.getElementById(inputId);
      const label = input?.closest('.ribbon-color');
      if (!input || !label) return;
      label.addEventListener('click', event => {
        event.preventDefault();
        this.showColorDialog(input.value, color => {
          input.value = color;
          input.dispatchEvent(new Event('change', { bubbles: true }));
        });
      });
    };
    bindColorInput('area-fill-color');
    bindColorInput('area-line-color');
    bindColorInput('text-color');
    bindColorInput('selected-line-color');

    bindTextStyle('selected-text-value', 'text');

    // --- Zoom Controls ---
    const zoomRange = document.getElementById('zoom-range');
    if (zoomRange) {
      zoomRange.addEventListener('input', (e) => {
        const cx = this.app.canvas.viewWidth / 2;
        const cy = this.app.canvas.viewHeight / 2;
        this.app.zoomTo(e.target.value / 100.0 * this.app.drawingScaleFactor(), cx, cy);
      });
    }

    // Drawing scale "1:N": N is the base scale (1:10 is the
    // default view) and the zoom percentage stays relative to it, so 1:20
    // shows the drawing at half size while the readout keeps its percent.
    const scaleInput = document.getElementById('scale-input');
    if (scaleInput) {
      const applyScale = () => {
        const next = Number(scaleInput.value);
        if (!Number.isFinite(next) || next <= 0) { this.updateZoomDisplay(); return; }
        this.app.setDrawingScale(next);
      };
      scaleInput.addEventListener('change', applyScale);
      scaleInput.addEventListener('keydown', event => { if (event.key === 'Enter') { applyScale(); scaleInput.blur(); } });
    }

    const btnZoomIn = document.getElementById('btn-zoom-in');
    if (btnZoomIn) btnZoomIn.addEventListener('click', () => this.app.zoomIn());
    
    const btnZoomOut = document.getElementById('btn-zoom-out');
    if (btnZoomOut) btnZoomOut.addEventListener('click', () => this.app.zoomOut());

    // --- Status Bar Buttons ---
    const btnCenter = document.getElementById('btn-center');
    if (btnCenter) btnCenter.addEventListener('click', () => this.app.centerSketch());

    const btnCenterFit = document.getElementById('btn-center-fit');
    if (btnCenterFit) btnCenterFit.addEventListener('click', () => this.app.centerAndFit());

    const btnNorth = document.getElementById('btn-north');
    if (btnNorth) btnNorth.addEventListener('click', async () => {
      const current = String(Math.round(this.app.state.northAngle || 0));
      const value = await this.promptDialog('North direction in degrees (0 = up, clockwise):', current);
      if (value === null) return;
      this.app.setNorthAngle(value);
    });

    const btnGrid = document.getElementById('btn-grid');
    if (btnGrid) btnGrid.addEventListener('click', () => this.app.toggleGrid());

    const btnSnap = document.getElementById('btn-snap');
    if (btnSnap) btnSnap.addEventListener('click', () => this.app.toggleSnap());

    const btnAreaSnap = document.getElementById('btn-area-snap');
    if (btnAreaSnap) btnAreaSnap.addEventListener('click', () => this.app.toggleAreaGridSnap());

    const snapPitch = document.getElementById('area-snap-pitch');
    if (snapPitch) snapPitch.addEventListener('change', (e) => this.app.setAreaGridSnapSize(e.target.value));

    // --- Ribbon Tabs ---
    document.querySelectorAll('.tab').forEach(tab => {
      tab.addEventListener('click', (e) => {
        document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
        e.target.classList.add('active');

        const fileMenu = document.getElementById('file-menu');
        
        if (e.target.classList.contains('file-tab')) {
          if (fileMenu) {
            const opening = (fileMenu.style.display === 'none' || fileMenu.style.display === '');
            fileMenu.style.display = opening ? 'flex' : 'none';
            if (opening) this.renderRecentFiles();
            if (!opening) {
              const submenu = document.getElementById('file-recent-submenu');
              if (submenu) submenu.style.display = 'none';
            }
          }
          return;
        } else {
          if (fileMenu) {
            fileMenu.style.display = 'none';
            const submenu = document.getElementById('file-recent-submenu');
            if (submenu) submenu.style.display = 'none';
          }
        }

        document.querySelectorAll('.ribbon-content').forEach(content => {
          content.style.display = 'none';
        });

        const targetId = e.target.getAttribute('data-target');
        if (targetId && targetId !== 'tab-file') {
          const targetContent = document.getElementById(targetId);
          if (targetContent) {
            targetContent.style.display = 'flex';
          }
        }
      });
    });

    // --- Apply Area Button ---
    const applyBtn = document.getElementById('btn-apply-area');
    if (applyBtn) {
      applyBtn.addEventListener('click', async () => {
        if (!this.app.state.selectedAreaType) {
          await this.alertDialog('Please select an area type from the tree above.');
          return;
        }

        const nameInput = document.getElementById('area-name-input');
        const customName = nameInput ? nameInput.value.trim() : '';
        const areaName = customName || this.app.state.selectedAreaName;
        const areaCode = this.app.state.selectedAreaType;

        const targetPolygons = [];
        if (this.app.state.selectedPolyIds.size > 0) {
          this.app.state.polygons.forEach(poly => {
            if (this.app.state.selectedPolyIds.has(poly.id)) {
              targetPolygons.push(poly);
            }
          });
        }

        if (targetPolygons.length === 0) {
          const und = this.app.state.polygons.find(p => p.type === 'UND');
          if (und) targetPolygons.push(und);
        }

        if (targetPolygons.length > 0) {
          this.app.pushUndo();
          targetPolygons.forEach(poly => {
            poly.type = areaCode;
            poly.name = areaName;
          });
          this.app.recalcAutoSubtract();
          this.app.clearSelection();
          this.app.render();
          this.updateCalcsPanel();
          
          if (nameInput) nameInput.value = '';
          document.querySelectorAll('.area-btn').forEach(b => b.classList.remove('selected'));
          this.app.state.selectedAreaType = null;
        } else {
          await this.alertDialog('No area to apply to. Draw and close a polygon first.');
        }
      });
    }

    // --- Redefine Area Button ---
    const redefineBtn = document.getElementById('btn-redefine-area');
    if (redefineBtn) {
      redefineBtn.addEventListener('click', async () => {
        if (!this.app.state.selectedAreaType) {
          await this.alertDialog('Click a type in the Areas palette first, then Redefine.');
          return;
        }
        if (this.app.state.selectedPolyIds.size === 0) {
          await this.alertDialog('Select a defined area on the canvas first.');
          return;
        }

        const nameInput = document.getElementById('area-name-input');
        const customName = nameInput ? nameInput.value.trim() : '';
        const areaName = customName || this.app.state.selectedAreaName;
        const areaCode = this.app.state.selectedAreaType;

        this.app.pushUndo();
        this.app.state.polygons.forEach(poly => {
          if (this.app.state.selectedPolyIds.has(poly.id)) {
            poly.type = areaCode;
            poly.name = areaName;
          }
        });

        this.app.recalcAutoSubtract();
        this.app.clearSelection();
        this.app.render();
        this.updateCalcsPanel();
      });
    }

    const cloneAreaBtn = document.getElementById('btn-clone-area');
    if (cloneAreaBtn) cloneAreaBtn.addEventListener('click', () => this.app.cloneSelectedAreas());

    const reopenAreaBtn = document.getElementById('btn-reopen-area');
    if (reopenAreaBtn) reopenAreaBtn.addEventListener('click', () => this.app.reopenSelectedArea());

    const detectRegionsBtn = document.getElementById('btn-detect-regions');
    if (detectRegionsBtn) detectRegionsBtn.addEventListener('click', () => this.app.detectRegions());

    const editAreaBtn = document.getElementById('btn-edit-area');
    if (editAreaBtn) {
      editAreaBtn.addEventListener('click', async () => {
        if (this.app.state.selectedAreaType) {
          this.showAreaDefinitionEditor(this.app.state.selectedAreaType);
          return;
        }
        const selected = this.app.state.polygons.filter(poly => this.app.state.selectedPolyIds.has(poly.id));
        if (selected.length === 0) { this.notify('Select an area to edit.'); return; }
        const name = await this.promptDialog('Area name:', selected[0].name || '');
        if (name === null) return;
        const factorValue = await this.promptDialog('Area factor override (blank uses the code default):',
          selected[0].factorOverride == null ? '' : String(selected[0].factorOverride));
        if (factorValue === null) return;
        const factor = factorValue.trim() === '' ? null : parseFloat(factorValue);
        if (factor !== null && (!Number.isFinite(factor) || factor < 0)) { this.notify('Factor must be zero or greater.'); return; }
        const modeValue = await this.promptDialog('Subtraction mode: auto, always, or never', selected[0].subtractMode || 'auto');
        if (modeValue === null) return;
        const subtractMode = modeValue.trim().toLowerCase();
        if (!['auto', 'always', 'never'].includes(subtractMode)) { this.notify('Subtraction mode must be auto, always, or never.'); return; }
        let subtractTargetId = selected[0].subtractTargetId ?? null;
        if (subtractMode === 'always') {
          const targetValue = await this.promptDialog('Deduct from area id (blank uses spatial containment):', subtractTargetId == null ? '' : String(subtractTargetId));
          if (targetValue === null) return;
          subtractTargetId = targetValue.trim() === '' ? null : Number(targetValue);
          if (subtractTargetId !== null && !Number.isFinite(subtractTargetId)) { this.notify('Deduction target must be an area id.'); return; }
        } else subtractTargetId = null;
        this.app.pushUndo();
        selected.forEach(poly => {
          poly.name = name || poly.name;
          poly.factorOverride = factor;
          poly.subtractMode = subtractMode;
          poly.subtractTargetId = subtractTargetId;
        });
        this.app.recalcAutoSubtract();
        this.updateCalcsPanel();
        this.app.render();
      });
    }

    const addCodeBtn = document.getElementById('btn-add-area-code');
    if (addCodeBtn) {
      addCodeBtn.addEventListener('click', () => this.showAddAreaCode());
    }

    const autoPostBindings = {
      'autopost-code': 'code',
      'autopost-name': 'name',
      'autopost-calcs': 'calcs',
      'autopost-base': 'base',
      'autopost-factor': 'factor'
    };
    Object.entries(autoPostBindings).forEach(([id, key]) => {
      const checkbox = document.getElementById(id);
      if (!checkbox) return;
      checkbox.addEventListener('change', () => {
        this.app.state.autoPost[key] = checkbox.checked;
        this.app.render();
      });
    });
    const suffix = document.getElementById('autopost-suffix');
    if (suffix) suffix.addEventListener('change', () => {
      this.app.state.autoPost.suffix = suffix.value;
      this.app.render();
    });

    // --- Left Toolbar ---
    document.querySelectorAll('.tool-btn').forEach(btn => {
      btn.addEventListener('click', (e) => {
        const id = e.currentTarget.id;
        if ((id === 'tool-draw-exterior' || id === 'tool-draw-interior') && e.target.closest('.tool-caret')) {
          e.stopPropagation();
          this.showWallShapeMenu(e.currentTarget, id === 'tool-draw-exterior' ? 'exterior' : 'interior');
          return;
        }
        if (id === 'tool-select') this.app.setMode('select');
        else if (id === 'tool-draw-exterior') this.app.setMode('draw-exterior');
        else if (id === 'tool-draw-interior') this.app.setMode('draw-interior');
        else if (id === 'tool-pan') this.app.setMode('pan');
        else if (id === 'tool-fence') this.app.setMode('fence');
        else if (id === 'tool-lasso') this.app.setMode('lasso');
        else if (id === 'tool-insert-vertex') this.app.setMode('insert-vertex');
        else if (id === 'tool-curve') this.app.setMode('curve');
        else if (id === 'tool-measure') this.app.setMode('measure');
        else if (id === 'tool-chain-dim') this.app.setMode('chain-dim');
        else if (id === 'tool-shapes') this.showShapes();
        else if (id === 'tool-text') {
          this.showPanel('panel-text-library');
        }
        else if (id === 'tool-symbols') {
          this.showPanel('panel-symbol-library');
        }
        else if (id === 'tool-define-area') {
          this.app.areaWorkflow?.activate();
          this.showPanel('panel-define-area');
        }
        else if (id === 'tool-touch') this.showTouchPanel();
        else if (id === 'tool-geo') this.showGeoMenu(e.currentTarget);
      });
      if (btn.id === 'tool-draw-exterior' || btn.id === 'tool-draw-interior') btn.addEventListener('contextmenu', event => {
        event.preventDefault();
        this.showWallShapeMenu(btn, btn.id === 'tool-draw-exterior' ? 'exterior' : 'interior');
      });
    });

    document.getElementById('selected-line-style')?.addEventListener('change', event => this.app.setSelectedLineAppearance({ lineStyle:event.target.value }));
    document.getElementById('selected-line-color')?.addEventListener('change', event => this.app.setSelectedLineAppearance({ color:event.target.value }));
    document.getElementById('selected-line-length')?.addEventListener('keydown', event => {
      if (event.key === 'Enter') this.app.setSelectedLineLength(event.currentTarget.value);
    });
    document.getElementById('area-status-select')?.addEventListener('change', event => {
      const poly = this.app.state.polygons.find(item => String(item.id) === event.currentTarget.value);
      if (!poly) return;
      this.app.clearSelection(false);
      this.app.state.selectedPolyIds.add(poly.id);
      this.updateAreaActionStates();
      this.app.render();
    });

    // --- Text Library Search ---
    const textSearch = document.getElementById('text-lib-search');
    if (textSearch) {
      textSearch.addEventListener('input', (e) => {
        this.populateTextLibrary(e.target.value);
      });
      textSearch.addEventListener('keydown', event => {
        if (event.key !== 'Enter') return;
        const value = event.currentTarget.value.trim();
        if (!value) return;
        event.preventDefault();
        this.app.startTextPlacement(value);
        this.notify(`Click the drawing to place “${value}”.`);
      });
    }

    document.querySelectorAll('.alpha-filter-btn').forEach(button => {
      button.addEventListener('click', () => {
        document.querySelectorAll('.alpha-filter-btn').forEach(item => item.classList.remove('active'));
        button.classList.add('active');
        this.activeTextRange = button.dataset.range || 'all';
        this.populateTextLibrary(textSearch?.value || '');
      });
    });
    document.getElementById('btn-edit-text-library')?.addEventListener('click', () => this.showTextLibraryManager());

    // --- Symbol category, search, preview, and organizer ---
    const symbolCategory = document.getElementById('symbol-category-select');
    symbolCategory?.addEventListener('change', event => {
      this.populateSymbolLibrary(event.target.value, document.getElementById('symbol-lib-search')?.value || '');
    });
    const symSearch = document.getElementById('symbol-lib-search');
    symSearch?.addEventListener('input', event => this.populateSymbolLibrary(this.activeSymbolCategory, event.target.value));
    document.getElementById('btn-organize-symbols')?.addEventListener('click', () => this.showSymbolOrganizer());

    // --- File Menu Operations ---
    const fileNew = document.getElementById('file-new');
    if (fileNew) fileNew.addEventListener('click', () => this.app.newSketch());

    const fileOpen = document.getElementById('file-open');
    const fileOpenInput = document.getElementById('file-open-input');
    if (fileOpen && fileOpenInput) {
      fileOpen.addEventListener('click', () => fileOpenInput.click());
      fileOpenInput.addEventListener('change', (e) => {
        if (e.target.files.length > 0) {
          this.app.loadFromFile(e.target.files[0]);
          e.target.value = '';
        }
      });
    }

    const fileSave = document.getElementById('file-save');
    if (fileSave) fileSave.addEventListener('click', () => this.app.saveToFile());

    const fileSaveAs = document.getElementById('file-save-as');
    if (fileSaveAs) fileSaveAs.addEventListener('click', () => this.app.saveAs());

    const fileImport = document.getElementById('file-import');
    const fileImportInput = document.getElementById('file-import-input');
    if (fileImport && fileImportInput) {
      fileImport.addEventListener('click', () => fileImportInput.click());
      fileImportInput.addEventListener('change', (e) => {
        if (e.target.files.length > 0) {
          this.app.importFromFile(e.target.files[0]);
          e.target.value = '';
        }
      });
    }

    const fileExportPng = document.getElementById('file-export-png');
    if (fileExportPng) fileExportPng.addEventListener('click', () => this.app.exportPNG());
    
    const fileExportSvg = document.getElementById('file-export-svg');
    if (fileExportSvg) fileExportSvg.addEventListener('click', () => this.app.exportSVG());
    
    const fileExportDxf = document.getElementById('file-export-dxf');
    if (fileExportDxf) fileExportDxf.addEventListener('click', () => this.app.exportDXF());
    
    const fileExportPdf = document.getElementById('file-export-pdf');
    if (fileExportPdf) fileExportPdf.addEventListener('click', () => this.app.exportPDF());

    const recentClear = document.getElementById('recent-files-clear');
    if (recentClear) recentClear.addEventListener('click', () => this.clearRecentFiles());

    const fileRecent = document.getElementById('file-recent');
    const fileRecentSubmenu = document.getElementById('file-recent-submenu');
    if (fileRecent && fileRecentSubmenu) {
      fileRecent.addEventListener('click', (e) => {
        e.stopPropagation();
        const isVisible = fileRecentSubmenu.style.display !== 'none';
        fileRecentSubmenu.style.display = isVisible ? 'none' : 'flex';
        if (!isVisible) this.renderRecentFiles();
      });
    }

    // --- Page Navigation ---
    const btnPageAdd = document.getElementById('btn-page-add');
    if (btnPageAdd) btnPageAdd.addEventListener('click', () => this.app.addPage());

    const underlaySelect = document.getElementById('underlay-select');
    if (underlaySelect) underlaySelect.addEventListener('change', () => this.app.setUnderlay(parseInt(underlaySelect.value, 10)));

    const btnPageDel = document.getElementById('btn-page-del');
    if (btnPageDel) btnPageDel.addEventListener('click', () => this.app.deletePage());

    const btnPagePrev = document.getElementById('btn-page-prev');
    if (btnPagePrev) btnPagePrev.addEventListener('click', () => {
      if (this.app.currentPage > 0) this.app.goToPage(this.app.currentPage - 1);
      else this.notify('This is the first page.');
    });

    const btnPageNext = document.getElementById('btn-page-next');
    if (btnPageNext) btnPageNext.addEventListener('click', async () => {
      if (this.app.currentPage < this.app.pages.length - 1) this.app.goToPage(this.app.currentPage + 1);
      else if (await this.confirmDialog('Add a page after the current page?')) this.app.addPage();
    });

    const canvas = document.getElementById('sketchCanvas');
    if (canvas) {
      canvas.addEventListener('dragover', event => {
        event.preventDefault();
        if (!this.app.state.dragSymbol) return;
        // Mouse events don't fire during a drag, so feed the preview with
        // the dragover coordinates and redraw.
        const rect = canvas.getBoundingClientRect();
        this.app.canvas.mouseScreen = { x: event.clientX - rect.left, y: event.clientY - rect.top };
        this.app.render();
      });
      canvas.addEventListener('dragleave', () => {
        if (!this.app.state.dragSymbol) return;
        this.app.canvas.mouseScreen = null;
        this.app.render();
      });
      canvas.addEventListener('drop', event => {
        event.preventDefault();
        this.app.state.dragSymbol = null;
        const raw = event.dataTransfer.getData('application/x-sketch-item');
        if (!raw) return;
        let item;
        try { item = JSON.parse(raw); } catch (_) { return; }
        const rect = canvas.getBoundingClientRect();
        const worldPt = window.Geometry.toWorld(
          { x: event.clientX - rect.left, y: event.clientY - rect.top },
          this.app.state.pan,
          this.app.state.zoom
        );
        if (item.kind === 'area') {
          this.app.defineAreaAt(worldPt, item.code, item.name);
        } else if (item.kind === 'text') {
          this.app.pushUndo();
          this.app.state.labels.push({
            x: worldPt.x, y: worldPt.y, text: item.text, fontSize: 12,
            rotation: 0, id: this.app.genId(),
            layerId: this.app.getAreaLayerIdAt(worldPt)
          });
          this.app.render();
        } else if (item.kind === 'symbol') {
          // Route drops through the regular placement so wall snap, swing
          // orientation and wall openings apply exactly as when clicking.
          this.app.placeSymbolAt(worldPt, item.symbolType);
        }
      });
    }

    // --- Area-backed Layers panel ---
    this.bindLayerTreeEvents();

    // --- Calcs panel expand/collapse all ---
    const btnExpandAll = document.getElementById('btn-expand-all');
    if (btnExpandAll) {
      btnExpandAll.addEventListener('click', () => {
        document.querySelectorAll('.calcs-group, .calcs-area-node').forEach(node => {
          node.classList.add('expanded');
          const header = node.querySelector(':scope > .calcs-group-header, :scope > .calcs-area-header');
          header?.setAttribute('aria-expanded', 'true');
          const icon = header?.querySelector('i');
          if (icon) icon.className = 'ph-fill ph-caret-down';
        });
      });
    }
    const btnCollapseAll = document.getElementById('btn-collapse-all');
    if (btnCollapseAll) {
      btnCollapseAll.addEventListener('click', () => {
        document.querySelectorAll('.calcs-group, .calcs-area-node').forEach(node => {
          node.classList.remove('expanded');
          const header = node.querySelector(':scope > .calcs-group-header, :scope > .calcs-area-header');
          header?.setAttribute('aria-expanded', 'false');
          const icon = header?.querySelector('i');
          if (icon) icon.className = 'ph-fill ph-caret-right';
        });
      });
    }
    document.getElementById('btn-calculation-details')?.addEventListener('click', () => {
      this.app.breakdown.show(this.app.buildExportModel(), { notify: message => this.notify(message), visual: true });
    });

    // --- Reference image ---
    const btnPhotoLoad = document.getElementById('btn-photo-load');
    const photoFileInput = document.getElementById('photo-file-input');
    if (btnPhotoLoad && photoFileInput) {
      btnPhotoLoad.addEventListener('click', () => photoFileInput.click());
      photoFileInput.addEventListener('change', (e) => {
        if (e.target.files.length > 0) {
          this.app.loadBgImage(e.target.files[0]);
          e.target.value = '';
        }
      });
    }

    const btnPhotoShow = document.getElementById('btn-photo-show');
    if (btnPhotoShow) btnPhotoShow.addEventListener('click', () => this.app.showBgImage());

    const btnPhotoHide = document.getElementById('btn-photo-hide');
    if (btnPhotoHide) btnPhotoHide.addEventListener('click', () => this.app.hideBgImage());

    const btnPhotoRemove = document.getElementById('btn-photo-remove');
    if (btnPhotoRemove) btnPhotoRemove.addEventListener('click', () => this.app.removeBgImage());

    const btnPhotoSetScale = document.getElementById('btn-photo-set-scale');
    if (btnPhotoSetScale) btnPhotoSetScale.addEventListener('click', () => this.app.startSetScale());

    const btnPhotoSmaller = document.getElementById('btn-photo-smaller');
    if (btnPhotoSmaller) btnPhotoSmaller.addEventListener('click', () => this.app.scaleBgImageBy(0.8));

    const btnPhotoLarger = document.getElementById('btn-photo-larger');
    if (btnPhotoLarger) btnPhotoLarger.addEventListener('click', () => this.app.scaleBgImageBy(1.25));

    const btnPhotoRotateLeft = document.getElementById('btn-photo-rotate-left');
    if (btnPhotoRotateLeft) btnPhotoRotateLeft.addEventListener('click', () => this.app.rotateBgImage(-90));

    const btnPhotoRotateRight = document.getElementById('btn-photo-rotate-right');
    if (btnPhotoRotateRight) btnPhotoRotateRight.addEventListener('click', () => this.app.rotateBgImage(90));

    const btnPhotoFlipH = document.getElementById('btn-photo-flip-h');
    if (btnPhotoFlipH) btnPhotoFlipH.addEventListener('click', () => this.app.flipBgImageH());

    const btnPhotoFlipV = document.getElementById('btn-photo-flip-v');
    if (btnPhotoFlipV) btnPhotoFlipV.addEventListener('click', () => this.app.flipBgImageV());

    const photoIntensity = document.getElementById('photo-intensity');
    if (photoIntensity) {
      photoIntensity.addEventListener('input', (e) => this.app.setBgIntensity(parseInt(e.target.value)));
    }
  }

  executeCommand(command, event = null) {
    const commands = {
      save: () => this.app.saveToFile(),
      print: () => this.app.exportPDF(),
      undo: () => this.app.undo(),
      redo: () => this.app.redo(),
      'undo-history': event => this.showUndoHistory(event?.currentTarget || document.querySelector('[data-command="undo-history"]')),
      cut: () => this.app.cutSelected(),
      copy: () => this.app.copySelected(),
      paste: () => this.app.paste(),
      'select-all': () => this.app.selectAll(),
      'select-by-type': event => this.showSelectMenu(event?.currentTarget || document.querySelector('[data-command="select-by-type"]')),
      'delete-selected': () => this.app.deleteSelected(),
      'rotate-sketch': () => this.app.rotateSketch(90),
      'flip-sketch': event => this.showFlipMenu(event?.currentTarget || document.querySelector('[data-command="flip-sketch"]')),
      'flip-selected-area': () => this.app.flipSelectedAreas('horizontal'),
      'apply-line-length': () => this.app.setSelectedLineLength(document.getElementById('selected-line-length')?.value),
      'curve-selected-line': () => this.app.setMode('curve'),
      'area-properties': () => this.showSelectedAreaProperties(),
      'selected-area-attributes': () => this.showSelectedAreaAttributes(),
      'auto-subtract-selected': () => this.app.toggleSelectedAreaAutoSubtract(),
      'text-bold': () => this.app.toggleSelectedTextStyle('bold'),
      'text-italic': () => this.app.toggleSelectedTextStyle('italic'),
      'text-underline': () => this.app.toggleSelectedTextStyle('underline'),
      'text-grow': () => this.app.adjustSelectedTextSize(1),
      'text-shrink': () => this.app.adjustSelectedTextSize(-1),
      'draw-color': () => this.toggleDrawColorPalette(),
      'subject-info': () => this.editSubjectInfo(),
      'area-attributes': () => this.editAreaAttributes(),
      'extended-data': () => this.editExtendedData(),
      'mini-map': () => this.toggleMiniMap(),
      'toggle-calcs': () => this.toggleCalcsPanel(),
      'area-breakdown': () => this.app.breakdown.show(this.app.buildExportModel(), { notify: message => this.notify(message) }),
      'area-code-settings': () => this.showAreaCodeEditor(),
      'page-prev': () => this.app.currentPage > 0
        ? this.app.goToPage(this.app.currentPage - 1)
        : this.notify('This is the first page.'),
      'page-next': async () => this.app.currentPage < this.app.pages.length - 1
        ? this.app.goToPage(this.app.currentPage + 1)
        : (await this.confirmDialog('Add a page after the current page?') && this.app.addPage()),
      'page-first': () => this.app.goToPage(0),
      'page-last': () => this.app.goToPage(this.app.pages.length - 1),
      'page-add': () => this.app.addPage(),
      'page-delete': () => this.app.deletePage(),
      'toggle-canvas-background': event => this.showCanvasBackgroundMenu(event?.currentTarget || document.querySelector('[data-command="toggle-canvas-background"]')),
      'interface-theme': event => this.showInterfaceThemeMenu(event?.currentTarget || document.querySelector('[data-command="interface-theme"]')),
      'window-minimize': () => this.notify('Minimize is controlled by the browser or operating system.'),
      'window-maximize': () => document.fullscreenElement
        ? document.exitFullscreen?.()
        : document.documentElement.requestFullscreen?.(),
      'cycle-dimensions': () => this.app.cycleDimensions(),
      'toggle-grid': () => this.app.toggleGrid(),
      filter: () => this.toggleFilterPanel(),
      'disto-connect': () => this.app.connectDisto(),
      options: () => this.showOptions(),
      'photo-edit': () => this.app.togglePhotoEditMode(),
      'photo-orient': () => this.app.startPhotoOrient(),
      geo: event => this.showGeoMenu(event?.currentTarget || document.querySelector('[data-command="geo"]')),
      'activate-module': () => this.setPhotometricsModule(true),
      'deactivate-module': () => this.setPhotometricsModule(false),
      email: () => this.app.emailSupport(),
      'print-preview': () => this.showPrintPreview(),
      'page-layout': () => this.showPageLayout(),
      exit: () => this.app.exitApp(),
      'zoom-fence': () => this.app.setMode('zoom-fence'),
      'touch-panel': () => this.showTouchPanel(),
      shortcuts: () => this.showShortcuts(),
      'flip-opening': () => this.app.flipOpening(),
      'opening-wider': () => this.app.adjustOpeningWidth(0.5),
      'opening-narrower': () => this.app.adjustOpeningWidth(-0.5)
    };

    const action = commands[command];
    if (action) action();
    else this.notify(`${command} is not available.`);
  }

  // --- Recent Files ---
  getRecentFiles() {
    try {
      const parsed = JSON.parse(window.localStorage?.getItem('sketch.recentFiles') || 'null');
      return Array.isArray(parsed)
        ? parsed.filter(entry => entry && typeof entry.name === 'string' && typeof entry.data === 'string')
        : [];
    } catch (_) { return []; }
  }

  recordRecentFile(name, data) {
    try {
      if (!name || typeof data !== 'string' || data.length > 1536000) return; // ~1.5MB cap
      let recent = this.getRecentFiles();
      recent = recent.filter(entry => entry.name !== name);
      recent.unshift({ name, lastOpened: new Date().toISOString(), data });
      if (recent.length > 10) recent = recent.slice(0, 10);
      window.localStorage?.setItem('sketch.recentFiles', JSON.stringify(recent));
      this.renderRecentFiles();
    } catch (_) { /* Storage may be unavailable in embedded contexts. */ }
  }

  renderRecentFiles() {
    const list = document.getElementById('recent-files-list');
    const empty = document.getElementById('recent-files-empty');
    if (!list || !empty) return;
    const recent = this.getRecentFiles();
    list.innerHTML = '';
    empty.style.display = recent.length ? 'none' : 'block';
    recent.forEach(entry => {
      const item = document.createElement('button');
      item.type = 'button';
      item.className = 'recent-file-item';
      item.title = entry.name;

      const icon = document.createElement('i');
      icon.className = 'ph-fill ph-file';

      const info = document.createElement('div');
      info.className = 'recent-file-info';

      const nameEl = document.createElement('span');
      nameEl.className = 'recent-file-name';
      nameEl.textContent = entry.name;

      const dateEl = document.createElement('span');
      dateEl.className = 'recent-file-date';
      dateEl.textContent = new Date(entry.lastOpened).toLocaleString();

      info.appendChild(nameEl);
      info.appendChild(dateEl);
      item.appendChild(icon);
      item.appendChild(info);
      item.addEventListener('click', () => this.openRecentFile(entry));
      list.appendChild(item);
    });
  }

  openRecentFile(entry) {
    try {
      const file = new File([entry.data], entry.name, { type: 'application/json' });
      this.app.loadFromFile(file);
    } catch (_) {
      this.notify('Could not open that recent file.');
    }
  }

  clearRecentFiles() {
    try { window.localStorage?.removeItem('sketch.recentFiles'); } catch (_) { /* Storage may be unavailable. */ }
    this.renderRecentFiles();
    this.notify('Recent files cleared.');
  }

  notify(message) {
    let toast = document.getElementById('app-toast');
    if (!toast) {
      toast = document.createElement('div');
      toast.id = 'app-toast';
      toast.className = 'app-toast';
      document.body.appendChild(toast);
    }
    toast.textContent = message;
    toast.classList.add('show');
    clearTimeout(this.toastTimer);
    this.toastTimer = setTimeout(() => toast.classList.remove('show'), 3200);
  }

  // --- Modal dialogs (replace native alert/confirm/prompt) ---

  alertDialog(message, title = 'SketchQuoder') {
    return this.showDialog({ message, title, buttons: ['OK'] }).then(() => {});
  }

  confirmDialog(message, title = 'SketchQuoder') {
    return this.showDialog({ message, title, buttons: ['Cancel', 'OK'] }).then(result => result.ok);
  }

  promptDialog(message, defaultValue = '', title = 'SketchQuoder') {
    return this.showDialog({ message, title, input: true, inputValue: String(defaultValue ?? ''), buttons: ['Cancel', 'OK'] })
      .then(result => result.ok ? result.value : null);
  }

  showDialog({ message, title = 'SketchQuoder', input = false, inputValue = '', buttons = ['OK'] }) {
    document.querySelector('.app-dialog-overlay')?.remove();
    const overlay = document.createElement('div');
    overlay.className = 'app-dialog-overlay';
    const hasCancel = buttons.includes('Cancel');
    const okLabel = buttons[buttons.length - 1] || 'OK';
    overlay.innerHTML = `
      <div class="app-dialog" role="dialog" aria-modal="true" aria-labelledby="app-dialog-title">
        <div class="app-dialog-title" id="app-dialog-title">${this.escapeHTML(title)}</div>
        <div class="app-dialog-message">${this.escapeHTML(message)}</div>
        ${input ? `<input type="text" class="app-dialog-input" spellcheck="false" value="${this.escapeHTML(inputValue)}">` : ''}
        <div class="app-dialog-actions">
          ${hasCancel ? '<button type="button" class="app-dialog-btn" data-dialog-result="cancel">Cancel</button>' : ''}
          <button type="button" class="app-dialog-btn primary" data-dialog-result="ok">${this.escapeHTML(okLabel)}</button>
        </div>
      </div>`;
    document.body.appendChild(overlay);
    const previousFocus = document.activeElement;
    const inputEl = overlay.querySelector('.app-dialog-input');
    const okBtn = overlay.querySelector('[data-dialog-result="ok"]');
    const cancelBtn = overlay.querySelector('[data-dialog-result="cancel"]');

    return new Promise(resolve => {
      const close = result => {
        const value = inputEl ? inputEl.value : null;
        overlay.remove();
        document.removeEventListener('keydown', onKey, true);
        if (previousFocus && previousFocus.focus) previousFocus.focus?.();
        resolve({ ok: result === 'ok', value });
      };
      const onKey = event => {
        if (event.key === 'Escape') {
          event.stopPropagation();
          event.preventDefault();
          close(hasCancel ? 'cancel' : 'ok');
        } else if (event.key === 'Enter') {
          event.stopPropagation();
          event.preventDefault();
          if (document.activeElement === cancelBtn) close('cancel');
          else close('ok');
        }
      };
      document.addEventListener('keydown', onKey, true);
      okBtn.addEventListener('click', () => close('ok'));
      cancelBtn?.addEventListener('click', () => close('cancel'));
      overlay.addEventListener('mousedown', event => {
        if (event.target === overlay) close(hasCancel ? 'cancel' : 'ok');
      });
      if (inputEl) { inputEl.focus(); inputEl.select(); }
      else okBtn.focus();
    });
  }

  showCommandPopup(anchor, items, className = 'command-popup') {
    document.querySelectorAll('.command-popup,.sketch-context-menu').forEach(item => item.remove());
    const popup = document.createElement('div');
    popup.className = className;
    popup.innerHTML = items.map(item => item.separator ? '<hr>' :
      `<button type="button" data-action="${this.escapeHTML(item.action)}"><i class="ph ${item.icon || 'ph-caret-right'}"></i><span>${this.escapeHTML(item.label)}</span></button>`).join('');
    document.body.appendChild(popup);
    const rect = anchor?.getBoundingClientRect?.() || { left:8, bottom:8 };
    popup.style.left = `${Math.min(innerWidth - popup.offsetWidth - 8, Math.max(8, rect.left))}px`;
    popup.style.top = `${Math.min(innerHeight - popup.offsetHeight - 8, Math.max(8, rect.bottom + 2))}px`;
    popup.querySelectorAll('[data-action]').forEach(button => button.addEventListener('click', () => {
      const item = items.find(entry => entry.action === button.dataset.action);
      popup.remove();
      item?.run?.();
    }));
    setTimeout(() => document.addEventListener('pointerdown', event => {
      if (!popup.contains(event.target)) popup.remove();
    }, { once:true }), 0);
    return popup;
  }

  showSelectMenu(anchor) {
    this.showCommandPopup(anchor, [
      ['all','Select All','ph-selection-all'], ['areas','Select Areas','ph-polygon'], ['text','Select Text','ph-text-t'],
      ['dimensions','Select Dimensions','ph-ruler'], ['symbols','Select Symbols','ph-couch'], ['freeform','Select Freeform Lines','ph-line-segment']
    ].map(([action,label,icon]) => ({ action,label,icon,run:() => this.app.selectByType(action) })));
  }

  showFlipMenu(anchor) {
    this.showCommandPopup(anchor, [
      { action:'horizontal', label:'Flip Horizontal', icon:'ph-arrows-left-right', run:() => this.app.flipSketch('horizontal') },
      { action:'vertical', label:'Flip Vertical', icon:'ph-arrows-down-up', run:() => this.app.flipSketch('vertical') }
    ]);
  }

  showWallShapeMenu(anchor, wallType) {
    const make = async shape => {
      if (shape === 'line') { this.app.setMode(`draw-${wallType}`); return; }
      const width = Number(await this.promptDialog(`${shape === 'circle' ? 'Diameter' : 'Width'} in feet`, '20'));
      if (!width) return;
      const height = shape === 'circle' ? width : Number(await this.promptDialog('Height in feet', '20'));
      if (!height) return;
      const sides = shape === 'polygon' ? Number(await this.promptDialog('Number of sides', '8')) : undefined;
      this.app.addWallShape(shape, wallType, { width, height, sides });
    };
    this.showCommandPopup(anchor, [
      { action:'line',label:'Line',icon:'ph-line-segment',run:() => make('line') },
      { action:'rectangle',label:'Rectangle',icon:'ph-rectangle',run:() => make('rectangle') },
      { action:'circle',label:'Circle',icon:'ph-circle',run:() => make('circle') },
      { action:'polygon',label:'Polygon',icon:'ph-polygon',run:() => make('polygon') }
    ]);
  }

  showGeoMenu(anchor) {
    this.showCommandPopup(anchor, [
      { action:'load',label:'Load Map Image…',icon:'ph-image',run:() => document.getElementById('photo-file-input')?.click() },
      { action:'coordinate',label:'Auto Locate by Coordinate…',icon:'ph-crosshair',run:() => this.showGeoReferenceDialog('coordinate') },
      { action:'address',label:'Auto Locate by Address…',icon:'ph-map-pin',run:() => this.showGeoReferenceDialog('address') }
    ]);
  }

  async showGeoReferenceDialog(mode) {
    const info = this.app.state.subjectInfo || {};
    const value = mode === 'coordinate'
      ? (String(info.gpsCoordinates || '').trim() || formatGpsPair(info.gpsLatitude, info.gpsLongitude))
      : (info.address || '');
    const entered = await this.promptDialog(mode === 'coordinate' ? 'Latitude, Longitude' : 'Property address', value);
    if (!entered) return;
    this.app.state.subjectInfo = { ...info };
    if (mode === 'coordinate') {
      const pair = parseGpsPair(entered);
      if (!pair) {
        this.notify('Enter coordinates as decimal degrees (47.7737, -122.6539) or DMS (47°46\'24.92"N 122°39\'14.18"W).');
        return;
      }
      this.app.state.subjectInfo.gpsCoordinates = entered.trim();
      this.app.state.subjectInfo.gpsLatitude = String(pair.lat);
      this.app.state.subjectInfo.gpsLongitude = String(pair.lon);
      this.notify(`Georeference saved at ${pair.lat}, ${pair.lon}. Load a map image to trace it.`);
    } else {
      this.app.state.subjectInfo.address = entered.trim();
      this.notify('Georeference address saved. Load a map image to trace it.');
    }
  }

  showCanvasContextMenu(clientX, clientY) {
    const items = [];
    const hostedSelected = this.app.state.symbols.filter(sym =>
      this.app.state.selectedSymbolIds.has(sym.id) && sym.wallId);
    if (hostedSelected.length) {
      items.push(
        { action:'flip-opening',label:'Flip Swing',icon:'ph-arrows-left-right',run:() => this.app.flipOpening() },
        { action:'opening-wider',label:'Opening Wider',icon:'ph-plus-circle',run:() => this.app.adjustOpeningWidth(0.5) },
        { action:'opening-narrower',label:'Opening Narrower',icon:'ph-minus-circle',run:() => this.app.adjustOpeningWidth(-0.5) },
        { separator:true }
      );
    }
    const areaUnderCursor = this.app.areaWorkflow?.polyAt?.(clientX, clientY);
    if (areaUnderCursor) items.push(
      { action:'edit-area-definition',label:'Edit Definition...',icon:'ph-tag',run:() => this.app.areaWorkflow.open(areaUnderCursor, { x:clientX, y:clientY }) },
      { separator:true }
    );
    items.push(
      { action:'make-negative',label:'Make Negative',icon:'ph-minus-circle',run:() => this.app.makeNegative() },
      { separator:true },
      { action:'delete',label:'Delete',icon:'ph-trash',run:() => this.app.deleteSelected() },
      { action:'cut',label:'Cut',icon:'ph-scissors',run:() => this.app.cutSelected() },
      { action:'copy',label:'Copy',icon:'ph-copy',run:() => this.app.copySelected() },
      { action:'paste',label:'Paste',icon:'ph-clipboard-text',run:() => this.app.paste() },
      { separator:true },
      { action:'send-to-back',label:'Send to Back',icon:'ph-arrow-line-down',run:() => this.app.sendToBack() },
      { action:'bring-to-front',label:'Bring to Front',icon:'ph-arrow-line-up',run:() => this.app.bringToFront() },
      { separator:true },
      { action:'flip-h',label:'Flip Horz',icon:'ph-arrows-left-right',run:() => this.app.flipSelectedAreas('horizontal') },
      { action:'flip-v',label:'Flip Vert',icon:'ph-arrows-down-up',run:() => this.app.flipSelectedAreas('vertical') },
      { action:'rotate',label:'Rotate 90°',icon:'ph-arrow-clockwise',run:() => this.app.rotateSelected(90) },
      { separator:true },
      { action:'clone',label:'Clone Area',icon:'ph-copy-simple',run:() => this.app.cloneSelected() },
      { action:'auto-subtract',label:'Auto-Subtract Area',icon:'ph-selection-background',run:() => this.app.toggleSelectedAreaAutoSubtract() },
      { separator:true },
      { action:'properties',label:'Properties…',icon:'ph-sliders',run:() => this.app.state.selectedPolyIds.size ? this.showSelectedAreaProperties() : this.notify('Use Edit Tools to change the selected item.') }
    );
    const anchor = { getBoundingClientRect:() => ({ left:clientX, bottom:clientY }) };
    return this.showCommandPopup(anchor, items, 'sketch-context-menu');
  }

  showPrintPreview() {
    document.getElementById('print-preview-overlay')?.remove();
    const model = this.app.buildExportModel();
    const overlay = document.createElement('div');
    overlay.id = 'print-preview-overlay';
    overlay.className = 'print-preview-overlay';
    const paperName = model.layout?.paperSize === 'a4' ? 'A4' : `${(model.layout?.paperSize || 'legal')[0].toUpperCase()}${(model.layout?.paperSize || 'legal').slice(1)}`;
    overlay.innerHTML = `<style>${window.SketchReport.REPORT_CSS}</style><div class="print-preview-toolbar report-preview-only"><button data-print><i class="ph ph-printer"></i> Print / Save PDF</button><button data-close>Close Preview</button><span>${model.pages.length} ${model.pages.length === 1 ? 'page' : 'pages'} &middot; ${paperName} portrait</span></div><div class="print-preview-scroll">${window.SketchReport.render(model, this.app.areaCodes)}</div>`;
    document.body.appendChild(overlay);
    overlay.querySelector('[data-close]').addEventListener('click', () => overlay.remove());
    overlay.querySelector('[data-print]').addEventListener('click', () => this.app.exportPDF());
  }

  // --- Keyboard Shortcuts Cheat Sheet (F1 / Help tab) ---
  showShortcuts() {
    document.getElementById('shortcuts-overlay')?.remove();
    const groups = [
      ['File', [
        ['Ctrl + N', 'New sketch'], ['Ctrl + O or F3', 'Open sketch'], ['Ctrl + S or F2', 'Save'],
        ['Ctrl + F12 or F12', 'Save As'], ['Ctrl + F1', 'Email support']
      ]],
      ['Edit', [
        ['Ctrl + Z', 'Undo'], ['Ctrl + Shift + Z or Ctrl + Y', 'Redo'],
        ['Ctrl + X / C / V', 'Cut / Copy / Paste'], ['Ctrl + A', 'Select all'],
        ['Delete', 'Delete selection'], ['Esc', 'Cancel draw / clear selection / exit tool'],
        ['Arrow keys', 'Nudge selection (Shift = fine)']
      ]],
      ['Drawing', [
        ['E', 'Draw exterior walls'], ['I or F5', 'Draw interior walls'],
        ['Enter', 'Start exterior / commit wall'], ['10 then arrow', 'Draw 10 ft in arrow direction'],
        ['10x5', 'Rise 10, run 5 diagonal'], ['10<45', '10 ft at 45 degrees'],
        ['10l / 10r', 'Turn left / right (default 90 degrees)'], ['Tab', 'Decimal point while typing'],
        ['A', 'Auto-close area while drawing'], ['B', 'Bay window mirror'],
        ['J', 'Jump cursor to nearest endpoint'], ['D', 'Toggle curve degree/height mode'],
        ['Insert', 'Insert vertex tool'], ['Backspace', 'Erase last typed character']
      ]],
      ['Measure', [
        ['Click', 'Add a point to the run'], ['Shift + drag', 'Lock the leg to 45 degrees'],
        ['Enter / double-click', 'Finish the run'], ['Shift + Enter', 'Close the run and read its area'],
        ['Backspace', 'Undo the last point'], ['Delete', 'Clear all measurements'],
        ['Ctrl + C', 'Copy measurements as text'], ['Esc', 'Cancel run, clear, then exit']
      ]],
      ['Selection', [
        ['P', 'Select tool'], ['Shift + R', 'Rotate selection 5 degrees'],
        ['Ctrl + Shift + R', 'Rotate selection 1 degree'], ['H / V', 'Orient selection horizontal / vertical'],
        ['+ / -', 'Scale selection up / down'], ['Space (tap)', 'Toggle dimension labels']
      ]],
      ['View', [
        ['G', 'Toggle grid'], ['C', 'Center sketch'], ['Shift + C', 'Center & fit'],
        ['Space (hold)', 'Temporary pan'], ['F / M', 'Flip sketch horizontal / vertical'],
        ['L / R', 'Rotate sketch left / right 90 degrees']
      ]],
      ['Panels', [
        ['F4', 'Define Area panel'], ['F6', 'Calculations panel'],
        ['S', 'Symbol library'], ['T', 'Text library'], ['F1', 'This cheat sheet']
      ]]
    ];
    const overlay = document.createElement('div');
    overlay.id = 'shortcuts-overlay';
    overlay.className = 'app-dialog-overlay';
    overlay.innerHTML = `
      <div class="shortcuts-dialog" role="dialog" aria-modal="true" aria-labelledby="shortcuts-title">
        <div class="app-dialog-title" id="shortcuts-title">Keyboard Shortcuts</div>
        <div class="shortcuts-grid">
          ${groups.map(([name, rows]) => `
            <div class="shortcut-group">
              <div class="shortcut-group-title">${this.escapeHTML(name)}</div>
              ${rows.map(([key, desc]) => `
                <div class="shortcut-row">
                  <span class="shortcut-key">${this.escapeHTML(key)}</span>
                  <span class="shortcut-desc">${this.escapeHTML(desc)}</span>
                </div>`).join('')}
            </div>`).join('')}
        </div>
        <div class="app-dialog-actions">
          <button type="button" class="app-dialog-btn primary" data-close>Close</button>
        </div>
      </div>`;
    document.body.appendChild(overlay);
    const close = () => {
      overlay.remove();
      document.removeEventListener('keydown', onKey, true);
    };
    const onKey = event => {
      if (event.key === 'Escape' || event.key === 'F1') {
        event.stopPropagation();
        event.preventDefault();
        close();
      }
    };
    document.addEventListener('keydown', onKey, true);
    overlay.querySelector('[data-close]').addEventListener('click', close);
    overlay.addEventListener('mousedown', event => { if (event.target === overlay) close(); });
    overlay.querySelector('[data-close]').focus();
  }

  // Parse a unit-aware length field into feet (world units). Accepts decimal
  // feet ("10", "10.5") or feet-and-inches ("10'6\""). For metric sketches the
  // value is interpreted as metres and converted to feet.
  parseShapeLength(text) {
    const str = String(text == null ? '' : text).trim();
    if (!str) return NaN;
    const metric = (this.app.state.options || {}).unitSystem === 'metric';
    let feet;
    const m = str.match(/^(-?\d+(?:\.\d+)?)\s*(?:'\s*(\d+(?:\.\d+)?)\s*"?)?$/);
    if (m && str.indexOf("'") !== -1) {
      feet = parseFloat(m[1]) + (parseFloat(m[2] || '0') / 12);
    } else {
      feet = parseFloat(str);
    }
    if (!isFinite(feet)) return NaN;
    return metric ? feet / 0.3048 : feet;
  }

  // --- Parametric Room Templates (Shapes dialog) ---
  showShapes() {
    document.getElementById('shapes-overlay')?.remove();
    const options = this.app.state.options || {};
    const overlay = document.createElement('div');
    overlay.id = 'shapes-overlay';
    overlay.className = 'app-dialog-overlay';
    overlay.innerHTML = `
      <div class="app-dialog shapes-dialog" role="dialog" aria-modal="true" aria-labelledby="shapes-title">
        <div class="app-dialog-title" id="shapes-title">Insert Room Shape</div>
        <div class="shapes-form">
          <label class="shapes-row"><span>Shape</span>
            <select id="shape-kind">
              <option value="rectangle">Rectangle</option>
              <option value="lshape">L-Shape</option>
            </select>
          </label>
          <label class="shapes-row"><span>Width</span><input id="shape-width" type="text" value="12"></label>
          <label class="shapes-row"><span>Depth</span><input id="shape-depth" type="text" value="10"></label>
          <label class="shapes-row shape-l-only"><span>Notch Width</span><input id="shape-cutw" type="text" value="4"></label>
          <label class="shapes-row shape-l-only"><span>Notch Depth</span><input id="shape-cutd" type="text" value="4"></label>
          <label class="shapes-row"><span>Orientation</span>
            <select id="shape-rotation">
              <option value="0">0&#176;</option><option value="90">90&#176;</option>
              <option value="180">180&#176;</option><option value="270">270&#176;</option>
            </select>
          </label>
          <label class="shapes-row"><span>Define as area</span><input id="shape-define" type="checkbox" ${options.quickShapeDefine ? 'checked' : ''}></label>
        </div>
        <div class="app-dialog-actions">
          <button type="button" class="app-dialog-btn" data-close>Cancel</button>
          <button type="button" class="app-dialog-btn primary" data-place>Place</button>
        </div>
      </div>`;
    document.body.appendChild(overlay);
    const dialog = overlay.querySelector('.shapes-dialog');
    const kindSel = overlay.querySelector('#shape-kind');
    const syncKind = () => dialog.classList.toggle('shapes-l', kindSel.value === 'lshape');
    kindSel.addEventListener('change', syncKind);
    syncKind();
    const close = () => {
      overlay.remove();
      document.removeEventListener('keydown', onKey, true);
    };
    const onKey = event => {
      if (event.key === 'Escape') { event.stopPropagation(); event.preventDefault(); close(); }
    };
    document.addEventListener('keydown', onKey, true);
    overlay.querySelector('[data-close]').addEventListener('click', close);
    overlay.addEventListener('mousedown', event => { if (event.target === overlay) close(); });
    overlay.querySelector('[data-place]').addEventListener('click', () => {
      const width = this.parseShapeLength(overlay.querySelector('#shape-width').value);
      const depth = this.parseShapeLength(overlay.querySelector('#shape-depth').value);
      const cutWidth = this.parseShapeLength(overlay.querySelector('#shape-cutw').value);
      const cutDepth = this.parseShapeLength(overlay.querySelector('#shape-cutd').value);
      if (!(width > 0) || !(depth > 0)) { this.notify('Enter a valid width and depth.'); return; }
      this.app.state.pendingShape = {
        kind: kindSel.value,
        width, depth,
        cutWidth: kindSel.value === 'lshape' ? cutWidth : 0,
        cutDepth: kindSel.value === 'lshape' ? cutDepth : 0,
        rotation: Number(overlay.querySelector('#shape-rotation').value) || 0,
        defineAsArea: overlay.querySelector('#shape-define').checked
      };
      close();
      this.app.setMode('place-shape');
    });
    overlay.querySelector('#shape-width').focus();
  }

  async selectByType() {
    const choice = await this.promptDialog('Select: All, Areas, Text, Dimensions, Symbols, or Freeform Lines', 'All');
    if (!choice) return;
    const normalized = choice.trim().toLowerCase();
    const aliases = {
      all: 'all', area: 'areas', areas: 'areas', text: 'text', labels: 'text',
      dimensions: 'dimensions', dimension: 'dimensions', symbols: 'symbols', symbol: 'symbols',
      freeform: 'freeform', lines: 'freeform', 'freeform lines': 'freeform'
    };
    this.app.selectByType(aliases[normalized] || 'all');
  }

  editSubjectInfo() {
    document.getElementById('subject-info-overlay')?.remove();
    const legacy = typeof this.app.state.subjectInfo === 'string' ? this.app.state.subjectInfo : '';
    const info = typeof this.app.state.subjectInfo === 'object' && this.app.state.subjectInfo
      ? this.app.state.subjectInfo : { generalComments: legacy };
    const fields = [
      ['parcelNo', 'Parcel No.'], ['fileNo', 'File No.'], ['propertyAddress', 'Property Address'],
      ['city', 'City'], ['state', 'State'], ['zipCode', 'ZipCode'], ['owner', 'Owner'],
      ['county', 'County'], ['client', 'Client'], ['clientAddress', 'Client Address'],
      ['appraiserName', 'Appraiser Name'], ['inspectionDate', 'Inspection Date'],
      ['gpsCoordinates', 'GPS Coordinates']
    ];
    const gpsValue = String(info.gpsCoordinates || '').trim()
      || formatGpsPair(info.gpsLatitude, info.gpsLongitude);
    const comments = Array.isArray(info.commentTables) ? info.commentTables : ['', '', ''];
    const additional = Array.isArray(info.additionalFields) ? info.additionalFields : [];
    const overlay = document.createElement('div');
    overlay.id = 'subject-info-overlay';
    overlay.className = 'sidebar-manager-overlay';
    overlay.innerHTML = `
      <form class="sidebar-manager-dialog subject-info-dialog" role="dialog" aria-modal="true" aria-labelledby="subject-info-title">
        <div class="sidebar-manager-title"><strong id="subject-info-title">Subject Info</strong><button type="button" data-close>&times;</button></div>
        <div class="subject-tabs" role="tablist">
          <button type="button" class="active" data-subject-tab="parcel">Parcel Info</button>
          <button type="button" data-subject-tab="comments">General Comments</button>
          <button type="button" data-subject-tab="tables">Comment Tables</button>
        </div>
        <section class="subject-panel active" data-subject-panel="parcel">
          ${this.app.state.subjectInfoAutoFilled ? '' : `<div class="subject-autofill" data-autofill-row><button type="button" data-auto-fill><i class="ph ph-arrow-square-in"></i>Auto-Fill</button><span>Pull subject, client and appraiser details from the open suite report.</span></div>`}
          <div class="subject-field-grid">${fields.map(([name, label]) => `<label><span>${label}</span><input name="${name}" value="${this.escapeHTML(name === 'gpsCoordinates' ? gpsValue : info[name] || '')}" ${name === 'inspectionDate' ? 'type="date"' : ''}${name === 'gpsCoordinates' ? ' placeholder="47.7737, -122.6539 or 47°46&#39;24.92&quot;N 122°39&#39;14.18&quot;W"' : ''}></label>`).join('')}</div>
          <div class="subject-metadata"><span>Created By: Appraisal Sketch</span><span>Modified By: Appraisal Sketch</span><span>Format Version: Sketch-Web</span></div>
          <details class="subject-additional"><summary>Expand additional fields</summary>
            <div class="subject-additional-header"><span>Name</span><span>Value</span></div>
            ${[0,1,2].map(index => `<div><input name="extraName${index}" value="${this.escapeHTML(additional[index]?.name || '')}"><input name="extraValue${index}" value="${this.escapeHTML(additional[index]?.value || '')}"></div>`).join('')}
          </details>
        </section>
        <section class="subject-panel" data-subject-panel="comments">
          <label class="subject-comments"><span>Comments</span><textarea name="generalComments">${this.escapeHTML(info.generalComments || '')}</textarea></label>
          <div class="subject-default-actions"><button type="button" data-save-default>Save Default</button><button type="button" data-apply-default>Apply Default</button></div>
        </section>
        <section class="subject-panel" data-subject-panel="tables">
          ${[0,1,2].map(index => `<label class="comment-table"><span>Comment Table ${index + 1}</span><textarea name="commentTable${index}">${this.escapeHTML(Array.isArray(comments[index]) ? comments[index].join('\n') : (comments[index] || ''))}</textarea></label>`).join('')}
        </section>
        <div class="sidebar-manager-actions"><button type="button" data-close>Cancel</button><button type="submit">OK</button></div>
      </form>`;
    document.body.appendChild(overlay);
    overlay.querySelectorAll('[data-close]').forEach(button => button.addEventListener('click', () => overlay.remove()));
    overlay.querySelectorAll('[data-subject-tab]').forEach(button => button.addEventListener('click', () => {
      overlay.querySelectorAll('[data-subject-tab]').forEach(item => item.classList.toggle('active', item === button));
      overlay.querySelectorAll('[data-subject-panel]').forEach(panel => panel.classList.toggle('active', panel.dataset.subjectPanel === button.dataset.subjectTab));
    }));
    const autoFillButton = overlay.querySelector('[data-auto-fill]');
    if (autoFillButton) autoFillButton.addEventListener('click', () => this.autoFillSubjectInfo(overlay));
    const commentsArea = overlay.querySelector('[name="generalComments"]');
    overlay.querySelector('[data-save-default]').addEventListener('click', () => {
      try { localStorage.setItem('sketch.defaultSubjectComments', commentsArea.value); } catch (_) { /* unavailable */ }
      this.notify('Default comments saved.');
    });
    overlay.querySelector('[data-apply-default]').addEventListener('click', () => {
      try { commentsArea.value = localStorage.getItem('sketch.defaultSubjectComments') || ''; } catch (_) { commentsArea.value = ''; }
    });
    overlay.querySelector('form').addEventListener('submit', event => {
      event.preventDefault();
      const data = new FormData(event.currentTarget);
      const value = Object.fromEntries(fields.map(([name]) => [name, data.get(name) || '']));
      const gpsPair = parseGpsPair(value.gpsCoordinates);
      if (value.gpsCoordinates && !gpsPair) {
        this.notify('GPS Coordinates must be decimal degrees (47.7737, -122.6539) or DMS (47°46\'24.92"N 122°39\'14.18"W).');
        return;
      }
      value.gpsLatitude = gpsPair ? String(gpsPair.lat) : '';
      value.gpsLongitude = gpsPair ? String(gpsPair.lon) : '';
      value.generalComments = data.get('generalComments') || '';
      value.commentTables = [0,1,2].map(index => String(data.get(`commentTable${index}`) || '').split(/\r?\n/));
      value.additionalFields = [0,1,2].map(index => ({ name: data.get(`extraName${index}`) || '', value: data.get(`extraValue${index}`) || '' })).filter(item => item.name || item.value);
      this.app.state.subjectInfo = value;
      overlay.remove();
      this.notify('Subject information updated.');
    });
  }

  /* On-demand pull of the suite's report subject block for sketches that the
     shell's background sync has not auto-filled yet. */
  autoFillSubjectInfo(overlay) {
    const host = window.parent && window.parent !== window ? window.parent : null;
    if (!host || typeof host.__collectSketchSubjectInfo !== 'function') {
      this.notify('Auto-Fill is available when the sketch is open in the appraisal suite.');
      return;
    }
    let info = null;
    try { info = host.__collectSketchSubjectInfo(); } catch (_) { info = null; }
    if (!info || !Object.keys(info).some(key => info[key] != null && String(info[key]).trim())) {
      this.notify('The suite report has no subject data to auto-fill yet.');
      return;
    }
    mergeSubjectInfo(this.app.state, info);
    this.app.state.subjectInfoAutoFilled = true;
    const model = this.app.state.subjectInfo;
    Object.keys(model).forEach(key => {
      const input = overlay.querySelector(`[name="${key}"]`);
      if (input) input.value = model[key];
    });
    overlay.querySelector('[data-autofill-row]')?.remove();
    this.notify('Subject info auto-filled from the suite report.');
  }

  hsvToRgb(h, s, v) {
    const c = v * s;
    const x = c * (1 - Math.abs((h / 60) % 2 - 1));
    const m = v - c;
    let r, g, b;
    if (h < 60) { r = c; g = x; b = 0; }
    else if (h < 120) { r = x; g = c; b = 0; }
    else if (h < 180) { r = 0; g = c; b = x; }
    else if (h < 240) { r = 0; g = x; b = c; }
    else if (h < 300) { r = x; g = 0; b = c; }
    else { r = c; g = 0; b = x; }
    return { r: Math.round((r + m) * 255), g: Math.round((g + m) * 255), b: Math.round((b + m) * 255) };
  }

  rgbToHsv(r, g, b) {
    const rNorm = r / 255, gNorm = g / 255, bNorm = b / 255;
    const max = Math.max(rNorm, gNorm, bNorm), min = Math.min(rNorm, gNorm, bNorm);
    const d = max - min;
    let h = 0, s = max === 0 ? 0 : d / max, v = max;
    if (d !== 0) {
      if (max === rNorm) h = ((gNorm - bNorm) / d + 6) % 6;
      else if (max === gNorm) h = (bNorm - rNorm) / d + 2;
      else h = (rNorm - gNorm) / d + 4;
      h *= 60;
    }
    return { h, s, v };
  }

  rgbToHex(r, g, b) {
    return '#' + [r, g, b].map(c => Math.max(0, Math.min(255, c)).toString(16).padStart(2, '0')).join('');
  }

  hexToRgb(hex) {
    const match = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex || '');
    return match ? { r: parseInt(match[1], 16), g: parseInt(match[2], 16), b: parseInt(match[3], 16) } : { r: 0, g: 0, b: 0 };
  }

  toggleDrawColorPalette() {
    this.showColorDialog(this.app.state.drawColor, color => {
      this.app.state.drawColor = color;
      const input = document.getElementById('draw-color-input');
      if (input) input.value = color;
      this.app.render();
      this.notify('Drawing color updated.');
    });
  }

  showColorDialog(initialColor, onSelect) {
    const rgb = this.hexToRgb(initialColor);
    let hsv = this.rgbToHsv(rgb.r, rgb.g, rgb.b);

    const overlay = document.createElement('div');
    overlay.className = 'color-dialog-overlay';
    overlay.innerHTML = `
      <div class="color-dialog" role="dialog" aria-label="Color">
        <div class="color-dialog-header">
          <span>Color</span>
          <button type="button" data-close aria-label="Close">&times;</button>
        </div>
        <div class="color-dialog-tabs">
          <button type="button" data-tab="standard" class="active">Standard</button>
          <button type="button" data-tab="custom">Custom</button>
        </div>
        <div class="color-dialog-body">
          <div class="color-dialog-panel active" data-panel="standard">
            <div class="color-dialog-row">
              <div class="color-wheel-wrap">
                <canvas class="color-wheel-canvas" width="200" height="200"></canvas>
                <span class="color-cursor" hidden></span>
              </div>
              <div class="color-dialog-side">
                <div class="color-dialog-preview">
                  <span>New</span>
                  <div class="color-preview-box"></div>
                </div>
              </div>
            </div>
            <div class="color-dialog-presets"></div>
          </div>
          <div class="color-dialog-panel" data-panel="custom">
            <div class="color-dialog-row">
              <div class="color-wheel-wrap" style="width:200px;height:200px;">
                <canvas class="color-sv-canvas" width="200" height="200"></canvas>
                <span class="color-cursor" hidden></span>
              </div>
              <canvas class="color-hue-canvas" width="24" height="200"></canvas>
              <div class="color-dialog-side">
                <div class="color-dialog-preview">
                  <span>New</span>
                  <div class="color-preview-box"></div>
                </div>
              </div>
            </div>
            <div class="color-dialog-rgb">
              <label>Red</label><input type="number" name="r" min="0" max="255">
              <label>Green</label><input type="number" name="g" min="0" max="255">
              <label>Blue</label><input type="number" name="b" min="0" max="255">
            </div>
          </div>
        </div>
        <div class="color-dialog-footer">
          <button type="button" data-close>Cancel</button>
          <button type="submit">OK</button>
        </div>
      </div>
    `;
    document.body.appendChild(overlay);
    const standardWheel = overlay.querySelector('.color-wheel-canvas');
    const standardCursor = overlay.querySelector('[data-panel="standard"] .color-cursor');
    const svCanvas = overlay.querySelector('.color-sv-canvas');
    const svCursor = overlay.querySelector('[data-panel="custom"] .color-cursor');
    const hueCanvas = overlay.querySelector('.color-hue-canvas');
    const previewBoxes = overlay.querySelectorAll('.color-preview-box');
    const presetGrid = overlay.querySelector('.color-dialog-presets');
    const rgbInputs = {
      r: overlay.querySelector('[name="r"]'),
      g: overlay.querySelector('[name="g"]'),
      b: overlay.querySelector('[name="b"]')
    };

    const presets = [
      '#000000','#7f0000','#007f00','#7f7f00','#00007f','#7f007f','#007f7f','#c0c0c0',
      '#ff0000','#00ff00','#ffff00','#0000ff','#ff00ff','#00ffff','#ffffff','#7f7f7f',
      '#800000','#008000','#808000','#000080','#800080','#008080','#666666','#999999',
      '#ff6a6a','#6aff6a','#ffff6a','#6a6aff','#ff6aff','#6affff','#cccccc','#333333',
      '#a52a2a','#d2691e','#ff8c00','#ffd700','#8fbc8f','#2e8b57','#00ced1','#4682b4',
      '#4169e1','#6a5acd','#9932cc','#c71585','#ff1493','#ffa07a','#f5f5dc','#deb887'
    ];

    const updatePreview = () => {
      const { r, g, b } = this.hsvToRgb(hsv.h, hsv.s, hsv.v);
      const hex = this.rgbToHex(r, g, b);
      previewBoxes.forEach(box => box.style.backgroundColor = hex);
      rgbInputs.r.value = r;
      rgbInputs.g.value = g;
      rgbInputs.b.value = b;
    };

    const positionCursor = (cursor, canvas, x, y) => {
      const wrap = canvas.parentElement;
      const cRect = canvas.getBoundingClientRect();
      const wRect = wrap.getBoundingClientRect();
      cursor.hidden = false;
      cursor.style.left = `${x + cRect.left - wRect.left}px`;
      cursor.style.top = `${y + cRect.top - wRect.top}px`;
    };

    const updateStandardCursor = () => {
      const w = standardWheel.width, h = standardWheel.height;
      const cx = w / 2, cy = h / 2;
      const radius = Math.min(cx, cy) - 2;
      const angle = hsv.h * Math.PI / 180;
      const sector = Math.abs(angle) % (Math.PI / 3);
      const hexRadius = radius * Math.cos(Math.PI / 6) / Math.cos(sector - Math.PI / 6);
      const dist = hsv.s * hexRadius;
      const x = cx + Math.cos(angle) * dist;
      const y = cy + Math.sin(angle) * dist;
      positionCursor(standardCursor, standardWheel, x, y);
    };

    const updateSvCursor = () => {
      const w = svCanvas.width, h = svCanvas.height;
      const x = hsv.s * (w - 1);
      const y = (1 - hsv.v) * (h - 1);
      positionCursor(svCursor, svCanvas, x, y);
    };

    const drawStandardWheel = () => {
      const ctx = standardWheel.getContext('2d');
      const w = standardWheel.width, h = standardWheel.height;
      const image = ctx.createImageData(w, h);
      const cx = w / 2, cy = h / 2;
      const radius = Math.min(cx, cy) - 2;
      for (let y = 0; y < h; y++) {
        for (let x = 0; x < w; x++) {
          const dx = x - cx, dy = y - cy;
          const angle = Math.atan2(dy, dx);
          const dist = Math.hypot(dx, dy);
          const sector = Math.abs(angle) % (Math.PI / 3);
          const hexRadius = radius * Math.cos(Math.PI / 6) / Math.cos(sector - Math.PI / 6);
          if (dist <= hexRadius) {
            const hue = (angle * 180 / Math.PI + 360) % 360;
            const sat = Math.min(1, dist / hexRadius);
            const { r, g, b } = this.hsvToRgb(hue, sat, 1);
            const idx = (y * w + x) * 4;
            image.data[idx] = r; image.data[idx + 1] = g; image.data[idx + 2] = b; image.data[idx + 3] = 255;
          }
        }
      }
      ctx.putImageData(image, 0, 0);
    };

    const drawCustom = () => {
      const ctx = svCanvas.getContext('2d');
      const w = svCanvas.width, h = svCanvas.height;
      const image = ctx.createImageData(w, h);
      for (let y = 0; y < h; y++) {
        const v = 1 - y / (h - 1);
        for (let x = 0; x < w; x++) {
          const s = x / (w - 1);
          const { r, g, b } = this.hsvToRgb(hsv.h, s, v);
          const idx = (y * w + x) * 4;
          image.data[idx] = r; image.data[idx + 1] = g; image.data[idx + 2] = b; image.data[idx + 3] = 255;
        }
      }
      ctx.putImageData(image, 0, 0);

      const hCtx = hueCanvas.getContext('2d');
      const hh = hueCanvas.height, hw = hueCanvas.width;
      const hImage = hCtx.createImageData(hw, hh);
      for (let y = 0; y < hh; y++) {
        const hue = (1 - y / (hh - 1)) * 360;
        const { r, g, b } = this.hsvToRgb(hue, 1, 1);
        for (let x = 0; x < hw; x++) {
          const idx = (y * hw + x) * 4;
          hImage.data[idx] = r; hImage.data[idx + 1] = g; hImage.data[idx + 2] = b; hImage.data[idx + 3] = 255;
        }
      }
      hCtx.putImageData(hImage, 0, 0);
    };

    const updateFromStandard = (clientX, clientY) => {
      const rect = standardWheel.getBoundingClientRect();
      const cx = standardWheel.width / 2, cy = standardWheel.height / 2;
      const x = clientX - rect.left;
      const y = clientY - rect.top;
      const dx = x - cx, dy = y - cy;
      const angle = Math.atan2(dy, dx);
      const dist = Math.hypot(dx, dy);
      const radius = Math.min(cx, cy) - 2;
      const sector = Math.abs(angle) % (Math.PI / 3);
      const hexRadius = radius * Math.cos(Math.PI / 6) / Math.cos(sector - Math.PI / 6);
      if (dist <= hexRadius) {
        hsv.h = (angle * 180 / Math.PI + 360) % 360;
        hsv.s = Math.min(1, Math.max(0, dist / hexRadius));
        hsv.v = 1;
        updatePreview();
        updateStandardCursor();
      }
    };

    const updateFromSv = (clientX, clientY) => {
      const rect = svCanvas.getBoundingClientRect();
      const w = svCanvas.width, h = svCanvas.height;
      const x = Math.min(Math.max(clientX - rect.left, 0), w - 1);
      const y = Math.min(Math.max(clientY - rect.top, 0), h - 1);
      hsv.s = x / (w - 1);
      hsv.v = 1 - y / (h - 1);
      updatePreview();
      drawCustom();
      updateSvCursor();
    };

    const updateFromHue = (clientY) => {
      const rect = hueCanvas.getBoundingClientRect();
      const hh = hueCanvas.height;
      const y = Math.min(Math.max(clientY - rect.top, 0), hh - 1);
      hsv.h = (1 - y / (hh - 1)) * 360;
      updatePreview();
      drawCustom();
      updateSvCursor();
    };

    const applyRgb = () => {
      const r = Math.max(0, Math.min(255, parseInt(rgbInputs.r.value, 10) || 0));
      const g = Math.max(0, Math.min(255, parseInt(rgbInputs.g.value, 10) || 0));
      const b = Math.max(0, Math.min(255, parseInt(rgbInputs.b.value, 10) || 0));
      hsv = this.rgbToHsv(r, g, b);
      updatePreview();
      drawCustom();
      updateStandardCursor();
      updateSvCursor();
    };

    presetGrid.innerHTML = presets.map(color =>
      `<button type="button" data-preset="${color}" style="background:${color}" aria-label="${color}"></button>`
    ).join('');

    let activeDrag = null;
    const onMouseMove = event => {
      if (!activeDrag) return;
      if (activeDrag === 'standard') updateFromStandard(event.clientX, event.clientY);
      else if (activeDrag === 'sv') updateFromSv(event.clientX, event.clientY);
      else if (activeDrag === 'hue') updateFromHue(event.clientY);
    };
    const onMouseUp = () => { activeDrag = null; window.removeEventListener('mousemove', onMouseMove); window.removeEventListener('mouseup', onMouseUp); };

    standardWheel.addEventListener('mousedown', event => {
      activeDrag = 'standard';
      updateFromStandard(event.clientX, event.clientY);
      window.addEventListener('mousemove', onMouseMove);
      window.addEventListener('mouseup', onMouseUp);
    });

    svCanvas.addEventListener('mousedown', event => {
      activeDrag = 'sv';
      updateFromSv(event.clientX, event.clientY);
      window.addEventListener('mousemove', onMouseMove);
      window.addEventListener('mouseup', onMouseUp);
    });

    hueCanvas.addEventListener('mousedown', event => {
      activeDrag = 'hue';
      updateFromHue(event.clientY);
      window.addEventListener('mousemove', onMouseMove);
      window.addEventListener('mouseup', onMouseUp);
    });

    presetGrid.querySelectorAll('[data-preset]').forEach(btn => {
      btn.addEventListener('click', () => {
        const rgb = this.hexToRgb(btn.dataset.preset);
        hsv = this.rgbToHsv(rgb.r, rgb.g, rgb.b);
        updatePreview();
        drawCustom();
        updateStandardCursor();
        updateSvCursor();
      });
    });

    Object.values(rgbInputs).forEach(input => {
      input.addEventListener('change', applyRgb);
      input.addEventListener('input', applyRgb);
    });

    overlay.querySelectorAll('[data-tab]').forEach(tab => {
      tab.addEventListener('click', () => {
        overlay.querySelectorAll('[data-tab]').forEach(t => t.classList.remove('active'));
        tab.classList.add('active');
        overlay.querySelectorAll('.color-dialog-panel').forEach(panel => {
          panel.classList.toggle('active', panel.dataset.panel === tab.dataset.tab);
        });
      });
    });

    const close = () => overlay.remove();
    const confirmColor = () => {
      const { r, g, b } = this.hsvToRgb(hsv.h, hsv.s, hsv.v);
      onSelect(this.rgbToHex(r, g, b));
      close();
    };

    overlay.querySelector('button[type="submit"]').addEventListener('click', event => {
      event.preventDefault();
      confirmColor();
    });
    overlay.querySelectorAll('[data-close]').forEach(btn => btn.addEventListener('click', close));
    overlay.addEventListener('click', event => { if (event.target === overlay) close(); });

    drawStandardWheel();
    drawCustom();
    updatePreview();
    updateStandardCursor();
    updateSvCursor();
  }

  editAreaAttributes() {
    const selected = this.app.state.polygons.filter(poly => this.app.state.selectedPolyIds.has(poly.id));
    if (selected.length === 0) {
      this.notify('Select an area before editing its attributes.');
      return;
    }
    this.showSelectedAreaAttributes(false);
  }

  closeEditorDialog(overlay) {
    overlay?.remove();
  }

  showSelectedAreaProperties() {
    const selected = this.app.state.polygons.filter(poly => this.app.state.selectedPolyIds.has(poly.id));
    if (!selected.length) { this.notify('Select an area before editing its properties.'); return; }
    const area = selected[0];
    const post = { ...this.app.state.autoPost, dims: this.app.state.dimensionsEnabled, ...(area.autoPost || {}) };
    const options = (this.app.areaCodes || []).map(code =>
      `<option value="${this.escapeHTML(code.code)}" ${code.code === area.type ? 'selected' : ''}>${this.escapeHTML(code.code)} - ${this.escapeHTML(code.name)}</option>`
    ).join('');
    const overlay = document.createElement('div');
    overlay.className = 'sidebar-manager-overlay';
    overlay.innerHTML = `
      <form class="sidebar-manager-dialog property-editor-dialog" role="dialog" aria-modal="true">
        <div class="sidebar-manager-title"><strong>Define Area</strong><button type="button" data-close>&times;</button></div>
        <label class="editor-row"><span>Area</span><select name="type">${options}</select></label>
        <label class="editor-row"><span>Name</span><input name="name" value="${this.escapeHTML(area.name || '')}"></label>
        <fieldset><legend>Calculation Adjustments</legend>
          <label class="editor-row"><span>Factor</span><input name="factor" type="number" step="0.01" value="${Math.abs(Number(area.factor) || 1)}"></label>
          <div class="editor-inline"><label><input name="sign" type="radio" value="positive" ${Number(area.factor) >= 0 ? 'checked' : ''}> Positive</label><label><input name="sign" type="radio" value="negative" ${Number(area.factor) < 0 ? 'checked' : ''}> Negative</label></div>
          <label class="editor-row"><span>Auto-Subtract</span><select name="autoSubtract"><option value="auto" ${area.subtractMode !== 'never' ? 'selected' : ''}>Automatic</option><option value="none" ${area.subtractMode === 'never' ? 'selected' : ''}>None</option></select></label>
          <label class="editor-row"><span>Measure To</span><select name="measureTo"><option value="center" ${(area.measureTo || 'center') === 'center' ? 'selected' : ''}>Centerline</option><option value="interior" ${area.measureTo === 'interior' ? 'selected' : ''}>Interior Face</option><option value="exterior" ${area.measureTo === 'exterior' ? 'selected' : ''}>Exterior Face</option></select></label>
        </fieldset>
        <fieldset><legend>Automatic labels</legend>
          <div class="editor-check-grid">
            <label><input name="postCode" type="checkbox" ${post.code ? 'checked' : ''}> Code</label>
            <label><input name="postName" type="checkbox" ${post.name ? 'checked' : ''}> Name</label>
            <label><input name="postDims" type="checkbox" ${post.dims ? 'checked' : ''}> Dims</label>
            <label><input name="postCalcs" type="checkbox" ${post.calcs ? 'checked' : ''}> Calcs</label>
            <label><input name="postBase" type="checkbox" ${post.base ? 'checked' : ''}> Base</label>
            <label><input name="postFactor" type="checkbox" ${post.factor ? 'checked' : ''}> Factor</label>
          </div>
          <label class="editor-row"><span>Suffix</span><input name="suffix" value="${this.escapeHTML(post.suffix || 'sf')}"></label>
        </fieldset>
        <div class="sidebar-manager-actions"><button type="button" data-close>Cancel</button><button type="submit">Apply</button></div>
      </form>`;
    document.body.appendChild(overlay);
    overlay.querySelectorAll('[data-close]').forEach(button => button.addEventListener('click', () => this.closeEditorDialog(overlay)));
    overlay.querySelector('form').addEventListener('submit', event => {
      event.preventDefault();
      const data = new FormData(event.currentTarget);
      const values = Object.fromEntries(data.entries());
      this.app.updateSelectedAreaProperties({
        type: values.type,
        name: values.name,
        factor: (values.sign === 'negative' ? -1 : 1) * Math.max(0, Number(values.factor) || 0),
        autoSubtract: values.autoSubtract !== 'none',
        measureTo: values.measureTo,
        autoPost: {
          code: data.has('postCode'), name: data.has('postName'), dims: data.has('postDims'),
          calcs: data.has('postCalcs'), base: data.has('postBase'), factor: data.has('postFactor'),
          suffix: values.suffix || 'sf'
        }
      });
      this.closeEditorDialog(overlay);
    });
  }

  showSelectedAreaAttributes(editable = true) {
    const selected = this.app.state.polygons.filter(poly => this.app.state.selectedPolyIds.has(poly.id));
    if (!selected.length) { this.notify('Select an area before editing its attributes.'); return; }
    const names = ['planRefNumber', 'floorCode', 'floorCodeText', 'yearBuilt'];
    const labels = ['Plan Ref Number', 'Floor Code', 'FloorCodeText', 'Year Built'];
    const first = selected[0].attributes || {};
    const overlay = document.createElement('div');
    overlay.className = 'sidebar-manager-overlay';
    overlay.innerHTML = `
      <form class="sidebar-manager-dialog property-editor-dialog" role="dialog" aria-modal="true">
        <div class="sidebar-manager-title"><strong>Area Attributes</strong><button type="button" data-close>&times;</button></div>
        <div class="attribute-header"><span>Area</span><span>Attribute</span><span>Value</span></div>
        <div class="attribute-grid">
          ${names.map((name, index) => `<span>${index === 0 ? 'All Selected Areas' : ''}</span><label for="attr-${name}">${labels[index]}</label><input id="attr-${name}" name="${name}" value="${this.escapeHTML(first[name] || '')}" ${editable ? '' : 'readonly'}>`).join('')}
        </div>
        <div class="sidebar-manager-actions"><button type="button" data-close>Cancel</button><button type="submit">OK</button></div>
      </form>`;
    document.body.appendChild(overlay);
    overlay.querySelectorAll('[data-close]').forEach(button => button.addEventListener('click', () => this.closeEditorDialog(overlay)));
    overlay.querySelector('form').addEventListener('submit', event => {
      event.preventDefault();
      if (editable) {
        const data = new FormData(event.currentTarget);
        this.app.updateSelectedAreaAttributes(Object.fromEntries(data.entries()));
      }
      this.closeEditorDialog(overlay);
    });
  }

  editExtendedData() {
    this.notify('Extended Data is password-protected in the desktop application and was intentionally omitted.');
  }

  showAreaDefinitionEditor(code) {
    const definition = this.app.areaCodes.find(item => item.code === code);
    if (!definition) { this.notify('Select an area definition to edit.'); return; }
    const overlay = document.createElement('div');
    overlay.className = 'sidebar-manager-overlay';
    overlay.innerHTML = `
      <form class="sidebar-manager-dialog area-definition-dialog" role="dialog" aria-modal="true">
        <div class="sidebar-manager-title"><strong>Edit ${this.escapeHTML(definition.code)}</strong><button type="button" data-close>&times;</button></div>
        <label class="editor-row"><span>Area name</span><input name="name" value="${this.escapeHTML(definition.name)}" required></label>
        <label class="editor-row"><span>Line color</span><input type="hidden" name="color" value="${definition.color || '#718096'}"><button type="button" class="color-swatch" data-color-for="color" style="background-color: ${definition.color || '#718096'}" aria-label="Choose line color"></button></label>
        <label class="editor-row"><span>Fill color</span><input type="hidden" name="fillColor" value="${definition.fillColor || definition.color || '#718096'}"><button type="button" class="color-swatch" data-color-for="fillColor" style="background-color: ${definition.fillColor || definition.color || '#718096'}" aria-label="Choose fill color"></button></label>
        <label class="editor-row"><span>Fill style</span><select name="fillStyle"><option value="solid">Solid</option><option value="none">None</option><option value="diagonal">Diagonal</option><option value="crosshatch">Crosshatch</option><option value="dots">Dots</option></select></label>
        <label class="editor-row"><span>Fill opacity</span><span class="opacity-editor"><input name="fillOpacity" type="range" min="0" max="100" value="${definition.fillOpacity ?? 10}"><output>${definition.fillOpacity ?? 10}%</output></span></label>
        <div class="area-style-preview" data-preview>Area style preview</div>
        <div class="sidebar-manager-actions"><button type="button" data-close>Cancel</button><button type="submit">Save Permanently</button></div>
      </form>`;
    document.body.appendChild(overlay);
    overlay.querySelector('[name="fillStyle"]').value = definition.fillStyle || 'solid';
    overlay.querySelectorAll('.color-swatch').forEach(button => {
      button.addEventListener('click', () => {
        const input = overlay.querySelector(`[name="${button.dataset.colorFor}"]`);
        this.showColorDialog(input.value, color => {
          input.value = color;
          button.style.backgroundColor = color;
          updatePreview();
        });
      });
    });
    const updatePreview = () => {
      const form = overlay.querySelector('form');
      const opacity = Number(form.elements.fillOpacity.value) / 100;
      const preview = overlay.querySelector('[data-preview]');
      preview.style.borderColor = form.elements.color.value;
      preview.style.backgroundColor = this.hexToRgba(form.elements.fillColor.value, opacity);
      preview.dataset.pattern = form.elements.fillStyle.value;
      overlay.querySelector('output').value = `${form.elements.fillOpacity.value}%`;
    };
    overlay.querySelectorAll('input,select').forEach(input => input.addEventListener('input', updatePreview));
    overlay.querySelectorAll('[data-close]').forEach(button => button.addEventListener('click', () => overlay.remove()));
    overlay.querySelector('form').addEventListener('submit', event => {
      event.preventDefault();
      const data = new FormData(event.currentTarget);
      const nextCodes = this.app.areaCodes.map(item => item.code === code ? {
        ...item, name: String(data.get('name') || item.name).trim(), color: data.get('color'),
        fillColor: data.get('fillColor'), fillStyle: data.get('fillStyle'), fillOpacity: Number(data.get('fillOpacity'))
      } : item);
      this.app.setAreaCodes(nextCodes);
      try { localStorage.setItem('sketch.areaCodes', JSON.stringify(this.app.areaCodes)); } catch (_) { /* unavailable */ }
      this.app.state.polygons.filter(poly => poly.type === code).forEach(poly => {
        poly.name = nextCodes.find(item => item.code === code).name;
        (poly.edges || []).forEach(edge => {
          const line = this.app.state.lines.find(item => item.id === edge.lineId);
          if (line) line.color = data.get('color');
        });
      });
      this.populateAreaCodes(document.getElementById('area-code-search')?.value || '');
      this.app.state.selectedAreaType = code;
      this.app.state.selectedAreaName = String(data.get('name'));
      { const el = document.getElementById('area-name-input'); if (el) el.value = String(data.get('name')); }
      overlay.remove();
      this.notify(`${code} settings saved permanently.`);
      this.app.render();
    });
    updatePreview();
  }

  showAddAreaCode(preferredGroup) {
    const overlay = document.createElement('div');
    overlay.className = 'sidebar-manager-overlay';
    const groups = [...new Set((this.app.areaCodes || []).map(c => c.group))].sort();
    const groupOptions = groups.map(g => `<option value="${this.escapeHTML(g)}">`).join('');
    const selGroup = preferredGroup || groups[0] || 'OTH';
    overlay.innerHTML = `
      <form class="sidebar-manager-dialog area-definition-dialog" role="dialog" aria-modal="true">
        <div class="sidebar-manager-title"><strong>New Area Code</strong><button type="button" data-close>&times;</button></div>
        <label class="editor-row"><span>Code</span><input name="code" value="" required placeholder="e.g. GLA5" autocomplete="off" style="text-transform:uppercase"></label>
        <label class="editor-row"><span>Name</span><input name="name" value="" required placeholder="e.g. Fifth Floor"></label>
        <label class="editor-row"><span>Description</span><input name="description" value="" placeholder="Defaults to name"></label>
        <label class="editor-row"><span>Group</span><input name="group" list="area-group-list" value="${this.escapeHTML(selGroup)}" required placeholder="Group name"></label>
        <datalist id="area-group-list">${groupOptions}</datalist>
        <label class="editor-row"><span>Multiplier</span><input name="multiplier" type="number" step="0.01" value="1" min="0"></label>
        <label class="editor-row"><span>Include in GLA</span><input name="includeInGLA" type="checkbox"></label>
        <label class="editor-row"><span>Line color</span><input type="hidden" name="color" value="#718096"><button type="button" class="color-swatch" data-color-for="color" style="background-color:#718096" aria-label="Choose line color"></button></label>
        <label class="editor-row"><span>Fill color</span><input type="hidden" name="fillColor" value="#718096"><button type="button" class="color-swatch" data-color-for="fillColor" style="background-color:#718096" aria-label="Choose fill color"></button></label>
        <label class="editor-row"><span>Fill style</span><select name="fillStyle"><option value="solid">Solid</option><option value="none">None</option><option value="diagonal">Diagonal</option><option value="crosshatch">Crosshatch</option><option value="dots">Dots</option></select></label>
        <label class="editor-row"><span>Fill opacity</span><span class="opacity-editor"><input name="fillOpacity" type="range" min="0" max="100" value="10"><output>10%</output></span></label>
        <div class="area-style-preview" data-preview>Area style preview</div>
        <div class="sidebar-manager-actions"><button type="button" data-close>Cancel</button><button type="submit">Create Code</button></div>
      </form>`;
    document.body.appendChild(overlay);
    overlay.querySelectorAll('.color-swatch').forEach(button => {
      button.addEventListener('click', () => {
        const input = overlay.querySelector(`[name="${button.dataset.colorFor}"]`);
        this.showColorDialog(input.value, color => {
          input.value = color;
          button.style.backgroundColor = color;
          updatePreview();
        });
      });
    });
    const updatePreview = () => {
      const form = overlay.querySelector('form');
      const opacity = Number(form.elements.fillOpacity.value) / 100;
      const preview = overlay.querySelector('[data-preview]');
      preview.style.borderColor = form.elements.color.value;
      preview.style.backgroundColor = this.hexToRgba(form.elements.fillColor.value, opacity);
      preview.dataset.pattern = form.elements.fillStyle.value;
      overlay.querySelector('output').value = `${form.elements.fillOpacity.value}%`;
    };
    overlay.querySelectorAll('input,select').forEach(input => input.addEventListener('input', updatePreview));
    overlay.querySelectorAll('[data-close]').forEach(button => button.addEventListener('click', () => overlay.remove()));
    overlay.querySelector('form').addEventListener('submit', event => {
      event.preventDefault();
      const data = new FormData(event.currentTarget);
      const code = String(data.get('code') || '').trim().toUpperCase();
      const name = String(data.get('name') || '').trim();
      if (!code) { this.notify('Code is required.'); return; }
      if (!name) { this.notify('Name is required.'); return; }
      if (this.app.areaCodes.some(c => c.code === code)) {
        this.notify(`Area code "${code}" already exists.`);
        return;
      }
      const description = String(data.get('description') || '').trim() || name;
      const group = String(data.get('group') || '').trim().toUpperCase() || 'OTH';
      const multiplier = Math.max(0, Number(data.get('multiplier')) || 1);
      const includeInGLA = data.get('includeInGLA') === 'on';
      const newEntry = window.AreaCodes.normalizeAreaCode({
        code, name, description, group, multiplier, includeInGLA,
        color: data.get('color'), fillColor: data.get('fillColor'),
        fillStyle: data.get('fillStyle'), fillOpacity: Number(data.get('fillOpacity'))
      });
      const nextCodes = [...this.app.areaCodes, newEntry];
      this.app.setAreaCodes(nextCodes);
      try { localStorage.setItem('sketch.areaCodes', JSON.stringify(this.app.areaCodes)); } catch (_) { /* unavailable */ }
      this.app.state.selectedAreaType = code;
      this.app.state.selectedAreaName = name;
      { const el = document.getElementById('area-name-input'); if (el) el.value = name; }
      overlay.remove();
      this.notify(`Area code "${code}" created.`);
    });
    updatePreview();
  }

  hexToRgba(hex, opacity) {
    const value = parseInt(String(hex || '#000000').replace('#', ''), 16);
    return `rgba(${value >> 16}, ${(value >> 8) & 255}, ${value & 255}, ${Math.max(0, Math.min(1, opacity))})`;
  }

  showAreaCodeEditor() {
    document.getElementById('area-code-overlay')?.remove();
    const overlay = document.createElement('div');
    overlay.id = 'area-code-overlay';
    overlay.className = 'breakdown-overlay';
    overlay.innerHTML = `
      <div class="breakdown-dialog area-code-dialog">
        <div class="breakdown-title"><strong>Area code table</strong><button type="button" data-close>&times;</button></div>
        <p>Edit the JSON table, or import/export a reusable code set.</p>
        <textarea spellcheck="false"></textarea>
        <input type="file" accept="application/json,.json" data-file hidden>
        <div class="breakdown-actions">
          <button type="button" data-apply>Apply</button>
          <button type="button" data-import>Import JSON</button>
          <button type="button" data-export>Export JSON</button>
          <button type="button" data-reset>Restore defaults</button>
        </div>
      </div>`;
    const textarea = overlay.querySelector('textarea');
    textarea.value = window.AreaCodes.toJSON(this.app.areaCodes);
    const close = () => overlay.remove();
    overlay.querySelector('[data-close]').onclick = close;
    overlay.querySelector('[data-apply]').onclick = () => {
      try {
        this.app.setAreaCodes(window.AreaCodes.fromJSON(textarea.value));
        this.notify('Area code table updated.');
        close();
      } catch (error) { this.notify(error.message); }
    };
    const fileInput = overlay.querySelector('[data-file]');
    overlay.querySelector('[data-import]').onclick = () => fileInput.click();
    fileInput.onchange = () => {
      const file = fileInput.files && fileInput.files[0];
      if (!file) return;
      const reader = new FileReader();
      reader.onload = () => { textarea.value = String(reader.result || ''); };
      reader.readAsText(file);
    };
    overlay.querySelector('[data-export]').onclick = () => {
      try {
        const body = window.AreaCodes.toJSON(window.AreaCodes.fromJSON(textarea.value));
        const url = URL.createObjectURL(new Blob([body], { type: 'application/json' }));
        const anchor = document.createElement('a');
        anchor.href = url; anchor.download = 'sketch-area-codes.json'; anchor.click();
        URL.revokeObjectURL(url);
      } catch (error) { this.notify(error.message); }
    };
    overlay.querySelector('[data-reset]').onclick = () => {
      textarea.value = window.AreaCodes.toJSON(window.AreaCodes.DEFAULT_AREA_CODES);
    };
    document.body.appendChild(overlay);
  }

  toggleMiniMap() {
    const existing = document.getElementById('mini-map-overlay');
    if (existing) {
      existing.remove();
      return;
    }
    const overlay = document.createElement('div');
    overlay.id = 'mini-map-overlay';
    overlay.className = 'mini-map-overlay';
    const close = document.createElement('button');
    close.type = 'button';
    close.textContent = '×';
    close.title = 'Close Mini-Map';
    close.addEventListener('click', () => overlay.remove());
    const image = document.createElement('img');
    image.alt = 'Sketch mini-map';
    image.src = this.app.canvas.canvas.toDataURL('image/png');
    overlay.append(close, image);
    document.querySelector('.canvas-container')?.appendChild(overlay);
  }

  showCanvasBackgroundMenu(anchor) {
    const current = this.app.state.canvasTheme;
    const popup = this.showCommandPopup(anchor, [
      { action: 'light', label: 'White', icon: 'ph-sun', run: () => { this.app.canvas.setBackgroundTheme('light'); this.notify('Drawing background: White'); } },
      { action: 'dark', label: 'Dark', icon: 'ph-moon', run: () => { this.app.canvas.setBackgroundTheme('dark'); this.notify('Drawing background: Dark'); } },
      { action: 'blue', label: 'Blue', icon: 'ph-drop', run: () => { this.app.canvas.setBackgroundTheme('blue'); this.notify('Drawing background: Blue'); } }
    ], 'command-popup');
    popup.querySelector(`[data-action="${current}"]`)?.classList.add('active');
  }

  applyInterfaceTheme(theme) {
    let valid = theme === 'light' || theme === 'blue' ? theme : 'dark';
    if (theme == null) {
      try {
        const stored = window.localStorage?.getItem('sketch.interfaceTheme');
        valid = stored === 'light' || stored === 'blue' ? stored : 'dark';
      } catch (_) { valid = 'dark'; }
    }
    document.body.classList.remove('dark-theme', 'blue-theme', 'light-theme');
    document.body.classList.add(valid === 'light' ? 'light-theme' : valid === 'blue' ? 'blue-theme' : 'dark-theme');
    try { window.localStorage?.setItem('sketch.interfaceTheme', valid); } catch (_) { /* Storage may be unavailable in embedded contexts. */ }
    return valid;
  }

  showInterfaceThemeMenu(anchor) {
    const current = document.body.classList.contains('blue-theme') ? 'blue' : document.body.classList.contains('light-theme') ? 'light' : 'dark';
    const popup = this.showCommandPopup(anchor, [
      { action: 'light', label: 'Light', icon: 'ph-sun', run: () => { this.applyInterfaceTheme('light'); this.notify('Interface: Light'); } },
      { action: 'dark', label: 'Dark', icon: 'ph-moon', run: () => { this.applyInterfaceTheme('dark'); this.notify('Interface: Dark'); } },
      { action: 'blue', label: 'Blue', icon: 'ph-drop', run: () => { this.applyInterfaceTheme('blue'); this.notify('Interface: Blue'); } }
    ], 'command-popup');
    popup.querySelector(`[data-action="${current}"]`)?.classList.add('active');
  }

  toggleFilterPanel() {
    const existing = document.getElementById('filter-popover');
    if (existing) {
      existing.remove();
      return;
    }
    const popover = document.createElement('div');
    popover.id = 'filter-popover';
    popover.className = 'filter-popover';
    popover.innerHTML = '<strong>Show in sketch</strong>';
    const labels = {
      areas: 'Area fills', exterior: 'Exterior walls', interior: 'Interior lines',
      dimensions: 'Dimensions', labels: 'Text labels', symbols: 'Symbols'
    };
    Object.entries(labels).forEach(([key, label]) => {
      const row = document.createElement('label');
      const checkbox = document.createElement('input');
      checkbox.type = 'checkbox';
      checkbox.checked = this.app.state.filters[key] !== false;
      checkbox.addEventListener('change', () => {
        this.app.state.filters[key] = checkbox.checked;
        this.app.render();
      });
      row.append(checkbox, document.createTextNode(` ${label}`));
      popover.appendChild(row);
    });
    document.body.appendChild(popover);
  }

  showOptions() {
    document.getElementById('options-overlay')?.remove();
    const o = { ...this.defaultOptions, ...(this.app.state.options || {}), defineFirst: this.app.state.defineFirst, gridEnabled: this.app.state.gridEnabled };
    const checked = key => o[key] ? 'checked' : '';
    const selected = (key, value) => String(o[key]) === String(value) ? 'selected' : '';
    const radio = (key, value) => String(o[key]) === String(value) ? 'checked' : '';
    const fonts = ['Arial', 'Calibri', 'Georgia', 'Segoe UI', 'Times New Roman', 'Verdana'];
    const fontOptions = key => fonts.map(font => `<option ${selected(key, font)}>${font}</option>`).join('');
    const sizes = [8,9,10,11,12,14,16,18,20,22,24,28,36,48,72];
    const sizeOptions = key => sizes.map(size => `<option value="${size}" ${selected(key, size)}>${size}</option>`).join('');
    const booleans = ['promptOnDeletion','inkConfirmation','autoPlacement','stripTrailingZeroes','sameColorAsLine','alignmentLines','alignmentPoints','testCloseIndicators','gridEnabled','minimumDimensionEnabled','autoShade','selectionHighlight','autoSaveEnabled','appendPageNumber','disableVideoTutorial','autoHealFiles','dimensionBold','dimensionItalic','dimensionUnderline','textBold','textItalic','textUnderline','autoPostBold','autoPostItalic','autoPostUnderline','circleAreaCalcs','displayOnClose','defineFirst','autoSubtractOverlapped','quickShapeDefine','redefine','panZoomDelete','saveImageWithSketch','splitWallsAtJunctions','wallSolids'];
    const numbers = ['decimalPrecision','dimensionOption','orthoAngle','minimumDimension','autoShadePercent','autoSaveInterval','imageDepth','dimensionFontSize','textFontSize','autoPostFontSize','textBackgroundOpacity','exteriorWidth','interiorWidth','exteriorThickness','interiorThickness','labelOpacity','imageQuality','pointerSize'];
    const collectOptions = form => {
      const data = new FormData(form);
      const next = { ...o };
      booleans.forEach(key => next[key] = data.has(key));
      numbers.forEach(key => next[key] = Number(data.get(key)));
      ['inputMode','unitSystem','stepUnit','areaLabelSuffix','alignmentColor','alignmentStyle','distoDiscovery','distoComPort','distoRounding','distoDrawingMode','imageSize','dimensionFont','textFont','textColor','autoPostFont','textBackground','exteriorColor','interiorColor','witnessBaseColor','witnessInteriorColor','witnessCurveColor'].forEach(key => { if (data.get(key) != null) next[key] = data.get(key); });
      next.imageFormats = ['PNG','JPG','BMP'].filter(format => data.has(`format${format}`)).map(format => format.toLowerCase());
      return next;
    };
    const overlay = document.createElement('div');
    overlay.id = 'options-overlay';
    overlay.className = 'sidebar-manager-overlay';
    overlay.innerHTML = `
      <form class="sidebar-manager-dialog options-dialog" role="dialog" aria-modal="true" aria-labelledby="options-title">
        <div class="sidebar-manager-title"><strong id="options-title">Options</strong><button type="button" data-close>&times;</button></div>
        <div class="options-body">
          <nav class="options-tabs" aria-label="Option categories">
            ${['General','Display','Hardware','System','Fonts','Drawing'].map((label,index) => `<button type="button" class="${index === 0 ? 'active' : ''}" data-option-tab="${label.toLowerCase()}">${label}</button>`).join('')}
          </nav>
          <div class="options-panels">
            <section class="option-panel active" data-option-panel="general">
              <fieldset><legend>Input Options</legend><label>Input mode <select name="inputMode"><option value="pen-ink" ${selected('inputMode','pen-ink')}>Pen as Ink</option><option value="pen-mouse" ${selected('inputMode','pen-mouse')}>Pen as Mouse</option><option value="touch" ${selected('inputMode','touch')}>Touch</option></select></label><label><input type="checkbox" name="promptOnDeletion" ${checked('promptOnDeletion')}> Prompt on Deletion</label></fieldset>
              <fieldset><legend>Ink Options</legend><label><input type="checkbox" name="inkConfirmation" ${checked('inkConfirmation')}> Ink Confirmation</label><label>Ink Recognition <select name="inkRecognition"><option>0.1</option><option>0.25</option><option>0.5</option></select></label></fieldset>
              <fieldset><legend>Unit of Measure</legend><label><input type="radio" name="unitSystem" value="imperial" ${radio('unitSystem','imperial')}> Imperial</label><label><input type="radio" name="unitSystem" value="metric" ${radio('unitSystem','metric')}> Metric</label></fieldset>
              <div class="option-split"><fieldset><legend>Decimal Precision</legend>${[1,.1,.01,.001,.0001].map(value => `<label><input type="radio" name="decimalPrecision" value="${value}" ${radio('decimalPrecision',value)}> ${value}</label>`).join('')}</fieldset><fieldset><legend>Step Unit</legend>${['tenths','half','whole'].map(value => `<label><input type="radio" name="stepUnit" value="${value}" ${radio('stepUnit',value)}> ${value[0].toUpperCase()+value.slice(1)}</label>`).join('')}</fieldset></div>
              <fieldset><legend>Dimension Options</legend><label>Dimension <input type="number" name="dimensionOption" step="0.1" value="${o.dimensionOption}"></label><label>Ortho <input type="number" name="orthoAngle" min="1" max="90" value="${o.orthoAngle}">°</label><label><input type="checkbox" name="autoPlacement" ${checked('autoPlacement')}> Auto Placement</label><label><input type="checkbox" name="stripTrailingZeroes" ${checked('stripTrailingZeroes')}> Strip Trailing Zeroes</label><label><input type="checkbox" name="sameColorAsLine" ${checked('sameColorAsLine')}> Same Color As Line</label><label>Area Label Suffix <input name="areaLabelSuffix" value="${this.escapeHTML(o.areaLabelSuffix)}"></label></fieldset>
            </section>
            <section class="option-panel" data-option-panel="display">
              <fieldset><legend>Alignment Indicators</legend><label><input type="checkbox" name="alignmentLines" ${checked('alignmentLines')}> Lines Visible</label><label><input type="checkbox" name="alignmentPoints" ${checked('alignmentPoints')}> Points Visible</label><label><input type="checkbox" name="testCloseIndicators" ${checked('testCloseIndicators')}> Test Close Indicators</label><label>Color <input type="color" name="alignmentColor" value="${o.alignmentColor}"></label><label>Style <select name="alignmentStyle"><option value="dash" ${selected('alignmentStyle','dash')}>Dash</option><option value="solid" ${selected('alignmentStyle','solid')}>Solid</option></select></label></fieldset>
              <fieldset><legend>Grid</legend><label><input type="checkbox" name="gridEnabled" ${checked('gridEnabled')}> Enabled</label></fieldset>
              <fieldset><legend>Minimum Dimension Properties</legend><label><input type="checkbox" name="minimumDimensionEnabled" ${checked('minimumDimensionEnabled')}> Enabled</label><label>Minimum length <input type="number" name="minimumDimension" min="0" step="0.1" value="${o.minimumDimension}"></label></fieldset>
              <fieldset><legend>Auto-Shade</legend><label><input type="checkbox" name="autoShade" ${checked('autoShade')}> Enabled</label><label>Percent <input type="range" name="autoShadePercent" min="0" max="100" value="${o.autoShadePercent}"></label><label><input type="checkbox" name="selectionHighlight" ${checked('selectionHighlight')}> Selection Highlight</label></fieldset>
            </section>
            <section class="option-panel" data-option-panel="hardware">
              <fieldset><legend>DISTO Discovery</legend><label><input type="radio" name="distoDiscovery" value="automatic" ${radio('distoDiscovery','automatic')}> Automatic</label><label><input type="radio" name="distoDiscovery" value="manual" ${radio('distoDiscovery','manual')}> Manual</label><label>COM Port <input name="distoComPort" value="${this.escapeHTML(o.distoComPort || '')}"></label></fieldset>
              <fieldset><legend>Configuration</legend><label>Rounding <select name="distoRounding"><option value="none">None</option><option value="nearestHalf" ${selected('distoRounding','nearestHalf')}>Nearest Half</option><option value="nearestHundredth" ${selected('distoRounding','nearestHundredth')}>Nearest Hundredth</option><option value="nearestTenth" ${selected('distoRounding','nearestTenth')}>Nearest Tenth</option><option value="nearestWhole" ${selected('distoRounding','nearestWhole')}>Nearest Whole</option></select></label><label>Drawing Mode <select name="distoDrawingMode"><option value="standard">Standard</option><option value="continuous" ${selected('distoDrawingMode','continuous')}>Continuous</option></select></label></fieldset>
            </section>
            <section class="option-panel" data-option-panel="system">
              <fieldset><legend>Auto-Save</legend><label><input type="checkbox" name="autoSaveEnabled" ${checked('autoSaveEnabled')}> Enabled</label><label>Interval (minutes) <input type="number" name="autoSaveInterval" min="1" max="120" value="${o.autoSaveInterval}"></label><small>Auto-save data is kept in this browser.</small></fieldset>
              <fieldset><legend>Image Files</legend><label>Size <select name="imageSize"><option ${selected('imageSize','4800x4800')}>4800x4800</option><option ${selected('imageSize','3200x3200')}>3200x3200</option><option ${selected('imageSize','2048x1536')}>2048x1536</option><option ${selected('imageSize','1920x1080')}>1920x1080</option><option ${selected('imageSize','1280x720')}>1280x720</option></select></label><label><input type="radio" name="imageDepth" value="8" ${radio('imageDepth',8)}> 8-Bit</label><label><input type="radio" name="imageDepth" value="24" ${radio('imageDepth',24)}> 24-Bit</label><label><input type="checkbox" name="appendPageNumber" ${checked('appendPageNumber')}> Append Page Number</label><label><input type="checkbox" name="formatPNG" ${(o.imageFormats||[]).includes('png')?'checked':''}> PNG</label><label><input type="checkbox" name="formatJPG" ${(o.imageFormats||[]).includes('jpg')?'checked':''}> JPG</label><label><input type="checkbox" name="formatBMP" ${(o.imageFormats||[]).includes('bmp')?'checked':''}> BMP</label></fieldset>
              <fieldset><legend>Application</legend><label><input type="checkbox" name="disableVideoTutorial" ${checked('disableVideoTutorial')}> Disable Video Tutorial</label><label><input type="checkbox" name="autoHealFiles" ${checked('autoHealFiles')}> Auto Heal Files</label></fieldset>
            </section>
            <section class="option-panel" data-option-panel="fonts">
              ${[['dimension','Dimension Properties'],['text','Text Label Properties'],['autoPost','Automatic labels Properties']].map(([prefix,title]) => `<fieldset><legend>${title}</legend><label>Font <select name="${prefix}Font">${fontOptions(`${prefix}Font`)}</select></label><label>Size <select name="${prefix}FontSize">${sizeOptions(`${prefix}FontSize`)}</select></label><label><input type="checkbox" name="${prefix}Bold" ${checked(`${prefix}Bold`)}> Bold</label><label><input type="checkbox" name="${prefix}Italic" ${checked(`${prefix}Italic`)}> Italic</label><label><input type="checkbox" name="${prefix}Underline" ${checked(`${prefix}Underline`)}> Underline</label>${prefix === 'text' ? `<label>Color <input type="color" name="textColor" value="${o.textColor}"></label>` : ''}${prefix === 'autoPost' ? `<label><input type="checkbox" name="circleAreaCalcs" ${checked('circleAreaCalcs')}> Circle Area Calcs</label>` : ''}</fieldset>`).join('')}
              <fieldset><legend>Text Background</legend><label>Color <input type="color" name="textBackground" value="${o.textBackground}"></label><label>Opacity <input type="number" name="textBackgroundOpacity" min="0" max="100" value="${o.textBackgroundOpacity}"></label></fieldset>
            </section>
            <section class="option-panel" data-option-panel="drawing">
              <div class="option-split"><fieldset><legend>Area Line Properties</legend><label>Drawing Color <input type="color" name="exteriorColor" value="${o.exteriorColor}"></label><label>Thickness <input type="number" name="exteriorWidth" min="2" max="12" value="${o.exteriorWidth}"></label></fieldset><fieldset><legend>Interior Line Properties</legend><label>Drawing Color <input type="color" name="interiorColor" value="${o.interiorColor}"></label><label>Thickness <input type="number" name="interiorWidth" min="1" max="6" value="${o.interiorWidth}"></label></fieldset></div>
              <fieldset><legend>Wall Solids</legend><label><input type="checkbox" name="wallSolids" ${checked('wallSolids')}> Enable wall solids</label><label>Exterior thickness (in) <input type="number" name="exteriorThickness" min="0" step="0.5" value="${o.exteriorThickness ?? 6}"></label><label>Interior thickness (in) <input type="number" name="interiorThickness" min="0" step="0.5" value="${o.interiorThickness ?? 4}"></label><small>When disabled, walls render as legacy single-line strokes.</small></fieldset>
              <fieldset><legend>Witness Line Properties</legend><label>Base <input type="color" name="witnessBaseColor" value="${o.witnessBaseColor}"></label><label>Interior <input type="color" name="witnessInteriorColor" value="${o.witnessInteriorColor}"></label><label>Curve <input type="color" name="witnessCurveColor" value="${o.witnessCurveColor}"></label><label>Label Opacity <input type="number" name="labelOpacity" min="0" max="100" value="${o.labelOpacity}"></label></fieldset>
              <fieldset><legend>Define Area Properties</legend><label><input type="checkbox" name="displayOnClose" ${checked('displayOnClose')}> Display on Close</label><label><input type="checkbox" name="defineFirst" ${checked('defineFirst')}> Define First</label><label><input type="checkbox" name="autoSubtractOverlapped" ${checked('autoSubtractOverlapped')}> Auto-Subtract Overlapped Areas</label><label><input type="checkbox" name="quickShapeDefine" ${checked('quickShapeDefine')}> Quick-Shape Define</label><label><input type="checkbox" name="redefine" ${checked('redefine')}> Re-Define</label><label><input type="checkbox" name="splitWallsAtJunctions" ${checked('splitWallsAtJunctions')}> Split wall dimensions at area junctions</label></fieldset>
              <fieldset><legend>Keyboard Modifiers</legend><label><input type="checkbox" name="panZoomDelete" ${checked('panZoomDelete')}> Pan-Zoom / Delete</label></fieldset>
              <fieldset><legend>Drawing Pointer</legend><label>Size <select name="pointerSize"><option value="128" ${selected('pointerSize',128)}>128x128</option><option value="64" ${selected('pointerSize',64)}>64x64</option><option value="32" ${selected('pointerSize',32)}>32x32</option></select></label></fieldset>
              <fieldset><legend>Reference image</legend><label><input type="checkbox" name="saveImageWithSketch" ${checked('saveImageWithSketch')}> Save Image With Sketch</label><label>Image Quality <input type="number" name="imageQuality" min="1" max="100" value="${o.imageQuality}"></label></fieldset>
            </section>
          </div>
        </div>
        <div class="options-actions"><button type="button" data-defaults>Restore Defaults</button><button type="button" data-save-defaults>Save as Default</button><span></span><button type="button" data-close>Cancel</button><button type="submit">OK</button></div>
      </form>`;
    document.body.appendChild(overlay);
    overlay.querySelectorAll('[data-close]').forEach(button => button.addEventListener('click', () => overlay.remove()));
    overlay.querySelectorAll('[data-option-tab]').forEach(button => button.addEventListener('click', () => {
      overlay.querySelectorAll('[data-option-tab]').forEach(tab => tab.classList.toggle('active', tab === button));
      overlay.querySelectorAll('[data-option-panel]').forEach(panel => panel.classList.toggle('active', panel.dataset.optionPanel === button.dataset.optionTab));
    }));
    overlay.querySelector('[data-defaults]').addEventListener('click', () => {
      this.app.applyOptions(JSON.parse(JSON.stringify(this.defaultOptions)));
      overlay.remove();
      this.notify('Options restored to defaults.');
    });
    overlay.querySelector('[data-save-defaults]').addEventListener('click', () => {
      const next = collectOptions(overlay.querySelector('form'));
      this.app.applyOptions(next);
      this.defaultOptions = JSON.parse(JSON.stringify(next));
      this.storedDefaultOptions = next;
      try {
        window.localStorage?.setItem('sketch.defaultOptions', JSON.stringify(next));
        this.notify('Current options saved as defaults.');
      } catch (_) {
        this.notify('Current options applied, but could not be saved as defaults.');
      }
    });
    overlay.querySelector('form').addEventListener('submit', event => {
      event.preventDefault();
      this.app.applyOptions(collectOptions(event.currentTarget));
      overlay.remove();
      this.notify('Options applied.');
    });
  }

  setPhotometricsModule(enabled) {
    const tab = document.querySelector('[data-target="tab-photometrics"]');
    if (tab) tab.hidden = !enabled;
    if (!enabled && tab?.classList.contains('active')) {
      document.querySelector('[data-target="tab-home"]')?.click();
    }
    this.notify(`Reference image module ${enabled ? 'activated' : 'deactivated'}.`);
  }

  showPageLayout() {
    document.getElementById('page-layout-overlay')?.remove();
    const current = this.app.normalizePageLayout(this.app.state.pageLayout);
    const formLabels = {
      standard: size => `Standard ${size}`,
      'full-page': size => `Full Page ${size}`,
      'sketch-only': size => `${size} Size Sketch Only`,
      'with-calculations': size => `${size} W/ Calculations`
    };
    const paperLabel = paper => paper === 'a4' ? 'A4' : `${paper[0].toUpperCase()}${paper.slice(1)}`;
    const formOptions = (paper, selected) => Object.entries(formLabels).map(([key, label]) => {
      const value = `${key}-${paper}`;
      return `<option value="${value}" ${selected === value ? 'selected' : ''}>${label(paperLabel(paper))}</option>`;
    }).join('');
    const captionFields = [
      ['sketchPageTitle','Sketch Page Title:'], ['subjectLabel','Subject Label:'],
      ['sketchLabel','Sketch Label:'], ['summaryLabel','Summary Label:'],
      ['commentTable1Label','Comment Table 1 Label:'], ['commentTable2Label','Comment Table 2 Label:'],
      ['commentTable3Label','Comment Table 3 Label:']
    ];
    const overlay = document.createElement('div');
    overlay.id = 'page-layout-overlay';
    overlay.className = 'sidebar-manager-overlay';
    overlay.innerHTML = `<form class="sidebar-manager-dialog page-layout-dialog" role="dialog" aria-modal="true" aria-labelledby="page-layout-title">
      <div class="sidebar-manager-title"><strong id="page-layout-title">Page Layout Settings</strong><button type="button" data-close aria-label="Close">&times;</button></div>
      <div class="page-layout-upper">
        <fieldset><legend>Form Selection</legend>
          <div class="page-size-row"><span>Paper Size:</span>${['legal','letter','a4'].map(paper => `<label><input type="radio" name="paperSize" value="${paper}" ${current.paperSize === paper ? 'checked' : ''}> ${paperLabel(paper)}</label>`).join('')}</div>
          <label class="page-current-form"><span>Current Form:</span><select name="currentForm">${formOptions(current.paperSize, current.currentForm)}</select></label>
        </fieldset>
        <fieldset><legend>Additional Options</legend>
          ${[['showSketchComments','Show Sketch Comments'],['showCommentTables','Show Comment Tables'],['showPageNumbers','Show Page Numbers'],['calculationAddendum','Calculation Addendum'],['deedCallsAddendum','Deed Calls Addendum']].map(([key,label]) => `<label><input type="checkbox" name="${key}" ${current[key] ? 'checked' : ''}> ${label}</label>`).join('')}
        </fieldset>
      </div>
      <fieldset class="page-layout-captions"><legend>Form Captions</legend>${captionFields.map(([key,label]) => `<label><span>${label}</span><input name="${key}" value="${this.escapeHTML(current.captions[key])}"></label>`).join('')}</fieldset>
      <div class="page-layout-actions"><button type="button" data-defaults>Restore Defaults</button><span></span><button type="submit">Apply</button><button type="button" data-close>Cancel</button></div>
    </form>`;
    document.body.appendChild(overlay);
    const form = overlay.querySelector('form');
    const formSelect = form.elements.currentForm;
    const refreshForms = paper => { formSelect.innerHTML = formOptions(paper, `standard-${paper}`); };
    form.querySelectorAll('[name="paperSize"]').forEach(radio => radio.addEventListener('change', () => refreshForms(radio.value)));
    overlay.querySelectorAll('[data-close]').forEach(button => button.addEventListener('click', () => overlay.remove()));
    overlay.querySelector('[data-defaults]').addEventListener('click', () => {
      const defaults = this.app.getDefaultPageLayout();
      form.elements.paperSize.value = defaults.paperSize;
      refreshForms(defaults.paperSize);
      form.elements.currentForm.value = defaults.currentForm;
      ['showSketchComments','showCommentTables','showPageNumbers','calculationAddendum','deedCallsAddendum'].forEach(key => { form.elements[key].checked = defaults[key]; });
      captionFields.forEach(([key]) => { form.elements[key].value = defaults.captions[key]; });
    });
    form.addEventListener('submit', event => {
      event.preventDefault();
      const data = new FormData(form);
      const layout = {
        paperSize: data.get('paperSize'), currentForm: data.get('currentForm'),
        showSketchComments: data.has('showSketchComments'), showCommentTables: data.has('showCommentTables'),
        showPageNumbers: data.has('showPageNumbers'), calculationAddendum: data.has('calculationAddendum'),
        deedCallsAddendum: data.has('deedCallsAddendum'), captions: Object.fromEntries(captionFields.map(([key]) => [key, String(data.get(key) || '')]))
      };
      this.app.setPageLayout(layout);
      overlay.remove();
      this.notify(`Page layout applied: ${paperLabel(layout.paperSize)}.`);
    });
  }

  showTouchPanel() {
    const existing = document.getElementById('touch-panel-floating');
    if (existing) { existing.remove(); return; }
    const panel = document.createElement('div');
    panel.id = 'touch-panel-floating';
    panel.className = 'touch-panel-floating';
    panel.innerHTML = `
      <div class="touch-panel-title"><span>Touch Drawing</span><button type="button" data-touch="close">×</button></div>
      <input type="number" min="0" step="0.1" value="1" aria-label="Distance" class="touch-distance">
      <div class="touch-direction-grid">
        <button type="button" data-touch="up">↑</button>
        <button type="button" data-touch="left">←</button>
        <button type="button" data-touch="down">↓</button>
        <button type="button" data-touch="right">→</button>
      </div>
      <div class="touch-row">
        <button type="button" data-touch="turn-left">Turn Left</button>
        <button type="button" data-touch="turn-right">Turn Right</button>
      </div>
      <div class="touch-row">
        <button type="button" data-touch="curve-left">Curve Left</button>
        <button type="button" data-touch="curve-right">Curve Right</button>
      </div>
      <div class="touch-row">
        <button type="button" data-touch="delete">Delete</button>
        <button type="button" data-touch="insert">Insert Vertex</button>
      </div>`;
    const getDistance = () => Math.max(0.1, parseFloat(panel.querySelector('.touch-distance').value) || 1);
    const direction = {
      up: [0, 1, 0], down: [0, -1, 180], left: [-1, 0, 270], right: [1, 0, 90]
    };
    panel.addEventListener('click', event => {
      const action = event.target.closest('[data-touch]')?.dataset.touch;
      if (!action) return;
      if (action === 'close') { panel.remove(); return; }
      if (direction[action]) {
        if (this.app.state.mode !== 'draw-exterior' && this.app.state.mode !== 'draw-interior') {
          this.app.setMode('draw-exterior');
        }
        if (!this.app.state.penDown) this.app.penDown();
        const [mx, my, angle] = direction[action];
        const distance = getDistance();
        this.app.drawLine({
          x: this.app.state.cursor.x + mx * distance,
          y: this.app.state.cursor.y + my * distance
        }, distance, angle);
      } else if (action === 'turn-left' || action === 'turn-right') {
        this.app.input.buffer = `${getDistance()}${action === 'turn-left' ? 'l' : 'r'}90`;
        this.notify('Turn command loaded. Press Enter to draw.');
      } else if (action === 'curve-left' || action === 'curve-right') {
        this.app.adjustSelectedCurves(action === 'curve-left' ? 15 : -15);
      } else if (action === 'delete') {
        this.app.deleteSelected();
      } else if (action === 'insert') {
        this.app.setMode('insert-vertex');
      }
    });
    document.body.appendChild(panel);
  }

  syncSettingsControls() {
    const dims = document.getElementById('autopost-dims');
    const placement = document.getElementById('dimension-placement');
    const drawColor = document.getElementById('draw-color-input');
    if (dims) dims.checked = this.app.state.dimensionsEnabled;
    if (placement) placement.value = this.app.state.dimensionPlacement;
    if (drawColor) drawColor.value = this.app.state.drawColor;
    const bindings = {
      'autopost-code': 'code', 'autopost-name': 'name', 'autopost-calcs': 'calcs',
      'autopost-base': 'base', 'autopost-factor': 'factor'
    };
    Object.entries(bindings).forEach(([id, key]) => {
      const input = document.getElementById(id);
      if (input) input.checked = Boolean(this.app.state.autoPost[key]);
    });
    const suffix = document.getElementById('autopost-suffix');
    if (suffix) suffix.value = this.app.state.autoPost.suffix || 'sf';
  }

  updateUndoRedoControls() {
    const canUndo = this.app.state.undoStack.length > 0;
    const canRedo = this.app.state.redoStack.length > 0;

    document.querySelectorAll('[data-command="undo"]').forEach(btn => {
      btn.disabled = !canUndo;
      btn.setAttribute('aria-disabled', String(!canUndo));
    });

    document.querySelectorAll('[data-command="redo"]').forEach(btn => {
      btn.disabled = !canRedo;
      btn.setAttribute('aria-disabled', String(!canRedo));
    });

    document.querySelectorAll('[data-command="undo-history"]').forEach(btn => {
      btn.disabled = !canUndo;
      btn.setAttribute('aria-disabled', String(!canUndo));
    });
  }

  // --- Undo History Panel ---
  showUndoHistory(anchor) {
    const stack = this.app.state.undoStack;
    if (!stack.length) { this.notify('Nothing to undo yet.'); return; }
    // Newest action first; each entry rewinds to the state before that action.
    const items = stack.map((entry, index) => ({
      action: `undo-to-${index}`,
      label: `${index + 1}. ${entry.label || 'Edit'}`,
      icon: 'ph-arrow-u-up-left',
      run: () => this.app.undoTo(index)
    })).reverse();
    this.showCommandPopup(anchor, items, 'command-popup undo-history-popup');
  }

  // --- HUD ---
  updateHUD(distBuf, angleBuf, isDown) {
    const elLen = document.getElementById('hud-len');
    const elAng = document.getElementById('hud-ang');
    const elState = document.getElementById('hud-state');

    if (elLen && distBuf !== null) elLen.innerText = (distBuf || "0.0") + "'";
    if (elAng && angleBuf !== null) elAng.innerText = (angleBuf || "0.0") + "°";
    
    if (elState) {
      if (isDown) {
        elState.innerText = "DOWN";
        elState.classList.add('down');
      } else {
        elState.innerText = "UP";
        elState.classList.remove('down');
      }
    }
  }

  updateZoomDisplay() {
    const zoomRange = document.getElementById('zoom-range');
    const scaleInfo = document.getElementById('scale-info');
    const pct = Math.round(this.app.state.zoom / this.app.drawingScaleFactor() * 100);
    if (zoomRange) zoomRange.value = pct;
    if (scaleInfo) scaleInfo.textContent = `+ ${pct}% (Imp) 1:`;
    const scaleInput = document.getElementById('scale-input');
    if (scaleInput && document.activeElement !== scaleInput) scaleInput.value = String(this.app.state.drawingScale || 10);
  }

  // --- North Compass ---
  updateNorthDisplay() {
    const angle = Number(this.app.state.northAngle) || 0;
    const arrow = document.getElementById('north-arrow');
    const label = document.getElementById('north-degrees');
    if (arrow) arrow.style.transform = `rotate(${angle}deg)`;
    if (label) label.textContent = `${Math.round(angle)}\u00B0`;
  }

  updatePageDisplay() {
    const el = document.getElementById('page-display');
    if (el) el.textContent = `${this.app.currentPage + 1} of ${this.app.pages.length}`;

    // Rebuild the underlay dropdown around the current page.
    const underlay = document.getElementById('underlay-select');
    if (underlay) {
      underlay.innerHTML = '';
      const none = document.createElement('option');
      none.value = '-1';
      none.textContent = 'Underlay: None';
      underlay.appendChild(none);
      this.app.pages.forEach((page, index) => {
        if (index === this.app.currentPage) return;
        const option = document.createElement('option');
        option.value = String(index);
        option.textContent = `Underlay: Page ${index + 1}`;
        underlay.appendChild(option);
      });
      const active = this.app.underlayPageIndex;
      underlay.value = String(active == null || active === this.app.currentPage || active >= this.app.pages.length ? -1 : active);
    }

    // Disable the Previous button when already on the first page.
    const canGoPrev = this.app.currentPage > 0;
    document.querySelectorAll('[data-command="page-prev"], #btn-page-prev').forEach(btn => {
      btn.disabled = !canGoPrev;
      btn.setAttribute('aria-disabled', String(!canGoPrev));
    });
    if (window.__pagesNavRefresh) window.__pagesNavRefresh();
  }

  // --- Layers Panel ---

  updateLayersPanel() {
    const tree = document.getElementById('layers-tree');
    if (!tree) return;
    // Only closed areas are user-facing layers. Working, symbol, imported,
    // and legacy layers remain internal so older files keep rendering, but
    // they no longer flood the Layers tab.
    const layers = this.app.getAreaLayersForCurrentPage();
    const activeId = this.app.state.activeLayerId;
    const rows = [];
    layers.forEach(layer => {
      const chain = this.app.layerAncestry(layer.id);
      const depth = chain.length - 1;
      const hiddenByGroup = chain.slice(0, -1).some(anc => anc.isGroup && anc.collapsed);
      if (hiddenByGroup) return;
      const classes = ['layer-row'];
      if (layer.isGroup) classes.push('is-group');
      if (layer.id === activeId) classes.push('active');
      if (!layer.visible) classes.push('layer-hidden');
      if (layer.locked) classes.push('layer-locked');
      const caret = layer.isGroup
        ? '<button class="layer-caret" type="button" data-action="caret" title="' + (layer.collapsed ? 'Expand group' : 'Collapse group') + '"><i class="ph ' + (layer.collapsed ? 'ph-caret-right' : 'ph-caret-down') + '"></i></button>'
        : '<span class="layer-caret"></span>';
      const icon = layer.isGroup ? 'ph-folder-simple' : 'ph-shapes';
      const eyeIcon = layer.visible ? 'ph-eye' : 'ph-eye-slash';
      const lockIcon = layer.locked ? 'ph-lock-simple-fill' : 'ph-lock-simple-open';
      const eyeTitle = layer.visible ? 'Hide layer' : 'Show layer';
      const lockTitle = layer.locked ? 'Unlock layer' : 'Lock layer';
      rows.push(
        '<div class="' + classes.join(' ') + '" data-layer-id="' + layer.id + '" draggable="true" role="treeitem" aria-selected="' + (layer.id === activeId) + '" style="--depth:' + depth + '">' +
        caret +
        '<i class="ph ' + icon + '"></i>' +
        '<span class="layer-name" title="' + this.escapeHtml(layer.name) + '">' + this.escapeHtml(layer.name) + '</span>' +
        '<span class="layer-tools">' +
        '<button class="layer-eye" type="button" data-action="eye" title="' + eyeTitle + '"><i class="ph ' + eyeIcon + '"></i></button>' +
        '<button class="layer-lock" type="button" data-action="lock" title="' + lockTitle + '"><i class="ph ' + lockIcon + '"></i></button>' +
        '</span>' +
        '</div>'
      );
    });
    tree.innerHTML = rows.length
      ? rows.join('')
      : '<div class="library-empty">Closed areas create layers automatically.</div>';
  }

  escapeHtml(value) {
    return String(value).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
  }

  bindLayerTreeEvents() {
    const tree = document.getElementById('layers-tree');
    if (!tree || tree.dataset.layersBound) return;
    tree.dataset.layersBound = '1';

    tree.addEventListener('click', event => {
      const row = event.target.closest('.layer-row');
      if (!row) return;
      const layerId = row.dataset.layerId;
      const actionEl = event.target.closest('[data-action]');
      const action = actionEl ? actionEl.dataset.action : null;
      const layer = this.app.getLayerById(layerId);
      if (!layer) return;
      if (action === 'eye') {
        this.app.setLayerVisible(layerId, !layer.visible);
      } else if (action === 'lock') {
        this.app.setLayerLocked(layerId, !layer.locked);
      } else if (action === 'caret') {
        this.app.setLayerCollapsed(layerId, !layer.collapsed);
      } else {
        this.app.setActiveLayer(layerId);
      }
    });

    tree.addEventListener('dblclick', event => {
      const nameEl = event.target.closest('.layer-name');
      if (!nameEl) return;
      const row = nameEl.closest('.layer-row');
      if (!row) return;
      this.startLayerRename(row.dataset.layerId);
    });

    tree.addEventListener('contextmenu', event => {
      const row = event.target.closest('.layer-row');
      if (!row) return;
      event.preventDefault();
      const layerId = row.dataset.layerId;
      if (!this.app.getLayerById(layerId)) return;
      this.app.setActiveLayer(layerId);
      this.showLayerContextMenu(event.clientX, event.clientY, layerId);
    });

    let dragRow = null;
    const clearDropMarks = () => {
      tree.querySelectorAll('.drop-before, .drop-after, .drop-inside').forEach(el => {
        el.classList.remove('drop-before', 'drop-after', 'drop-inside');
      });
    };
    tree.addEventListener('dragstart', event => {
      const row = event.target.closest('.layer-row');
      if (!row) return;
      dragRow = row;
      row.classList.add('dragging');
      event.dataTransfer.effectAllowed = 'move';
      try { event.dataTransfer.setData('text/plain', row.dataset.layerId); } catch (_) {}
    });
    tree.addEventListener('dragend', () => {
      if (dragRow) dragRow.classList.remove('dragging');
      dragRow = null;
      clearDropMarks();
    });
    tree.addEventListener('dragover', event => {
      const row = event.target.closest('.layer-row');
      if (!row || !dragRow || row === dragRow) return;
      event.preventDefault();
      event.dataTransfer.dropEffect = 'move';
      clearDropMarks();
      const rect = row.getBoundingClientRect();
      const ratio = (event.clientY - rect.top) / rect.height;
      if (row.classList.contains('is-group') && ratio > 0.25 && ratio < 0.75) {
        row.classList.add('drop-inside');
      } else if (ratio < 0.5) {
        row.classList.add('drop-before');
      } else {
        row.classList.add('drop-after');
      }
    });
    tree.addEventListener('drop', event => {
      const row = event.target.closest('.layer-row');
      if (!row || !dragRow || row === dragRow) return;
      event.preventDefault();
      const fromId = dragRow.dataset.layerId;
      const toId = row.dataset.layerId;
      let position = 'after';
      if (row.classList.contains('drop-inside')) position = 'inside';
      else if (row.classList.contains('drop-before')) position = 'before';
      this.app.moveLayer(fromId, toId, position);
    });
  }

  // Swaps the layer name in the tree for an inline input; shared by the
  // double-click handler and the Rename entry of the layer context menu.
  startLayerRename(layerId) {
    const layer = this.app.getLayerById(layerId);
    const row = document.querySelector('#layers-tree .layer-row[data-layer-id="' + layerId + '"]');
    const nameEl = row && row.querySelector('.layer-name');
    if (!layer || !nameEl) return;
    const input = document.createElement('input');
    input.type = 'text';
    input.className = 'layer-rename';
    input.value = layer.name;
    input.setAttribute('aria-label', 'Rename layer');
    let cancelled = false;
    const finish = () => {
      if (!cancelled && input.value.trim()) this.app.renameLayer(layer.id, input.value);
      nameEl.style.display = '';
      input.remove();
    };
    input.addEventListener('keydown', keyEvent => {
      keyEvent.stopPropagation();
      if (keyEvent.key === 'Enter') finish();
      else if (keyEvent.key === 'Escape') { cancelled = true; finish(); }
    });
    input.addEventListener('blur', finish);
    nameEl.style.display = 'none';
    nameEl.parentNode.insertBefore(input, nameEl);
    input.focus();
    input.select();
  }

  showLayerContextMenu(clientX, clientY, layerId) {
    const layer = this.app.getLayerById(layerId);
    if (!layer) return;
    const items = [
      { action:'rename', label:'Rename', icon:'ph-text-t', run:() => this.startLayerRename(layerId) },
      { action:'hide', label:layer.visible ? 'Hide' : 'Show', icon:layer.visible ? 'ph-eye' : 'ph-eye-slash', run:() => this.app.setLayerVisible(layerId, !layer.visible) },
      { action:'lock', label:layer.locked ? 'Unlock' : 'Lock', icon:layer.locked ? 'ph-lock-simple-open' : 'ph-lock-simple', run:() => this.app.setLayerLocked(layerId, !layer.locked) },
      { separator:true },
      { action:'export', label:'Export', icon:'ph-export', run:() => this.app.exportLayer(layerId) }
    ];
    const anchor = { getBoundingClientRect:() => ({ left:clientX, bottom:clientY }) };
    return this.showCommandPopup(anchor, items, 'sketch-context-menu');
  }

  pickLayerImportFile() {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = 'application/json,.json';
    input.style.display = 'none';
    input.addEventListener('change', () => {
      const file = input.files && input.files[0];
      input.remove();
      if (file) this.app.importLayerFile(file);
    });
    document.body.appendChild(input);
    input.click();
  }

  // --- Calculations Panel ---

  toggleCalcsPanel() {
    const panel = document.getElementById('panel-calcs');
    const dock = document.getElementById('sidebar-dock');
    if (!panel || !dock) return;
    if (!panel.hidden && dock.classList.contains('open')) this.setSidebarOpen(false);
    else this.showPanel('panel-calcs');
  }

  updateCalcsPanel() {
    const calcsTree = document.getElementById('calcs-tree');
    const calcsSummary = document.getElementById('calcs-summary');
    if (!calcsTree || !calcsSummary) return;

    const polys = this.app.state.polygons.filter(p => p.type && p.type !== 'UND');
    const groups = new Map();
    polys.forEach(poly => {
      const code = poly.type;
      if (!groups.has(code)) groups.set(code, []);
      groups.get(code).push(poly);
    });

    let treeHTML = '';
    for (const [code, items] of groups) {
      const groupArea = items
        .filter(poly => poly.subtractedFromId == null || (Number(poly.net) || 0) > 0)
        .reduce((sum, poly) => sum + (Number(poly.net) || 0), 0);
      const ordered = items.slice().sort((a, b) => {
        if (a.containerId === b.id) return 1;
        if (b.containerId === a.id) return -1;
        return (a.containerId != null) - (b.containerId != null);
      });
      const areasHTML = ordered.map(poly => {
        const nested = poly.containerId != null;
        const baseArea = Number(poly.gross) || 0;
        const factor = Number.isFinite(Number(poly.effectiveMultiplier)) ? Number(poly.effectiveMultiplier) : 1;
        const total = baseArea * factor * (poly.sign === -1 ? -1 : 1);
        const netArea = Number(poly.net) || 0;
        return `<div class="calcs-area-node ${nested ? 'calcs-child' : ''}" data-poly-id="${this.escapeHTML(poly.id)}">
          <button class="calcs-area-header" type="button" aria-expanded="false"><i class="ph-fill ph-caret-right" aria-hidden="true"></i><span>${this.escapeHTML(poly.name || code)} : ${netArea.toFixed(2)}</span></button>
          <div class="calcs-area-details">
            <div class="calcs-row"><span class="calcs-label">Base Area</span><span class="calcs-value">${baseArea.toFixed(2)}</span></div>
            <div class="calcs-row"><span class="calcs-label">Factor</span><span class="calcs-value">${factor.toFixed(2)}</span></div>
            <div class="calcs-row"><span class="calcs-label">Total</span><span class="calcs-value">${total.toFixed(2)}</span></div>
            ${poly.subtracted ? `<div class="calcs-row calcs-subtracted"><span class="calcs-label">Subtracted</span><span class="calcs-value">-${Number(poly.subtracted).toFixed(2)}</span></div>` : ''}
            <div class="calcs-row"><span class="calcs-label">Net Area</span><span class="calcs-value">${netArea.toFixed(2)}</span></div>
            <div class="calcs-row"><span class="calcs-label">Perimeter</span><span class="calcs-value">${(Number(poly.perimeter) || 0).toFixed(2)}</span></div>
          </div>
        </div>`;
      }).join('');
      treeHTML += `
        <div class="calcs-group expanded">
          <button class="calcs-group-header" type="button" aria-expanded="true"><span><i class="ph-fill ph-caret-down" aria-hidden="true"></i> ${this.escapeHTML(code)}</span><span class="calcs-area-val">${groupArea.toFixed(2)}</span></button>
          <div class="calcs-group-body">${areasHTML}</div>
        </div>
      `;
    }

    if (polys.length === 0) {
      treeHTML = '<div style="color:var(--text-muted);padding:8px;font-size:11px;">No defined areas yet.</div>';
    }

    calcsTree.innerHTML = treeHTML;
    calcsTree.querySelectorAll('.calcs-group-header').forEach(header => header.addEventListener('click', () => {
      const group = header.closest('.calcs-group');
      group.classList.toggle('expanded');
      const expanded = group.classList.contains('expanded');
      header.setAttribute('aria-expanded', String(expanded));
      const icon = header.querySelector('i');
      if (icon) icon.className = `ph-fill ${expanded ? 'ph-caret-down' : 'ph-caret-right'}`;
    }));
    calcsTree.querySelectorAll('.calcs-area-header').forEach(header => header.addEventListener('click', event => {
      const node = header.closest('.calcs-area-node');
      node.classList.toggle('expanded');
      const expanded = node.classList.contains('expanded');
      header.setAttribute('aria-expanded', String(expanded));
      const icon = header.querySelector('i');
      if (icon) icon.className = `ph-fill ${expanded ? 'ph-caret-down' : 'ph-caret-right'}`;
      if (event.detail > 1) return;
      this.app.clearSelection(false);
      this.app.state.selectedPolyIds.add(Number(node.dataset.polyId));
      this.app.ui.updateAreaActionStates?.();
      this.app.render();
    }));

    const undCount = this.app.state.polygons.filter(p => p.type === 'UND').length;
    const totals = this.app.areaTotals || { totalGLA: 0, totalAllAreas: 0 };
    const totalGBA = Number(totals.totalsByGroup?.GBA) || 0;
    calcsSummary.innerHTML = `
      <div class="calcs-summary-row">
        <span>Defined Areas</span>
        <span class="calcs-value">${polys.length}</span>
      </div>
      ${undCount > 0 ? `<div class="calcs-summary-row"><span>Undefined Areas</span><span class="calcs-value" style="color:#E53E3E">${undCount}</span></div>` : ''}
      <div class="calcs-summary-row">
        <span>Total Gross Living Area</span>
        <span class="calcs-value">${(Number(totals.totalGLA) || 0).toFixed(1)} sf</span>
      </div>
      <div class="calcs-summary-row">
        <span>Total Gross Building Area</span>
        <span class="calcs-value">${totalGBA.toFixed(1)} sf</span>
      </div>
    `;
    const chip = document.getElementById('totals-chip');
    if (chip) {
      const gla = document.getElementById('totals-chip-gla');
      const count = document.getElementById('totals-chip-count');
      const chipTotals = this.app.areaTotals || { totalGLA: 0 };
      if (gla) gla.textContent = 'GLA ' + (Number(chipTotals.totalGLA) || 0).toFixed(1) + ' sf';
      if (count) count.textContent = polys.length + ' area' + (polys.length === 1 ? '' : 's');
    }
    const statusSelect = document.getElementById('area-status-select');
    if (statusSelect) {
      const previousId = statusSelect.value;
      const selectedPoly = polys.find(poly => this.app.state.selectedPolyIds.has(poly.id));
      const codeWidth = Math.max(3, ...polys.map(poly => String(poly.type || '').length));
      const nameWidth = Math.max(4, ...polys.map(poly => String(poly.name || poly.type || '').length));
      statusSelect.replaceChildren();
      if (polys.length === 0) {
        const option = document.createElement('option');
        option.value = '';
        option.textContent = 'No defined areas    0.0';
        statusSelect.appendChild(option);
        statusSelect.disabled = true;
      } else {
        polys.forEach(poly => {
          const option = document.createElement('option');
          const code = String(poly.type || '').padEnd(codeWidth);
          const name = String(poly.name || poly.type || '').padEnd(nameWidth);
          const area = Number(poly.gross ?? poly.area) || 0;
          option.value = String(poly.id);
          option.textContent = `${code}  ${name}  ${area.toFixed(1)}`;
          statusSelect.appendChild(option);
        });
        statusSelect.disabled = false;
        const preferredId = selectedPoly ? String(selectedPoly.id) : previousId;
        if (polys.some(poly => String(poly.id) === preferredId)) statusSelect.value = preferredId;
        else statusSelect.value = String(polys[polys.length - 1].id);
      }
    }
  }
}

window.UIManager = UIManager;



;
/**
 * Sketch - Application Coordinator
 * Central state manager connecting UI, Input, Canvas, and Geometry.
 */

const DEFAULT_PAGE_LAYOUT = Object.freeze({
  paperSize: 'legal',
  currentForm: 'standard-legal',
  showSketchComments: true,
  showCommentTables: true,
  showPageNumbers: true,
  calculationAddendum: false,
  deedCallsAddendum: false,
  captions: Object.freeze({
    sketchPageTitle: 'SKETCH/AREA TABLE ADDENDUM',
    subjectLabel: 'SUBJECT INFO',
    sketchLabel: 'SKETCH',
    summaryLabel: 'AREA CALCULATIONS SUMMARY',
    commentTable1Label: 'COMMENT TABLE 1',
    commentTable2Label: 'COMMENT TABLE 2',
    commentTable3Label: 'COMMENT TABLE 3'
  })
});

