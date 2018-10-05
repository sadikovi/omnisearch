/** @babel */

import {CompositeDisposable, Disposable, TextEditor} from 'atom';
import ProjectSelectorView from './project-selector-view';
import ResultFileView from './result-file-view';

export default class SearchView {
  constructor(serializedState) {
    this.disposables = new CompositeDisposable();

    // List of currently available repositories (projects)
    this.paths = atom.project.getPaths();

    // Path selector panel
    this.projectSelector = new ProjectSelectorView();
    this.projectSelector.updateForPaths(this.paths);
    this.disposables.add(this.projectSelector.onDidChangeSelection(path => {
      console.log(path);
    }));

    // Main text editor to search
    this.editor = new TextEditor({mini: true, placeholderText: 'Type to search...'});
    this.disposables.add(this.editor.onDidStopChanging(() => {
      this.search({
        paths: this.paths,
        pattern: this.editor.getText()
      });
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

    const fileView = new ResultFileView();
    fileView.updateForFiles([
      {
        path: "/Users/sadikovi/Developer/spark/core/src/main/java/org/apache/spark/JobExecutionStatus.java",
        ext: "java"
      },
      {
        path: "/Users/sadikovi/Developer/spark/core/src/main/scala/org/apache/spark/memory/ExecutionMemoryPool.scala",
        ext: "scala"
      },
      {
        path: "/Users/sadikovi/Developer/spark/sql/core/src/test/scala/org/apache/spark/sql/TestQueryExecutionListener.scala",
        ext: "scala"
      }
    ], '5+');

    this.element.appendChild(controlPanel);
    this.element.appendChild(fileView.getElement());
  }

  search(data) {
    console.log(data);
  }

  // Tear down any state and detach
  destroy() {
    this.disposables.dispose();
    this.projectSelector.destroy();
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
