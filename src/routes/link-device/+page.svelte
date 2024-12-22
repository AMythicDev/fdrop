<script lang="ts">
  import DeviceLink from "../../components/DeviceLink.svelte";
  import { refresh_available_devices } from "$lib/networking.svelte";

  refresh_available_devices();
</script>

<div class="flex flex-col gap-4 p-3">
  <DeviceLink />
</div>
