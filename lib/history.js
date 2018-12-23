/** @babel */

export default class History {
  constructor(state) {
    // current position in the buffer
    this.current = 0;
    // buffer of previous searches
    this.buffer = [];
  }

  append(text) {
    if (this.buffer.length == 0 || this.buffer[this.buffer.length - 1] != text) {
      this.buffer.push(text);
    }
    this.current = this.buffer.length - 1;
  }

  previous() {
    // Indicates that there is no history there.
    if (this.current >= this.buffer.length) {
      return null;
    }
    if (this.current > 0) {
      this.current--;
    }
    return this.buffer[this.current];
  }

  next() {
    // Indicates that there is no history there.
    if (this.current >= this.buffer.length) {
      return null;
    }
    // If next is called on the index that is already the last, return empty string.
    if (this.current == this.buffer.length - 1) {
      return "";
    }
    if (this.current < this.buffer.length - 1) {
      this.current++;
    }
    return this.buffer[this.current];
  }

  serialize() {
    return {
      'buffer': this.buffer,
      'pos': this.current
    };
  }
}
