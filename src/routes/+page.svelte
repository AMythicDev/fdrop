<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import Button from "flowbite-svelte/Button.svelte";
  import {
    available_devices,
    type ConnectionInfo,
  } from "$lib/networking.svelte";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

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

<div class="grid grid-cols-[1fr,2fr] h-[100vh]">
  <div class="shadow-sm shadow-gray-700 p-2 flex flex-col justify-between">
    {#each linked_devices as device}
      <ul>
        <li>{device.name}</li>
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
      class="!bg-blue-400"
      onclick={() => invoke("open_link_device_window", {})}>Link Device</Button
    >
  </div>
  <div></div>
</div>
