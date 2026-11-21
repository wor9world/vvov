<script lang="ts">
	import { Bot, Folder, MessagesCircle, PanelRight } from '@lucide/svelte/icons';
	import { buttonVariants } from '#lib/components/ui/button';
	import { cn } from '#lib/utils';
	import type { ContextSummary } from '#lib/context';

	let {
		context,
		open,
		onToggle
	}: { context: ContextSummary; open: boolean; onToggle: () => void } = $props();

	/** ต่ำกว่า xl แผงเป็น sheet — ปิดหลังกดลิงก์เพื่อไม่ให้บังหน้าใหม่ */
	function closeOnNavigate() {
		if (typeof window !== 'undefined' && window.matchMedia('(max-width: 1279px)').matches) {
			onToggle();
		}
	}

	function handleKeydown(event: KeyboardEvent) {
		if (
			open &&
			event.key === 'Escape' &&
			typeof window !== 'undefined' &&
			window.matchMedia('(max-width: 1279px)').matches
		) {
			onToggle();
		}
	}

	const heading = $derived(
		context.kind === 'chat'
			? context.title
			: context.kind === 'agent'
				? context.name
				: context.kind === 'project'
					? context.name
					: 'Workspace'
	);

	const kindLabel = $derived(
		context.kind === 'chat'
			? 'แชท'
			: context.kind === 'agent'
				? 'Agent'
				: context.kind === 'project'
					? 'Project'
					: 'ภาพรวม'
	);
</script>

{#snippet meta(label: string, value: string | null)}
	<div class="flex flex-col gap-0.5">
		<dt class="text-[11px] font-medium tracking-wide text-muted-foreground uppercase">{label}</dt>
		<dd class="truncate text-sm">{value || '—'}</dd>
	</div>
{/snippet}

{#if open}
	<button
		type="button"
		class="fixed inset-0 z-40 bg-black/40 xl:hidden"
		aria-label="ปิดแผงบริบท"
		onclick={onToggle}
	></button>
{/if}

<svelte:window onkeydown={handleKeydown} />

<aside
	id="context-panel"
	aria-label="บริบทของหน้าปัจจุบัน"
	class={cn(
		'fixed inset-y-0 right-0 z-50 flex shrink-0 border-l border-border bg-sidebar text-sidebar-foreground transition-transform duration-200 xl:static xl:z-auto xl:translate-x-0',
		open ? 'translate-x-0' : 'translate-x-full invisible xl:visible'
	)}
>
	{#if open}
		<div class="flex w-72 flex-col">
			<div class="flex items-center gap-2 border-b border-border px-3 py-2">
				<h2 class="min-w-0 flex-1 truncate text-sm font-medium">{heading}</h2>
				<button
					type="button"
					class={cn(buttonVariants({ variant: 'ghost', size: 'icon' }), 'text-muted-foreground')}
					onclick={onToggle}
					aria-label="ซ่อนแผงบริบท"
					aria-expanded={open}
				>
					<PanelRight />
				</button>
			</div>

			<div class="min-h-0 flex-1 overflow-y-auto p-3">
				<p class="mb-3 text-[11px] font-medium tracking-wide text-muted-foreground uppercase">
					{kindLabel}
				</p>

				{#if context.kind === 'chat'}
					<dl class="flex flex-col gap-3">
						{@render meta('โมเดล', context.modelId ?? 'ค่าเริ่มต้นของระบบ')}
						{@render meta('Agent', context.agentName)}
						{@render meta('Project', context.projectName)}
						{@render meta('ข้อความ', `${context.messageCount} ข้อความ`)}
					</dl>

					<a
						href="/chat"
						onclick={closeOnNavigate}
						class={cn(buttonVariants({ variant: 'outline', size: 'sm' }), 'mt-4 w-full justify-start')}
					>
						<MessagesCircle data-icon="inline-start" />
						ดูแชททั้งหมด
					</a>
				{:else if context.kind === 'agent'}
					{#if context.description}
						<p class="mb-3 text-sm text-muted-foreground">{context.description}</p>
					{/if}
					<dl class="flex flex-col gap-3">
						{@render meta('โมเดล', context.modelId ?? 'ค่าเริ่มต้นของระบบ')}
						{@render meta('แชท', `${context.chatCount} แชท`)}
					</dl>
				{:else if context.kind === 'project'}
					{#if context.description}
						<p class="mb-3 text-sm text-muted-foreground">{context.description}</p>
					{/if}
					<dl class="flex flex-col gap-3">
						{@render meta('แชท', `${context.chatCount} แชท`)}
					</dl>
				{:else}
					<dl class="flex flex-col gap-3">
						{@render meta('แชท', `${context.counts.chats}`)}
						{@render meta('Agents', `${context.counts.agents}`)}
						{@render meta('Projects', `${context.counts.projects}`)}
					</dl>

					<div class="mt-4 flex flex-col gap-1">
						<a
							href="/agent"
							onclick={closeOnNavigate}
							class={cn(buttonVariants({ variant: 'ghost', size: 'sm' }), 'w-full justify-start')}
						>
							<Bot data-icon="inline-start" />
							Agents
						</a>
						<a
							href="/project"
							onclick={closeOnNavigate}
							class={cn(buttonVariants({ variant: 'ghost', size: 'sm' }), 'w-full justify-start')}
						>
							<Folder data-icon="inline-start" />
							Projects
						</a>
					</div>
				{/if}
			</div>

			<p class="border-t border-border px-3 py-2 text-[11px] text-muted-foreground">
				local-first · ข้อมูลอยู่ในเครื่องนี้
			</p>
		</div>
	{:else}
		<div class="flex w-12 flex-col items-center py-3">
			<button
				type="button"
				class={cn(buttonVariants({ variant: 'ghost', size: 'icon' }), 'text-muted-foreground')}
				onclick={onToggle}
				aria-label="เปิดแผงบริบท"
				aria-expanded={open}
				aria-controls="context-panel"
				title="บริบท"
			>
				<PanelRight />
			</button>
		</div>
	{/if}
</aside>
