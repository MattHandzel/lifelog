import { useState, useEffect } from 'react';
import { lifelogGrpcClient } from '../../lib/grpcWebClient';
import { Button } from './button';
import { Card, CardContent, CardHeader, CardTitle, CardDescription, CardFooter } from './card';

// Define the types for the data we expect from gRPC
interface LoggerStatus {
  name: string;
  enabled: boolean;
  running: boolean;
  lastActive: string;
  dataPoints: number;
  error: string;
}

interface LoggerStatusResponse {
  loggers: LoggerStatus[];
  systemStats: Record<string, string>;
}

export function GrpcExample() {
  const [loggers, setLoggers] = useState<LoggerStatus[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [grpcEndpoint, setGrpcEndpoint] = useState<string>('');

  useEffect(() => {
    // Get and display gRPC endpoint for debugging
    const endpoint = import.meta.env.VITE_GRPC_API_URL || 'http://localhost:50051';
    setGrpcEndpoint(endpoint);
    
    fetchLoggerStatus();
  }, []);

  const fetchLoggerStatus = async () => {
    try {
      setLoading(true);
      setError(null);
      setSuccess(null);
      
      console.log('Fetching logger status...');
      console.log('Using gRPC endpoint:', import.meta.env.VITE_GRPC_API_URL || 'http://localhost:50051');
      
      // Add a direct fetch as a fallback test to check if the server is reachable
      try {
        const resp = await fetch(`${import.meta.env.VITE_GRPC_API_URL || 'http://localhost:50051'}/healthz`, { 
          method: 'GET',
          mode: 'cors'
        });
        console.log('Health check response:', resp.status, resp.statusText);
      } catch (e) {
        console.warn('Health check failed:', e);
      }
      
      const response = await lifelogGrpcClient.getLoggerStatus({});
      console.log('Received response:', response);
      
      setLoggers(response.loggers || []);
      setSuccess('Successfully connected to gRPC server and retrieved logger status');
    } catch (err) {
      console.error('Error fetching logger status:', err);
      setError(`Error fetching logger status: ${err instanceof Error ? err.message : String(err)}`);
    } finally {
      setLoading(false);
    }
  };

  const handleToggleLogger = async (logger: LoggerStatus) => {
    try {
      setLoading(true);
      setError(null);
      setSuccess(null);
      const response = await lifelogGrpcClient.toggleLogger({
        loggerName: logger.name,
        enable: !logger.enabled
      });
      
      if (response.success && response.status) {
        setLoggers(prevLoggers => 
          prevLoggers.map(l => l.name === logger.name ? response.status : l)
        );
        setSuccess(`Successfully ${!logger.enabled ? 'enabled' : 'disabled'} logger: ${logger.name}`);
      } else {
        setError(`Failed to toggle logger: ${response.errorMessage}`);
      }
    } catch (err) {
      setError(`Error toggling logger: ${err instanceof Error ? err.message : String(err)}`);
      console.error('Error toggling logger:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleTakeSnapshot = async () => {
    try {
      setLoading(true);
      setError(null);
      setSuccess(null);
      const response = await lifelogGrpcClient.takeSnapshot({
        loggers: loggers.filter(l => l.enabled).map(l => l.name)
      });
      
      if (response.success) {
        setSuccess(`Snapshot taken successfully! ID: ${response.snapshotId}`);
      } else {
        setError(`Failed to take snapshot: ${response.errorMessage}`);
      }
    } catch (err) {
      setError(`Error taking snapshot: ${err instanceof Error ? err.message : String(err)}`);
      console.error('Error taking snapshot:', err);
    } finally {
      setLoading(false);
    }
  };

  return (
    <Card className="w-full max-w-3xl mx-auto">
      <CardHeader>
        <CardTitle>Logger Status (gRPC-Web)</CardTitle>
        <CardDescription>
          This component demonstrates using the gRPC-Web client to interact with the server.
        </CardDescription>
      </CardHeader>
      <CardContent>
        {error && (
          <div className="bg-red-100 border-l-4 border-red-500 text-red-700 p-4 mb-4">
            {error}
          </div>
        )}
        
        {success && (
          <div className="bg-green-100 border-l-4 border-green-500 text-green-700 p-4 mb-4">
            {success}
          </div>
        )}
        
        {loading ? (
          <div className="flex justify-center py-8">
            <div className="animate-spin rounded-full h-10 w-10 border-b-2 border-primary"></div>
          </div>
        ) : (
          <div className="space-y-4">
            {loggers.length === 0 ? (
              <p className="text-center text-gray-500">No loggers found</p>
            ) : (
              loggers.map(logger => (
                <div key={logger.name} className="border rounded-lg p-4 flex justify-between items-center">
                  <div>
                    <h3 className="font-medium">{logger.name}</h3>
                    <div className="text-sm text-gray-500">
                      Status: {logger.running ? 'Running' : 'Stopped'} | 
                      Last active: {new Date(logger.lastActive).toLocaleString()} |
                      Data points: {logger.dataPoints}
                    </div>
                    {logger.error && (
                      <div className="text-sm text-red-500 mt-1">
                        Error: {logger.error}
                      </div>
                    )}
                  </div>
                  <Button 
                    variant={logger.enabled ? "destructive" : "default"}
                    size="sm"
                    onClick={() => handleToggleLogger(logger)}
                    disabled={loading}
                  >
                    {logger.enabled ? 'Disable' : 'Enable'}
                  </Button>
                </div>
              ))
            )}
          </div>
        )}
      </CardContent>
      <CardFooter className="flex justify-between">
        <Button onClick={fetchLoggerStatus} variant="outline" disabled={loading}>
          Refresh
        </Button>
        <Button onClick={handleTakeSnapshot} disabled={loading || loggers.filter(l => l.enabled).length === 0}>
          Take Snapshot
        </Button>
      </CardFooter>
    </Card>
  );
} 