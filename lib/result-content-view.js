/** @babel */

import CodeItem from './code-item';
import ResultView from './result-view';

export default class ResultContentView {
  constructor() {
    this.contentList = document.createElement('ul');
    this.contentList.classList.add('list-group', 'content-view');
    this.view = new ResultView('Content', this.contentList, '0');
    this.blocks = [];
  }

  updateForBlocks(blocks, badgeText) {
    while (this.blocks.length > 0) {
      this.blocks.pop().destroy();
    }
    // Update to the new badge
    this.view.updateBadge(badgeText);
    // Create file objects
    if (!blocks) return;
    for (const block of blocks) {
      const item = new CodeItem(block);
      this.blocks.push(item);
      const listItem = document.createElement('li');
      listItem.classList.add('list-item');
      listItem.appendChild(item.getElement());
      this.contentList.appendChild(listItem);
    }
  }

  // Returns root element.
  getElement() {
    return this.view.getElement();
  }

  // Releases resources.
  destroy() {
    this.view.destroy();
  }
}
