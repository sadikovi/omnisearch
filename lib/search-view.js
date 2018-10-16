/** @babel */

import {CompositeDisposable, Disposable, TextEditor} from 'atom';
import ExtensionView from './extension-view';
import MetricsView from './metrics-view';
import ProjectSelectorView from './project-selector-view';
import Query from './query';
import ResultContentView from './result-content-view';
import ResultFileView from './result-file-view';
import ServerProcess from './server-process';

export default class SearchView {
  constructor(serializedState) {
    this.serverProcess = new ServerProcess();

    // Global search query that we modify in order to send to the server.
    this.query = new Query();

    // List of disposables that we create.
    this.disposables = new CompositeDisposable();

    // List of currently available repositories (projects)
    this.paths = atom.project.getPaths();

    // Path selector panel
    this.projectSelector = new ProjectSelectorView();
    this.projectSelector.updateForPaths(this.paths);
    // Set initial path for the query.
    this.query.setPath(this.projectSelector.getSelection());
    this.disposables.add(this.projectSelector.onDidChangeSelection(path => {
      this.query.setPath(path);
    }));

    // Main text editor to search
    this.editor = new TextEditor({
      mini: true,
      placeholderText: 'Type query and hit Enter to search...'
    });
    // Add marker class, so we can bind event
    this.editor.getElement().classList.add('omnisearch');
    this.disposables.add(this.editor.onDidChange(() => {
      this.query.setPattern(this.editor.getText());
    }));

    this.disposables.add(atom.project.onDidChangePaths(projectPaths => {
      this.paths = projectPaths;
      this.projectSelector.updateForPaths(this.paths);
    }));

    this.element = document.createElement('div');
    this.element.classList.add('omnisearch', 'search-view', 'pane-item');
    this.element.setAttribute('tabIndex', '-1');

    this.metricsView = new MetricsView();

    const controlPanel = document.createElement('section');
    controlPanel.classList.add('bordered', 'control-panel');
    controlPanel.appendChild(this.projectSelector.getElement());
    controlPanel.appendChild(this.editor.getElement());
    controlPanel.appendChild(this.metricsView.getElement());

    this.extensionView = new ExtensionView(this.onExtensionSelected.bind(this));

    this.fileView = new ResultFileView();

    this.contentView = new ResultContentView();

    this.element.appendChild(controlPanel);
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
        this.editor.getElement().focus();
      })
    );

    // Also focus editor when view is active
    this.disposables.add(atom.workspace.onDidChangeActivePaneItem(item => {
      if (this.isEqual(item)) {
        this.editor.getElement().focus();
      }
    }));
  }

  triggerSearch() {
    const data = {
      dir: this.query.getPath(),
      pattern: this.query.getPattern()
    };

    this.metricsView.update({runStatus: 'Running'});
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

    this.metricsView.update({runStatus: `Done, took ${json.time_sec.toFixed(2)} sec`});
    this.extensionView.update(Array.from(extensions).sort());
    this.fileView.updateForFiles(json.files, countBadge(json.file_matches));
    this.contentView.updateForBlocks(json.content, countBadge(json.content_matches));
  }

  // Triggers in case of any error.
  onSearchError(err, isJson) {
    const msg = isJson ? err.msg : `${err}`;
    atom.notifications.addError(`Request error: ${msg}`, {dismissable: true});
    this.metricsView.update({runStatus: 'Failed'});
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
    this.projectSelector.destroy();
    this.metricsView.destroy();
    this.extensionView.destroy();
    this.fileView.destroy();
    this.contentView.destroy();
    this.element.remove();
    this.serverProcess.stop();
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
