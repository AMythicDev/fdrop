<script lang="ts">
    import { Indicator } from "flowbite-svelte";
    import { CheckCircleSolid } from "flowbite-svelte-icons";
    let { text, border = false, complete = false, active = false } = $props();
</script>

{#if complete}
    <div class="flex items-center">
        <Indicator
            size="xl"
            color="none"
            class="bg-blue-200 dark:bg-blue-500 z-10 ring-0 ring-white sm:ring-8 dark:ring-gray-900 shrink-0"
        >
            <CheckCircleSolid
                class="w-6 h-6 text-blue-600 dark:text-blue-300"
            />
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
        <Indicator
            size="xl"
            color="gray"
            class="dark:bg-gray-700 z-10 ring-0 ring-white sm:ring-8 dark:ring-gray-900 shrink-0"
        >
            {#if active}
                <Indicator class="w-5 h-5 bg-green-500 dark:text-gray-300" />
            {:else}
                <CheckCircleSolid
                    class="w-6 h-6 text-gray-800 dark:text-gray-300"
                />
            {/if}
        </Indicator>
        {#if border}
            <div class="flex w-full bg-blue-400 h-0.5"></div>
        {/if}
    </div>
    <div class="mt-3">
        <h3 class="font-medium text-gray-900 dark:text-white">{text}</h3>
    </div>
{/if}
