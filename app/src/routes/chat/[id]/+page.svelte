<script lang="ts">
	import { ChevronLeft } from '@lucide/svelte/icons';
	import { Badge } from '#lib/components/ui/badge';
	import ChatView from '#lib/components/chat/ChatView.svelte';

	let { data } = $props();
</script>

<div class="flex h-full min-h-0 flex-col">
	<header class="flex items-center gap-2 border-b border-border px-4 py-2">
		<a
			href="/chat"
			class="flex shrink-0 items-center gap-1 text-xs text-muted-foreground hover:text-foreground"
			aria-label="กลับไปรวมแชท"
		>
			<ChevronLeft size={14} />
			<span class="hidden sm:inline">แชท</span>
		</a>

		<span class="min-w-0 flex-1 truncate text-sm font-medium">{data.chat.title}</span>

		{#if data.agentName}
			<Badge variant="secondary" class="hidden shrink-0 md:inline-flex">agent: {data.agentName}</Badge>
		{/if}
		{#if data.projectName}
			<Badge variant="outline" class="hidden shrink-0 md:inline-flex">project: {data.projectName}</Badge>
		{/if}
	</header>

	<div class="min-h-0 flex-1">
		{#key data.chat.id}
			<ChatView
				chatId={data.chat.id}
				messages={data.messages}
				models={data.models}
				modelId={data.chat.model_id ?? 'default'}
				persistModel
			/>
		{/key}
	</div>
</div>
