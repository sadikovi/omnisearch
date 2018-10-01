/** @babel */

import {TextEditor} from 'atom';

export default class SearchInfoView {
  constructor(serializedState) {
    this.element = document.createElement('div');
    this.element.classList.add('search-view', 'pane-item');
    this.element.setAttribute('tabIndex', '-1');
    this.element.appendChild(this.panel());
    console.log(this.fetchResults());
  }

  fetchResults() {
    return fetch("/Users/sadikovi/developer/omnisearch/example.json")
      .then(response => response.json())
      .done(json => {
        // collect all extensions
        console.log(json);
      });
  }

  panel() {
    const section = document.createElement('section');
    section.classList.add('search-panel');
    section.appendChild(this.usage());
    section.appendChild(this.search());
    section.appendChild(this.extensions(['scala', 'java']));
    return section;
  }

  search() {
    const search = document.createElement('div');
    search.classList.add('search-container');
    const editor = new TextEditor({mini: true, placeholderText: 'Type to search...'});
    search.appendChild(editor.element);
    return search;
  }

  usage() {
    const note = document.createElement('p');
    note.classList.add('usage');
    note.textContent = 'Type search query and hit Enter to start the search!';
    return note;
  }

  extensions(list) {
    let extensions = document.createElement('div');
    extensions.classList.add('extensions-list', 'btn-toolbar');
    for (let name of list) {
      const extension = document.createElement('div');
      extension.classList.add('btn-group');
      const button = document.createElement('button');
      button.classList.add('btn')
      button.textContent = `${name}`;
      extension.appendChild(button);
      extensions.appendChild(extension);
    }
    return extensions;
  }

  // Tear down any state and detach
  destroy() {
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
    return other instanceof SearchInfoView;
  }
}
