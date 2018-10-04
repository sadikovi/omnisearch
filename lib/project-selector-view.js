/** @babel */

import {CompositeDisposable, Disposable, Emitter} from 'atom';
import ProjectView from './project-view';

export default class ProjectSelectorView {
  constructor() {
    this.emitter = new Emitter();
    this.disposables = new CompositeDisposable();
    this.projects = [];
    this.element = this._createElement();
  }

  // Takes list of paths and updates the internal state as well as DOM.
  updateForPaths(paths) {
    if (!paths) return;
    // For simplicity we are going to remove all existing elements and add new ones.
    while (this.projects.length > 0) {
      const projectElement = this.projects.pop();
      projectElement.destroy();
    }
    // Register all paths.
    // Select the first path we encounter as active.
    let selected = true;
    for (path of paths) {
      const projectElement = new ProjectView(path, this._selectionHandler.bind(this));
      if (selected) {
        projectElement.markChecked();
      }
      selected = false;
      this.projects.push(projectElement);
    }
    // Add all elements to the parent element.
    for (projectElement of this.projects) {
      this.element.appendChild(projectElement.element);
    }
  }

  // Returns the root element of the project selector view.
  getElement() {
    return this.element;
  }

  // Returns currently selected path.
  getSelection() {
    for (projectElement of this.projects) {
      if (projectElement.isChecked()) {
        return projectElement.path;
      }
    }
    return null;
  }

  // Method is invoked whenever user changes the currently selected path.
  onDidChangeSelection(callback) {
    return this.emitter.on('did-change-path-selection', callback);
  }

  destroy() {
    this.emitter.dispose();
    this.emitter.clear();
    this.disposables.dispose();
    this.element.remove();
  }

  _createElement() {
    const ul = document.createElement('ul');
    ul.classList.add('list-group', 'project-selector');
    return ul;
  }

  // Triggered when active selection changes
  _selectionHandler(activeProject) {
    for (projectElement of this.projects) {
      projectElement.markUnchecked();
    }
    activeProject.markChecked();
    this.emitter.emit('did-change-path-selection', activeProject.path);
  }
}
