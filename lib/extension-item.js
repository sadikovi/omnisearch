/** @babel */

import {Disposable} from 'atom';

export default class ExtensionItem {
  constructor(extension, onSelected) {
    this.extension = extension;
    this.element = document.createElement('button');
    this.element.classList.add('inline-block', 'btn');
    this.element.textContent = `${extension}`;

    const clickHandler = (event) => {
      onSelected(this.extension)
    }
    this.element.addEventListener('click', clickHandler);
    this.disposable = new Disposable(() => {
      this.element.removeEventListener('click', clickHandler)
    });
  }

  // Enables/disables this item.
  toggle() {
    if (this.element.classList.contains('selected')) {
      this.element.classList.remove('selected');
    } else {
      this.element.classList.add('selected');
    }
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
