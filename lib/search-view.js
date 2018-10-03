/** @babel */

import {CompositeDisposable, Disposable, TextEditor} from 'atom';

export default class SearchView {
  constructor(serializedState) {
    this.disposables = new CompositeDisposable();
    // List of currently available repositories (projects)
    this.paths = atom.project.getPaths();
    this.disposables.add(atom.project.onDidChangePaths(projectPaths => {
      this.paths = projectPaths;
      this.update();
    }));

    const {editor, controlPanelElement} = this.createControlPanel();
    // Main text editor to search
    this.disposables.add(editor.onDidStopChanging(() => {
      this.search({
        paths: this.paths,
        pattern: editor.getText()
      });
    }));

    // Whenever our search view is active, update the current state.
    this.disposables.add(atom.workspace.onDidStopChangingActivePaneItem(pane => {
      if (pane == this) {
        this.update();
      }
    }));

    this.element = document.createElement('div');
    this.element.classList.add('search-view', 'pane-item');
    this.element.setAttribute('tabIndex', '-1');
    this.element.appendChild(controlPanelElement);
  }

  search(data) {
    console.log(data);
  }

  // Creates the main control panel.
  createControlPanel() {
    const editor = new TextEditor({mini: true, placeholderText: 'Type to search...'});

    // Create a panel of choosing the search path
    const header = document.createElement('p');
    if (this.paths) {
      const ul = document.createElement('ul');
      ul.classList.add('list-group');
      for (const path of this.paths) {
        const li = document.createElement('li');
        li.classList.add('list-item');

        const p = document.createElement('label');
        p.classList.add('input-label');

        const radio = document.createElement('input');
        radio.classList.add('input-radio');
        radio.setAttribute('type', 'radio');
        radio.setAttribute('checked', 'true');

        const label = document.createElement('span');
        label.textContent = ` ${path}`;

        p.appendChild(radio);
        p.appendChild(label);
        li.appendChild(p);
        ul.appendChild(li);
      }
      header.appendChild(ul);
    }

    const controlPanel = document.createElement('section');
    controlPanel.classList.add('control-panel');
    controlPanel.appendChild(header);
    controlPanel.appendChild(editor.element);
    return {editor: editor, controlPanelElement: controlPanel};
  }

  update() {
    // First, we need to update current projects that are available.
  }

  // Tear down any state and detach
  destroy() {
    if (this.disposables) {
      this.disposables.dispose();
    }
    if (this.element) {
      this.element.remove();
    }
  }

  getTitle() {
    return 'Search';
  }

  getIconName() {
    return 'search'
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
    return 'atom://omnisearch';
  }

  getElement() {
    return this.element;
  }

  isEqual(other) {
    return other instanceof SearchView;
  }

  serialize() {
    // Should match definition in package.json.
    return {
      deserializer: 'omnisearch/search-view/SearchView'
    };
  }
}
