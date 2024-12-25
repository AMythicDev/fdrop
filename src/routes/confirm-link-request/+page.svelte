<script lang="ts">
  import Button from "flowbite-svelte/Button.svelte";
  import ButtonGroup from "flowbite-svelte/ButtonGroup.svelte";
  import Link from "$lib/icons//link.svelte";
  import CheckOutline from "flowbite-svelte-icons/CheckOutline.svelte";
  import CloseOutline from "flowbite-svelte-icons/CloseOutline.svelte";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

  const webview = getCurrentWebviewWindow();
  let device_name = localStorage.getItem("device-name");

  function accept() {
    webview.emitTo(webview.label, "link-response", "accepted");
    webview.close()
  }
  function reject() {
    webview.emitTo(webview.label, "link-response", "rejected");
    webview.close()
  }
</script>

<div class="flex px-3 pt-4">
  <Link class="fill-none stroke-blue-300 w-32 mt-5" />
  <div class="flex flex-col gap-5">
    <p>
      Incoming Link Request from <span class="font-semibold">{device_name}</span
      >
    </p>
    <p>
      Accept this request if you trust the device and would like to link it to
      this device
    </p>
    <ButtonGroup class="shadow-none flex gap-1 justify-end">
      <Button class="!bg-red-400 text-white" onclick={reject}><CloseOutline />Reject</Button>
      <Button class="!bg-green-400 text-white" onclick={accept}><CheckOutline /> Accept</Button>
    </ButtonGroup>
  </div>
</div>
