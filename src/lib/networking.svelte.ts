import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { SvelteSet } from "svelte/reactivity";

export let available_devices = new SvelteSet<string>();

/* Attach listenerer for device events
 * NOTE: Always call this before `enable_networking` otherwise certain events may get lost
 */
export function listen_device_events() {
  listen<string>("device-discovered", (event) => {
    available_devices.add(event.payload);
  });
  listen<string>("device-removed", (event) => {
    available_devices.delete(event.payload);
  });
}

export function enable_networking() {
  invoke("enable_networking");
}

export async function get_available_devices(): Promise<string[]> {
  return await invoke("get_available_connections");
}

export async function refresh_available_devices() {
  const devices = await get_available_devices();
  devices.forEach((d) => available_devices.add(d));
}
