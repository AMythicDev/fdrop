<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getActivePage } from "../../lib/welcome";
  import { listen } from "@tauri-apps/api/event";
  import Buttons from "./Buttons.svelte";
  import Listgroup from "flowbite-svelte/Listgroup.svelte";
  import { Button } from "flowbite-svelte";
  import { SvelteSet } from "svelte/reactivity";

  let { pageIndex, ref = $bindable() } = $props();
  let activePage = getActivePage();

  let devices = new SvelteSet<string>();

  listen<string>("device-discovered", (event) => {
    devices.add(event.payload);
  });

  function link_device(device: string) {
    invoke("link_device_by_name", { name: device });
  }
</script>

<div
  bind:this={ref}
  class="flex flex-col gap-4 transition-transform {$activePage == pageIndex
    ? 'flex'
    : 'hidden'}"
>
  <span class="text-xl">Now let's get your devices paired</span>
  <span class="mx-auto text-xl">List of devices</span>
  <Listgroup active items={Array.from(devices)} let:item class="min-h-80">
    <div class="w-full flex justify-between text-black">
      {item}
      {#if item}
        <Button class="bg-blue-400" onclick={() => link_device(item)}
          >Link</Button
        >
      {/if}
    </div>
  </Listgroup>
  <Buttons continueButtonText="Continue Without Linking" />
</div>
