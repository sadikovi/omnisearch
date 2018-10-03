/** @babel */

import {CompositeDisposable, Disposable, TextEditor} from 'atom';

export default class SearchView {
  constructor(serializedState) {
    this.disposables = new CompositeDisposable();
    // Register repositories in the current workspace
    this.paths = atom.project.getPaths();
    this.disposables.add(atom.project.onDidChangePaths(projectPaths => {
      this.paths = projectPaths;
    }));
    // Set active directory for which we should search.
    /*
    this.activePath = null;
    if (atom.treeView) {
      const firstSelectedPath =
      this.activePath =
    }
    */

    this.element = document.createElement('div');
    this.element.classList.add('search-view', 'pane-item');
    this.element.setAttribute('tabIndex', '-1');
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
