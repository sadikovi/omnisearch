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
      const item = new FileItem(file.path, file.ext);
      this.files.push(item);
      this.fileList.appendChild(item.getElement());
    }
  }

  // Filter for extension, ext can be null.
  filterForExtension(ext) {
    if (ext == null) {
      for (item of this.files) {
        item.show();
      }
    } else {
      for (item of this.files) {
        if (item.hasExtension(ext)) {
          item.show();
        } else {
          item.hide();
        }
      }
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
