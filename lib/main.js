'use babel';

import {CompositeDisposable, Disposable} from 'atom';
import SearchInfoView from './search';

export default {
  subscriptions: null,

  activate(state) {
    this.subscriptions = new CompositeDisposable(
      atom.workspace.addOpener(uri => {
        if (uri === 'atom://omnisearch') {
          return new SearchInfoView();
        }
      }),

      atom.commands.add('atom-workspace', {
        'omnisearch:toggle': () => this.toggle()
      }),

      new Disposable(() => {
        atom.workspace.getPaneItems().forEach(item => {
          if (item instanceof SearchInfoView) {
            item.destroy();
          }
        });
      })
    );
  },

  deactivate() {
    if (this.subscriptions) {
      this.subscriptions.dispose();
    }
  },

  toggle() {
    atom.workspace.toggle('atom://omnisearch');
  }
};
