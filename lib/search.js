/** @babel */
/** @jsx etch.dom */

import {TextEditor} from 'atom';
import {getSettingDescription} from './rich-description';

export default class SearchInfoView {
  constructor(serializedState) {
    this.element = document.createElement('div');
    this.element.classList.add('search-view', 'pane-item');
    this.element.setAttribute('tabIndex', '-1');

    const section = document.createElement('section');
    section.classList.add('section', 'search-panel');

    const container = document.createElement('div');
    container.classList.add('section-container');

    const title = document.createElement('div');
    title.classList.add('block', 'section-heading', 'icon', 'icon-code');
    title.textContent = 'Editor Settings';

    const subtitle = document.createElement('div');
    subtitle.classList.add('text', 'icon', 'icon-question');
    subtitle.setAttribute('tabIndex', '-1');
    subtitle.textContent =
      'These settings are related to text editing. ' +
      'Some of these can be overriden on a per-language basis. ' +
      'Check language settings by clicking its package card in the';

    const body = document.createElement('div');
    body.classList.add('section-body');

    body.appendChild(this.elementForEditor('editor', 'search', '11'));
    container.appendChild(title);
    container.appendChild(subtitle);
    container.appendChild(body);
    section.appendChild(container);
    this.element.appendChild(section);
  }

  elementForEditor(namespace, name, value) {
    let keyPath = `${namespace}.${name}`
    let type = 'string'

    const fragment = document.createDocumentFragment()

    const label = document.createElement('label')
    label.classList.add('control-label')

    const titleDiv = document.createElement('div')
    titleDiv.classList.add('setting-title')
    titleDiv.textContent = name
    label.appendChild(titleDiv)

    const descriptionDiv = document.createElement('div')
    descriptionDiv.classList.add('setting-description')
    descriptionDiv.innerHTML = getSettingDescription(keyPath)
    label.appendChild(descriptionDiv)
    fragment.appendChild(label)

    const controls = document.createElement('div')
    controls.classList.add('controls')

    const editorContainer = document.createElement('div')
    editorContainer.classList.add('editor-container')

    const editor = new TextEditor({mini: true})
    editor.element.id = keyPath
    editor.element.setAttribute('type', type)
    editorContainer.appendChild(editor.element)
    controls.appendChild(editorContainer)
    fragment.appendChild(controls)

    return fragment
  }

  // Tear down any state and detach
  destroy() {
    this.element.remove();
  }

  getTitle() {
    return 'Search';
  }

  getIconName () {
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
    return 'atom://omnisearch'
  }

  getElement() {
    return this.element;
  }
}
