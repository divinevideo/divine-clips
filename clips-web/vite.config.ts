import adapter from '@sveltejs/adapter-cloudflare';
import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { relative, sep } from 'node:path';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [
		tailwindcss(),
		sveltekit({
			compilerOptions: {
				// Default to runes in app components without forcing them on dependencies.
				runes: ({ filename }) => {
					const parts = relative(import.meta.dirname, filename).toLowerCase().split(sep);
					return parts.includes('node_modules') ? undefined : true;
				}
			},
			adapter: adapter()
		})
	]
});
