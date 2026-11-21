<script lang="ts">
	import { Pencil, Plus, Trash2 } from '@lucide/svelte/icons';
	import { Button, buttonVariants } from '#lib/components/ui/button';
	import * as Empty from '#lib/components/ui/empty';
	import { Input } from '#lib/components/ui/input';

	let { data } = $props();

	let editingId = $state<string | null>(null);
	let renameInput = $state<HTMLInputElement | null>(null);

	$effect(() => {
		if (editingId && renameInput) {
			renameInput.focus();
			renameInput.select();
		}
	});

	function confirmRemove(event: SubmitEvent) {
		if (!confirm('ลบแชทนี้? ข้อความทั้งหมดจะหายไปด้วย')) event.preventDefault();
	}
</script>

<div class="mx-auto flex h-full max-w-3xl flex-col gap-6 overflow-y-auto p-6">
	<header class="flex items-end justify-between gap-4">
		<div class="flex flex-col gap-1">
			<h1 class="text-xl font-semibold tracking-tight">แชททั้งหมด</h1>
			<p class="text-sm text-muted-foreground">{data.chats.length} แชท</p>
		</div>

		<a href="/chat/new" class={buttonVariants()}>
			<Plus data-icon="inline-start" />
			แชทใหม่
		</a>
	</header>

	{#if data.chats.length === 0}
		<Empty.Root class="mt-16">
			<Empty.Header>
				<Empty.Title>ยังไม่มีแชท</Empty.Title>
				<Empty.Description>กด "แชทใหม่" เพื่อเริ่มคุยกับโมเดล</Empty.Description>
			</Empty.Header>
		</Empty.Root>
	{:else}
		<ul class="flex flex-col gap-2">
			{#each data.chats as chat (chat.id)}
				<li class="group flex items-center gap-1 rounded-lg border border-border p-2 pl-3">
					{#if editingId === chat.id}
						<form method="POST" action="?/rename" class="flex min-w-0 flex-1 items-center gap-1">
							<input type="hidden" name="id" value={chat.id} />
							<Input
								bind:ref={renameInput}
								name="title"
								value={chat.title}
								class="h-8 min-w-0 flex-1"
								aria-label="ชื่อแชท"
							/>
							<Button type="submit" size="sm">บันทึก</Button>
							<Button type="button" variant="ghost" size="sm" onclick={() => (editingId = null)}>
								ยกเลิก
							</Button>
						</form>
					{:else}
						<a href="/chat/{chat.id}" class="flex min-w-0 flex-1 flex-col gap-0.5">
							<span class="truncate text-sm font-medium">{chat.title}</span>
							<span class="truncate text-xs text-muted-foreground">
								{chat.when}
								{#if chat.agent_name}· agent: {chat.agent_name}{/if}
								{#if chat.project_name}· project: {chat.project_name}{/if}
							</span>
						</a>

						<button
							type="button"
							class="rounded p-1.5 text-muted-foreground transition-opacity hover:bg-muted hover:text-foreground focus-visible:opacity-100 sm:opacity-0 sm:group-hover:opacity-100"
							onclick={() => (editingId = chat.id)}
							aria-label="เปลี่ยนชื่อแชท"
						>
							<Pencil size={14} />
						</button>

						<form method="POST" action="?/remove" onsubmit={confirmRemove}>
							<input type="hidden" name="id" value={chat.id} />
							<button
								type="submit"
								class="rounded p-1.5 text-muted-foreground transition-opacity hover:bg-muted hover:text-destructive focus-visible:opacity-100 sm:opacity-0 sm:group-hover:opacity-100"
								aria-label="ลบแชท"
							>
								<Trash2 size={14} />
							</button>
						</form>
					{/if}
				</li>
			{/each}
		</ul>
	{/if}
</div>
