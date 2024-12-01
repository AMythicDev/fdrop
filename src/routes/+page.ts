import { invoke } from "@tauri-apps/api/core";
import type { PageData } from "./$types";

export function load(params: any) {
  invoke("check_first_launch", {}).then((res) => {
    if (!res) {
      window.location.href = "/welcome";
    } else {
      invoke("launch_discovery_service");
    }
  });
}
