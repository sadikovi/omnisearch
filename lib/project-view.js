/** @babel */

import {Disposable} from 'atom';

export default class ProjectView {
  constructor(path, callback) {
    this.path = path;

    this.element = document.createElement('span');
    this.element.setAttribute('path', path);
    this.element.appendChild(document.createTextNode(` ${path}`));
    this.markUnchecked();

    const clickHandler = (event) => {
      callback(this)
    }
    this.element.addEventListener('click', clickHandler);
    this.disposable = new Disposable(() => {
      this.element.removeEventListener('click', clickHandler)
    });
  }

  // Marks current view checked (active).
  markChecked() {
    this.element.classList.remove('icon-file-directory');
    this.element.classList.add('icon-check');
  }

  // Marks current view unchecked (default state).
  markUnchecked() {
    this.element.classList.remove('icon-check');
    this.element.classList.add('icon-file-directory');
  }

  isChecked() {
    return this.element.classList.contains('icon-check');
  }

  // Returns root element.
  getElement() {
    return this.element;
  }

  // Releases resources.
  destroy() {
    this.path = null;
    this.element.remove();
    this.disposable.dispose();
  }
}
