<script lang="ts">
  import { type Transfer, Sender, TransferType } from "$lib/networking.svelte";
  import { filename } from "$lib/utils";
  let { transfer }: { transfer: Transfer } = $props();
  $inspect(transfer);
</script>

<div>
  <div
    class="{transfer.sentby == Sender.Local
      ? 'float-right bg-blue-400'
      : 'bg-green-400'} w-max text-white py-0.5 px-2.5 rounded-md"
  >
    {#if transfer.ttype == TransferType.PrepareFileTransfer}
      <div class="flex flex-col gap-2 py-2">
        {#each transfer.display_content.file_paths as file}
          <div
            class="h-14 {transfer.sentby == Sender.Local
              ? 'float-right bg-blue-300'
              : 'bg-green-300'} px-2"
          >
            {filename(file)}
          </div>
        {/each}
      </div>
      {transfer.display_content.assoc_text}
    {:else if transfer.ttype == TransferType.TextMessage}
      {transfer.display_content}
    {:else}
      Error
    {/if}
  </div>
</div>
