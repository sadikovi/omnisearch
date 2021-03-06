/** @babel */

import {CompositeDisposable, Disposable} from 'atom';

// Content kind
const BEFORE = 'before';
const AFTER = 'after';
const MATCH = 'match';

export default class CodeItem {
  constructor(block) {
    this.block = block;

    this.disposables = new CompositeDisposable();

    this.element = document.createElement('div');
    this.element.classList.add('file');

    // Collapsible icon
    this.icon = document.createElement('span');
    this.icon.classList.add('icon', 'icon-chevron-down');

    const extension = document.createElement('span');
    extension.classList.add('inline-block', 'highlight-warning');
    extension.textContent = `${block.ext}`;

    const title = document.createElement('a');
    title.textContent = `${block.path}`;
    // Add click for the file path
    const titleClickHandler = (event) => {
      atom.workspace.open(block.path);
      // This is done, so we do not trigger toggle on the header.
      event.stopPropagation();
    }
    title.addEventListener('click', titleClickHandler);
    this.disposables.add(new Disposable(() => {
      title.removeEventListener('click', titleClickHandler);
    }));

    this.header = document.createElement('div');
    this.header.classList.add('file-header');
    this.header.appendChild(this.icon);
    this.header.appendChild(extension);
    this.header.appendChild(title);

    const clickHandler = (event) => {
      this.toggle()
    }
    this.header.addEventListener('click', clickHandler);
    this.disposables.add(new Disposable(() => {
      this.header.removeEventListener('click', clickHandler)
    }));

    this.code = document.createElement('div');
    this.code.classList.add('code-wrapper');
    const table = document.createElement('table');
    const tbody = document.createElement('tbody');

    for (let i = 0; i < block.matches.length; i++) {
      this.addMatch(tbody, block.matches[i], block.path);
      if (i < block.matches.length - 1) {
        this.addSeparator(tbody);
      }
    }

    table.appendChild(tbody);
    this.code.appendChild(table);
    this.element.appendChild(this.header);
    this.element.appendChild(this.code);
  }

  // Toggles/collapses code view.
  toggle() {
    if (this.element.classList.contains('collapsed')) {
      this.icon.classList.add('icon-chevron-down');
      this.icon.classList.remove('icon-chevron-right');
      this.code.classList.remove('hidden');
      this.header.classList.remove('collapsed');
      this.element.classList.remove('collapsed');
    } else {
      this.icon.classList.remove('icon-chevron-down');
      this.icon.classList.add('icon-chevron-right');
      this.code.classList.add('hidden');
      this.header.classList.add('collapsed');
      this.element.classList.add('collapsed');
    }
  }

  // Returns true, if item has extension.
  hasExtension(ext) {
    return this.block && this.block.ext == ext;
  }

  // Marks element as visible.
  show() {
    this.element.classList.remove('hidden');
  }

  // Marks element as hidden.
  hide() {
    this.element.classList.add('hidden');
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
      if (line.kind == MATCH && line.before_bytes != null && line.after_bytes != null) {
        const before = document.createElement('span');
        before.textContent = `${line.before_bytes}`;
        const matched = document.createElement('span');
        matched.classList.add('highlight-info');
        matched.textContent = `${line.bytes}`;
        const after = document.createElement('span');
        after.textContent = `${line.after_bytes}`;

        code.appendChild(before);
        code.appendChild(matched);
        code.appendChild(after);
      } else {
        code.textContent = `${line.bytes}`;
      }

      // Update condition-based classes
      if (line.kind == BEFORE) {
        icon.classList.add('icon-arrow-small-right', 'text-success');
      } else if (line.kind == AFTER) {
        icon.classList.add('icon-arrow-small-left', 'text-warning');
      } else if (line.kind == MATCH) {
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
        tr.removeEventListener('click', clickHandler);
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
