/** @babel */

import {CompositeDisposable, Disposable} from 'atom';
import ErrorView from './error-view';
import SearchView from './search-view';
import ServerProcess from './server-process';

export default {
  subscriptions: null,
  process: null,

  activate(state) {
    // Start the process, if applicable
    this.getOrCreateProcess();
    console.log('activate', this.process);

    this.subscriptions = new CompositeDisposable(
      atom.workspace.addOpener(uri => {
        if (uri === 'atom://omnisearch') {
          if (!this.isPlatformSupported()) {
            const msg = `Platform ${process.platform} is not supported by omnisearch`;
            atom.notifications.addError(msg);
            return new ErrorView(msg);
          } else if (!this.getOrCreateProcess()) {
            const msg = `Internal error: process is not running`;
            atom.notifications.addError(msg);
            return new ErrorView(msg);
          } else {
            const view = new SearchView();
            view.setProcess(this.getOrCreateProcess());
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
    console.log('deactivate', this.process);
    if (this.subscriptions) {
      this.subscriptions.dispose();
      this.subscriptions = null;
    }
    this.stopProcess();
  },

  toggle() {
    atom.workspace.toggle('atom://omnisearch');
  },

  deserialize(state) {
    const view = new SearchView(state);
    view.setProcess(this.getOrCreateProcess());
    return view;
  },

  isPlatformSupported() {
    return {'darwin': 'mac', 'linux': 'linux'}[process.platform] != null;
  },

  getOrCreateProcess() {
    if (!this.process && this.isPlatformSupported()) {
      this.process = new ServerProcess();
    }
    return this.process;
  },

  stopProcess() {
    if (this.process) {
      this.process.stop();
      this.process = null;
    }
  }
};
