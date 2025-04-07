# Lifelog Server

The Lifelog Server provides a REST API for the Lifelog application, serving as the intermediary between the frontend and the various logger components.

## Features

- **REST API**: Provides endpoints for all logger operations
- **Authentication**: JWT-based authentication system
- **CORS Support**: Configurable CORS settings for secure cross-origin requests
- **Error Handling**: Comprehensive error handling and logging
- **Static File Serving**: Serves logger files (images, audio, etc.)

## Setup

### Environment Variables

The server uses environment variables for configuration. Create a `.env` file in the server directory with the following variables:

```
# Server Configuration
SERVER_IP=localhost
SERVER_PORT=8080

# Authentication
JWT_SECRET=your_secure_random_string
JWT_EXPIRES_IN=86400  # 24 hours in seconds
DEFAULT_ADMIN_PASSWORD=admin

# Cross-Origin Resource Sharing (CORS)
ALLOWED_ORIGINS=http://localhost:3000,http://localhost:8080

# Logging
LOG_LEVEL=info  # Options: trace, debug, info, warn, error

# Development Mode
DEVELOPMENT_MODE=true
```

### Running the Server

```bash
# Build the server
cargo build

# Run the server
cargo run
```

## API Documentation

### Authentication

#### Login

```
POST /api/auth/login
```

Request body:
```json
{
  "username": "admin",
  "password": "admin"
}
```

Response:
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": {
    "id": "1",
    "username": "admin",
    "role": "admin"
  }
}
```

#### Get Current User Profile

```
GET /api/auth/profile
```

Headers:
```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

Response:
```json
{
  "user": {
    "id": "1",
    "username": "admin",
    "role": "admin"
  }
}
```

### Logger Operations

All logger endpoints require authentication with a valid JWT token in the Authorization header.

#### Get All Loggers

```
GET /api/loggers
```

#### Get Logger Status

```
GET /api/logger/{name}/status
```

#### Start Logger

```
POST /api/logger/{name}/start
```

#### Stop Logger

```
POST /api/logger/{name}/stop
```

#### Get Logger Configuration

```
GET /api/logger/{name}/config
```

#### Update Logger Configuration

```
PUT /api/logger/{name}/config
```

#### Get Logger Data

```
GET /api/logger/{name}/data
```

Query parameters:
- `limit`: Maximum number of records to return
- `filter`: SQL filter condition
- `page`: Page number for pagination
- `page_size`: Number of items per page

#### Get Logger Files

```
GET /api/logger/{name}/files/{path}
```

### Special Actions

#### Capture Camera Frame

```
POST /api/logger/camera/capture
```

#### Capture Screen

```
POST /api/logger/screen/capture
```

#### Microphone Recording

```
POST /api/logger/microphone/record/start
POST /api/logger/microphone/record/stop
POST /api/logger/microphone/record/pause
POST /api/logger/microphone/record/resume
```

## Security Considerations

- The JWT secret should be a strong, random string in production
- In production, set `DEVELOPMENT_MODE` to `false` for stricter CORS rules
- Consider updating the default admin password immediately after first login
- Use HTTPS in production to secure token transmission

## Error Handling

The API returns standardized error responses:

```json
{
  "status": "error",
  "message": "The error message",
  "error_code": "ERROR_CODE",
  "details": "Optional additional details"
}
```

HTTP status codes are properly set according to the error type (400, 401, 403, 404, 500, etc.). 