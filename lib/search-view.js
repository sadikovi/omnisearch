/** @babel */

import {CompositeDisposable, Disposable, TextEditor} from 'atom';
import ProjectSelectorView from './project-selector-view';

export default class SearchView {
  constructor(serializedState) {
    this.disposables = new CompositeDisposable();

    // List of currently available repositories (projects)
    this.paths = atom.project.getPaths();

    // Path selector panel
    this.projectSelector = new ProjectSelectorView();
    this.projectSelector.updateForPaths(this.paths);
    this.disposables.add(this.projectSelector.onDidChangeSelection(path => {
      console.log(path);
    }));

    // Main text editor to search
    this.editor = new TextEditor({mini: true, placeholderText: 'Type to search...'});
    this.disposables.add(this.editor.onDidStopChanging(() => {
      this.search({
        paths: this.paths,
        pattern: this.editor.getText()
      });
    }));

    // Whenever our search view is active, update the current state.
    this.disposables.add(atom.workspace.onDidStopChangingActivePaneItem(pane => {
      if (pane == this) {
        this.update();
      }
    }));
    this.disposables.add(atom.project.onDidChangePaths(projectPaths => {
      this.paths = projectPaths;
      this.update();
    }));

    this.element = document.createElement('div');
    this.element.classList.add('search-view', 'pane-item');
    this.element.setAttribute('tabIndex', '-1');
    const controlPanel = document.createElement('section');
    controlPanel.classList.add('control-panel');
    controlPanel.appendChild(this.projectSelector.element);
    controlPanel.appendChild(this.editor.element);
    this.element.appendChild(controlPanel);
  }

  search(data) {
    console.log(data);
  }

  update() {
    // First, we need to update current projects that are available.
  }

  // Tear down any state and detach
  destroy() {
    this.disposables.dispose();
    this.projectSelector.destroy();
    this.element.remove();
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
    return {
      // Should match definition in package.json.
      deserializer: 'omnisearch/search-view/SearchView'
    };
  }
}
