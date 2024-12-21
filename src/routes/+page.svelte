<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import Button from "flowbite-svelte/Button.svelte";
  import { SvelteSet } from "svelte/reactivity";

  let linked_devices = new SvelteSet<string>();
</script>

<div class="grid grid-cols-[1fr,2fr] h-[100vh]">
  <div class="shadow-sm shadow-gray-700 p-2 flex flex-col justify-between">
    {#if linked_devices.size == 0}
      <div class="flex flex-col items-center justify-center h-full gap-2">
        <span>No Linked Devices</span>
        <span class="text-gray-400 text-center">
          Click the Link Device button to link a new device
        </span>
      </div>
    {/if}
    <Button class="!bg-blue-400" onclick={() => invoke("open_link_device_window", {})}>Link Device</Button>
  </div>
  <div></div>
</div>

