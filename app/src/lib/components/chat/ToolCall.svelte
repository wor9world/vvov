<script lang="ts">
	import { Badge } from '#lib/components/ui/badge';
	import type { ToolPartLike } from './tool-part';

	let { part }: { part: ToolPartLike } = $props();

	const name = $derived(
		part.type === 'dynamic-tool' ? (part.toolName ?? 'tool') : part.type.replace(/^tool-/, '')
	);

	const state = $derived(part.state ?? 'input-available');

	const label = $derived(
		state === 'output-available'
			? 'เสร็จแล้ว'
			: state === 'output-error'
				? 'ผิดพลาด'
				: state === 'input-streaming'
					? 'กำลังรับค่า'
					: 'กำลังทำงาน'
	);

	const variant = $derived(
		state === 'output-available' ? 'secondary' : state === 'output-error' ? 'destructive' : 'outline'
	);
</script>

<div class="my-2 overflow-hidden rounded-lg border border-border bg-muted/30">
	<div class="flex items-center gap-2 border-b border-border px-3 py-1.5 text-xs">
		<span class="font-mono font-medium">{name}</span>
		<Badge {variant} class="ml-auto">{label}</Badge>
	</div>

	<div class="space-y-2 px-3 py-2">
		{#if part.input}
			<pre class="overflow-x-auto rounded bg-background/70 p-2 font-mono text-[11px] leading-relaxed">{JSON.stringify(
					part.input,
					null,
					2
				)}</pre>
		{/if}

		{#if part.output}
			<pre class="overflow-x-auto rounded bg-background/70 p-2 font-mono text-[11px] leading-relaxed">{JSON.stringify(
					part.output,
					null,
					2
				)}</pre>
		{/if}

		{#if part.errorText}
			<p class="text-xs text-destructive">{part.errorText}</p>
		{/if}
	</div>
</div>
