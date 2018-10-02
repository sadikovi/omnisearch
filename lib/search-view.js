/** @babel */

import {CompositeDisposable, Disposable, TextEditor} from 'atom';
import find from './search';

export default class SearchView {
  constructor(serializedState) {
    this.disposables = new CompositeDisposable();

    this.element = document.createElement('div');
    this.element.classList.add('search-view', 'pane-item');
    this.element.setAttribute('tabIndex', '-1');

    // Register repositories in the current workspace
    this.paths = atom.project.getPaths();
    this.disposables.add(atom.project.onDidChangePaths(projectPaths => {
      this.paths = projectPaths;
    }));

    const editor = new TextEditor({mini: true, placeholderText: 'Type to search...'});
    this.element.appendChild(editor.element);
    this.disposables.add(editor.onDidStopChanging(() => {
      this.callback(editor.getText())
    }));
  }

  callback(text) {
    console.log('editor text', text);
    find.find("localhost:8080/search", "/Users/sadikovi/developer/omnisearch", "query", null,
      (json) => console.log("json"),
      (err) => console.log(err));
    /*
    fetch('/Users/sadikovi/developer/omnisearch/example.json')
      .then(response => response.json())
      .then(json => {
        // collect all extensions
        console.log(json);
      });
    */
  }

  // Tear down any state and detach
  destroy() {
    this.disposables.dispose();
    if (this.element) {
      this.element.remove();
    }
  }

  getTitle() {
    return 'Search';
  }

  getIconName () {
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
    return 'atom://omnisearch'
  }

  getElement() {
    return this.element;
  }

  isEqual(other) {
    return other instanceof SearchView;
  }
}
