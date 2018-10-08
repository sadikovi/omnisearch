/** @babel */

import {Emitter} from 'atom';
import ProjectItem from './project-item';

export default class ProjectSelectorView {
  constructor() {
    this.emitter = new Emitter();
    this.projects = [];
    this.element = this._createElement();
  }

  // Takes list of paths and updates the internal state as well as DOM.
  updateForPaths(paths) {
    if (!paths) return;

    let selected = null;
    let remaining = [];
    for (const projectElement of this.projects) {
      if (projectElement.isChecked()) {
        selected = projectElement.path;
      }
      projectElement.markUnchecked();
      if (projectElement.path in paths) {
        remaining.push(projectElement);
      } else {
        projectElement.destroy();
      }
    }

    this.projects = [];
    let i = 0;
    for (const path of paths) {
      if (i < remaining.length && remaining[i].path == path) {
        this.projects.push(remaining[i]);
        i++;
      } else {
        const projectElement = new ProjectItem(path, this._selectionHandler.bind(this));
        this.projects.push(projectElement);
        if (i < remaining.length) {
          this.element.insertBefore(projectElement.element, remaining[i]);
        } else {
          this.element.appendChild(projectElement.element);
        }
      }
    }

    for (projectElement of this.projects) {
      if (!selected || projectElement.path == selected) {
        projectElement.markChecked();
        return;
      }
    }
    // If we have removed selected project, just mark the first one as selected.
    if (this.projects.length > 0) {
      this.projects[0].markChecked();
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
    this.element.remove();
    for (const project of this.projects) {
      project.destroy();
    }
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
