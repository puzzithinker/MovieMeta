<script lang="ts">
	import '../app.css';
	import { page } from '$app/stores';
</script>

<div class="app">
	<nav class="navbar">
		<div class="container">
			<div class="nav-content">
				<h1 class="logo">MDC</h1>
				<div class="nav-links">
					<a href="/" class:active={$page.url.pathname === '/'}>Dashboard</a>
					<a href="/jobs" class:active={$page.url.pathname === '/jobs'}>Jobs</a>
					<a href="/scan" class:active={$page.url.pathname === '/scan'}>Scan</a>
					<a href="/config" class:active={$page.url.pathname === '/config'}>Config</a>
				</div>
			</div>
		</div>
	</nav>

	<main class="main-content">
		<slot />
	</main>
</div>

<style>
	.app {
		min-height: 100vh;
		display: flex;
		flex-direction: column;
	}

	.navbar {
		background: var(--bg-secondary);
		border-bottom: 1px solid var(--border);
		padding: 1rem 0;
	}

	.nav-content {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.logo {
		font-size: 1.5rem;
		font-weight: bold;
		color: var(--primary);
	}

	.nav-links {
		display: flex;
		gap: 2rem;
	}

	.nav-links a {
		color: var(--text-secondary);
		text-decoration: none;
		font-weight: 500;
		transition: color 0.2s;
	}

	.nav-links a:hover,
	.nav-links a.active {
		color: var(--primary);
	}

	.main-content {
		flex: 1;
		padding: 2rem 0;
	}

	@media (max-width: 768px) {
		.nav-links {
			gap: 1rem;
		}
	}
</style>
