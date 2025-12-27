<script lang="ts">
	import { onMount } from 'svelte';

	let stats = {
		total_jobs: 0,
		completed: 0,
		failed: 0,
		pending: 0
	};

	let recentJobs: any[] = [];
	let loading = true;

	onMount(async () => {
		await loadStats();
		await loadRecentJobs();
		loading = false;
	});

	async function loadStats() {
		try {
			const response = await fetch('/api/stats');
			if (response.ok) {
				stats = await response.json();
			}
		} catch (error) {
			console.error('Failed to load stats:', error);
		}
	}

	async function loadRecentJobs() {
		try {
			const response = await fetch('/api/jobs');
			if (response.ok) {
				const data = await response.json();
				recentJobs = data.slice(0, 5);
			}
		} catch (error) {
			console.error('Failed to load jobs:', error);
		}
	}
</script>

<div class="container">
	<h1>Dashboard</h1>

	{#if loading}
		<p>Loading...</p>
	{:else}
		<div class="stats-grid">
			<div class="stat-card">
				<h3>Total Jobs</h3>
				<p class="stat-value">{stats.total_jobs}</p>
			</div>
			<div class="stat-card">
				<h3>Completed</h3>
				<p class="stat-value success">{stats.completed}</p>
			</div>
			<div class="stat-card">
				<h3>Failed</h3>
				<p class="stat-value error">{stats.failed}</p>
			</div>
			<div class="stat-card">
				<h3>Pending</h3>
				<p class="stat-value">{stats.pending}</p>
			</div>
		</div>

		<div class="recent-jobs">
			<h2>Recent Jobs</h2>
			{#if recentJobs.length === 0}
				<p>No jobs yet</p>
			{:else}
				<div class="jobs-list">
					{#each recentJobs as job}
						<div class="job-item">
							<div class="job-info">
								<strong>{job.file_path || 'Unknown'}</strong>
								<span class="status-badge status-{job.status}">{job.status}</span>
							</div>
							{#if job.error}
								<p class="job-error">{job.error}</p>
							{/if}
						</div>
					{/each}
				</div>
			{/if}
			<a href="/jobs" class="view-all">View all jobs →</a>
		</div>
	{/if}
</div>

<style>
	h1 {
		margin-bottom: 2rem;
	}

	.stats-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
		gap: 1.5rem;
		margin-bottom: 3rem;
	}

	.stat-card {
		background: var(--bg-secondary);
		padding: 1.5rem;
		border-radius: 0.5rem;
		border: 1px solid var(--border);
	}

	.stat-card h3 {
		font-size: 0.875rem;
		color: var(--text-secondary);
		margin-bottom: 0.5rem;
	}

	.stat-value {
		font-size: 2rem;
		font-weight: bold;
	}

	.stat-value.success {
		color: var(--success);
	}

	.stat-value.error {
		color: var(--error);
	}

	.recent-jobs {
		background: var(--bg-secondary);
		padding: 1.5rem;
		border-radius: 0.5rem;
		border: 1px solid var(--border);
	}

	.recent-jobs h2 {
		margin-bottom: 1rem;
	}

	.jobs-list {
		display: flex;
		flex-direction: column;
		gap: 1rem;
		margin-bottom: 1rem;
	}

	.job-item {
		background: var(--bg-tertiary);
		padding: 1rem;
		border-radius: 0.375rem;
	}

	.job-info {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.job-error {
		margin-top: 0.5rem;
		color: var(--error);
		font-size: 0.875rem;
	}

	.view-all {
		color: var(--primary);
		text-decoration: none;
		font-weight: 500;
	}

	.view-all:hover {
		text-decoration: underline;
	}
</style>
