/** @babel */

import {CompositeDisposable, Disposable, Emitter, TextEditor} from 'atom';

import History from './history';
import MetricsView from './metrics-view';
import ProjectSelectorView from './project-selector-view';

// Class for the main control panel
export default class ControlView {
  constructor() {
    this.disposables = new CompositeDisposable();
    this.emitter = new Emitter();

    this.projectSelector = new ProjectSelectorView();

    this.editor = new TextEditor({
      mini: true,
      placeholderText: 'Type query and hit Enter to search...'
    });
    // Add marker class, so we can bind event
    this.editor.getElement().classList.add('omnisearch', 'inline-block');
    this.editor.history = new History();

    this.metricsView = new MetricsView();

    this.useRegexControl = document.createElement('button');
    this.useRegexControl.classList.add('btn', 'inline-block');
    this.useRegexControl.textContent = 'Use Regex';

    const clickHandler = (event) => {
      if (this.useRegexControl.classList.contains('selected')) {
        this.useRegexControl.classList.remove('selected');
        this.emitter.emit('omnisearch-on-use-regex', false);
      } else {
        this.useRegexControl.classList.add('selected');
        this.emitter.emit('omnisearch-on-use-regex', true);
      }
    }
    this.useRegexControl.addEventListener('click', clickHandler);
    this.disposables.add(new Disposable(() => {
      this.useRegexControl.removeEventListener('click', clickHandler)
    }));

    const editorView = document.createElement('div');
    editorView.classList.add('block');
    editorView.appendChild(this.editor.getElement());
    editorView.appendChild(this.useRegexControl);

    this.element = document.createElement('section');
    this.element.classList.add('bordered', 'control-panel');
    this.element.appendChild(this.projectSelector.getElement());
    this.element.appendChild(editorView);
    this.element.appendChild(this.metricsView.getElement());
  }

  destroy() {
    this.disposables.dispose();
    this.emitter.dispose();
    this.element.remove();
  }

  // Returns true if regex mode is enabled.
  useRegex() {
    return this.useRegexControl.classList.contains('selected');
  }

  // Fires when useRegex changes.
  onUseRegex(func) {
    return this.emitter.on('omnisearch-on-use-regex', func);
  }

  // Returns instance of editor history.
  getHistory() {
    return this.editor.history;
  }

  getProjectSelector() {
    return this.projectSelector;
  }

  getEditor() {
    return this.editor;
  }

  getMetrics() {
    return this.metricsView;
  }

  getElement() {
    return this.element;
  }
}
