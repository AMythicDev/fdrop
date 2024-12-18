<script lang="ts">
  import Button from "flowbite-svelte/Button.svelte";
  import CheckOutline from "flowbite-svelte-icons/CheckOutline.svelte";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import CloseOutline from "flowbite-svelte-icons/CloseOutline.svelte";
  import ButtonGroup from "flowbite-svelte/ButtonGroup.svelte";

  const webview = getCurrentWebviewWindow();
  let device_name = localStorage.getItem("device-name");
</script>

<div class="flex px-3 pt-4">
  <CloseOutline class="text-red-400 h-32 w-32" />
  <div class="flex flex-col gap-2">
    <p>
      <span class="font-semibold">{device_name}</span> rejected the link request
    </p>
    <p>
      You can try again by clicking the <span class="font-semibold">Link</span> button
      if it got cancelled by accident
    </p>

    <ButtonGroup class="flex justify-end shadow-none">
    <Button class="!bg-blue-400 text-white w-20" onclick={webview.close}
      ><CheckOutline /> Ok</Button
    >
    </ButtonGroup>
  </div>
</div>
