<script lang="ts">
  import Button from "flowbite-svelte/Button.svelte";
  import Helper from "flowbite-svelte/Helper.svelte";
  import Kbd from "flowbite-svelte/Kbd.svelte";
  import Input from "flowbite-svelte/Input.svelte";
  import InputAddon from "flowbite-svelte/InputAddon.svelte";
  import ButtonGroup from "flowbite-svelte/ButtonGroup.svelte";
  import Send from "../components/Send.svelte";
  import { type Transfer, Sender } from "$lib/networking.svelte";

  let chat_message: string = $state("");

  function send_message() {
    console.log("send message");
    if (chat_message.length == 0) return;
    transfers.push({
      content: chat_message,
      sentby: Sender.Local,
    });
  }

  let transfers: Transfer[] = $state([]);
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
          {transfer.content}
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
