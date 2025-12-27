# MDC Web UI

Modern web interface for Movie Data Capture built with SvelteKit.

## Features

- ✅ **Dashboard** - Real-time statistics and recent jobs overview
- ✅ **Job Queue** - Live monitoring with WebSocket updates
- ✅ **Folder Scanning** - Easy-to-use interface for batch processing
- ✅ **Configuration** - Edit settings through the UI
- ✅ **Responsive Design** - Works on desktop and mobile
- ✅ **Dark Theme** - Modern, eye-friendly interface

## Quick Start

### Development

```bash
cd mdc-web
npm install
npm run dev
```

The UI will be available at http://localhost:5173

**Note**: Make sure the MDC API server is running at http://localhost:3000

### Production Build

```bash
npm run build
npm run preview
```

## Architecture

### Pages

- **`/`** - Dashboard with stats and recent jobs
- **`/jobs`** - Job queue with real-time updates via WebSocket
- **`/scan`** - Folder scanning interface
- **`/config`** - Configuration editor

### API Integration

The UI connects to the Rust API server through:
- REST API endpoints (`/api/*`)
- WebSocket for real-time progress (`/ws/progress`)

The Vite proxy automatically forwards requests to the API server.

## Technology Stack

- **SvelteKit** - Full-stack framework
- **TypeScript** - Type-safe JavaScript
- **Vite** - Fast build tool
- **WebSocket** - Real-time updates

## Development

### Project Structure

```
mdc-web/
├── src/
│   ├── routes/          # Pages
│   │   ├── +layout.svelte
│   │   ├── +page.svelte      # Dashboard
│   │   ├── jobs/
│   │   │   └── +page.svelte  # Job queue
│   │   ├── scan/
│   │   │   └── +page.svelte  # Folder scanner
│   │   └── config/
│   │       └── +page.svelte  # Configuration
│   ├── app.html         # HTML template
│   └── app.css          # Global styles
├── package.json
├── svelte.config.js
├── vite.config.ts
└── tsconfig.json
```

### Adding New Pages

1. Create a new directory in `src/routes/`
2. Add a `+page.svelte` file
3. Add link in `+layout.svelte` navigation

### Styling

Global styles are in `src/app.css` with CSS custom properties for theming.

## Docker

The UI can be served through the Docker setup:

```bash
cd ..
docker-compose up mdc-server
```

Then access the UI at http://localhost:3000

## Contributing

1. Follow SvelteKit conventions
2. Keep components simple and focused
3. Use TypeScript for type safety
4. Test with the API server running

## License

Same as main MDC project
