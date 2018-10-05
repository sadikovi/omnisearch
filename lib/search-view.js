/** @babel */

import {CompositeDisposable, Disposable, TextEditor} from 'atom';
import ProjectSelectorView from './project-selector-view';
import ResultContentView from './result-content-view';
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

    const contentView = new ResultContentView();
    contentView.updateForBlocks([
      {
        "path": "/Users/sadikovi/Developer/spark/pom.xml",
        "ext": "xml",
        "matches": [
          {
            "lines": [
              {
                "kind": "before",
                "num": 224,
                "bytes": "      things breaking.\n",
                "truncated": false
              },
              {
                "kind": "before",
                "num": 225,
                "bytes": "    -->\n",
                "truncated": false
              },
              {
                "kind": "match",
                "num": 226,
                "bytes": "    <spark.test.home>${session.executionRootDirectory}</spark.test.home>\n",
                "truncated": false
              },
              {
                "kind": "after",
                "num": 227,
                "bytes": "\n",
                "truncated": false
              },
              {
                "kind": "after",
                "num": 228,
                "bytes": "    <CodeCacheSize>512m</CodeCacheSize>\n",
                "truncated": false
              }
            ]
          },
          {
            "lines": [
              {
                "kind": "before",
                "num": 2008,
                "bytes": "          <artifactId>maven-enforcer-plugin</artifactId>\n",
                "truncated": false
              },
              {
                "kind": "before",
                "num": 2009,
                "bytes": "          <version>3.0.0-M1</version>\n",
                "truncated": false
              },
              {
                "kind": "match",
                "num": 2010,
                "bytes": "          <executions>\n",
                "truncated": false
              },
              {
                "kind": "match",
                "num": 2011,
                "bytes": "            <execution>\n",
                "truncated": false
              },
              {
                "kind": "after",
                "num": 2012,
                "bytes": "              <id>enforce-versions</id>\n",
                "truncated": false
              },
              {
                "kind": "after",
                "num": 2013,
                "bytes": "              <goals>\n",
                "truncated": false
              }
            ]
          },
          {
            "lines": [
              {
                "kind": "before",
                "num": 2039,
                "bytes": "                </rules>\n",
                "truncated": false
              },
              {
                "kind": "before",
                "num": 2040,
                "bytes": "              </configuration>\n",
                "truncated": false
              },
              {
                "kind": "match",
                "num": 2041,
                "bytes": "            </execution>\n",
                "truncated": false
              },
              {
                "kind": "match",
                "num": 2042,
                "bytes": "          </executions>\n",
                "truncated": false
              },
              {
                "kind": "after",
                "num": 2043,
                "bytes": "        </plugin>\n",
                "truncated": false
              },
              {
                "kind": "after",
                "num": 2044,
                "bytes": "        <plugin>\n",
                "truncated": false
              }
            ]
          }
        ]
      }
    ], '4+');

    this.element.appendChild(controlPanel);
    this.element.appendChild(fileView.getElement());
    this.element.appendChild(contentView.getElement());
  }

  search(data) {
    console.log(data);
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
