/** @babel */

import {CompositeDisposable, Disposable} from 'atom';
import SearchView from './search-view';

export default {
  searchView: null,
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
      })
    );
  },

  deactivate() {
    if (this.subscriptions) {
      this.subscriptions.dispose();
      this.subscriptions = null;
    }
    if (this.searchView) {
      this.searchView.destroy();
      this.searchView = null;
    }
  },

  toggle() {
    atom.workspace.toggle('atom://omnisearch');
  },

  deserialize(serialized) {
    return new SearchView(serialized);
  },

  consumeTreeView(treeView) {
    // Assign tree view to global atom environment,
    // so we can reference it in search view.
    atom.treeView = treeView;
  }
};
