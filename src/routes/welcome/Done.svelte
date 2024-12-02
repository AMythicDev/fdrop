<script>
  import { getActivePage } from "../../lib/welcome";
  let { pageIndex, ref = $bindable() } = $props();
  let activePage = getActivePage();
  import { Button, Heading } from "flowbite-svelte";
  import CheckOutline from "flowbite-svelte-icons/CheckOutline.svelte";

  function go_home() {
    window.location.href = "/";
  }
</script>

<div
  bind:this={ref}
  class="flex-col items-center gap-4 transition-transform {$activePage ==
  pageIndex
    ? 'flex'
    : 'hidden'}"
>
  <CheckOutline class="h-36 w-36 text-gray-300" />
  <Heading class="text-4xl text-gray-300 text-center"
    >You are Ready to Go</Heading
  >
  <Button class="bg-blue-400 w-72" onclick={go_home}>Go Home</Button>
</div>
