<script lang="ts">
  import DeviceLink from "$lib/components/DeviceLink.svelte";
  import {
    refresh_available_devices,
    listen_device_events,
  } from "$lib/networking.svelte";

  listen_device_events();
  refresh_available_devices();
</script>

<div class="flex flex-col gap-4 p-3">
  <DeviceLink />
</div>
