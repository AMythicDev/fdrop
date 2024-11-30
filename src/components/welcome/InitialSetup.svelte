<script lang="ts">
  import { P, Input, Label } from "flowbite-svelte";
  import { getActivePage, type UserConfig } from "../../lib/welcome";
  import Buttons from "./Buttons.svelte";
  import { FolderOpenOutline } from "flowbite-svelte-icons";
  import { open } from "@tauri-apps/plugin-dialog";
  import { invoke } from "@tauri-apps/api/core";

  let activePage = getActivePage();

  let { pageIndex, details, ref = $bindable() } = $props();

  function openFolderSelector() {
    open({
      directory: true,
    }).then((selection: string | null) => {
      if (selection !== null && !Array.isArray(selection)) {
        details.fdrop_dir = selection;
      }
    });
  }
  function initial_setup() {
    invoke("initial_setup", { config: details }).catch((e) => console.log(e));
  }
</script>

<div
  bind:this={ref}
  class="flex-col gap-6 transition-transform {$activePage == pageIndex
    ? 'flex'
    : 'hidden'}"
>
  <P class="text-xl md:w-10/12 lg:w-1/2">
    Let's start by entering your full name and an alias for this device. This
    will be used to identify that's its you on other devices.
  </P>
  <div class="mb-5 flex flex-col gap-3">
    <Label class="w-96">
      <span>Your Name</span>
      <Input
        bind:value={details.user}
        onclick={(ev) => ev.target.select()}
        class="pl-2 outline-none border-2 border-gray-400 focus:border-blue-800 !bg-transparent"
      />
    </Label>
    <Label class="w-96">
      <span>Name of this device</span>
      <Input
        class="outline-none border-2 border-gray-400 focus:border-blue-800 !bg-transparent"
        onclick={(ev) => ev.target.select()}
        bind:value={details.instance_name}
      />
    </Label>
    <P class="text-xl md:w-10/12 lg:w-1/2">
      Next, choose a folder where FDrop will put your files. You can enter a
      path or use the folder selector icon to select a folder. Make sure that
      the folder you choose is completely empty.
    </P>
    <Label class="w-96">
      <span>Enter folder path</span>
      <Input
        class="outline-none border-2 !px-0 !pl-2.5 border-gray-400 focus:border-blue-800 !bg-transparent"
        onclick={(ev) => ev.target.select()}
        bind:value={details.fdrop_dir}
      >
        <button
          class="bg-blue-700 px-4 h-full rounded-r-lg translate-x-2.5"
          slot="right"
          onclick={openFolderSelector}
        >
          <FolderOpenOutline />
        </button>
      </Input>
    </Label>

    <Buttons exitButton extraContinueCallback={initial_setup} />
  </div>
</div>
