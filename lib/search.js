'use babel';

export default class SearchInfoView {
  constructor(serializedState) {
    // Create root element
    this.element = document.createElement('div');
    this.element.classList.add('omnisearch-view');
    this.element.classList.add('pane-item');

    // Create search input
    const search = document.createElement('div');
    search.classList.add('block');

    const searchIcon = document.createElement('span');
    searchIcon.classList.add('icon');
    searchIcon.classList.add('icon-search');
    searchIcon.textContent = 'search';
    search.append(searchIcon);

    const searchInput = document.createElement('input');
    searchInput.classList.add('input-search');
    searchInput.setAttribute('type', 'search');
    searchInput.setAttribute('placeholder', 'Search');
    search.append(searchInput);

    this.element.append(search);

    // Create message element
    const message = document.createElement('div');
    message.textContent = 'The Search View';
    message.classList.add('message');
    this.element.appendChild(message);

    this.subscriptions = atom.workspace.getCenter().observeActivePaneItem(item => {
      if (!atom.workspace.isTextEditor(item)) return;
      message.innerHTML = `
        <h2>${item.getFileName() || 'untitled'}</h2>
        <ul>
          <li><b>Soft Wrap:</b> ${item.softWrapped}</li>
          <li><b>Tab Length:</b> ${item.getTabLength()}</li>
          <li><b>Encoding:</b> ${item.getEncoding()}</li>
          <li><b>Line Count:</b> ${item.getLineCount()}</li>
        </ul>
      `;
    });
  }

  getTitle() {
    return 'Search';
  }

  getDefaultLocation() {
    // This location will be used if the user hasn't overridden it by dragging the item elsewhere.
    // Valid values are "left", "right", "bottom", and "center" (the default).
    return 'center';
  }

  getAllowedLocations() {
    // The locations into which the item can be moved.
    return ['left', 'right', 'center', 'bottom'];
  }

  getURI() {
    return 'atom://omnisearch'
  }

  // Tear down any state and detach
  destroy() {
    this.element.remove();
    this.subscriptions.dispose();
  }

  getElement() {
    return this.element;
  }
}
