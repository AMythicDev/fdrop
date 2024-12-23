<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import Button from "flowbite-svelte/Button.svelte";
  import {
    available_devices,
    realname,
    type ConnectionInfo,
  } from "$lib/networking.svelte";
  import { PaneGroup, Pane, PaneResizer } from "paneforge";
  import { onMount } from "svelte";

  let resizer: HTMLElement | undefined = $state();
  onMount(() => {
    resizer!.classList.add("w-1", "bg-gray-200");
  });

  let selected: EventTarget | undefined = $state(undefined);
  let prev_selected: EventTarget | undefined = $state(undefined);

  $effect(() => {
    if (selected == undefined) return;
    let classes = ["bg-blue-400", "text-white"];
    if (selected != undefined && prev_selected == undefined) prev_selected = selected;
    prev_selected!.classList.remove(...classes);
    selected!.classList.add(...classes);
    prev_selected = selected;
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
        {#if linked_devices.length > 0}
          <ul class="overflow-y-scroll">
            {#each linked_devices as device}
              <li class="border-b-2 border-gray-100 h-16">
                <button
                  class="h-full w-full px-3 py-1 text-left"
                  onclick={(e) => (selected = e.target!)}
                >
                  {realname(device)}
                </button>
              </li>
            {/each}
          </ul>
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
      <div class="h-full">Hello World</div>
    </Pane>
  </PaneGroup>
</div>
