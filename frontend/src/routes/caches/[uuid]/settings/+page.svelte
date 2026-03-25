<script lang="ts">
  import { Button } from 'positron-components/components/ui/button';
  import { deleteCache, editCache } from '$lib/backend/cache.svelte.js';
  import BaseForm from 'positron-components/components/form/base-form.svelte';
  import { cacheSchema } from './schema.svelte.js';
  import FormInput from 'positron-components/components/form/form-input.svelte';
  import { Spinner } from 'positron-components/components/ui/spinner';
  import Save from '@lucide/svelte/icons/save';
  import FormSwitch from 'positron-components/components/form/form-switch.svelte';
  import type { FormValue } from 'positron-components/components/form/types';
  import { toast } from 'positron-components/components/util/general';
  import { RequestError } from 'positron-components/backend';

  const { data } = $props();

  let readonly = $derived(!data.cacheInfo.has_write_access);

  const onsubmit = async (form: FormValue<typeof cacheSchema>) => {
    let res = await editCache(data.cacheInfo.uuid, {
      ...data.cacheInfo,
      ...form
    });

    if (res) {
      if (res === RequestError.Conflict) {
        return { error: 'This cache name is already in use', field: 'name' };
      } else {
        return { error: 'Failed to update cache' };
      }
    } else {
      toast.success(`Cache ${data.cacheInfo.name} updated successfully`);
      return { error: '' };
    }
  };
</script>

<BaseForm
  schema={cacheSchema}
  {onsubmit}
  initialValue={data.cacheInfo}
  class="flex-1"
>
  {#snippet children({ props })}
    <div class="grid grid-cols-1 gap-4 lg:grid-cols-[1fr_auto_1fr]">
      <div>
        <FormInput
          {...props}
          key="name"
          label="Cache Name"
          placeholder="Enter cahce name"
          {readonly}
        />
        <FormInput
          {...props}
          key="priority"
          label="Priority"
          placeholder="Enter priority"
          type="number"
          {readonly}
        />
        <FormSwitch
          disabled={readonly || props.disabled}
          formData={props.formData}
          key="allow_force_push"
          label="Allow Force Push"
        />
      </div>
    </div>
  {/snippet}
  {#snippet footer({ isLoading }: { isLoading: boolean })}
    <div class="mt-4 grid w-full grid-cols-1 gap-8 lg:grid-cols-2">
      <Button
        class="ml-auto cursor-pointer"
        type="submit"
        disabled={isLoading || readonly}
      >
        {#if isLoading}
          <Spinner />
        {:else}
          <Save />
        {/if}
        Save Changes</Button
      >
    </div>
  {/snippet}
</BaseForm>
