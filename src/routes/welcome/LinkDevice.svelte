<script lang="ts">
  import { getActivePage } from "../../lib/welcome";
  import Buttons from "./Buttons.svelte";

  let { pageIndex, ref = $bindable() } = $props();
  let activePage = getActivePage();
</script>

<div
  bind:this={ref}
  class="flex flex-col gap-4 transition-transform {$activePage == pageIndex
    ? 'flex'
    : 'hidden'}"
>
  <span class="text-xl">Now let's get your devices paired</span>
  <span class="mx-auto text-xl"> List of devices </span>
  <div class="min-h-80 border-gray-300 border-2 rounded-lg"></div>
  <Buttons continueButtonText="Continue Without Linking" />
</div>
