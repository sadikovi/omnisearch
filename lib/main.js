/** @babel */

import {CompositeDisposable, Disposable} from 'atom';
import SearchView from './search-view';

export default {
  subscriptions: null,

  activate(state) {
    this.subscriptions = new CompositeDisposable(
      atom.workspace.addOpener(uri => {
        if (uri === 'atom://omnisearch') {
          return new SearchView();
        }
      }),

      atom.commands.add('atom-workspace', {
        'omnisearch:toggle': () => this.toggle()
      }),

      new Disposable(() => {
        atom.workspace.getPaneItems().forEach(item => {
          if (item instanceof SearchView) {
            item.destroy();
          }
        });
      })
    );
  },

  deactivate() {
    if (this.subscriptions) {
      this.subscriptions.dispose();
      this.subscriptions = null;
    }
  },

  toggle() {
    atom.workspace.toggle('atom://omnisearch');
  },

  deserialize(state) {
    return new SearchView(state);
  },

  consumeTreeView(treeView) {
    // Assign tree view to global atom environment,
    // so we can reference it in search view.
    atom.treeView = treeView;
  }
};
