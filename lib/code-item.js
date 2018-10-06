/** @babel */

import {Disposable} from 'atom';

export default class CodeItem {
  constructor(block) {
    console.log(block);
    this.element = document.createElement('div');
    this.element.classList.add('file');

    const header = document.createElement('div');
    header.classList.add('file-header');
    header.textContent = `${block.path}`;

    const code = document.createElement('div');
    code.classList.add('code-wrapper');
    const table = document.createElement('table');
    const tbody = document.createElement('tbody');

    for (let i = 0; i < block.matches.length; i++) {
      this.addMatch(tbody, block.matches[i]);
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
  addMatch(tbody, match) {
    for (const line of match.lines) {
      const lineNumber = document.createElement('td');
      lineNumber.classList.add('code-line-num');
      lineNumber.textContent = `${line.num}`;

      const label = document.createElement('td');
      label.classList.add('code-line-label');
      const icon = document.createElement('span');
      icon.classList.add('icon');
      if (line.kind == 'before') {
        icon.classList.add('icon-arrow-small-right');
      } else if (line.kind == 'after') {
        icon.classList.add('icon-arrow-small-left');
      } else {
        icon.classList.add('icon-versions');
      }
      label.appendChild(icon);

      const code = document.createElement('td');
      code.classList.add('code-blob');
      code.textContent = `${line.bytes}`;

      const tr = document.createElement('tr');
      tr.appendChild(lineNumber);
      tr.appendChild(label);
      tr.appendChild(code);

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
    this.element.remove();
  }
}
