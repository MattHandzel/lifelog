import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import TimelineDashboard from '../components/TimelineDashboard';
import { mockInvoke, clearInvokeMocks } from './setup';

describe('TimelineDashboard', () => {
  beforeEach(() => {
    clearInvokeMocks();
  });

  afterEach(() => {
    clearInvokeMocks();
  });

  it('renders the timeline header and search controls', () => {
    render(<TimelineDashboard />);
    expect(screen.getByText('Timeline')).toBeInTheDocument();
    expect(screen.getByPlaceholderText('Enter search query...')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /search/i })).toBeInTheDocument();
    expect(screen.getByText('Reset')).toBeInTheDocument();
  });

  it('shows "No results found" initially', () => {
    render(<TimelineDashboard />);
    expect(screen.getByText('No results found')).toBeInTheDocument();
  });

  it('calls query_timeline on search and displays results', async () => {
    const mockEntries = [
      { uuid: 'abc12345-def6-7890-ghij-klmnopqrstuv', origin: 'collector:Screen', modality: 'Screen', timestamp: 1700000000 },
      { uuid: 'xyz98765-uvwx-4321-abcd-efghijklmnop', origin: 'collector:Audio', modality: 'Audio', timestamp: 1700001000 },
    ];

    mockInvoke('query_timeline', () => mockEntries);

    render(<TimelineDashboard />);

    const searchInput = screen.getByPlaceholderText('Enter search query...');
    fireEvent.change(searchInput, { target: { value: 'test query' } });

    const searchButton = screen.getByRole('button', { name: /search/i });
    fireEvent.click(searchButton);

    await waitFor(() => {
      expect(screen.getByText('Screen')).toBeInTheDocument();
      expect(screen.getByText('Audio')).toBeInTheDocument();
    });

    // UUIDs are truncated to 16 chars + "..."
    expect(screen.getByText('abc12345-def6-78...')).toBeInTheDocument();
  });

  it('passes collectorId to invoke when provided', async () => {
    let capturedArgs: unknown = null;
    mockInvoke('query_timeline', (args) => {
      capturedArgs = args;
      return [];
    });

    render(<TimelineDashboard collectorId="test-collector-01" />);

    const searchButton = screen.getByRole('button', { name: /search/i });
    fireEvent.click(searchButton);

    await waitFor(() => {
      expect(capturedArgs).toBeDefined();
      expect((capturedArgs as Record<string, unknown>).collectorId).toBe('test-collector-01');
    });
  });

  it('converts date inputs to unix seconds', async () => {
    let capturedArgs: unknown = null;
    mockInvoke('query_timeline', (args) => {
      capturedArgs = args;
      return [];
    });

    render(<TimelineDashboard />);

    // Set a start date — find the datetime-local input inside the "Start Date" label's parent
    const startLabel = screen.getByText('Start Date');
    const startInput = startLabel.parentElement!.querySelector('input[type="datetime-local"]')!;
    fireEvent.change(startInput, { target: { value: '2025-01-15T10:00' } });

    const searchButton = screen.getByRole('button', { name: /search/i });
    fireEvent.click(searchButton);

    await waitFor(() => {
      const args = capturedArgs as Record<string, unknown>;
      expect(args.startTime).toBeTypeOf('number');
      // Should be a reasonable unix timestamp (2025 = ~1.7 billion seconds)
      expect(args.startTime as number).toBeGreaterThan(1700000000);
    });
  });

  it('resets all fields when Reset is clicked', async () => {
    mockInvoke('query_timeline', () => [
      { uuid: 'test-uuid-123456789', origin: 'c:Screen', modality: 'Screen', timestamp: null },
    ]);

    render(<TimelineDashboard />);

    // Search first
    fireEvent.change(screen.getByPlaceholderText('Enter search query...'), { target: { value: 'hello' } });
    fireEvent.click(screen.getByRole('button', { name: /search/i }));

    await waitFor(() => {
      expect(screen.getByText('Screen')).toBeInTheDocument();
    });

    // Reset
    fireEvent.click(screen.getByText('Reset'));

    expect(screen.getByPlaceholderText('Enter search query...')).toHaveValue('');
    expect(screen.getByText('No results found')).toBeInTheDocument();
  });

  it('handles invoke errors gracefully', async () => {
    mockInvoke('query_timeline', () => {
      throw new Error('gRPC connection failed');
    });

    render(<TimelineDashboard />);

    fireEvent.click(screen.getByRole('button', { name: /search/i }));

    await waitFor(() => {
      // Should show empty state, not crash
      expect(screen.getByText('No results found')).toBeInTheDocument();
    });
  });

  it('shows loading state while searching', async () => {
    // Use a slow mock to catch the loading state
    mockInvoke('query_timeline', () => new Promise(resolve => setTimeout(() => resolve([]), 100)));

    render(<TimelineDashboard />);
    fireEvent.click(screen.getByRole('button', { name: /search/i }));

    // Loading state should appear
    expect(screen.getByText('Searching timeline...')).toBeInTheDocument();

    // Wait for it to resolve
    await waitFor(() => {
      expect(screen.getByText('No results found')).toBeInTheDocument();
    });
  });
});
