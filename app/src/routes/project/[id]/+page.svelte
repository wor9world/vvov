<script lang="ts">
	import { ChevronLeft, Folder, MessageSquarePlus, Trash2 } from '@lucide/svelte/icons';
	import { Button } from '#lib/components/ui/button';
	import * as Card from '#lib/components/ui/card';
	import * as Field from '#lib/components/ui/field';
	import { Input } from '#lib/components/ui/input';
	import { Textarea } from '#lib/components/ui/textarea';

	let { data } = $props();
</script>

<div class="mx-auto flex h-full max-w-3xl flex-col gap-6 overflow-y-auto p-6">
	<a href="/project" class="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground">
		<ChevronLeft size={14} />
		Projects
	</a>

	<header class="flex items-start gap-4">
		<div class="flex size-14 shrink-0 items-center justify-center rounded-full bg-muted">
			<Folder class="text-muted-foreground" />
		</div>
		<div class="flex min-w-0 flex-col gap-1">
			<h1 class="text-xl font-semibold tracking-tight">{data.project.name}</h1>
			<p class="text-sm text-muted-foreground">{data.project.description || 'ไม่มีคำอธิบาย'}</p>
		</div>
	</header>

	<div class="flex flex-wrap gap-2">
		<form method="POST" action="?/newChat">
			<Button type="submit">
				<MessageSquarePlus data-icon="inline-start" />
				แชทใหม่ใน project นี้
			</Button>
		</form>

		<form method="POST" action="?/remove">
			<Button type="submit" variant="outline" class="text-destructive">
				<Trash2 data-icon="inline-start" />
				ลบ project
			</Button>
		</form>
	</div>

	<Card.Root>
		<Card.Header>
			<Card.Title class="text-base">แก้ไข project</Card.Title>
			<Card.Description>instructions จะถูกส่งให้โมเดลทุกแชทใน project นี้</Card.Description>
		</Card.Header>
		<Card.Content>
			<form method="POST" action="?/update">
				<Field.Group>
					<Field.Field>
						<Field.Label for="project-name">ชื่อ</Field.Label>
						<Input id="project-name" name="name" value={data.project.name} required />
					</Field.Field>

					<Field.Field>
						<Field.Label for="project-description">คำอธิบาย</Field.Label>
						<Textarea id="project-description" name="description" rows={2} value={data.project.description} />
					</Field.Field>

					<Field.Field>
						<Field.Label for="project-instructions">Instructions</Field.Label>
						<Textarea id="project-instructions" name="instructions" rows={6} value={data.project.instructions} />
					</Field.Field>

					<Button type="submit" variant="secondary" size="sm">บันทึก</Button>
				</Field.Group>
			</form>
		</Card.Content>
	</Card.Root>

	<section class="flex flex-col gap-2">
		<h2 class="text-sm font-medium text-muted-foreground">แชทใน project นี้ ({data.chats.length})</h2>

		{#if data.chats.length === 0}
			<p class="text-sm text-muted-foreground">ยังไม่มีแชท</p>
		{:else}
			<ul class="flex flex-col gap-1">
				{#each data.chats as chat (chat.id)}
					<li>
						<a href="/chat/{chat.id}" class="flex items-center justify-between rounded-lg px-3 py-2 hover:bg-muted">
							<span class="truncate text-sm">{chat.title}</span>
							<span class="text-xs text-muted-foreground">{chat.when}</span>
						</a>
					</li>
				{/each}
			</ul>
		{/if}
	</section>
</div>
