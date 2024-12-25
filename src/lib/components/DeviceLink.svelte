<script lang="ts">
  import { Button } from "flowbite-svelte";
  import { SvelteSet } from "svelte/reactivity";
  import Spinner from "flowbite-svelte/Spinner.svelte";
  import Listgroup from "flowbite-svelte/Listgroup.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import Circle from "$lib/icons/Circle.svelte";
  import { available_devices, realname } from "$lib/networking.svelte";
  import { emitTo, listen } from "@tauri-apps/api/event";

  let link_devices = new SvelteSet<string>();

  listen<string>("device-linked", (event) => {
    link_devices.add(event.payload);
  });

  async function link_device(name: string) {
    let link_resp = await invoke("link_device_by_name", { name: name });
    if (link_resp == "accepted") {
      await emitTo("main", "device-linked", name);
      // TODO: handle this
      // any_device_linked = true;
      return;
    } else if (link_resp == "rejected") {
      link_devices.delete(name);
    }
  }
</script>

<span class="mx-auto text-xl">List of devices</span>

<Listgroup
  active
  items={Array.from(available_devices.values())}
  let:item
  class="min-h-80"
>
  <div class="w-full flex justify-between text-black">
    {#if item}
      {realname(item)}
      {#if link_devices.has(item.name)}
        {#await link_device(item.name)}
          <Spinner currentFill="#31c48d" currentColor="#d1d5db" />
        {:then}
          <div>
            <Circle class="fill-green-400 h-2 w-2 inline mr-1" /><span
              class="text-gray-400">Active</span
            >
          </div>
        {/await}
      {:else}
        <Button class="bg-blue-400" onclick={() => link_devices.add(item.name)}
          >Link</Button
        >
      {/if}
    {/if}
  </div>
</Listgroup>
