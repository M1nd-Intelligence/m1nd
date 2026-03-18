import {defineConfig} from 'vite';
import mc from '@motion-canvas/vite-plugin';

// CJS interop: the plugin may land on .default or directly
const motionCanvas = (mc as any).default ?? mc;

export default defineConfig({
  plugins: [motionCanvas()],
});
