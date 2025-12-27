<script lang="ts">
	import { onMount, onDestroy } from 'svelte';

	let jobs: any[] = [];
	let loading = true;
	let ws: WebSocket | null = null;

	onMount(async () => {
		await loadJobs();
		connectWebSocket();
		loading = false;
	});

	onDestroy(() => {
		if (ws) {
			ws.close();
		}
	});

	async function loadJobs() {
		try {
			const response = await fetch('/api/jobs');
			if (response.ok) {
				jobs = await response.json();
			}
		} catch (error) {
			console.error('Failed to load jobs:', error);
		}
	}

	function connectWebSocket() {
		const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
		ws = new WebSocket(`${protocol}//${window.location.host}/ws/progress`);

		ws.onmessage = (event) => {
			const message = JSON.parse(event.data);
			if (message.type === 'progress') {
				updateJobProgress(message);
			}
		};

		ws.onerror = (error) => {
			console.error('WebSocket error:', error);
		};

		ws.onclose = () => {
			// Reconnect after 5 seconds
			setTimeout(connectWebSocket, 5000);
		};
	}

	function updateJobProgress(message: any) {
		const index = jobs.findIndex((j) => j.id === message.job_id);
		if (index !== -1) {
			jobs[index] = { ...jobs[index], ...message };
			jobs = jobs;
		}
	}

	async function retryJob(jobId: string) {
		try {
			const response = await fetch(`/api/jobs/${jobId}/retry`, { method: 'POST' });
			if (response.ok) {
				await loadJobs();
			}
		} catch (error) {
			console.error('Failed to retry job:', error);
		}
	}

	async function cancelJob(jobId: string) {
		try {
			const response = await fetch(`/api/jobs/${jobId}/cancel`, { method: 'POST' });
			if (response.ok) {
				await loadJobs();
			}
		} catch (error) {
			console.error('Failed to cancel job:', error);
		}
	}
</script>

<div class="container">
	<div class="header">
		<h1>Jobs</h1>
		<button class="btn btn-primary" on:click={loadJobs}>Refresh</button>
	</div>

	{#if loading}
		<p>Loading jobs...</p>
	{:else if jobs.length === 0}
		<div class="empty-state">
			<p>No jobs found</p>
			<a href="/scan" class="btn btn-primary">Start scanning</a>
		</div>
	{:else}
		<div class="jobs-table">
			{#each jobs as job}
				<div class="job-row">
					<div class="job-main">
						<div class="job-title">
							<strong>{job.file_path || job.id}</strong>
							<span class="status-badge status-{job.status}">{job.status}</span>
						</div>
						{#if job.number}
							<p class="job-number">Number: {job.number}</p>
						{/if}
						{#if job.progress}
							<div class="progress-bar">
								<div class="progress-fill" style="width: {job.progress}%"></div>
							</div>
							<p class="progress-text">{job.progress}%</p>
						{/if}
						{#if job.error}
							<p class="error-message">{job.error}</p>
						{/if}
					</div>
					<div class="job-actions">
						{#if job.status === 'failed'}
							<button class="btn btn-secondary" on:click={() => retryJob(job.id)}>Retry</button>
						{/if}
						{#if job.status === 'pending' || job.status === 'processing'}
							<button class="btn btn-secondary" on:click={() => cancelJob(job.id)}>Cancel</button>
						{/if}
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 2rem;
	}

	.empty-state {
		text-align: center;
		padding: 4rem 2rem;
	}

	.empty-state p {
		margin-bottom: 1rem;
		color: var(--text-secondary);
	}

	.jobs-table {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.job-row {
		background: var(--bg-secondary);
		border: 1px solid var(--border);
		border-radius: 0.5rem;
		padding: 1.5rem;
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
	}

	.job-main {
		flex: 1;
	}

	.job-title {
		display: flex;
		align-items: center;
		gap: 1rem;
		margin-bottom: 0.5rem;
	}

	.job-number {
		color: var(--text-secondary);
		font-size: 0.875rem;
		margin-bottom: 0.5rem;
	}

	.progress-bar {
		width: 100%;
		height: 0.5rem;
		background: var(--bg-tertiary);
		border-radius: 9999px;
		overflow: hidden;
		margin: 0.5rem 0;
	}

	.progress-fill {
		height: 100%;
		background: var(--primary);
		transition: width 0.3s ease;
	}

	.progress-text {
		font-size: 0.875rem;
		color: var(--text-secondary);
	}

	.error-message {
		color: var(--error);
		font-size: 0.875rem;
		margin-top: 0.5rem;
	}

	.job-actions {
		display: flex;
		gap: 0.5rem;
	}

	@media (max-width: 768px) {
		.job-row {
			flex-direction: column;
		}

		.job-actions {
			width: 100%;
			margin-top: 1rem;
		}

		.job-actions button {
			flex: 1;
		}
	}
</style>
