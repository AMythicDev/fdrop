<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import Button from "flowbite-svelte/Button.svelte";
  import {
    available_devices,
    type ConnectionInfo,
  } from "$lib/networking.svelte";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { PaneGroup, Pane, PaneResizer } from "paneforge";
  import { onMount } from "svelte";

  let resizer: HTMLElement | undefined = $state();
  onMount(() => {
    resizer!.classList.add("w-1", "bg-gray-200");
  })

  const webview = getCurrentWebviewWindow();
  webview.listen<string>("device-linked", (event) => {
    let device = available_devices.get(event.payload);
    device!.linked = true;
    available_devices.set(event.payload, device!);
  });

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
        {#each linked_devices as device}
          <ul>
            <li class="border-b-2 border-gray-100 px-2 py-1">{device.name}</li>
          </ul>
        {:else}
          <div class="flex flex-col items-center justify-center h-full gap-2">
            <span>No Linked Devices</span>
            <span class="text-gray-400 text-center">
              Click the Link Device button to link a new device
            </span>
          </div>
        {/each}
        <Button
          class="!bg-blue-400 m-2"
          onclick={() => invoke("open_link_device_window", {})}
          >Link Device</Button
        >
      </div>
    </Pane>
    <PaneResizer bind:el={resizer} />
    <Pane defaultSize={65} minSize={40}>
      <div class="h-full">Hello World</div>
    </Pane>
  </PaneGroup>
</div>
