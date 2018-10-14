/** @babel */

import cp from 'child_process';
import path from 'path';

export default class ServerProcess {
  constructor() {
    this.process = null;
    this.closed = false;
    // Server address
    this.address = null;
    // Start the process
    this.start();
  }

  // Returns current working directory.
  workdir() {
    return path.join(__dirname, '..', 'server', 'target', 'release');
  }

  // Returns command to run.
  command() {
    return path.join(this.workdir(), 'omnisearch');
  }

  // Start the process.
  start() {
    this.process = cp.spawn(this.command(), [], {cwd: this.workdir()});
    this.process.on('exit', (code, signal) => {
      if (code >= 1) {
        atom.notifications.addError(`Fatal error: Process did not exit correctly (${code})`);
      }
      this.closed = true;
    });
    this.process.on('error', (err) => {
      atom.notifications.addError(`Fatal error: ${err}`);
      this.closed = true;
    });

    this.process.stdout.on('data', (data) => {
      if (!this.address) {
        this.address = data.toString();
        console.log(`Listening on ${this.address}`);
      }
    });
  }

  send(input) {
    const options = {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json; charset=utf-8'
      },
      body: JSON.stringify(input)
    };
    return fetch(`http://${this.address}/search`, options);
  }

  // Returns true if process is running.
  isRunning() {
    return this.process != null && !this.closed;
  }

  // Closes this process.
  stop() {
    if (this.process) {
      this.process.kill();
      this.process = null;
    }
  }
}
