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
          return this.getOrCreateSearchView();
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
    }
    if (this.searchView) {
      this.searchView.destroy();
    }
  },

  getOrCreateSearchView(state) {
    if (!this.searchView) {
      this.searchView = new SearchView(state);
    }
    return this.searchView;
  },

  toggle() {
    atom.workspace.toggle('atom://omnisearch');
  },

  deserialize(serialized) {
    return this.getOrCreateSearchView(serialized);
  },

  consumeTreeView(treeView) {
    // Assign tree view to global atom environment,
    // so we can reference it in search view.
    atom.treeView = treeView;
  }
};
