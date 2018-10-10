/** @babel */

import {CompositeDisposable, Disposable} from 'atom';
import ErrorView from './error-view';
import SearchView from './search-view';

export default {
  subscriptions: null,

  activate(state) {
    const platform = {'darwin': 'mac', 'linux': 'linux'}[process.platform];

    this.subscriptions = new CompositeDisposable(
      atom.workspace.addOpener(uri => {
        if (uri === 'atom://omnisearch') {
          if (!platform) {
            const msg = `Platform ${process.platform} is not supported by omnisearch`;
            atom.notifications.addError(msg);
            return new ErrorView(msg);
          } else {
            return new SearchView();
          }
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
  }
};
