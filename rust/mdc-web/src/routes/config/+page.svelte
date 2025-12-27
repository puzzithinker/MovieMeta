<script lang="ts">
	import { onMount } from 'svelte';

	let config: any = {};
	let loading = true;
	let saving = false;
	let message = '';

	onMount(async () => {
		await loadConfig();
		loading = false;
	});

	async function loadConfig() {
		try {
			const response = await fetch('/api/config');
			if (response.ok) {
				config = await response.json();
			}
		} catch (error) {
			console.error('Failed to load config:', error);
		}
	}

	async function saveConfig() {
		saving = true;
		message = '';

		try {
			const response = await fetch('/api/config', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify(config)
			});

			if (response.ok) {
				message = 'Configuration saved successfully!';
			} else {
				message = 'Failed to save configuration';
			}
		} catch (error: any) {
			message = `Error: ${error.message}`;
		} finally {
			saving = false;
		}
	}
</script>

<div class="container">
	<h1>Configuration</h1>

	{#if loading}
		<p>Loading configuration...</p>
	{:else}
		<div class="config-form">
			<div class="form-section">
				<h2>General Settings</h2>

				<div class="form-group">
					<label for="main_mode">Main Mode</label>
					<select id="main_mode" bind:value={config.main_mode}>
						<option value="1">Scraping</option>
						<option value="2">Organizing</option>
						<option value="3">Analysis</option>
					</select>
				</div>

				<div class="form-group">
					<label for="link_mode">Link Mode</label>
					<select id="link_mode" bind:value={config.link_mode}>
						<option value="0">Move</option>
						<option value="1">Soft Link</option>
						<option value="2">Hard Link</option>
					</select>
				</div>

				<div class="form-group">
					<label for="success_folder">Success Folder</label>
					<input id="success_folder" type="text" bind:value={config.success_folder} />
				</div>

				<div class="form-group">
					<label for="failed_folder">Failed Folder</label>
					<input id="failed_folder" type="text" bind:value={config.failed_folder} />
				</div>
			</div>

			<div class="form-section">
				<h2>Naming Rules</h2>

				<div class="form-group">
					<label for="location_rule">Location Rule</label>
					<input
						id="location_rule"
						type="text"
						bind:value={config.location_rule}
						placeholder="actor/number"
					/>
					<small>Variables: number, title, actor, studio, director, series</small>
				</div>

				<div class="form-group">
					<label for="naming_rule">Naming Rule</label>
					<input
						id="naming_rule"
						type="text"
						bind:value={config.naming_rule}
						placeholder="number-title"
					/>
				</div>
			</div>

			<div class="form-section">
				<h2>Media Settings</h2>

				<div class="form-group">
					<label for="media_type">Media Types</label>
					<input id="media_type" type="text" bind:value={config.media_type} />
					<small>Comma-separated extensions (e.g., .mp4,.mkv,.avi)</small>
				</div>

				<div class="form-group">
					<label for="nfo_skip_days">NFO Skip Days</label>
					<input id="nfo_skip_days" type="number" bind:value={config.nfo_skip_days} min="0" />
					<small>Skip files with NFO modified within N days</small>
				</div>
			</div>

			<button class="btn btn-primary" on:click={saveConfig} disabled={saving}>
				{saving ? 'Saving...' : 'Save Configuration'}
			</button>

			{#if message}
				<p class="message {message.includes('Error') ? 'error' : 'success'}">{message}</p>
			{/if}
		</div>
	{/if}
</div>

<style>
	h1 {
		margin-bottom: 2rem;
	}

	.config-form {
		max-width: 800px;
	}

	.form-section {
		background: var(--bg-secondary);
		padding: 2rem;
		border-radius: 0.5rem;
		border: 1px solid var(--border);
		margin-bottom: 2rem;
	}

	.form-section h2 {
		margin-bottom: 1.5rem;
		font-size: 1.25rem;
	}

	.form-group {
		margin-bottom: 1.5rem;
	}

	.form-group label {
		display: block;
		margin-bottom: 0.5rem;
		font-weight: 500;
	}

	.form-group input,
	.form-group select {
		width: 100%;
		padding: 0.75rem;
		background: var(--bg-tertiary);
		border: 1px solid var(--border);
		border-radius: 0.375rem;
		color: var(--text);
		font-size: 1rem;
	}

	.form-group input:focus,
	.form-group select:focus {
		outline: none;
		border-color: var(--primary);
	}

	.form-group small {
		display: block;
		margin-top: 0.5rem;
		color: var(--text-secondary);
		font-size: 0.875rem;
	}

	.message {
		margin-top: 1rem;
		padding: 1rem;
		border-radius: 0.375rem;
	}

	.message.success {
		background: rgba(16, 185, 129, 0.1);
		color: var(--success);
	}

	.message.error {
		background: rgba(239, 68, 68, 0.1);
		color: var(--error);
	}
</style>
