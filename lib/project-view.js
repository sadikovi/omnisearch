/** @babel */

import {Disposable} from 'atom';

export default class ProjectView {
  constructor(path, callback) {
    this.path = path;

    this.project = document.createElement('span');
    this.project.classList.add('icon-file-directory');
    this.project.setAttribute('path', path);
    this.project.appendChild(document.createTextNode(` ${path}`));
    this.markUnchecked();

    const clickHandler = (event) => {
      callback(this)
    }
    this.project.addEventListener('click', clickHandler);
    this.disposable = new Disposable(() => {
      this.project.removeEventListener('click', clickHandler)
    });

    this.element = document.createElement('li');
    this.element.classList.add('list-item');
    this.element.appendChild(this.project);
  }

  // Marks current view checked (active).
  markChecked() {
    this.project.classList.add('active');
  }

  // Marks current view unchecked (default state).
  markUnchecked() {
    this.project.classList.remove('active');

  }

  isChecked() {
    return this.project.classList.contains('active');
  }

  // Returns root element.
  getElement() {
    return this.element;
  }

  // Releases resources.
  destroy() {
    this.disposable.dispose();
    this.element.remove();
    this.path = null;
  }
}
