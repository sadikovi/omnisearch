/** @babel */

export default class MetricsView {
  constructor() {
    this.element = document.createElement('div');
    this.element.classList.add('metrics', 'block');

    // Run status
    this.runStatus = document.createElement('span');
    this.runStatus.classList.add('inline-block');

    this.element.appendChild(this.runStatus);

    this.update({});
  }

  // Updates metrics.
  update(obj) {
    this._updateMetric(this.runStatus, obj.runStatus);
  }

  // Internal method to update metric.
  _updateMetric(metric, value) {
    if (value) {
      metric.textContent = `${value}`;
      metric.classList.remove('hidden');
    } else {
      metric.classList.add('hidden');
    }
  }

  destroy() {
    this.element.remove();
  }

  getElement() {
    return this.element;
  }
}
