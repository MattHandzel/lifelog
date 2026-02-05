# Refactoring Plan: Decoupling Frontend from Logger

## Goal
Decouple the frontend code from the logger by introducing a server layer that handles all logger-related operations, ensuring the frontend communicates exclusively with the backend server.

## Architecture

### Current Architecture
```
Frontend (Interface) ------> Logger Code (Direct invocation)
```

### Target Architecture
```
Frontend (Interface) ------> Backend Server ------> Logger Code
                             (REST API)
```

## What's Implemented

### Server API Endpoints
- Logger management endpoints:
  - `/api/loggers` - Get all available loggers
  - `/api/logger/:name/start` - Start a specific logger
  - `/api/logger/:name/stop` - Stop a specific logger
  - `/api/logger/:name/status` - Get logger status
  - `/api/logger/:name/config` - Get/update logger configuration

- Query endpoints:
  - `/api/logger/:name/data` - Get data from specific logger
  - `/api/logger/:name/files/:path` - Get files captured by logger

- Action-specific endpoints:
  - `/api/logger/camera/capture` - Trigger one-time camera capture
  - `/api/logger/screen/capture` - Trigger one-time screen capture
  - `/api/logger/microphone/record/start` - Start manual recording
  - `/api/logger/microphone/record/stop` - Stop manual recording
  - `/api/logger/microphone/record/pause` - Pause manual recording
  - `/api/logger/microphone/record/resume` - Resume manual recording

### Frontend Refactoring
- ✅ `CameraDashboard.tsx` - Refactored to use REST API
- ✅ `ScreenDashboard.tsx` - Refactored to use REST API
- ✅ `MicrophoneDashboard.tsx` - Refactored to use REST API
- ✅ `ProcessesDashboard.tsx` - Refactored to use REST API

## What Needs to Be Done

### Server Implementation
- Authentication for API endpoints
- Proper CORS handling
- Error handling and status codes
- Rate limiting
- Logging of API requests

### Logger Refactoring
- Ensure logger can run standalone without frontend
- Update logger configuration management

### Testing
- Test all API endpoints
- Test frontend components with mocked API
- Integration testing of frontend and server

## Implementation Strategy
1. ✅ Incrementally refactor each frontend component to use the REST API
2. ✅ Run the server alongside the frontend during development
3. ✅ Test functionality after each component is refactored
4. Implement remaining server features (authentication, CORS, etc.)
5. Create comprehensive tests for the API
6. Document the API endpoints and usage

## Benefits of New Architecture
1. **Cleaner Separation of Concerns**:
   - Frontend focuses on UI/UX
   - Server handles business logic and logger interaction
   - Logger focuses on data collection

2. **Independent Evolution**:
   - Frontend and logger can evolve independently
   - New logger features don't require frontend changes
   - UI updates don't impact logger functionality

3. **Easier Maintenance**:
   - Smaller, focused components
   - Clear API contracts between components
   - Simplified testing of each layer

4. **Better Scalability**:
   - Server can handle multiple frontend clients
   - Potential for distributed logging in the future
   - More efficient resource usage

5. **Improved Security**:
   - Controlled access to logger functionality
   - Authentication and authorization at API level
   - Limited direct access to system resources 