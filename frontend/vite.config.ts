import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    proxy: {
      '/api/gatherings': 'http://localhost:8080',
      '/api/menu-items': 'http://localhost:8080',
      '/health': 'http://localhost:8080',
    },
  },
});
