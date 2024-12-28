import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { SvelteMap } from "svelte/reactivity";

export type ConnectionInfo = {
  name: string,
  linked: boolean
  platform?: string,
}

export enum Sender {
  Local,
  Peer,
}

export enum TransferType {
  TextMessage,
  PrepareFileTransfer,
}

export function transferTypeFromString(s: string): TransferType {
  switch (s) {
    case "TextMessage":
      return TransferType.TextMessage;
    case "PrepareFileTransfer":
      return TransferType.PrepareFileTransfer;
  }
}

export type DisplayFileTransfer = {
  assoc_text?: string | null,
  file_path: string[]
}

export type Transfer = {
  ttype: TransferType,
  display_content: string | DisplayFileTransfer,
  sentby: Sender | undefined,
}

export function realname(conn: ConnectionInfo): string {
  let name_end = conn.name.indexOf("._fdrop");
  if (name_end == -1) name_end = conn.name.length;
  return conn.name.slice(0, name_end);
}

export let available_devices = new SvelteMap<string, ConnectionInfo>();

/* Attach listenerer for device events
 * NOTE: Always call this before `enable_networking` otherwise certain events may get lost
 */
export function listen_device_events() {
  listen<ConnectionInfo>("device-discovered", (event) => {
    const device = $state(event.payload);
    available_devices.set(event.payload.name, device);
  });
  listen<ConnectionInfo>("device-removed", (event) => {
    available_devices.delete(event.payload.name);
  });
  listen<ConnectionInfo>("device-linked", (event) => {
    let device = available_devices.get(event.payload.name);
    device!.linked = true;
    device!.platform = event.payload.platform;
    available_devices.set(event.payload.name, device!);
  });
}

export function enable_networking() {
  invoke("enable_networking");
}

export async function get_available_devices(): Promise<ConnectionInfo[]> {
  return JSON.parse(await invoke("get_available_connections"));
}

export async function refresh_available_devices() {
  const devices = await get_available_devices();
  console.log(typeof devices)
  devices.forEach((d) => available_devices.set(d.name, d));
}
