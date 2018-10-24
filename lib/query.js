/** @babel */

export default class Query {
  constructor() {
    this.pattern = '';
    this.path = '';
    this.regex = false;
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

  // Sets boolean flag for using regex.
  setUseRegex(useRegex) {
    this.regex = !!useRegex;
  }

  // Returns pattern.
  getPattern() {
    return this.pattern;
  }

  // Returns path.
  getPath() {
    return this.path;
  }

  // Returns true, if regex is enabled.
  useRegex() {
    return this.regex;
  }
}
