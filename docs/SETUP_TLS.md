# Setting up Lifelog with Mandatory TLS & Auth

As of 2026-03-01, Lifelog enforces mandatory TLS (HTTPS) and Token-based Authentication for all communication between collectors and the server.

## 1. Quick Start: Generate Certificates

You need a SSL certificate and private key. For local setups, a self-signed certificate is sufficient.

### Option A: Use the helper script
```bash
# This will create 'cert.pem' and 'key.pem' in the current directory
openssl req -x509 -newkey rsa:2048 -sha256 -days 3650 -nodes 
  -keyout key.pem -out cert.pem 
  -subj "/CN=localhost" 
  -addext "subjectAltName=DNS:localhost,IP:127.0.0.1"
```

## 2. Server Configuration

The server requires the following environment variables:

| Variable | Description | Example |
| :--- | :--- | :--- |
| `LIFELOG_TLS_CERT_PATH` | Path to your cert.pem | `./cert.pem` |
| `LIFELOG_TLS_KEY_PATH` | Path to your key.pem | `./key.pem` |
| `LIFELOG_AUTH_TOKEN` | Secret for authorized collectors | `my-secure-token` |
| `LIFELOG_ENROLLMENT_TOKEN` | Secret for new collector pairing | `enroll-me-123` |

**Example command:**
```bash
export LIFELOG_TLS_CERT_PATH=./cert.pem
export LIFELOG_TLS_KEY_PATH=./key.pem
export LIFELOG_AUTH_TOKEN=your-secret-token
export LIFELOG_ENROLLMENT_TOKEN=your-enrollment-token
just run-server
```

## 3. Collector Configuration

Collectors must connect via `https://` and provide a token.

| Variable | Description |
| :--- | :--- |
| `LIFELOG_SERVER_URL` | Must start with `https://` |
| `LIFELOG_AUTH_TOKEN` | The same token set on the server |
| `LIFELOG_TLS_CA_CERT_PATH` | (Optional) Path to server cert if self-signed |

**Example command:**
```bash
export LIFELOG_SERVER_URL=https://localhost:50051
export LIFELOG_AUTH_TOKEN=your-secret-token
export LIFELOG_TLS_CA_CERT_PATH=./cert.pem  # Required if using self-signed certs
just run-collector
```

## 4. Troubleshooting Common Errors

- **"Connection Refused / TLS Alert":** Ensure the collector is using `https://` and has `LIFELOG_TLS_CA_CERT_PATH` pointed to the server's certificate if it is self-signed.
- **"Unauthenticated":** Check that `LIFELOG_AUTH_TOKEN` matches exactly on both server and collector.
- **"Plaintext gRPC is not allowed":** The server will refuse to start if the TLS environment variables are missing.
