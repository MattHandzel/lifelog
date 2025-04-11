import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { auth } from '../lib/api';

export function Login() {
  const [credentials, setCredentials] = useState({
    username: 'admin',  // Hardcode for testing
    password: 'admin',  // Hardcode for testing
  });
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const [apiUrl, setApiUrl] = useState('');
  const [debugInfo, setDebugInfo] = useState<any>({});
  const navigate = useNavigate();

  useEffect(() => {
    // Get API URL from environment
    const url = import.meta.env.VITE_API_BASE_URL || 'Not set';
    setApiUrl(url);
    
    // Try to ping the server
    const pingServer = async () => {
      try {
        const response = await fetch(`${url}/api/health`);
        const data = await response.json();
        setDebugInfo(prev => ({ ...prev, serverPing: 'Success', serverResponse: data }));
      } catch (err) {
        setDebugInfo(prev => ({ ...prev, serverPing: 'Failed', error: String(err) }));
      }
    };
    
    pingServer();
  }, []);

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value } = e.target;
    setCredentials(prev => ({
      ...prev,
      [name]: value
    }));
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    setLoading(true);
    console.log('[DEBUG] Login attempt with:', credentials);
    setDebugInfo(prev => ({ ...prev, loginAttempt: credentials }));

    try {
      // Use the real authentication API
      const result = await auth.login(credentials);
      console.log('[DEBUG] Login successful:', result);
      setDebugInfo(prev => ({ ...prev, loginResult: result }));
      navigate('/camera');
    } catch (err) {
      console.error('[DEBUG] Login failed:', err);
      setError('Invalid username or password');
      setDebugInfo(prev => ({ ...prev, loginError: String(err) }));
    } finally {
      setLoading(false);
    }
  };

  console.log('[DEBUG] Rendering login component');

  return (
    <div className="flex items-center justify-center min-h-screen bg-gray-100">
      <div className="w-full max-w-md p-8 space-y-8 bg-white rounded-lg shadow">
        <div className="text-center">
          <h1 className="text-3xl font-bold text-gray-900">Lifelog</h1>
          <p className="mt-2 text-gray-600">Sign in to your account</p>
        </div>
        
        <form className="mt-8 space-y-6" onSubmit={handleSubmit}>
          {error && (
            <div className="p-3 text-sm text-red-600 bg-red-100 rounded">
              {error}
            </div>
          )}
          
          <div>
            <label htmlFor="username" className="block text-sm font-medium text-gray-700">
              Username
            </label>
            <input
              id="username"
              name="username"
              type="text"
              required
              value={credentials.username}
              onChange={handleChange}
              className="w-full px-3 py-2 mt-1 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
            />
          </div>
          
          <div>
            <label htmlFor="password" className="block text-sm font-medium text-gray-700">
              Password
            </label>
            <input
              id="password"
              name="password"
              type="password"
              required
              value={credentials.password}
              onChange={handleChange}
              className="w-full px-3 py-2 mt-1 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
            />
          </div>
          
          <div>
            <button
              type="submit"
              disabled={loading}
              className="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50"
            >
              {loading ? 'Signing in...' : 'Sign in'}
            </button>
          </div>

          <div className="text-xs text-center text-gray-500">
            <p>Demo credentials: admin / admin</p>
          </div>
        </form>
        
        <div className="mt-4 p-4 border border-gray-200 rounded bg-gray-50">
          <h3 className="text-sm font-medium text-gray-700">Debug Info:</h3>
          <div className="mt-2 text-xs">
            <p>API URL: {apiUrl}</p>
            <p>Server ping: {debugInfo.serverPing || 'Not attempted'}</p>
            <pre className="mt-2 p-2 bg-gray-100 overflow-auto text-xs">
              {JSON.stringify(debugInfo, null, 2)}
            </pre>
          </div>
        </div>
      </div>
    </div>
  );
} 