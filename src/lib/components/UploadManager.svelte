<script lang="ts">
  import { onDestroy } from 'svelte';
  import { Upload, X, Check, AlertCircle } from 'lucide-svelte';
  import { uploadManager } from '$lib/stores/misc';

  // Clean up intervals when component is destroyed
  onDestroy(() => {
    $uploadManager.cleanupIntervals();
  });

  // Get the status icon based on status
  function getStatusIcon(status: string) {
    switch (status) {
      case 'completed':
        return Check;
      case 'failed':
        return AlertCircle;
      default:
        return Upload;
    }
  }

  // Get the status color based on status
  function getStatusColor(status: string) {
    switch (status) {
      case 'completed':
        return 'text-green-500';
      case 'failed':
        return 'text-red-500';
      case 'processing':
        return 'text-yellow-500';
      default:
        return 'text-secondary-300';
    }
  }

  // Get a human-readable status message
  function getStatusMessage(item: { status: string; error?: string }) {
    if (item.status === 'failed' && item.error) {
      return `Failed: ${item.error}`;
    }

    switch (item.status) {
      case 'queued':
        return 'Queued for upload';
      case 'uploading':
        return 'Uploading...';
      case 'zipping':
        return 'Zipping Recording...';
      case 'processing':
        return 'Processing...';
      case 'completed':
        return 'Upload completed';
      case 'failed':
        return 'Upload failed';
      default:
        return 'Unknown status';
    }
  }

  // Format file size to human-readable format
  function formatFileSize(bytes: number | undefined): string {
    if (bytes === undefined) return '0 B';
    
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    let size = bytes;
    let unitIndex = 0;
    
    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }
    
    return `${size.toFixed(1)} ${units[unitIndex]}`;
  }

  // Subscribe to the queue store
  const queue = $uploadManager.queue;

  // Derive queue items from the store value
  $: queueItems = Object.entries($queue).map(([id, item]) => ({ id, ...item }));
</script>

{#if Object.keys($queue).length > 0}
  <div class="w-full flex justify-center relative group">
    <!-- Icon with badge -->
    <div class="relative">
      <div
        class="p-2 rounded-full hover:bg-white/10 transition-colors flex justify-center items-center">
        <Upload size={16} class="text-white" />
      </div>

      <!-- Status badge -->
      <div
        class="absolute -top-1 -right-1 w-3 h-3 rounded-full
        {queueItems.some((item) => item.status === 'failed')
          ? 'bg-red-500'
          : queueItems.some((item) => ['uploading', 'processing', 'zipping'].includes(item.status))
            ? 'bg-yellow-500'
            : queueItems.every((item) => item.status === 'completed')
              ? 'bg-green-500'
              : 'bg-secondary-300'}">
      </div>
    </div>

    <!-- Expanded view on hover -->
    <div
      class="invisible group-hover:visible opacity-0 group-hover:opacity-100 transition-all duration-200 absolute bottom-0 left-full ml-2 z-50 w-64 bg-gray-800 rounded-lg shadow-lg">
      <div class="bg-gray-700 px-4 py-2 rounded-t-lg flex justify-between items-center">
        <div class="text-white font-medium flex gap-2">
          <span class="text-sm">Upload Manager</span>
          <span class="text-xs text-gray-300 self-center">
            ({queueItems.length})
          </span>
        </div>
      </div>
      <div class="max-h-60 overflow-y-auto">
        {#each queueItems.reverse() as item}
          {@const StatusIcon = getStatusIcon(item.status)}
          <div class="px-4 py-3 border-b border-gray-700 last:border-0">
            <div class="flex justify-between items-start mb-1">
              <div class="flex items-center gap-2">
                <StatusIcon size={16} class={getStatusColor(item.status)} />
                <span class="text-white text-sm truncate max-w-[150px]">{item.name}</span>
              </div>
              <button
                class="text-gray-400 hover:text-white self-center"
                onclick={() => $uploadManager.removeFromQueue(item.id)}>
                <X size={14} />
              </button>
            </div>

            <div class="text-xs text-gray-400 mb-1">{getStatusMessage(item)}</div>

            {#if item.progress !== undefined}
              <div class="w-full bg-gray-700 rounded-full h-1.5 mb-1">
                <div
                  class="h-1.5 rounded-full {item.status === 'completed'
                    ? 'bg-green-500'
                    : 'bg-secondary-300'}"
                  style="width: {item.progress}%">
                </div>
              </div>
              
              {#if item.status === 'uploading' && item.uploadedBytes !== undefined && item.totalBytes !== undefined}
                <div class="text-xs text-gray-400 mb-1">
                  {formatFileSize(item.uploadedBytes)} of {formatFileSize(item.totalBytes)}
                </div>
              {/if}
            {/if}
          </div>
        {/each}
      </div>
    </div>
  </div>
{/if}
