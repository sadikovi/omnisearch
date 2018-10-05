/** @babel */

import {Disposable} from 'atom';

export default class ResultView {
  constructor(title, dom, badge) {
    this.icon = document.createElement('span');
    this.icon.classList.add('icon', 'icon-chevron-down');

    this.badge = document.createElement('span');
    this.badge.classList.add('badge');
    this.badge.textContent = `${badge}`;

    this.header = document.createElement('h2');
    this.header.classList.add('section-heading');
    this.header.appendChild(this.icon);
    this.header.appendChild(document.createTextNode(`${title}`));
    this.header.appendChild(this.badge);

    this.content = dom;

    this.element = document.createElement('section');
    this.element.classList.add('bordered');
    this.element.appendChild(this.header);
    this.element.appendChild(this.content);

    const clickHandler = (event) => {
      this.toggle()
    }
    this.header.addEventListener('click', clickHandler);
    this.disposable = new Disposable(() => {
      this.header.removeEventListener('click', clickHandler)
    });
  }

  // Updates badge text.
  updateBadge(text) {
    this.badge.textContent = `${text}`;
  }

  // Returns root element.
  getElement() {
    return this.element;
  }

  destroy() {
    this.disposable.dispose();
    this.element.remove();
  }

  // Toggle the result view.
  toggle() {
    if (this.element.classList.contains('collapsed')) {
      this.icon.classList.add('icon-chevron-down');
      this.icon.classList.remove('icon-chevron-right');
      this.content.classList.remove('hidden');
      this.element.classList.remove('collapsed');
    } else {
      this.icon.classList.remove('icon-chevron-down');
      this.icon.classList.add('icon-chevron-right');
      this.content.classList.add('hidden');
      this.element.classList.add('collapsed');
    }
  }
}
