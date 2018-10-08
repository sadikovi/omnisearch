/** @babel */

import FileItem from './file-item';
import ResultView from './result-view';

export default class ResultFileView {
  constructor() {
    this.fileList = document.createElement('ul');
    this.fileList.classList.add('list-group', 'file-view');
    this.view = new ResultView('Files', this.fileList, '0');
    this.files = [];
  }

  updateForFiles(files, badgeText) {
    while (this.files.length > 0) {
      this.files.pop().destroy();
    }
    // Update to the new badge
    this.view.updateBadge(badgeText);
    // Create file objects
    if (!files) return;
    for (const file of files) {
      const item = new FileItem(file.path);
      this.files.push(item);
      const listItem = document.createElement('li');
      listItem.classList.add('list-item');
      listItem.appendChild(item.getElement());
      this.fileList.appendChild(listItem);
    }
  }

  // Returns root element.
  getElement() {
    return this.view.getElement();
  }

  // Releases resources.
  destroy() {
    this.view.destroy();
    for (const file of this.files) {
      file.destroy();
    }
  }
}
