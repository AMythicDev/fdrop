<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import Button from "flowbite-svelte/Button.svelte";
  import Helper from "flowbite-svelte/Helper.svelte";
  import Kbd from "flowbite-svelte/Kbd.svelte";
  import Input from "flowbite-svelte/Input.svelte";
  import InputAddon from "flowbite-svelte/InputAddon.svelte";
  import ButtonGroup from "flowbite-svelte/ButtonGroup.svelte";
  import Send from "../components/Send.svelte";
  import { type Transfer, Sender, TransferType } from "$lib/networking.svelte";

  let { selected } = $props();

  let chat_message: string = $state("");

  function send_message() {
    console.log(selected);
    if (chat_message.length == 0) return;
    invoke("send_text_message", {
      name: selected.name,
      contents: chat_message,
    });
    transfers.push({
      type: TransferType.TextMessage,
      display_content: chat_message,
      sentby: Sender.Local,
    });
  }

  let transfers: Transfer[] = $state([]);

  listen<Transfer>("transfer", (event) => {
    let transfer = event.payload;
    transfer.sentby = Sender.Peer;
    transfers.push(transfer);
  });
</script>

<div class="h-full p-2">
  <div
    class="flex flex-col gap-1 justify-end h-[calc(100vh-6rem)] overflow-y-scroll m-3"
  >
    {#each transfers as transfer}
      <div>
        <div
          class="{transfer.sentby == Sender.Local
            ? 'float-right bg-blue-400'
            : 'bg-green-400'} w-max text-white py-0.5 px-2.5 rounded-md"
        >
          {transfer.display_content}
        </div>
      </div>
    {/each}
  </div>
  <form class="h-16">
    <ButtonGroup class="w-full">
      <Input bind:value={chat_message} placeholder="Enter text to send" />
      <InputAddon class="bg-green-400 border-green-400">
        <Button
          type="submit"
          class="!bg-transparent outline-none border-none"
          onclick={send_message}
        >
          <Send class="h-6 fill-white" />
        </Button>
      </InputAddon>
    </ButtonGroup>
    <Helper>Press <Kbd>Ctrl</Kbd> + <Kbd>Enter</Kbd> to send</Helper>
  </form>
</div>
