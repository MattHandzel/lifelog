import '@testing-library/jest-dom';
import { vi } from 'vitest';

// Mock Tauri's invoke API so components can be tested without the Tauri runtime.
// Individual tests can override invoke behavior via mockInvoke().
const invokeHandlers = new Map<string, (...args: unknown[]) => unknown>();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(async (cmd: string, args?: unknown) => {
    const handler = invokeHandlers.get(cmd);
    if (handler) return handler(args);
    console.warn(`[test] Unhandled invoke: ${cmd}`);
    return undefined;
  }),
}));

/**
 * Register a mock handler for a Tauri invoke command.
 * Call in beforeEach or at the top of a test to control what invoke returns.
 */
export function mockInvoke(cmd: string, handler: (...args: unknown[]) => unknown) {
  invokeHandlers.set(cmd, handler);
}

/**
 * Clear all invoke mock handlers.
 */
export function clearInvokeMocks() {
  invokeHandlers.clear();
}
