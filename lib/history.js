/** @babel */

export default class History {
  constructor(state) {
    // current position in the buffer, can equal to the length of the buffer, in this case
    // we show empty field.
    this.current = 0;
    // buffer of previous searches
    this.buffer = [];
  }

  assertPosition() {
    if (this.current < 0 || this.current > this.buffer.length) {
      throw Error("Invalid position " + this.current);
    }
  }

  append(text) {
    if (this.buffer.length == 0 || this.buffer[this.buffer.length - 1] != text) {
      this.buffer.push(text);
    }
    this.current = this.buffer.length;
  }

  previous() {
    this.assertPosition();
    if (this.current > 0) {
      return this.buffer[--this.current];
    }
    return null;
  }

  next() {
    this.assertPosition();
    if (this.current < this.buffer.length) {
      return this.buffer[this.current++];
    }
    return "";
  }

  serialize() {
    return {
      'buffer': this.buffer,
      'pos': this.current
    };
  }
}
