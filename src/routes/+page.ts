import { invoke } from "@tauri-apps/api/core";

export function load(params: any) {
  invoke("check_first_launch", {}).then((res) => {
    if (res) {
      window.location.href = "/welcome";
    } else {
      invoke("enable_networking");
    }
  });
}
