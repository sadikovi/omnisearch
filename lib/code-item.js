/** @babel */

import {CompositeDisposable, Disposable} from 'atom';

export default class CodeItem {
  constructor(block) {
    this.disposables = new CompositeDisposable();

    this.element = document.createElement('div');
    this.element.classList.add('file');

    const extension = document.createElement('span');
    extension.classList.add('inline-block', 'highlight-warning');
    extension.textContent = `${block.ext}`;
    const title = document.createElement('span');
    title.textContent = `${block.path}`;
    const header = document.createElement('div');
    header.classList.add('file-header');
    header.appendChild(extension);
    header.appendChild(title);

    const code = document.createElement('div');
    code.classList.add('code-wrapper');
    const table = document.createElement('table');
    const tbody = document.createElement('tbody');

    for (let i = 0; i < block.matches.length; i++) {
      this.addMatch(tbody, block.matches[i], block.path);
      if (i < block.matches.length - 1) {
        this.addSeparator(tbody);
      }
    }

    table.appendChild(tbody);
    code.appendChild(table);
    this.element.appendChild(header);
    this.element.appendChild(code);
  }

  // Adds matched lines.
  addMatch(tbody, match, path) {
    for (const line of match.lines) {
      // Line number
      const number = document.createElement('td');
      number.classList.add('code-line-num');
      number.textContent = `${line.num}`;

      // Line marker
      const marker = document.createElement('td');
      marker.classList.add('code-line-marker');
      const icon = document.createElement('span');
      icon.classList.add('icon');
      marker.appendChild(icon);

      // Line of code
      const code = document.createElement('td');
      code.classList.add('code-blob');
      code.textContent = `${line.bytes}`;

      // Update condition-based classes
      if (line.kind == 'before') {
        icon.classList.add('icon-arrow-small-right', 'text-success');
      } else if (line.kind == 'after') {
        icon.classList.add('icon-arrow-small-left', 'text-warning');
      } else {
        number.classList.add('text-highlight');
        icon.classList.add('icon-versions', 'text-highlight');
        code.classList.add('text-highlight');
      }

      const tr = document.createElement('tr');
      tr.appendChild(number);
      tr.appendChild(marker);
      tr.appendChild(code);

      const clickHandler = (event) => {
        atom.workspace.open(path, {initialLine: line.num - 1});
      }
      tr.addEventListener('click', clickHandler);
      this.disposables.add(new Disposable(() => {
        tr.removeEventListener('click', clickHandler)
      }));

      tbody.appendChild(tr);
    }
  }

  // Adds separator between matches.
  addSeparator(tbody) {
    const tr = document.createElement('tr');
    for (let i = 0; i < 3; i++) {
      const td = document.createElement('td');
      td.classList.add('match-separator');
      tr.appendChild(td);
    }
    tbody.appendChild(tr);
  }

  // Returns root element.
  getElement() {
    return this.element;
  }

  // Releases resources.
  destroy() {
    this.disposables.dispose();
    this.element.remove();
  }
}
