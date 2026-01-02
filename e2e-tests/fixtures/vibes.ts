import { test as base } from '@playwright/test';
import { spawn, ChildProcess } from 'child_process';
import { Readable } from 'stream';
import * as path from 'path';
import * as fs from 'fs';
import * as os from 'os';

export type VibesFixture = {
  serverUrl: string;
  serverPort: number;
  cli: (...args: string[]) => ChildProcess;
};

// Get the project root directory (parent of e2e-tests)
const projectRoot = path.resolve(import.meta.dirname, '../..');
const testConfigDir = path.join(projectRoot, '.vibes');
const testConfigPath = path.join(testConfigDir, 'config.toml');

// Find the vibes binary
// Priority: VIBES_BIN env var > debug build > release build
function findVibesBinary(): string {
  // Allow explicit override via environment variable
  if (process.env.VIBES_BIN) {
    if (!fs.existsSync(process.env.VIBES_BIN)) {
      throw new Error(`VIBES_BIN path does not exist: ${process.env.VIBES_BIN}`);
    }
    return process.env.VIBES_BIN;
  }

  // Prefer debug build (faster CI), fallback to release
  const debugPath = path.join(projectRoot, 'target/debug/vibes');
  const releasePath = path.join(projectRoot, 'target/release/vibes');

  if (fs.existsSync(debugPath)) {
    return debugPath;
  }
  if (fs.existsSync(releasePath)) {
    return releasePath;
  }
  throw new Error('vibes binary not found. Run "cargo build" or set VIBES_BIN.');
}

// Get a random port by binding to port 0 and reading the assigned port
async function getRandomPort(): Promise<number> {
  const { createServer } = await import('net');
  return new Promise((resolve, reject) => {
    const server = createServer();
    server.listen(0, '127.0.0.1', () => {
      const addr = server.address();
      if (addr && typeof addr === 'object') {
        const port = addr.port;
        server.close(() => resolve(port));
      } else {
        reject(new Error('Failed to get port'));
      }
    });
    server.on('error', reject);
  });
}

export const test = base.extend<VibesFixture>({
  serverPort: [async ({}, use) => {
    const vibesBin = findVibesBinary();
    console.log(`[vibes] Using binary: ${vibesBin}`);

    // Create isolated temp directory for Iggy data
    const testId = `vibes-e2e-${Date.now()}-${Math.random().toString(36).slice(2)}`;
    const iggyDataDir = path.join(os.tmpdir(), testId);
    fs.mkdirSync(iggyDataDir, { recursive: true });

    // Get random ports for Iggy to avoid conflicts between parallel tests
    const iggyPort = await getRandomPort();
    const iggyHttpPort = await getRandomPort();

    console.log(`[vibes] Using Iggy data dir: ${iggyDataDir}`);
    console.log(`[vibes] Using Iggy ports: TCP=${iggyPort}, HTTP=${iggyHttpPort}`);

    // Start server on random port with isolated Iggy config
    const server = spawn(
      vibesBin,
      ['serve', '--port', '0'],
      {
        cwd: projectRoot,
        env: {
          ...process.env,
          VIBES_IGGY_DATA_DIR: iggyDataDir,
          VIBES_IGGY_PORT: String(iggyPort),
          VIBES_IGGY_HTTP_PORT: String(iggyHttpPort),
        }
      }
    );

    // Log server stderr for debugging
    server.stderr?.on('data', (data) => {
      console.error(`[vibes server] ${data.toString().trim()}`);
    });

    const port = await waitForPort(server.stdout!);
    console.log(`[vibes] Server started on http://127.0.0.1:${port}`);

    // Backup existing config if present
    let existingConfig: string | null = null;
    try {
      existingConfig = fs.readFileSync(testConfigPath, 'utf-8');
    } catch {
      // No existing config
    }

    // Write config file for CLI to use
    fs.mkdirSync(testConfigDir, { recursive: true });
    fs.writeFileSync(testConfigPath, `[server]\nport = ${port}\n`);

    await use(port);

    server.kill('SIGTERM');
    console.log('[vibes] Server stopped');

    // Restore or cleanup config
    try {
      if (existingConfig !== null) {
        fs.writeFileSync(testConfigPath, existingConfig);
      } else {
        fs.unlinkSync(testConfigPath);
      }
    } catch {
      // Ignore cleanup errors
    }

    // Clean up Iggy data directory
    try {
      fs.rmSync(iggyDataDir, { recursive: true, force: true });
      console.log(`[vibes] Cleaned up: ${iggyDataDir}`);
    } catch {
      // Ignore cleanup errors
    }
  }, { scope: 'test' }],

  serverUrl: async ({ serverPort }, use) => {
    await use(`http://127.0.0.1:${serverPort}`);
  },

  cli: async ({ serverPort }, use) => {
    const processes: ChildProcess[] = [];
    const vibesBin = findVibesBinary();

    const spawnCli = (...args: string[]) => {
      // Use --no-serve since we have our own server running
      // Set VIBES_CONFIG to point to our test config
      const proc = spawn(
        vibesBin,
        [...args, '--no-serve'],
        {
          cwd: projectRoot,
          env: {
            ...process.env,
            // If the CLI supports VIBES_CONFIG, use it
            // Otherwise we rely on .vibes-test/config.toml
          }
        }
      );
      processes.push(proc);

      // Log CLI output for debugging
      proc.stderr?.on('data', (data) => {
        console.error(`[vibes cli] ${data.toString().trim()}`);
      });

      return proc;
    };

    await use(spawnCli);

    // Cleanup all spawned processes
    processes.forEach(p => p.kill('SIGTERM'));
  },
});

async function waitForPort(stdout: Readable): Promise<number> {
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(
      () => reject(new Error('Timeout waiting for server to start')),
      30000
    );

    let buffer = '';
    stdout.on('data', (data) => {
      buffer += data.toString();
      // Look for "listening on 127.0.0.1:XXXX" or similar patterns
      const match = buffer.match(/listening on (?:127\.0\.0\.1|0\.0\.0\.0):(\d+)/);
      if (match) {
        clearTimeout(timeout);
        resolve(parseInt(match[1], 10));
      }
    });

    stdout.on('error', (err) => {
      clearTimeout(timeout);
      reject(err);
    });

    stdout.on('close', () => {
      clearTimeout(timeout);
      reject(new Error('Server stdout closed before port was found'));
    });
  });
}

export { expect } from '@playwright/test';
