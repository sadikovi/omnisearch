/** @babel */

import {Disposable} from 'atom';

export default class FileItem {
  constructor(path, ext) {
    this.extension = ext;

    const pathElement = document.createElement('a');
    pathElement.textContent = `${path}`;

    const clickHandler = (event) => {
      atom.workspace.open(path);
    }
    pathElement.addEventListener('click', clickHandler);
    this.disposable = new Disposable(() => {
      pathElement.removeEventListener('click', clickHandler)
    });

    const icon = document.createElement('span');
    icon.classList.add('icon', 'icon-file-text');
    icon.appendChild(pathElement);

    this.element = document.createElement('li');
    this.element.classList.add('list-item');
    this.element.appendChild(icon);

    // All elements are rendered visible when initialised.
    this.show();
  }

  // Returns true, if item has extension.
  hasExtension(ext) {
    return this.extension == ext;
  }

  // Marks element as visible.
  show() {
    this.element.classList.remove('hidden');
  }

  // Marks element as hidden.
  hide() {
    this.element.classList.add('hidden');
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
