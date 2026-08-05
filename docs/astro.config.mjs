// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import flameGrammar from './src/syntax/flame.tmLanguage.json' with { type: 'json' };
import fmiGrammar from './src/syntax/fmi.tmLanguage.json' with { type: 'json' };

// https://astro.build/config
export default defineConfig({
	markdown: {
		shikiConfig: {
			langs: [
				// @ts-ignore
				{
					...flameGrammar,
					name: 'flame',
					aliases: ['fm', 'flamelang'],
				},
				// @ts-ignore
				{
					...fmiGrammar,
					name: 'fmi',
					aliases: ['flame-interface'],
				},
			],
		},
	},
	integrations: [
		starlight({
			title: 'Flame',
			logo: {
				src: './src/assets/flame.png',
				alt: 'Flame Logo',
			},
			customCss: ['./src/styles/custom.css'],
			social: [
				{ icon: 'github', label: 'GitHub', href: 'https://github.com/shoya-129/flame' }
			],
			sidebar: [
				{
					label: 'Getting Started',
					items: [
						{ label: 'Introduction', slug: 'getting-started/introduction' },
						{ label: 'Installation & Toolchain', slug: 'getting-started/installation' },
						{ label: 'Architecture & Analysis', slug: 'getting-started/architecture' },
					],
				},
				{
					label: 'Language Basics',
					items: [
						{ label: 'Variables & Constants', slug: 'language-basics/variables' },
						{ label: 'Data Types & Formulas', slug: 'language-basics/data-types' },
						{ label: 'Operators & Expressions', slug: 'language-basics/operators' },
						{ label: 'Nil Safety & Optionals', slug: 'language-basics/nil-safety' },
						{ label: 'Type Conversions', slug: 'language-basics/type-conversions' },
						{ label: 'Control Flow', slug: 'language-basics/control-flow' },
						{ label: 'Functions & Closures', slug: 'language-basics/functions' },
					],
				},
				{
					label: 'Types & Object-Oriented',
					items: [
						{ label: 'Structs', slug: 'types-and-traits/structs' },
						{ label: 'Impl & Methods', slug: 'types-and-traits/impl-and-methods' },
						{ label: 'Enums & Patterns', slug: 'types-and-traits/enums' },
						{ label: 'Traits & Interfaces', slug: 'types-and-traits/traits' },
					],
				},
				{
					label: 'Memory & Safety',
					items: [
						{ label: 'Ownership & Move Semantics', slug: 'memory-and-safety/ownership' },
						{ label: 'Borrowing & References', slug: 'memory-and-safety/borrowing' },
					],
				},
				{
					label: 'Concurrency',
					items: [
						{ label: 'Async & Await (I/O)', slug: 'concurrency/async-await' },
						{ label: 'Threads & Channels (Compute)', slug: 'concurrency/threads-and-channels' },
					],
				},
				{
					label: 'Packages & Native Rust',
					items: [
						{ label: 'Modules & Imports', slug: 'packages-and-native/modules-and-imports' },
						{ label: 'Using Native Rust Crates', slug: 'packages-and-native/native-rust-crates' },
						{ label: 'Native Plugins & FFI', slug: 'packages-and-native/native-plugins' },
						{ label: 'Native Macros (flame-macro)', slug: 'packages-and-native/native-macros' },
					],
				},
				{
					label: 'Annotations & Testing',
					items: [
						{ label: 'Built-in Annotations & CLI', slug: 'annotations-and-testing/builtin-annotations' },
						{ label: 'Custom Annotations & Scope Injection', slug: 'annotations-and-testing/custom-annotations' },
						{ label: 'Testing Framework', slug: 'annotations-and-testing/testing-framework' },
					],
				},
				{
					label: 'Standard Library',
					items: [
						{ label: 'Overview & Status', slug: 'standard-library/overview' },
						{ label: 'File System (std.fs)', slug: 'standard-library/filesystem' },
						{ label: 'Process & OS (std.process, std.os, std.env)', slug: 'standard-library/process-and-os' },
						{ label: 'Hardware & Desktop Automation', slug: 'standard-library/hardware-and-automation' },
						{ label: 'Threading & Math (std.thread, std.math, std.time)', slug: 'standard-library/threading-and-time' },
						{ label: 'Platform Devices (Bluetooth, Camera, HID, Serial)', slug: 'standard-library/platform-devices' },
					],
				},
			],
		}),
	],
});
