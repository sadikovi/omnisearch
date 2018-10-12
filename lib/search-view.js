/** @babel */

import {CompositeDisposable, Disposable, TextEditor} from 'atom';
import cp from 'child_process';
import path from 'path';
import ProjectSelectorView from './project-selector-view';
import Query from './query';
import ResultContentView from './result-content-view';
import ResultFileView from './result-file-view';

export default class SearchView {
  constructor(serializedState) {
    // Use this address to connect to the server.
    this.serverAddress = this.getServerAddress();

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
    this.editor = new TextEditor({mini: true, placeholderText: 'Type and Hit Enter to search...'});
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
    this.element.classList.add('search-view', 'pane-item');
    this.element.setAttribute('tabIndex', '-1');

    const controlPanel = document.createElement('section');
    controlPanel.classList.add('bordered', 'control-panel');
    controlPanel.appendChild(this.projectSelector.getElement());
    controlPanel.appendChild(this.editor.getElement());

    this.fileView = new ResultFileView();

    this.contentView = new ResultContentView();

    this.element.appendChild(controlPanel);
    this.element.appendChild(this.fileView.getElement());
    this.element.appendChild(this.contentView.getElement());

    // Main action to trigger the search
    this.disposables.add(
      atom.commands.add('atom-text-editor.omnisearch', 'omnisearch:search', () => {
        this.triggerSearch();
      })
    );
  }

  triggerSearch() {
    const data = {
      dir: this.query.getPath(),
      pattern: this.query.getPattern(),
      extensions: [] // TODO: handle extensions
    };

    const options = {
      method: "POST",
      headers: {
        "Content-Type": "application/json; charset=utf-8"
      },
      body: JSON.stringify(data)
    };

    fetch("/Users/sadikovi/developer/omnisearch/example.json", options)
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
    this.fileView.updateForFiles(json.files, countBadge(json.file_matches));
    this.contentView.updateForBlocks(json.content, countBadge(json.content_matches));
  }

  // Triggers in case of any error.
  onSearchError(err, isJson) {
    const msg = isJson ? err.msg : `${err}`;
    atom.notifications.addError(`Search error: ${msg}`, {dismissable: true});
    this.fileView.updateForFiles([], '0');
    this.contentView.updateForBlocks([], '0');
  }

  getServerAddress() {
    // We assume that server code has already been built.
    const serverHome = path.join(__dirname, '..', 'server', 'target', 'release');
    const serverPath = path.join(serverHome, 'omnisearch');
    const command = serverPath;

    const childProcess = cp.spawn(command, [], {cwd: serverHome});
    let stderr = '';
    let stdout = '';
    childProcess.stderr.on('data', chunk => {
      stderr += chunk.toString();
      console.log('stderr', chunk.toString());
    });
    childProcess.stdout.on('data', chunk => {
      stdout += chunk.toString();
      console.log('stdout', chunk.toString());
    });
    childProcess.on('close', exitCode => {
      if (exitCode == 0) {
        console.log('exited with ', exitCode, stdout);
      } else {
        console.log('Code', exitCode, stderr + '\n' + stdout);
      }
    });
    console.log("Spawned the process");
    return "127.0.0.1:8080";
  }

  // Tear down any state and detach
  destroy() {
    this.disposables.dispose();
    this.projectSelector.destroy();
    this.fileView.destroy();
    this.contentView.destroy();
    this.element.remove();
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
