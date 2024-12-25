<script lang="ts">
  import Linux from "$lib/icons/Linux.svelte";
  import Windows from "$lib/icons/Windows.svelte";
  import Macos from "$lib/icons/Macos.svelte";
  import { type ConnectionInfo, realname } from "$lib/networking.svelte";

  let {
    linked_devices,
    selected = $bindable(),
  }: {
    linked_devices: ConnectionInfo[];
    selected: ConnectionInfo | undefined;
  } = $props();

  let selected_elm: EventTarget | undefined = $state(undefined);
  let prev_selected_elm: EventTarget | undefined = $state(undefined);
  $effect(() => {
    if (selected_elm == undefined) return;
    let classes = ["bg-blue-400", "text-white"];
    if (selected_elm != undefined && prev_selected_elm == undefined)
      prev_selected_elm = selected_elm;
    prev_selected_elm!.classList.remove(...classes);
    selected_elm!.classList.add(...classes);
    prev_selected_elm = selected_elm;
  });
</script>

<ul class="overflow-y-scroll">
  {#each linked_devices as device}
    <li class="border-b-2 border-gray-100 h-16">
      <button
        class="h-full w-full px-3 py-1 text-left"
        onclick={(e) => {
          selected_elm = e.target!;
          selected = device;
        }}
      >
        {#if device.platform == "linux"}
          <Linux class="h-12 w-12 inline" />
        {:else if device.platform == "windows"}
          <Windows class="h-12 w-12 inline" />
        {:else if device.platform == "macos"}
          <Macos class="h-12 w-12 inline fill-gray-300" />
          <!-- 
            TODO: Add ios and android icons here
          -->
        {/if}
        {realname(device)}
      </button>
    </li>
  {/each}
</ul>
