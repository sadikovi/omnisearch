/** @babel */

export default class ErrorView {
  constructor(msg) {
    const li = document.createElement('li');
    li.textContent = `${msg}`;

    const ul = document.createElement('ul');
    ul.classList.add('background-message', 'centered');
    ul.appendChild(li);

    this.element = document.createElement('div');
    this.element.classList.add('search-view', 'pane-item');
    this.element.appendChild(ul);
  }

  destroy() {
    this.element.remove();
  }

  getTitle() {
    return 'Search';
  }

  getIconName() {
    return 'search'
  }

  getElement() {
    return this.element;
  }

  isEqual(other) {
    return other instanceof ErrorView;
  }
}
