import { ChildProcess } from 'child_process';
import { Readable } from 'stream';

/**
 * Read from a stream until specific text appears
 */
export async function readUntil(
  stream: Readable,
  text: string,
  timeoutMs = 10000
): Promise<string> {
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(
      () => reject(new Error(`Timeout waiting for "${text}"`)),
      timeoutMs
    );

    let buffer = '';
    const onData = (data: Buffer) => {
      buffer += data.toString();
      if (buffer.includes(text)) {
        clearTimeout(timeout);
        stream.removeListener('data', onData);
        resolve(buffer);
      }
    };

    stream.on('data', onData);
  });
}

/**
 * Wait for CLI to show the interactive prompt (>)
 */
export async function waitForPrompt(proc: ChildProcess, timeoutMs = 15000): Promise<void> {
  if (!proc.stdout) {
    throw new Error('Process has no stdout');
  }
  await readUntil(proc.stdout, '>', timeoutMs);
}

/**
 * Poll the API until a session with the given name appears
 */
export async function waitForSession(
  serverUrl: string,
  sessionName: string,
  timeoutMs = 15000
): Promise<{ id: string; name: string }> {
  const startTime = Date.now();

  while (Date.now() - startTime < timeoutMs) {
    try {
      const response = await fetch(`${serverUrl}/api/claude/sessions`);
      const data = await response.json() as { sessions: Array<{ id: string; name: string }> };
      const session = data.sessions?.find(s => s.name === sessionName);
      if (session) {
        return session;
      }
    } catch {
      // Server might not be ready
    }
    await new Promise(r => setTimeout(r, 500));
  }

  throw new Error(`Timeout waiting for session "${sessionName}"`);
}

/**
 * Collect all output from a stream
 */
export function collectOutput(stream: Readable): () => string {
  let buffer = '';
  stream.on('data', (data) => {
    buffer += data.toString();
  });
  return () => buffer;
}
