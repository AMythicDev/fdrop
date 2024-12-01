<script lang="ts">
  import { exit } from "@tauri-apps/plugin-process";

  import { ButtonGroup, Button } from "flowbite-svelte";
  import {
    CloseOutline,
    ArrowRightOutline,
    ArrowLeftOutline,
  } from "flowbite-svelte-icons";
  import { getActivePage } from "../../lib/welcome";

  let activePageStore = getActivePage();

  let activePage: number;
  getActivePage().subscribe((value) => (activePage = value));

  let {
    extraContinueCallback = undefined,
    exitButton = false,
    continueDisabled = false,
    continueButtonText = "Continue",
  } = $props();

  function continueCallback() {
    if (extraContinueCallback != undefined) {
      extraContinueCallback(activePage! + 1, activePage!);
    }
    activePageStore.set(activePage + 1);
  }
</script>

<ButtonGroup class="flex justify-between w-96 shadow-none">
  {#if exitButton}
    <Button class="!bg-red-400 px-6 py-2.5" onclick={() => exit(0)}>
      <CloseOutline color="white" /><span class="ml-2 text-white">Close</span
      ></Button
    >
  {:else}
    <Button
      class="!bg-gray-400 px-6 py-2.5"
      onclick={() => activePageStore.set(activePage - 1)}
    >
      <ArrowLeftOutline color="white" /><span class="ml-2 text-white">Back</span
      ></Button
    >
  {/if}
  <Button
    class="!bg-blue-400 px-6 py-2.5"
    disabled={continueDisabled}
    onclick={continueCallback}
    ><span class="mr-2 text-white">{continueButtonText}</span>
    <ArrowRightOutline color="white" />
  </Button>
</ButtonGroup>
