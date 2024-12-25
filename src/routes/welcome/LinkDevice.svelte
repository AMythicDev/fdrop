<script lang="ts">
  import { getActivePage } from "../../lib/welcome";
  import Buttons from "./Buttons.svelte";
  import DeviceLink from "$lib/components/DeviceLink.svelte";

  let { pageIndex, ref = $bindable() } = $props();
  let activePage = getActivePage();

  let any_device_linked = $state(false);
</script>

<div
  bind:this={ref}
  class="flex flex-col gap-4 transition-transform {$activePage == pageIndex
    ? 'flex'
    : 'hidden'}"
>
  <span class="text-xl">Now let's get your devices paired</span>

  <DeviceLink />
  {#if any_device_linked}
    <Buttons />
  {:else}
    <Buttons continueButtonText="Continue Without Linking" />
  {/if}
</div>
