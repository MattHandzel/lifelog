import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface InterfaceSettings {
  grpcServerAddress: string;
  configPath: string;
}

interface RetentionSettings {
  screenDays: number;
  audioDays: number;
  textDays: number;
}

export default function SettingsDashboard(): JSX.Element {
  const [settings, setSettings] = useState<InterfaceSettings | null>(null);
  const [serverAddressInput, setServerAddressInput] = useState('');
  const [retention, setRetention] = useState<RetentionSettings>({
    screenDays: 0,
    audioDays: 0,
    textDays: 0,
  });
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [isSavingRetention, setIsSavingRetention] = useState(false);
  const [isTesting, setIsTesting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  useEffect(() => {
    void loadSettings();
  }, []);

  async function loadSettings() {
    setIsLoading(true);
    setError(null);
    try {
      const result = await invoke<InterfaceSettings>('get_interface_settings');
      setSettings(result);
      setServerAddressInput(result.grpcServerAddress);
      const retentionResult = await invoke<RetentionSettings>('get_component_config', {
        collectorId: 'server',
        componentType: 'retention',
      });
      setRetention(retentionResult);
    } catch (err) {
      console.error('Failed to load interface settings:', err);
      setError(`Failed to load interface settings: ${String(err)}`);
    } finally {
      setIsLoading(false);
    }
  }

  async function saveRetentionSettings() {
    setIsSavingRetention(true);
    setError(null);
    setSuccessMessage(null);
    try {
      await invoke('set_component_config', {
        collectorId: 'server',
        componentType: 'retention',
        configValue: retention,
      });
      setSuccessMessage('Saved. Retention policy updated.');
    } catch (err) {
      console.error('Failed to save retention settings:', err);
      setError(`Failed to save retention settings: ${String(err)}`);
    } finally {
      setIsSavingRetention(false);
    }
  }

  async function saveSettings() {
    setIsSaving(true);
    setError(null);
    setSuccessMessage(null);
    try {
      await invoke('set_interface_settings', {
        grpcServerAddress: serverAddressInput.trim(),
      });
      await loadSettings();
      setSuccessMessage('Saved. Server connection updated.');
    } catch (err) {
      console.error('Failed to save interface settings:', err);
      setError(`Failed to save settings: ${String(err)}`);
    } finally {
      setIsSaving(false);
    }
  }

  async function testConnection() {
    setIsTesting(true);
    setError(null);
    setSuccessMessage(null);
    try {
      await invoke('test_interface_server_connection', {
        grpcServerAddress: serverAddressInput.trim(),
      });
      setSuccessMessage('Connection test succeeded.');
    } catch (err) {
      console.error('Connection test failed:', err);
      setError(`Connection test failed: ${String(err)}`);
    } finally {
      setIsTesting(false);
    }
  }

  return (
    <div className="p-8">
      <h2 className="title mb-2">Settings</h2>
      <p className="subtitle mb-8">Interface connection is independent from local collectors.</p>

      {isLoading ? (
        <div className="text-[#9CA3AF]">Loading settings...</div>
      ) : (
        <div className="space-y-6">
          {error && (
            <div className="bg-red-500/10 border border-red-500/50 text-red-300 px-4 py-3 rounded-lg">
              {error}
            </div>
          )}
          {successMessage && (
            <div className="bg-green-500/10 border border-green-500/50 text-green-300 px-4 py-3 rounded-lg">
              {successMessage}
            </div>
          )}

          <div className="card p-5 space-y-4">
            <h3 className="font-medium text-[#F9FAFB]">Server Connection</h3>

            <div className="space-y-2">
              <label className="text-sm text-[#9CA3AF]">gRPC server address</label>
              <input
                className="w-full rounded-lg bg-[#0F111A] border border-[#2A3142] px-3 py-2 text-[#F9FAFB]"
                value={serverAddressInput}
                onChange={(e) => setServerAddressInput(e.target.value)}
                placeholder="http://127.0.0.1:7182"
              />
              <p className="text-xs text-[#9CA3AF]">
                Example: `http://127.0.0.1:27182` (SSH tunnel to remote server gRPC port 7182)
              </p>
            </div>

            <div className="flex gap-3">
              <button
                onClick={() => void testConnection()}
                disabled={isTesting}
                className="btn btn-secondary"
              >
                {isTesting ? 'Testing...' : 'Test Connection'}
              </button>
              <button
                onClick={() => void saveSettings()}
                disabled={isSaving}
                className="btn btn-primary"
              >
                {isSaving ? 'Saving...' : 'Save'}
              </button>
            </div>

            <div className="text-xs text-[#9CA3AF]">
              <div>Config file:</div>
              <div className="break-all">{settings?.configPath ?? 'unknown'}</div>
            </div>
          </div>

          <div className="card p-5 space-y-4">
            <h3 className="font-medium text-[#F9FAFB]">Privacy &amp; Storage</h3>
            <p className="text-xs text-[#9CA3AF]">
              Retention values are in days. Use 0 to keep data forever.
            </p>

            <div className="grid gap-3 md:grid-cols-3">
              <label className="space-y-2">
                <span className="text-sm text-[#9CA3AF]">Screen</span>
                <input
                  type="number"
                  min={0}
                  className="w-full rounded-lg bg-[#0F111A] border border-[#2A3142] px-3 py-2 text-[#F9FAFB]"
                  value={retention.screenDays}
                  onChange={(e) =>
                    setRetention((prev) => ({
                      ...prev,
                      screenDays: Math.max(0, Number(e.target.value) || 0),
                    }))
                  }
                />
              </label>
              <label className="space-y-2">
                <span className="text-sm text-[#9CA3AF]">Audio</span>
                <input
                  type="number"
                  min={0}
                  className="w-full rounded-lg bg-[#0F111A] border border-[#2A3142] px-3 py-2 text-[#F9FAFB]"
                  value={retention.audioDays}
                  onChange={(e) =>
                    setRetention((prev) => ({
                      ...prev,
                      audioDays: Math.max(0, Number(e.target.value) || 0),
                    }))
                  }
                />
              </label>
              <label className="space-y-2">
                <span className="text-sm text-[#9CA3AF]">Text</span>
                <input
                  type="number"
                  min={0}
                  className="w-full rounded-lg bg-[#0F111A] border border-[#2A3142] px-3 py-2 text-[#F9FAFB]"
                  value={retention.textDays}
                  onChange={(e) =>
                    setRetention((prev) => ({
                      ...prev,
                      textDays: Math.max(0, Number(e.target.value) || 0),
                    }))
                  }
                />
              </label>
            </div>

            <div>
              <button
                onClick={() => void saveRetentionSettings()}
                disabled={isSavingRetention}
                className="btn btn-primary"
              >
                {isSavingRetention ? 'Saving...' : 'Save Retention'}
              </button>
            </div>
          </div>

          <div className="card p-5">
            <h3 className="font-medium text-[#F9FAFB] mb-1">Version</h3>
            <p className="text-sm text-[#9CA3AF]">Lifelog Interface • Version 0.1.0</p>
          </div>
        </div>
      )}
    </div>
  );
}
