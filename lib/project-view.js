/** @babel */

import {Disposable} from 'atom';

export default class ProjectView {
  constructor(path, callback) {
    this.path = path;

    this.element = document.createElement('span');
    this.element.classList.add('icon-file-directory');
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
    this.element.classList.add('active');
  }

  // Marks current view unchecked (default state).
  markUnchecked() {
    this.element.classList.remove('active');

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
