<script lang="ts">
	import './layout.css';
	import favicon from '#lib/assets/favicon.svg';
	import { onMount } from 'svelte';
	import { ModeWatcher } from 'mode-watcher';
	import { Menu, PanelRight, Plus } from '@lucide/svelte/icons';
	import { buttonVariants } from '#lib/components/ui/button';
	import { cn } from '#lib/utils';
	import type { LayoutProps } from './$types';
	import SidebarLeft from '#lib/components/layout/SidebarLeft.svelte';
	import SidebarRight from '#lib/components/layout/SidebarRight.svelte';

	let { children, data }: LayoutProps = $props();
	let mobileOpen = $state(false);

	const CONTEXT_KEY = 'vvov:context-panel';
	let contextOpen = $state(false);

	onMount(() => {
		contextOpen = localStorage.getItem(CONTEXT_KEY) === 'open';
	});

	function toggleContext() {
		contextOpen = !contextOpen;
		localStorage.setItem(CONTEXT_KEY, contextOpen ? 'open' : 'closed');
	}
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>VVOV — AI workspace</title>
	<meta
		name="description"
		content="พื้นที่ทำงาน AI แบบ local-first — แชท, agents และ projects ที่เก็บข้อมูลไว้ในเครื่อง"
	/>
</svelte:head>

<ModeWatcher />

<div class="flex h-dvh overflow-hidden bg-background text-foreground">
	<SidebarLeft
		chats={data.chats}
		counts={data.counts}
		bind:mobileOpen
		onToggleContext={toggleContext}
	/>

	<div class="flex min-w-0 flex-1 flex-col">
		<header class="flex items-center gap-2 border-b border-border px-2 py-1.5 md:hidden">
			<button
				type="button"
				class={cn(buttonVariants({ variant: 'ghost', size: 'icon' }), 'text-muted-foreground')}
				onclick={() => (mobileOpen = true)}
				aria-label="เปิดเมนู"
				aria-expanded={mobileOpen}
				aria-controls="app-sidebar"
			>
				<Menu />
			</button>

			<a href="/" class="text-sm font-semibold">VVOV</a>

			<div class="ml-auto flex items-center gap-1">
				<button
					type="button"
					class={cn(buttonVariants({ variant: 'ghost', size: 'icon' }), 'text-muted-foreground')}
					onclick={toggleContext}
					aria-label="เปิดแผงบริบท"
					aria-expanded={contextOpen}
					aria-controls="context-panel"
				>
					<PanelRight />
				</button>

				<a
					href="/chat/new"
					class={cn(
						buttonVariants({ variant: 'ghost', size: 'icon' }),
						'text-muted-foreground'
					)}
					aria-label="แชทใหม่"
				>
					<Plus />
				</a>
			</div>
		</header>

		<main class="min-h-0 min-w-0 flex-1">
			{@render children()}
		</main>
	</div>

	<SidebarRight context={data.context} open={contextOpen} onToggle={toggleContext} />
</div>
