<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLButtonAttributes } from 'svelte/elements';

	interface Props extends HTMLButtonAttributes {
		variant?: 'primary' | 'secondary' | 'danger';
		size?: 'sm' | 'md' | 'lg';
		children: Snippet;
	}

	let {
		variant = 'primary',
		size = 'md',
		disabled = false,
		children,
		class: className = '',
		...rest
	}: Props = $props();
</script>

<button {disabled} class="btn btn-{variant} btn-{size} {className}" {...rest}>
	{@render children()}
</button>

<style>
	.btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		border: none;
		border-radius: 6px;
		font-weight: 500;
		cursor: pointer;
	}
	.btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
	.btn-primary {
		background: #2563eb;
		color: white;
	}
	.btn-secondary {
		background: #e5e7eb;
		color: #111827;
	}
	.btn-danger {
		background: #dc2626;
		color: white;
	}
	.btn-sm {
		padding: 6px 12px;
		font-size: 0.875rem;
	}
	.btn-md {
		padding: 8px 16px;
		font-size: 1rem;
	}
	.btn-lg {
		padding: 12px 24px;
		font-size: 1.125rem;
	}
</style>
