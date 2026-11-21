<script lang="ts">
	import { Bot, Folder, MessagesCircle, Plus } from '@lucide/svelte/icons';
	import { buttonVariants } from '#lib/components/ui/button';
	import * as Card from '#lib/components/ui/card';

	let { data } = $props();

	const cards = $derived([
		{ label: 'แชท', count: data.counts.chats, href: '/chat', icon: MessagesCircle },
		{ label: 'Agents', count: data.counts.agents, href: '/agent', icon: Bot },
		{ label: 'Projects', count: data.counts.projects, href: '/project', icon: Folder }
	]);
</script>

<div class="mx-auto flex h-full max-w-4xl flex-col gap-8 overflow-y-auto p-6">
	<header class="flex items-start justify-between gap-4">
		<div class="flex flex-col gap-1">
			<h1 class="text-3xl font-semibold tracking-tight">VVOV</h1>
			<p class="text-sm text-muted-foreground">AI workspace บนเครื่องนี้ — local first</p>
		</div>

		<a href="/chat/new" class={buttonVariants()}>
			<Plus data-icon="inline-start" />
			แชทใหม่
		</a>
	</header>

	<div class="grid grid-cols-3 gap-2 sm:gap-3">
		{#each cards as item (item.label)}
			<Card.Root>
				<Card.Header class="flex-row items-center justify-between">
					<Card.Title class="text-xs font-medium text-muted-foreground sm:text-sm"
						>{item.label}</Card.Title
					>
					<item.icon class="hidden text-muted-foreground sm:block" />
				</Card.Header>
				<Card.Content class="pt-0">
					<a
						href={item.href}
						class="text-2xl font-semibold tracking-tight hover:underline sm:text-3xl">{item.count}</a
					>
				</Card.Content>
			</Card.Root>
		{/each}
	</div>

	<section class="flex flex-col gap-2">
		<div class="flex items-center justify-between">
			<h2 class="text-sm font-medium">แชทล่าสุด</h2>
			<a href="/chat" class="text-xs text-muted-foreground hover:text-foreground">ดูทั้งหมด</a>
		</div>

		{#if data.recent.length === 0}
			<p class="text-sm text-muted-foreground">ยังไม่มีแชท</p>
		{:else}
			<ul class="flex flex-col gap-1">
				{#each data.recent as chat (chat.id)}
					<li>
						<a href="/chat/{chat.id}" class="flex items-center justify-between gap-3 rounded-lg px-3 py-2 hover:bg-muted">
							<span class="min-w-0 flex-1 truncate text-sm">{chat.title}</span>
							<span class="shrink-0 text-xs whitespace-nowrap text-muted-foreground">{chat.when}</span>
						</a>
					</li>
				{/each}
			</ul>
		{/if}
	</section>

	<div class="grid gap-6 sm:grid-cols-2">
		<section class="flex flex-col gap-2">
			<div class="flex items-center justify-between">
				<h2 class="text-sm font-medium">Agents</h2>
				<a href="/agent" class="text-xs text-muted-foreground hover:text-foreground">จัดการ</a>
			</div>
			{#if data.agents.length === 0}
				<p class="text-sm text-muted-foreground">ยังไม่มี agent</p>
			{:else}
				<ul class="flex flex-col gap-1">
					{#each data.agents as agent (agent.id)}
						<li>
							<a href="/agent/{agent.id}" class="flex items-center gap-2 rounded-lg px-3 py-2 hover:bg-muted">
								<span class="text-base">{agent.avatar || '🤖'}</span>
								<span class="truncate text-sm">{agent.name}</span>
							</a>
						</li>
					{/each}
				</ul>
			{/if}
		</section>

		<section class="flex flex-col gap-2">
			<div class="flex items-center justify-between">
				<h2 class="text-sm font-medium">Projects</h2>
				<a href="/project" class="text-xs text-muted-foreground hover:text-foreground">จัดการ</a>
			</div>
			{#if data.projects.length === 0}
				<p class="text-sm text-muted-foreground">ยังไม่มี project</p>
			{:else}
				<ul class="flex flex-col gap-1">
					{#each data.projects as project (project.id)}
						<li>
							<a href="/project/{project.id}" class="flex items-center gap-2 rounded-lg px-3 py-2 hover:bg-muted">
								<Folder class="text-muted-foreground" />
								<span class="truncate text-sm">{project.name}</span>
							</a>
						</li>
					{/each}
				</ul>
			{/if}
		</section>
	</div>
</div>
