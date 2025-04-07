import { useState, useEffect } from 'react';
import axios from 'axios';

const API_BASE_URL = process.env.REACT_APP_API_URL || 'http://localhost:8000';

interface AppSettings {
  theme: 'light' | 'dark';
  autoRefresh: boolean;
  refreshInterval: number;
  logLevel: 'debug' | 'info' | 'warn' | 'error';
}

export default function SettingsDashboard() {
  const [settings, setSettings] = useState<AppSettings>({
    theme: 'dark',
    autoRefresh: true,
    refreshInterval: 30,
    logLevel: 'info',
  });
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    setIsLoading(true);
    setError(null);
    
    try {
      const response = await axios.get(`${API_BASE_URL}/api/settings`);
      setSettings(response.data);
    } catch (err) {
      console.error('Failed to load settings:', err);
      setError('Failed to load settings. Using defaults.');
      // Continue with default settings
    } finally {
      setIsLoading(false);
    }
  };

  const saveSettings = async () => {
    setIsSaving(true);
    setError(null);
    setSuccessMessage(null);
    
    try {
      await axios.put(`${API_BASE_URL}/api/settings`, settings);
      setSuccessMessage('Settings saved successfully');
      
      // Clear success message after 3 seconds
      setTimeout(() => {
        setSuccessMessage(null);
      }, 3000);
    } catch (err) {
      console.error('Failed to save settings:', err);
      setError('Failed to save settings. Please try again.');
    } finally {
      setIsSaving(false);
    }
  };

  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) => {
    const { name, value, type } = e.target as HTMLInputElement;
    
    setSettings(prev => ({
      ...prev,
      [name]: type === 'checkbox' 
        ? (e.target as HTMLInputElement).checked 
        : type === 'number' 
          ? parseInt(value, 10) 
          : value
    }));
  };

  return (
    <div className="p-6">
      <h1 className="text-2xl font-bold mb-6">Settings</h1>
      
      {isLoading ? (
        <div className="flex justify-center my-8">
          <div className="loading-spinner w-8 h-8"></div>
        </div>
      ) : (
        <div className="max-w-2xl mx-auto">
          {error && (
            <div className="bg-red-500 bg-opacity-10 border border-red-500 text-red-500 px-4 py-3 rounded mb-4">
              {error}
            </div>
          )}
          
          {successMessage && (
            <div className="bg-green-500 bg-opacity-10 border border-green-500 text-green-500 px-4 py-3 rounded mb-4">
              {successMessage}
            </div>
          )}
          
          <div className="card p-6 space-y-6">
            <div>
              <h2 className="text-lg font-medium mb-4">Appearance</h2>
              <div className="space-y-4">
                <div>
                  <label className="form-label">Theme</label>
                  <select 
                    name="theme"
                    value={settings.theme}
                    onChange={handleChange}
                    className="form-input"
                  >
                    <option value="light">Light</option>
                    <option value="dark">Dark</option>
                  </select>
                </div>
              </div>
            </div>
            
            <div className="border-t border-gray-700 pt-6">
              <h2 className="text-lg font-medium mb-4">Data Refresh</h2>
              <div className="space-y-4">
                <div className="flex items-center">
                  <input
                    type="checkbox"
                    id="autoRefresh"
                    name="autoRefresh"
                    checked={settings.autoRefresh}
                    onChange={handleChange}
                    className="h-4 w-4 rounded border-gray-600 text-blue-600 focus:ring-blue-500 bg-gray-700"
                  />
                  <label htmlFor="autoRefresh" className="ml-2 block text-sm text-gray-300">
                    Enable auto-refresh
                  </label>
                </div>
                
                {settings.autoRefresh && (
                  <div>
                    <label className="form-label">Refresh Interval (seconds)</label>
                    <input
                      type="number"
                      name="refreshInterval"
                      value={settings.refreshInterval}
                      onChange={handleChange}
                      min="5"
                      max="3600"
                      className="form-input"
                    />
                  </div>
                )}
              </div>
            </div>
            
            <div className="border-t border-gray-700 pt-6">
              <h2 className="text-lg font-medium mb-4">Logging</h2>
              <div>
                <label className="form-label">Log Level</label>
                <select 
                  name="logLevel"
                  value={settings.logLevel}
                  onChange={handleChange}
                  className="form-input"
                >
                  <option value="debug">Debug</option>
                  <option value="info">Info</option>
                  <option value="warn">Warning</option>
                  <option value="error">Error</option>
                </select>
              </div>
            </div>
            
            <div className="border-t border-gray-700 pt-6 flex justify-end">
              <button
                onClick={saveSettings}
                disabled={isSaving}
                className="btn btn-primary flex items-center"
              >
                {isSaving ? (
                  <>
                    <span className="loading-spinner mr-2"></span>
                    Saving...
                  </>
                ) : 'Save Settings'}
              </button>
            </div>
          </div>
          
          <div className="mt-8 card p-6">
            <h2 className="text-lg font-medium mb-4">About</h2>
            <div>
              <p className="text-gray-400">Lifelog Interface • Version 0.1.0</p>
              <p className="text-gray-400 mt-2">A personal life data tracking application.</p>
            </div>
          </div>
        </div>
      )}
    </div>
  );
} 