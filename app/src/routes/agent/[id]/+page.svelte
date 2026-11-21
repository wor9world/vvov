<script lang="ts">
	import { ChevronLeft, MessageSquarePlus, Trash2 } from '@lucide/svelte/icons';
	import { Button } from '#lib/components/ui/button';
	import * as Card from '#lib/components/ui/card';
	import * as Field from '#lib/components/ui/field';
	import { Input } from '#lib/components/ui/input';
	import * as Select from '#lib/components/ui/select';
	import { Textarea } from '#lib/components/ui/textarea';

	let { data } = $props();
	let model = $state(initialModel());

	function initialModel() {
		return data.agent.model_id ?? 'default';
	}
</script>

<div class="mx-auto flex h-full max-w-3xl flex-col gap-6 overflow-y-auto p-6">
	<a href="/agent" class="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground">
		<ChevronLeft size={14} />
		Agents
	</a>

	<header class="flex items-start gap-4">
		<div class="flex size-14 shrink-0 items-center justify-center rounded-full bg-muted text-2xl">
			{data.agent.avatar || '🤖'}
		</div>
		<div class="flex min-w-0 flex-col gap-1">
			<h1 class="text-xl font-semibold tracking-tight">{data.agent.name}</h1>
			<p class="text-sm text-muted-foreground">{data.agent.description || 'ไม่มีคำอธิบาย'}</p>
		</div>
	</header>

	<div class="flex flex-wrap gap-2">
		<form method="POST" action="?/newChat">
			<Button type="submit">
				<MessageSquarePlus data-icon="inline-start" />
				แชทใหม่กับ agent นี้
			</Button>
		</form>

		<form method="POST" action="?/remove">
			<Button type="submit" variant="outline" class="text-destructive">
				<Trash2 data-icon="inline-start" />
				ลบ agent
			</Button>
		</form>
	</div>

	<Card.Root>
		<Card.Header>
			<Card.Title class="text-base">แก้ไข persona</Card.Title>
			<Card.Description>system prompt นี้จะถูกส่งให้โมเดลทุกครั้งที่แชทกับ agent</Card.Description>
		</Card.Header>
		<Card.Content>
			<form method="POST" action="?/update">
				<Field.Group>
					<Field.Field>
						<Field.Label for="agent-name">ชื่อ</Field.Label>
						<Input id="agent-name" name="name" value={data.agent.name} required />
					</Field.Field>

					<Field.Field>
						<Field.Label for="agent-avatar">Emoji</Field.Label>
						<Input id="agent-avatar" name="avatar" value={data.agent.avatar} maxlength={4} />
					</Field.Field>

					<Field.Field>
						<Field.Label for="agent-description">คำอธิบาย</Field.Label>
						<Textarea id="agent-description" name="description" rows={2} value={data.agent.description} />
					</Field.Field>

					<Field.Field>
						<Field.Label for="agent-prompt">System prompt</Field.Label>
						<Textarea id="agent-prompt" name="system_prompt" rows={6} value={data.agent.system_prompt} />
					</Field.Field>

					<Field.Field>
						<Field.Label>โมเดล</Field.Label>
						<Select.Root type="single" name="model_id" bind:value={model}>
							<Select.Trigger class="w-full">{model === 'default' ? 'ค่าเริ่มต้น' : model}</Select.Trigger>
							<Select.Content>
								<Select.Item value="default">ค่าเริ่มต้นของระบบ</Select.Item>
								{#each data.models as item (item.id)}
									<Select.Item value={item.id}>{item.label}</Select.Item>
								{/each}
							</Select.Content>
						</Select.Root>
					</Field.Field>

					<Button type="submit" variant="secondary" size="sm">บันทึก</Button>
				</Field.Group>
			</form>
		</Card.Content>
	</Card.Root>

	<section class="flex flex-col gap-2">
		<h2 class="text-sm font-medium text-muted-foreground">แชทของ agent นี้ ({data.chats.length})</h2>

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
