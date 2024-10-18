import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import path from 'path';

export default defineConfig({
  plugins: [sveltekit()],
  server: {
    watch: {
      ignored: [path.resolve(__dirname, 'src-tauri/resource') + '/**'] // Игнорирование всех изменений в папке resource
    }
  }
});