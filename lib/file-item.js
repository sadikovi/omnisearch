/** @babel */

import {Disposable} from 'atom';

export default class FileItem {
  constructor(path) {
    const pathElement = document.createElement('a');
    pathElement.textContent = `${path}`;

    const clickHandler = (event) => {
      atom.workspace.open(path);
    }
    pathElement.addEventListener('click', clickHandler);
    this.disposable = new Disposable(() => {
      pathElement.removeEventListener('click', clickHandler)
    });

    this.element = document.createElement('span');
    this.element.classList.add('icon', 'icon-file-text');
    this.element.appendChild(pathElement);
  }

  // Returns root element.
  getElement() {
    return this.element;
  }

  // Releases resources.
  destroy() {
    this.disposable.dispose();
    this.element.remove();
  }
}
