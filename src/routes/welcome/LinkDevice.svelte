<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getActivePage } from "../../lib/welcome";
  import { listen } from "@tauri-apps/api/event";
  import Buttons from "./Buttons.svelte";
  import Listgroup from "flowbite-svelte/Listgroup.svelte";
  import Spinner from "flowbite-svelte/Spinner.svelte";
  import Circle from "../../components/Circle.svelte";
  import { Button } from "flowbite-svelte";
  import { SvelteSet } from "svelte/reactivity";

  let { pageIndex, ref = $bindable() } = $props();
  let activePage = getActivePage();

  let devices = new SvelteSet<string>();
  let devices_linking = new SvelteSet<string>();

  listen<string>("device-discovered", (event) => {
    devices.add(event.payload);
  });
  listen<string>("device-removed", (event) => {
    devices.delete(event.payload);
  });

  async function link_device(device: string) {
    let link_resp = await invoke("link_device_by_name", { name: device });
    if (link_resp == "accepted") {
      return;
    } else if (link_resp == "rejected") {
      devices_linking.delete(device)
    }
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
        {#if devices_linking.has(item)}
          {#await link_device(item)}
            <Spinner currentFill="#31c48d" currentColor="#d1d5db" />
          {:then}
            <div>
              <Circle class="fill-green-400 h-2 w-2 inline mr-1" /><span
                class="text-gray-400">Active</span
              >
            </div>
          {/await}
        {:else}
          <Button class="bg-blue-400" onclick={() => devices_linking.add(item)}
            >Link</Button
          >
        {/if}
      {/if}
    </div>
  </Listgroup>
  <Buttons continueButtonText="Continue Without Linking" />
</div>
