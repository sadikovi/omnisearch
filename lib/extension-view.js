/** @babel */

import ExtensionItem from './extension-item';

export default class ExtensionView {
  constructor(onSelected) {
    this.element = document.createElement('section');
    // Mark extensions view as hidden when it is initialised
    this.element.classList.add('bordered', 'extensions', 'block', 'hidden');
    this.extensions = [];
    this.activeExtension = null;
    this.onSelected = onSelected;
  }

  // Updates current list of extensions.
  update(extensions) {
    // Remove previous set of elements, if any.
    for (elem of this.extensions) {
      elem.destroy();
    }
    // Reset list
    this.extensions = [];
    if (extensions && extensions.length > 0) {
      // Add new ones.
      for (ext of extensions) {
        const elem = new ExtensionItem(ext, (tag) => {
          if (this.activeExtension) {
            this.activeExtension.toggle();
          }
          if (this.activeExtension == elem) {
            // unselect element
            this.activeExtension = null;
            this.onSelected(null);
          } else {
            elem.toggle();
            this.activeExtension = elem;
            this.onSelected(tag);
          }
        });
        this.extensions.push(elem);
        this.element.appendChild(elem.getElement());
      }
      this.element.classList.remove('hidden');
    } else {
      this.element.classList.add('hidden');
    }
  }

  destroy() {
    for (elem of this.extensions) {
      elem.destroy();
    }
    this.element.remove();
  }

  getElement() {
    return this.element;
  }
}
