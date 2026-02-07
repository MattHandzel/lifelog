import { describe, it, expect } from 'vitest';
import type {
  CollectorConfig,
  CollectorState,
  ServerState,
  ScreenConfig,
  MicrophoneConfig,
  SystemState,
  LifelogData,
  DataModality,
} from '../auto_generated_types';

/**
 * These tests verify that the proto-generated TypeScript types match
 * the shapes returned by Tauri invoke (prost+serde JSON serialization).
 * If a proto field is renamed or removed, these tests fail at compile time.
 */
describe('Proto-generated types compile-time checks', () => {
  it('CollectorConfig has expected fields', () => {
    // This test validates at compile time that the type has these fields.
    // If a proto field changes, TypeScript compilation fails here.
    const config: CollectorConfig = {
      id: 'test-collector',
      host: 'localhost',
      port: 7182,
      timestampFormat: '%Y-%m-%d',
    };
    expect(config.id).toBe('test-collector');
    expect(config.port).toBe(7182);
  });

  it('CollectorState has enriched fields', () => {
    const state: CollectorState = {
      name: 'test',
      sourceStates: [],
      sourceBufferSizes: [],
      totalBufferSize: 0,
      uploadLagBytes: 0,
    };
    // New enriched fields should exist as optional
    expect(state.lastSeen).toBeUndefined();
    expect(state.lastUploadTime).toBeUndefined();
    expect(state.uploadLagBytes).toBe(0);
  });

  it('ServerState has enriched fields', () => {
    const state: ServerState = {
      name: 'test-server',
      cpuUsage: 0.5,
      memoryUsage: 0.3,
      threads: 4,
      pendingActions: [],
      version: '0.1.0',
      totalFramesStored: 1000,
      diskUsageBytes: 500000,
    };
    expect(state.totalFramesStored).toBe(1000);
    expect(state.diskUsageBytes).toBe(500000);
    expect(state.uptimeSince).toBeUndefined();
  });

  it('ScreenConfig matches Tauri invoke shape', () => {
    const config: ScreenConfig = {
      enabled: true,
      interval: 60,
      outputDir: '/tmp/screens',
      program: 'grim',
      timestampFormat: '%Y-%m-%d_%H-%M-%S',
    };
    expect(config.enabled).toBe(true);
    expect(config.interval).toBe(60);
  });

  it('MicrophoneConfig matches Tauri invoke shape', () => {
    const config: MicrophoneConfig = {
      enabled: false,
      outputDir: '/tmp/audio',
      sampleRate: 44100,
      chunkDurationSecs: 60,
      timestampFormat: '%Y-%m-%d_%H-%M-%S',
      bitsPerSample: 16,
      channels: 1,
      captureIntervalSecs: 300,
    };
    expect(config.sampleRate).toBe(44100);
    expect(config.channels).toBe(1);
  });

  it('SystemState has nested collector/interface maps', () => {
    const state: SystemState = {
      collectorStates: {
        'collector-1': {
          name: 'collector-1',
          sourceStates: [],
          sourceBufferSizes: [],
          totalBufferSize: 0,
          uploadLagBytes: 0,
        },
      },
      interfaceStates: {},
    };
    expect(Object.keys(state.collectorStates)).toHaveLength(1);
  });

  it('LifelogData has oneof payload fields', () => {
    const data: LifelogData = {};
    expect(data.screenframe).toBeUndefined();
    expect(data.audioframe).toBeUndefined();
    expect(data.browserframe).toBeUndefined();
  });

  it('DataModality enum has expected values', () => {
    const browser: DataModality = 0; // Browser
    const screen: DataModality = 2; // Screen
    const audio: DataModality = 3;  // Audio
    expect(browser).toBe(0);
    expect(screen).toBe(2);
    expect(audio).toBe(3);
  });
});
