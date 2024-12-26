<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import Button from "flowbite-svelte/Button.svelte";
  import Chat from "./Chat.svelte";
  import DeviceList from "./DeviceList.svelte";

  import {
    available_devices,
    type ConnectionInfo,
  } from "$lib/networking.svelte";
  import { PaneGroup, Pane, PaneResizer } from "paneforge";
  import { onMount } from "svelte";

  let resizer: HTMLElement | undefined = $state();
  onMount(() => {
    resizer!.classList.add("w-1", "bg-gray-200");
  });

  let selected: ConnectionInfo | undefined = $state(undefined);

  let linked_devices = $derived.by(() => {
    let linked_devices: ConnectionInfo[] = [];
    for (const d of available_devices.values()) {
      if (d.linked) {
        linked_devices.push(d);
      }
    }
    return linked_devices;
  });
</script>

<div class="h-[100vh]">
  <PaneGroup direction="horizontal">
    <Pane defaultSize={35} minSize={20}>
      <div class="flex flex-col justify-between h-full">
        {#if linked_devices.length > 0}
          <DeviceList {linked_devices} bind:selected />
        {:else}
          <div class="flex flex-col items-center justify-center h-full gap-2">
            <span>No Linked Devices</span>
            <span class="text-gray-400 text-center">
              Click the Link Device button to link a new device
            </span>
          </div>
        {/if}
        <Button
          class="!bg-blue-400 m-2"
          onclick={() => invoke("open_link_device_window", {})}
          >Link Device</Button
        >
      </div>
    </Pane>
    <PaneResizer bind:el={resizer} />
    <Pane defaultSize={65} minSize={40}>
      {#if selected}
        <Chat {selected} />
      {:else}
        <div class="h-full flex flex-col justify-center items-center gap-2">
          <span>No devices selected</span>
          <span class="text-gray-400"
            >Select a device from the left panel to transfer content</span
          >
        </div>
      {/if}
    </Pane>
  </PaneGroup>
</div>
