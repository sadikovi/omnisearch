/** @babel */

import {CompositeDisposable, Disposable} from 'atom';
import ControlView from './control-view';
import ExtensionView from './extension-view';
import Query from './query';
import ResultContentView from './result-content-view';
import ResultFileView from './result-file-view';

export default class SearchView {
  constructor(serializedState) {
    // Use `setProcess` to assign currently running process.
    // Search view does not manage external process anymore, only uses currently active one.
    this.serverProcess = null;

    // Global search query that we modify in order to send to the server.
    this.query = new Query();

    // List of disposables that we create.
    this.disposables = new CompositeDisposable();

    // List of currently available repositories (projects)
    this.paths = atom.project.getPaths();

    this.controlView = new ControlView();

    // Path selector panel
    this.controlView.getProjectSelector().updateForPaths(this.paths);
    // Set initial path for the query.
    this.query.setPath(this.controlView.getProjectSelector().getSelection());
    this.disposables.add(this.controlView.getProjectSelector().onDidChangeSelection(path => {
      this.query.setPath(path);
    }));
    this.query.setUseRegex(this.controlView.useRegex());
    this.disposables.add(this.controlView.onUseRegex(useRegex => {
      this.query.setUseRegex(useRegex);
    }));

    // Main text editor to search
    const editor = this.controlView.getEditor();
    this.disposables.add(editor.onDidChange(() => {
      this.query.setPattern(editor.getText());
    }));

    this.disposables.add(atom.project.onDidChangePaths(projectPaths => {
      this.paths = projectPaths;
      this.controlView.getProjectSelector().updateForPaths(this.paths);
    }));

    this.element = document.createElement('div');
    this.element.classList.add('omnisearch', 'search-view', 'pane-item');
    this.element.setAttribute('tabIndex', '-1');

    this.extensionView = new ExtensionView(this.onExtensionSelected.bind(this));

    this.fileView = new ResultFileView();

    this.contentView = new ResultContentView();

    this.element.appendChild(this.controlView.getElement());
    this.element.appendChild(this.extensionView.getElement());
    this.element.appendChild(this.fileView.getElement());
    this.element.appendChild(this.contentView.getElement());

    // Main action to trigger the search
    this.disposables.add(
      atom.commands.add('atom-text-editor.omnisearch', 'omnisearch:search', () => {
        this.triggerSearch();
      })
    );

    // Focus to the editor
    this.disposables.add(
      atom.commands.add('div.omnisearch', 'omnisearch:focus', () => {
        editor.getElement().focus();
      })
    );

    // Also focus editor when view is active
    this.disposables.add(atom.workspace.onDidChangeActivePaneItem(item => {
      if (this.isEqual(item)) {
        editor.getElement().focus();
      }
    }));

    // Add handles for editor history selection
    this.disposables.add(
      atom.commands.add('atom-text-editor.omnisearch', 'omnisearch:history-prev', () => {
        const prev = this.controlView.getHistory().previous();
        if (prev != null) editor.setText(prev);
      }),
      atom.commands.add('atom-text-editor.omnisearch', 'omnisearch:history-next', () => {
        const next = this.controlView.getHistory().next();
        if (next != null) editor.setText(next);
      })
    );
  }

  // Sets currently running process for this search view.
  // Process lifecycle is outside of this view, and will be handled externally.
  setProcess(proc) {
    this.serverProcess = proc;
  }

  triggerSearch() {
    const data = {
      dir: this.query.getPath(),
      pattern: this.query.getPattern(),
      use_regex: this.query.useRegex()
    };

    this.controlView.getMetrics().update({runStatus: 'Running'});
    this.controlView.getHistory().append(this.query.getPattern());
    this.serverProcess.send(data)
      .then(response => response.json())
      .then(json => json.err ? this.onSearchError(json, true) : this.onSearchSuccess(json))
      .catch(err => this.onSearchError(err, false))
  }

  // Triggers on successful request.
  onSearchSuccess(json) {
    const countBadge = (obj) => {
      if (obj.match == 'exact') {
        return `${obj.count}`;
      } else {
        return `${obj.count}+`;
      }
    }

    // Collect extensions.
    let extensions = new Set([]);
    for (file of json.files) {
      if (!extensions.has(file.ext)) {
        extensions.add(file.ext);
      }
    }

    for (cnt of json.content) {
      if (!extensions.has(cnt.ext)) {
        extensions.add(cnt.ext);
      }
    }

    this.controlView.getMetrics().update({runStatus: `Done, took ${json.time_sec.toFixed(2)} sec`});
    this.extensionView.update(Array.from(extensions).sort());
    this.fileView.updateForFiles(json.files, countBadge(json.file_matches));
    this.contentView.updateForBlocks(json.content, countBadge(json.content_matches));
  }

  // Triggers in case of any error.
  onSearchError(err, isJson) {
    const msg = isJson ? err.msg : `${err}`;
    atom.notifications.addError(`Request error: ${msg}`, {dismissable: true});
    this.controlView.getMetrics().update({runStatus: 'Failed'});
    this.fileView.updateForFiles([], '0');
    this.contentView.updateForBlocks([], '0');
  }

  // Triggers when extension is selected.
  onExtensionSelected(ext) {
    this.fileView.filterForExtension(ext);
    this.contentView.filterForExtension(ext);
  }

  // Tear down any state and detach
  destroy() {
    this.disposables.dispose();
    this.controlView.destroy();
    this.extensionView.destroy();
    this.fileView.destroy();
    this.contentView.destroy();
    this.element.remove();
    // Do not stop external process.
    this.serverProcess = null;
  }

  getTitle() {
    return 'Search';
  }

  getIconName() {
    return 'search'
  }

  getDefaultLocation() {
    // This location will be used if the user hasn't overridden it by dragging the item elsewhere.
    // Valid values are "left", "right", "bottom", and "center" (the default).
    return 'center';
  }

  getAllowedLocations() {
    // The locations into which the item can be moved.
    return ['left', 'right', 'center', 'bottom'];
  }

  getURI() {
    return 'atom://omnisearch';
  }

  getElement() {
    return this.element;
  }

  isEqual(other) {
    return other instanceof SearchView;
  }

  serialize() {
    return {
      // Should match definition in package.json.
      deserializer: 'omnisearch/search-view/SearchView'
    };
  }
}
