<script lang="ts">
  import { onOpenUrl } from '@tauri-apps/plugin-deep-link';
  import '../app.css';
  import { onDestroy, onMount } from 'svelte';
  import { recordingState } from '$lib/stores/recording';
  import { RecordingState } from '$lib/types/gym';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import posthog from 'posthog-js';
  import { browser } from '$app/environment';
  import { afterNavigate } from '$app/navigation';

  let { children } = $props();
  let unlistenState: UnlistenFn | null = null;

  if (browser) {
    afterNavigate(() => posthog.capture('$pageview'));
  }

  onMount(async () => {
    // update recording state store
    unlistenState = await listen<{ state: RecordingState }>('recording-status', (event) => {
      $recordingState = event.payload.state;
    });
  });
  onDestroy(() => {
    unlistenState?.();
  });

  onOpenUrl(async (urls) => {
    console.log('Deep link opened!');
    console.log(urls);
  });
</script>

<svelte:head>viralmind desktop</svelte:head>

{@render children()}
