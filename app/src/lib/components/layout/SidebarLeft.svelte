<script lang="ts">
	import { onMount } from 'svelte';
	import { goto, refreshAll } from '$app/navigation';
	import { page } from '$app/state';
	import {
		Bot,
		Folder,
		MessagesCircle,
		Moon,
		Orbit,
		PanelLeft,
		PanelRight,
		Plus,
		Search,
		Sun,
		Trash2,
		X
	} from '@lucide/svelte/icons';
	import { buttonVariants } from '#lib/components/ui/button';
	import { Input } from '#lib/components/ui/input';
	import { mode, toggleMode } from 'mode-watcher';
	import { cn } from '#lib/utils';

	type GroupKey = 'today' | 'yesterday' | 'week' | 'older';

	interface ChatItem {
		id: string;
		title: string;
		updated_at: number;
		group: GroupKey;
	}

	interface Counts {
		chats: number;
		agents: number;
		projects: number;
	}

	let {
		chats,
		counts,
		mobileOpen = $bindable(false),
		onToggleContext
	}: {
		chats: ChatItem[];
		counts: Counts;
		mobileOpen: boolean;
		onToggleContext?: () => void;
	} = $props();

	const STORAGE_KEY = 'vvov:sidebar';
	let collapsed = $state(false);
	let query = $state('');
	let deleting = $state<string | null>(null);

	onMount(() => {
		collapsed = localStorage.getItem(STORAGE_KEY) === 'collapsed';
	});

	function toggleCollapsed() {
		collapsed = !collapsed;
		localStorage.setItem(STORAGE_KEY, collapsed ? 'collapsed' : 'open');
	}

	const sections = $derived([
		{ href: '/agent', icon: Bot, label: 'Agents', count: counts.agents },
		{ href: '/project', icon: Folder, label: 'Projects', count: counts.projects },
		{ href: '/chat', icon: MessagesCircle, label: 'Chats', count: counts.chats }
	]);

	const activeChatId = $derived(
		page.url.pathname.startsWith('/chat/') ? page.url.pathname.slice('/chat/'.length) : null
	);

	const groups = $derived.by(() => {
		const order: { key: GroupKey; label: string }[] = [
			{ key: 'today', label: 'วันนี้' },
			{ key: 'yesterday', label: 'เมื่อวาน' },
			{ key: 'week', label: '7 วันที่ผ่านมา' },
			{ key: 'older', label: 'เก่ากว่า' }
		];

		const q = query.trim().toLowerCase();
		const filtered = q ? chats.filter((chat) => chat.title.toLowerCase().includes(q)) : chats;

		return order
			.map((group) => ({ ...group, items: filtered.filter((chat) => chat.group === group.key) }))
			.filter((group) => group.items.length > 0);
	});

	async function removeChat(id: string, title: string) {
		if (deleting) return;
		if (!confirm(`ลบแชท "${title}" ?`)) return;

		deleting = id;
		try {
			const response = await fetch(`/api/chats/${id}`, { method: 'DELETE' });
			if (!response.ok) return;

			if (page.url.pathname === `/chat/${id}`) await goto('/');
			else await refreshAll();
		} finally {
			deleting = null;
		}
	}
</script>

{#if mobileOpen}
	<button
		type="button"
		class="fixed inset-0 z-40 bg-black/40 md:hidden"
		aria-label="ปิดเมนู"
		onclick={() => (mobileOpen = false)}
	></button>
{/if}

<aside
	id="app-sidebar"
	aria-label="แถบนำทางหลัก"
	class={cn(
		'fixed inset-y-0 left-0 z-50 flex w-64 shrink-0 flex-col gap-3 border-r border-border bg-sidebar p-3 text-sidebar-foreground transition-[width,transform] duration-200 md:static md:z-auto md:translate-x-0',
		collapsed ? 'md:w-14' : 'md:w-64',
		mobileOpen ? 'translate-x-0' : '-translate-x-full'
	)}
>
	<div class={cn('flex items-center gap-1', collapsed && 'md:flex-col')}>
		<button
			type="button"
			class={cn(buttonVariants({ variant: 'ghost', size: 'icon' }), 'text-muted-foreground')}
			onclick={() => goto('/')}
			aria-label="หน้าแรก"
			title="หน้าแรก"
		>
			<Orbit />
		</button>

		<a
			href="/"
			class={cn('min-w-0 flex-1 truncate px-1 text-sm font-semibold', collapsed && 'md:hidden')}
		>
			VVOV
		</a>

		<button
			type="button"
			class={cn(
				buttonVariants({ variant: 'ghost', size: 'icon' }),
				'hidden text-muted-foreground md:inline-flex'
			)}
			onclick={toggleCollapsed}
			aria-label={collapsed ? 'ขยายแถบด้านข้าง' : 'ยุบแถบด้านข้าง'}
			aria-expanded={!collapsed}
		>
			<PanelLeft />
		</button>

		<button
			type="button"
			class={cn(buttonVariants({ variant: 'ghost', size: 'icon' }), 'text-muted-foreground md:hidden')}
			onclick={() => (mobileOpen = false)}
			aria-label="ปิดเมนู"
		>
			<X />
		</button>
	</div>

	<a
		href="/chat/new"
		onclick={() => (mobileOpen = false)}
		class={cn(
			buttonVariants({ variant: 'secondary', size: 'lg' }),
			'w-full justify-start',
			collapsed && 'md:justify-center md:px-0'
		)}
		aria-label="แชทใหม่"
		title="แชทใหม่"
	>
		<Plus data-icon={collapsed ? undefined : 'inline-start'} />
		<span class={cn(collapsed && 'md:hidden')}>แชทใหม่</span>
	</a>

	<nav aria-label="หมวดหมู่" class="flex flex-col gap-0.5">
		{#each sections as section (section.href)}
			{@const active = page.url.pathname.startsWith(section.href)}
			<a
				href={section.href}
				onclick={() => (mobileOpen = false)}
				aria-current={active ? 'page' : undefined}
				title={section.label}
				class={cn(
					'flex items-center gap-2 rounded-md px-2 py-1.5 text-sm transition-colors',
					active
						? 'bg-accent font-medium text-accent-foreground'
						: 'text-muted-foreground hover:bg-muted hover:text-foreground',
					collapsed && 'md:justify-center md:px-0'
				)}
			>
				<section.icon size={16} class="shrink-0" />
				<span class={cn('truncate', collapsed && 'md:hidden')}>{section.label}</span>
				<span
					class={cn(
						'ml-auto text-xs tabular-nums',
						active ? 'text-accent-foreground' : 'text-muted-foreground',
						collapsed && 'md:hidden'
					)}
				>
					{section.count}
				</span>
			</a>
		{/each}
	</nav>

	<div class={cn('relative', collapsed && 'md:hidden')}>
		<Search
			size={14}
			class="pointer-events-none absolute top-1/2 left-2 -translate-y-1/2 text-muted-foreground"
		/>
		<Input bind:value={query} placeholder="ค้นหาแชท" aria-label="ค้นหาแชท" class="h-8 pl-7" />
	</div>

	<div class={cn('flex min-h-0 flex-1 flex-col gap-3 overflow-y-auto', collapsed && 'md:hidden')}>
		{#if chats.length === 0}
			<p class="px-2 text-sm text-muted-foreground">ยังไม่มีแชท</p>
		{:else if groups.length === 0}
			<p class="px-2 text-sm text-muted-foreground">ไม่พบแชทที่ค้นหา</p>
		{:else}
			{#each groups as group (group.key)}
				<section class="flex flex-col gap-0.5" aria-labelledby={`chat-group-${group.key}`}>
					<h2
						id={`chat-group-${group.key}`}
						class="px-2 pb-1 text-[11px] font-medium tracking-wide text-muted-foreground uppercase"
					>
						{group.label}
					</h2>

					{#each group.items as chat (chat.id)}
						<div class="group/row relative">
							<a
								href="/chat/{chat.id}"
								onclick={() => (mobileOpen = false)}
								aria-current={activeChatId === chat.id ? 'page' : undefined}
								class={cn(
									'block truncate rounded-md py-1.5 pr-8 pl-2 text-sm transition-colors',
									activeChatId === chat.id
										? 'bg-accent font-medium text-accent-foreground'
										: 'text-muted-foreground hover:bg-muted hover:text-foreground'
								)}
							>
								{chat.title}
							</a>

							<button
								type="button"
								class="absolute top-1/2 right-1 -translate-y-1/2 rounded p-1 text-muted-foreground opacity-0 transition-opacity hover:bg-background hover:text-destructive focus-visible:opacity-100 group-hover/row:opacity-100 disabled:opacity-50"
								onclick={() => removeChat(chat.id, chat.title)}
								disabled={deleting === chat.id}
								aria-label="ลบแชท {chat.title}"
							>
								<Trash2 size={14} />
							</button>
						</div>
					{/each}
				</section>
			{/each}
		{/if}
	</div>

	<div class={cn('flex items-center border-t border-border pt-2', collapsed && 'md:flex-col')}>
		<button
			type="button"
			class={cn(
				buttonVariants({ variant: 'ghost' }),
				'flex-1 justify-start text-muted-foreground xl:hidden',
				collapsed && 'md:flex-1 md:justify-center md:px-0'
			)}
			onclick={() => {
				mobileOpen = false;
				onToggleContext?.();
			}}
			aria-label="เปิดแผงบริบท"
			title="บริบท"
		>
			<PanelRight size={16} />
			<span class={cn(collapsed && 'md:hidden')}>บริบท</span>
		</button>

		<button
			type="button"
			class={cn(
				buttonVariants({ variant: 'ghost' }),
				'flex-1 justify-start text-muted-foreground',
				collapsed && 'md:flex-1 md:justify-center md:px-0'
			)}
			onclick={() => toggleMode()}
			aria-label="สลับธีมสว่าง/มืด"
			title="สลับธีมสว่าง/มืด"
		>
			{#if mode.current === 'dark'}
				<Sun size={16} />
			{:else}
				<Moon size={16} />
			{/if}
			<span class={cn(collapsed && 'md:hidden')}>สลับธีม</span>
		</button>
	</div>
</aside>
