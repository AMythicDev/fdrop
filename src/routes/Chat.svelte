<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import Button from "flowbite-svelte/Button.svelte";
  import Helper from "flowbite-svelte/Helper.svelte";
  import Kbd from "flowbite-svelte/Kbd.svelte";
  import Input from "flowbite-svelte/Input.svelte";
  import InputAddon from "flowbite-svelte/InputAddon.svelte";
  import ButtonGroup from "flowbite-svelte/ButtonGroup.svelte";
  import Send from "$lib/icons/Send.svelte";
  import FileCirclePlusSolid from "flowbite-svelte-icons/FileCirclePlusSolid.svelte";
  import { type Transfer, Sender, TransferType } from "$lib/networking.svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { onMount, tick } from "svelte";

  let { selected } = $props();

  let chat_message: string = $state("");
  let file_selected: string[] = $state([]);
  let transfers: Transfer[] = $state([]);

  let transfers_list: HTMLElement | undefined = $state(undefined);

  onMount(() => {
    transfers_list!.scrollTop =
      transfers_list!.scrollHeight - transfers_list!.offsetHeight;
  });

  function send_message() {
    if (chat_message.length == 0) return;
    invoke("send_text_message", {
      cname: selected.name,
      contents: chat_message,
    });
    transfers.push({
      type: TransferType.TextMessage,
      display_content: chat_message,
      sentby: Sender.Local,
    });
    scroll_transfer_list();
  }

  function select_file() {
    open({
      multiple: true,
    }).then((selection: string[] | null) => {
      if (selection !== null) {
        file_selected = selection;
      }
    });
  }

  function scroll_transfer_list() {
    if (
      transfers_list!.scrollHeight -
        transfers_list!.offsetHeight -
        transfers_list!.scrollTop <
      1
    ) {
      tick().then(() => {
        // NOTE: .scrollIntoView() does not work for some reason so we resort to this method for scrolling
        transfers_list!.scrollTop =
          transfers_list!.scrollHeight - transfers_list!.offsetHeight;
      });
    }
  }

  listen<Transfer>("transfer", (event) => {
    let transfer = event.payload;
    transfer.sentby = Sender.Peer;
    transfers.push(transfer);
    scroll_transfer_list();
  });
</script>

<div class="h-full p-2 flex flex-col">
  <div
    class="flex flex-col gap-1 h-[calc(100vh-6rem)] overflow-y-scroll m-3"
    bind:this={transfers_list}
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
  <form class="h-max" onsubmit={() => (chat_message = "")}>
    {#if file_selected.length != 0}
      <ul class="flex gap-2 overflow-x-scroll">
        {#each file_selected as file}
          <li class="w-10 overflow-x-hidden">{file}</li>
        {/each}
      </ul>
    {/if}
    <ButtonGroup class="w-full">
      <Button
        class="!bg-gray-100 border-2 border-gray-200 w-10"
        onclick={select_file}
      >
        <FileCirclePlusSolid class="h-6 fill-gray-400" />
      </Button>
      <Input
        bind:value={chat_message}
        class="focus:border-2 focus:border-gray-200 focus:ring-transparent"
        placeholder="Enter text to send"
      />
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
