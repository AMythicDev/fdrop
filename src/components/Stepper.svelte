<script lang="ts">
  import { Indicator } from "flowbite-svelte";
  import { CheckCircleSolid } from "flowbite-svelte-icons";
  let { text, border = false, complete = false, active = false } = $props();
</script>

{#if complete}
  <div class="flex items-center">
    <Indicator size="xl" color="none" class="bg-blue-200">
      <CheckCircleSolid class="w-6 h-6 text-blue-400 dark:text-blue-300" />
    </Indicator>
    {#if border}
      <div class="flex w-full bg-blue-400 h-0.5"></div>
    {/if}
  </div>
  <div class="mt-3">
    <h3 class="font-medium text-gray-900 dark:text-white">{text}</h3>
  </div>
{:else}
  <div class="flex items-center">
    {#if active}
      <Indicator size="xl" class="bg-green-200">
        <Indicator class="w-5 h-5 bg-green-400 dark:text-gray-300" />
      </Indicator>
    {:else}
      <Indicator size="xl" class="bg-gray-200">
        <CheckCircleSolid class="w-6 h-6 text-gray-800 dark:text-gray-300" />
      </Indicator>
    {/if}
    {#if border}
      <div class="flex w-full bg-blue-400 h-0.5"></div>
    {/if}
  </div>
  <div class="mt-3">
    <h3 class="font-medium text-gray-900 dark:text-white">{text}</h3>
  </div>
{/if}
