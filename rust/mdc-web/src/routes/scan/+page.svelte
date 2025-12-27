<script lang="ts">
	let path = '';
	let outputPath = '';
	let mode = '1';
	let linkMode = '0';
	let concurrent = 4;
	let scanning = false;
	let result: any = null;

	async function startScan() {
		if (!path) {
			alert('Please enter a path to scan');
			return;
		}

		scanning = true;
		result = null;

		try {
			const response = await fetch('/api/scan', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					path,
					output: outputPath || undefined,
					mode: parseInt(mode),
					link_mode: parseInt(linkMode),
					concurrent
				})
			});

			if (response.ok) {
				result = await response.json();
			} else {
				const error = await response.text();
				result = { error };
			}
		} catch (error: any) {
			result = { error: error.message };
		} finally {
			scanning = false;
		}
	}
</script>

<div class="container">
	<h1>Scan Folder</h1>

	<div class="scan-form">
		<div class="form-group">
			<label for="path">Input Path *</label>
			<input id="path" type="text" bind:value={path} placeholder="/path/to/movies" />
		</div>

		<div class="form-group">
			<label for="outputPath">Output Path (optional)</label>
			<input id="outputPath" type="text" bind:value={outputPath} placeholder="/path/to/output" />
		</div>

		<div class="form-row">
			<div class="form-group">
				<label for="mode">Processing Mode</label>
				<select id="mode" bind:value={mode}>
					<option value="1">Scraping (fetch metadata)</option>
					<option value="2">Organizing (move only)</option>
					<option value="3">Analysis (in-place)</option>
				</select>
			</div>

			<div class="form-group">
				<label for="linkMode">Link Mode</label>
				<select id="linkMode" bind:value={linkMode}>
					<option value="0">Move</option>
					<option value="1">Soft Link</option>
					<option value="2">Hard Link</option>
				</select>
			</div>

			<div class="form-group">
				<label for="concurrent">Concurrent Jobs</label>
				<input id="concurrent" type="number" bind:value={concurrent} min="1" max="16" />
			</div>
		</div>

		<button class="btn btn-primary" on:click={startScan} disabled={scanning}>
			{scanning ? 'Scanning...' : 'Start Scan'}
		</button>
	</div>

	{#if result}
		<div class="result-box {result.error ? 'error' : 'success'}">
			{#if result.error}
				<h3>Error</h3>
				<p>{result.error}</p>
			{:else}
				<h3>Scan Complete</h3>
				<p>Found {result.files_found || 0} files</p>
				<p>Jobs created: {result.jobs_created || 0}</p>
				<a href="/jobs" class="btn btn-primary">View Jobs</a>
			{/if}
		</div>
	{/if}
</div>

<style>
	h1 {
		margin-bottom: 2rem;
	}

	.scan-form {
		background: var(--bg-secondary);
		padding: 2rem;
		border-radius: 0.5rem;
		border: 1px solid var(--border);
		max-width: 800px;
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

	.form-row {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 1rem;
	}

	.result-box {
		margin-top: 2rem;
		padding: 1.5rem;
		border-radius: 0.5rem;
	}

	.result-box.success {
		background: rgba(16, 185, 129, 0.1);
		border: 1px solid var(--success);
	}

	.result-box.error {
		background: rgba(239, 68, 68, 0.1);
		border: 1px solid var(--error);
	}

	.result-box h3 {
		margin-bottom: 1rem;
	}

	.result-box p {
		margin-bottom: 0.5rem;
	}

	.result-box .btn {
		margin-top: 1rem;
	}

	@media (max-width: 768px) {
		.form-row {
			grid-template-columns: 1fr;
		}
	}
</style>
