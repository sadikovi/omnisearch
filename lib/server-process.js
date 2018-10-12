/** @babel */

import cp from 'child_process';

export default class ServerProcess {
  constructor(command, workdir) {
    this.command = command;
    this.workdir = workdir;
    this.process = null;
    this.stderr = '';
    this.stdout = '';
    this.exitCode = null;
  }

  // Start the process.
  spawn(onStdout, onStderr, onClose) {
    this.process = cp.spawn(this.command, [], {cwd: this.workdir});
    this.process.stderr.on('data', chunk => {
      const data = chunk.toString();
      this.stderr += data;
      if (onStderr) {
        onStderr(data);
      }
    });
    this.process.stdout.on('data', chunk => {
      const data = chunk.toString();
      this.stdout += data;
      if (onStdout) {
        onStdout(data);
      }
    });
    this.process.on('close', exitCode => {
      this.exitCode = exitCode;
      if (onClose) {
        onClose(exitCode, this.stdout, this.stderr);
      }
    });
  }

  // Blocks and waits for printed address.
  getServerAddress() {
    while (this.isRunning() && this.stdout.length == 0) { }
    return this.stdout.trim();
  }

  // Returns true if process is running.
  isRunning() {
    return this.process != null && this.exitCode != null;
  }

  // Closes this process.
  close() {
    if (this.process) {
      this.process.kill();
      this.process = null;
    }
  }
}
