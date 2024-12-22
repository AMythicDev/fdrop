import { invoke } from "@tauri-apps/api/core";
import { listen_device_events, enable_networking } from "$lib/networking.svelte";

export function load(params: any) {
  invoke("check_first_launch", {}).then((res) => {
    if (res) {
      window.location.href = "/welcome";
    } else {
      listen_device_events();
      enable_networking();
    }
  });
}
