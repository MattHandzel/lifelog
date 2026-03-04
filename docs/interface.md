# Lifelog Interface

The Lifelog Interface is the primary way for users to interact with their collected data.

## Features

- **Search**: Query your lifelog using natural language or structured queries.
- **Visualization**: View data from different modalities (images, audio, history) in a unified timeline.
- **Management**: Manage collectors and system configuration.

## Components

- **Frontend**: A React application built with Vite and Tailwind CSS.
- **Tauri Bridge**: A desktop application wrapper that provides system integration and secure gRPC communication.

## Development

```bash
cd interface
npm install
npm run tauri dev
```
