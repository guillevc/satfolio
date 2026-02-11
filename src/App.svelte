<script lang="ts">
	import './app.css';
	import svelteLogo from './assets/svelte.svg';
	import viteLogo from '/vite.svg';
	import Counter from './lib/Counter.svelte';
</script>

<main class="mx-auto max-w-3xl px-4 py-16 text-center">
	<div class="flex items-center justify-center gap-4">
		<a
			href="https://vite.dev"
			target="_blank"
			rel="noreferrer"
		>
			<img
				src={viteLogo}
				class="h-24 p-6 transition-[filter] duration-300 hover:drop-shadow-[0_0_2em_#646cffaa]"
				alt="Vite Logo"
			/>
		</a>

		<a
			href="https://svelte.dev"
			target="_blank"
			rel="noreferrer"
		>
			<img
				src={svelteLogo}
				class="h-24 p-6 transition-[filter] duration-300 hover:drop-shadow-[0_0_2em_#ff3e00aa]"
				alt="Svelte Logo"
			/>
		</a>
	</div>

	<h1 class="mt-8 text-4xl font-bold">Vite + Svelte</h1>
	<div class="mt-6"><Counter /></div>

	<p class="mt-6">
		Check out

		<a
			href="https://github.com/sveltejs/kit#readme"
			target="_blank"
			rel="noreferrer"
		>
			SvelteKit
		</a>
		, the official Svelte app framework powered by Vite!
	</p>

	<p class="mt-4 text-muted-foreground">Click on the Vite and Svelte logos to learn more</p>
</main>
