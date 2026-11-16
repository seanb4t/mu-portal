// @ts-check
import { defineConfig } from 'astro/config';
import react from '@astrojs/react';
import starlight from '@astrojs/starlight';
import { LikeC4VitePlugin } from 'likec4/vite-plugin';

export default defineConfig({
	integrations: [
		starlight({
			title: 'mu-portal',
			social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/seanb4t/mu-portal' }],
			sidebar: [
				{ label: 'Overview', slug: '' },
				{ label: 'Guides', items: [{ autogenerate: { directory: 'guides' } }] },
				{ label: 'Decisions', items: [{ autogenerate: { directory: 'decisions' } }] },
				{ label: 'Architecture', items: [{ autogenerate: { directory: 'architecture' } }] },
				{ label: 'Contributing', items: [{ autogenerate: { directory: 'contributing' } }] },
			],
		}),
		react(),
	],
	vite: {
		plugins: [LikeC4VitePlugin({ workspace: '../architecture' })],
	},
});
