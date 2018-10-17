/** @babel */

import {CompositeDisposable, Disposable} from 'atom';
import ErrorView from './error-view';
import SearchView from './search-view';
import ServerProcess from './server-process';

export default {
  subscriptions: null,
  process: null,

  activate(state) {
    const platform = {'darwin': 'mac', 'linux': 'linux'}[process.platform];
    // If platform is supported, start the process
    if (platform) {
      this.process = new ServerProcess();
    }

    this.subscriptions = new CompositeDisposable(
      atom.workspace.addOpener(uri => {
        if (uri === 'atom://omnisearch') {
          if (!platform) {
            const msg = `Platform ${process.platform} is not supported by omnisearch`;
            atom.notifications.addError(msg);
            return new ErrorView(msg);
          } else if (!this.process) {
            const msg = `Internal error: process is not running`;
            atom.notifications.addError(msg);
            return new ErrorView(msg);
          } else {
            const view = new SearchView();
            view.setProcess(this.process);
            return view;
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
    if (this.process) {
      this.process.stop();
      this.process = null;
    }
  },

  toggle() {
    atom.workspace.toggle('atom://omnisearch');
  },

  deserialize(state) {
    const view = new SearchView(state);
    view.setProcess(this.process);
    return view;
  }
};
