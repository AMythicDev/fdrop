<script lang="ts">
  import { getActivePage } from "../../lib/welcome";
  import { listen } from "@tauri-apps/api/event";
  import Buttons from "./Buttons.svelte";

  let { pageIndex, ref = $bindable() } = $props();
  let activePage = getActivePage();

  let devices: string[] = $state([]);

  listen<string>("device-discovered", (event) => {
    devices.push(event.payload);
  });
</script>

<div
  bind:this={ref}
  class="flex flex-col gap-4 transition-transform {$activePage == pageIndex
    ? 'flex'
    : 'hidden'}"
>
  <span class="text-xl">Now let's get your devices paired</span>
  <span class="mx-auto text-xl"> List of devices </span>
  <div class="min-h-80 border-gray-300 border-2 rounded-lg">
    {#each devices as device}
      <div>{device}</div>
    {/each}
  </div>
  <Buttons continueButtonText="Continue Without Linking" />
</div>
