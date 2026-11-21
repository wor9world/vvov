<script lang="ts">
	import { Badge } from '#lib/components/ui/badge';
	import ChatView from '#lib/components/chat/ChatView.svelte';

	let { data } = $props();
</script>

<div class="flex h-full min-h-0 flex-col">
	<header class="flex items-center gap-2 border-b border-border px-4 py-2">
		<a href="/chat" class="shrink-0 text-xs text-muted-foreground hover:text-foreground">แชท</a>
		<span class="min-w-0 truncate text-sm font-medium">แชทใหม่</span>
		<Badge variant="outline" class="hidden shrink-0 sm:inline-flex">ยังไม่บันทึก</Badge>
	</header>

	<div class="min-h-0 flex-1">
		<ChatView
			chatId={data.chatId}
			messages={[]}
			adoptUrl
			placeholder="เริ่มแชทใหม่…"
			models={data.models}
		/>
	</div>
</div>
