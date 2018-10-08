/** @babel */

export default class Query {
  constructor() {
    this.pattern = '';
    this.path = '';
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

  // Returns pattern.
  getPattern() {
    return this.pattern;
  }

  // Returns path.
  getPath() {
    return this.path;
  }
}
