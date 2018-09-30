'use babel';

import SearchClient from './client';

export default {
  activate(state) {
    this.client = new SearchClient();
  },

  deactivate() {
    if (this.client) {
      this.client.destroy();
      this.client = null;
    }
  },

  consumeStatusBar(statusBar) {
    if (this.client) {
      this.client.setStatusBar(statusBar);
    }
  }
};
