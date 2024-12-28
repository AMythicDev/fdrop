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
  import Tooltip from "flowbite-svelte/Tooltip.svelte";
  import {
    type Transfer,
    Sender,
    TransferType,
    transferTypeFromString,
  } from "$lib/networking.svelte";
  import { filename } from "$lib/utils";
  import { open } from "@tauri-apps/plugin-dialog";
  import { onMount, tick } from "svelte";
  import { SvelteSet } from "svelte/reactivity";
  import TransferDisplay from "./TransferDisplay.svelte";

  let { selected } = $props();

  let chat_message: string = $state("");
  let file_selected = new SvelteSet<string>();
  let transfers: Transfer[] = $state([]);

  let transfers_list: HTMLElement | undefined = $state(undefined);

  onMount(() => {
    transfers_list!.scrollTop =
      transfers_list!.scrollHeight - transfers_list!.offsetHeight;
  });

  function send_message() {
    if (file_selected.size == 0 && chat_message.length == 0) return;

    if (file_selected.size > 0) {
      let file_selected_arr = Array.from(file_selected);
      for (const file of file_selected_arr) {
        transfers.push({
          ttype: TransferType.PrepareFileTransfer,
          display_content: {
            assoc_text: chat_message != "" ? chat_message : null,
            file_path: file,
          },
          sentby: Sender.Local,
        });
      }
      invoke("send_files", {
        cname: selected.name,
        assocText: chat_message,
        filePaths: Array.from(file_selected),
      });
      scroll_transfer_list();
      return;
    }

    invoke("send_text_message", {
      cname: selected.name,
      contents: chat_message,
    });
    transfers.push({
      ttype: TransferType.TextMessage,
      display_content: chat_message,
      sentby: Sender.Local,
    });
    scroll_transfer_list();
  }

  function select_file(clear: boolean) {
    open({
      multiple: true,
    }).then((selection: string[] | null) => {
      if (selection !== null) {
        if (clear) {
          file_selected.clear();
        }
        for (const sel of selection) {
          file_selected.add(sel);
        }
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
    if (typeof transfer.ttype == "string")
      transfer.ttype = transferTypeFromString(transfer.ttype);
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
      <TransferDisplay {transfer} />
    {/each}
  </div>
  <form
    class="h-max"
    onsubmit={() => {
      chat_message = "";
      file_selected.clear();
    }}
  >
    {#if file_selected.size != 0}
      <div class="flex bg-gray-100">
        <div class="flex gap-2 overflow-x-scroll">
          {#each file_selected as file}
            <Button
              class="w-14 bg-transparent text-black overflow-x-hidden !border-r-4 !border-r-gray-200 !ring-transparent"
              >{filename(file)}</Button
            >
            <Tooltip>{file}</Tooltip>
          {/each}
        </div>
        <Button
          class="w-10 bg-transparent text-black overflow-x-hidde !ring-transparent"
          onclick={() => select_file(false)}
        >
          <FileCirclePlusSolid class="fill-gray-400" />
        </Button>
      </div>
    {/if}
    <ButtonGroup class="w-full">
      <Button
        class="!bg-gray-100 border-2 border-gray-200 w-10"
        onclick={() => select_file(true)}
      >
        <FileCirclePlusSolid class="h-6 fill-gray-400" />
      </Button>
      <Input
        bind:value={chat_message}
        class="focus:border-2 focus:border-gray-200 focus:ring-transparent"
        placeholder="Enter text to send"
      />
      <InputAddon class="bg-green-400 border-green-400 w-16 p-0">
        <Button
          type="submit"
          class="!bg-transparent outline-none border-none"
          onclick={send_message}
        >
          <Send class="fill-white w-6" />
        </Button>
      </InputAddon>
    </ButtonGroup>
    <Helper>Press <Kbd>Ctrl</Kbd> + <Kbd>Enter</Kbd> to send</Helper>
  </form>
</div>
