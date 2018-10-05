/** @babel */

import {Disposable} from 'atom';

export default class LineBlockItem {
  constructor(block) {
    console.log(block);
    this.element = document.createElement('div');
    this.element.classList.add('line-block');

    const header = document.createElement('div');
    header.classList.add('line-block-heading');
    header.textContent = '/path/to/a/file';

    const content = document.createElement('div');
    content.textContent = "CONTENT";

    this.element.appendChild(header);
    this.element.appendChild(content);
  }

  // Returns root element.
  getElement() {
    return this.element;
  }

  // Releases resources.
  destroy() {
    this.element.remove();
  }
}
