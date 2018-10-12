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
  spawn() {
    this.process = cp.spawn(this.command, [], {cwd: this.workdir});
    this.process.stderr.on('data', chunk => this.stderr += chunk.toString());
    this.process.stdout.on('data', chunk => this.stdout += chunk.toString());
    this.process.on('close', exitCode => this.exitCode = exitCode);
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
