<script lang="ts">
  import { P } from "flowbite-svelte";
  import { getActivePage } from "../../lib/welcome";
  import Buttons from "./Buttons.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { Button } from "flowbite-svelte";
  import CheckOutline from "flowbite-svelte-icons/CheckOutline.svelte";

  let { pageIndex, ref = $bindable() } = $props();

  let activePage = getActivePage();

  let key_generation_complete = $state(false);
  function generate_keys() {
    invoke("generate_keys", {});
    key_generation_complete = true;
  }
</script>

<div
  bind:this={ref}
  class="flex flex-col gap-6 transition-transform {$activePage == pageIndex
    ? 'flex'
    : 'hidden'}"
>
  <P class="text-xl md:w-10/12 lg:w-1/2">
    A key pair is required to send data securely across the network and protects
    it from malicious actors from reading or tampering it. Generate a new key
    pair for this setup by click on button below.
  </P>
  {#if key_generation_complete}
    <P
      ><CheckOutline class="inline text-green-400" /> Key generated successfully</P
    >
  {:else}
    <Button class="!bg-green-400 w-48" on:click={generate_keys}
      >Generate Keys</Button
    >
  {/if}
  <Buttons continueDisabled={!key_generation_complete} />
</div>
