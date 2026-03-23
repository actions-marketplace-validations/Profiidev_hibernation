<script lang="ts">
  import { Button } from 'positron-components/components/ui/button';
  import { deleteCache } from '$lib/backend/cache.svelte.js';

  const { data } = $props();
</script>

<div class="flex-1">
  <h4 class="mb-2">Settings</h4>
  <Button onclick={() => deleteCache({ uuid: data.cacheInfo.uuid })}
    >Test</Button
  >
</div>
