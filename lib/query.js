/** @babel */

export default class Query {
  constructor() {
    this.pattern = null;
    this.path = null;
  }

  // Whether or not query will be interpreted correctly.
  isValid() {
    return this.pattern && this.path;
  }

  // Sets search directory.
  setPath(path) {
    this.path = path;
  }

  // Sets search pattern.
  setPattern(pattern) {
    this.pattern = pattern;
  }
}
