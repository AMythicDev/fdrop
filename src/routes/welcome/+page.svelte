<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import {
    type UserConfig,
    getActivePage,
    createActivePage,
    getPages,
    transitionPage,
  } from "$lib/welcome";
  import { Heading, P } from "flowbite-svelte";

  import Stepper from "../../components/Stepper.svelte";
  import InitialSetup from "./InitialSetup.svelte";
  import KeyGeneration from "./KeyGeneration.svelte";
  import LinkDevice from "./LinkDevice.svelte";
  import Done from "./Done.svelte";

  createActivePage();

  let pages = getPages();

  let activePage: number = $state(0);

  getActivePage().subscribe((value: any) => {
    transitionPage(value, activePage!, pages);
    if (value == 2) {
      invoke("enable_networking");
    }
    activePage = value;
  });

  let details: UserConfig = $state({
    user: "",
    instance_name: "",
    fdrop_dir: "",
  });

  function get_device_details() {
    invoke("get_device_details", {}).then((d: any) => {
      details = d;
    });
  }

  get_device_details();
</script>

<div class="min-h-screen p-16">
  <div class="flex flex-col gap-6">
    <Heading color="text-blue-400" class="text-8xl">FDrop</Heading>
    <P class="text-4xl">Looks like you are new here</P>
    <ol class="flex items-center">
      {#each pages as page, i}
        <li class="relative w-full mb-6">
          <Stepper
            text={page.text}
            border={i + 1 < pages.length}
            complete={activePage > i}
            active={activePage == i}
          />
        </li>
      {/each}
    </ol>
  </div>

  <InitialSetup pageIndex={0} {details} bind:ref={pages[0].elm} />
  <KeyGeneration pageIndex={1} bind:ref={pages[1].elm} />
  <LinkDevice pageIndex={2} bind:ref={pages[2].elm} />
  <Done pageIndex={3} bind:ref={pages[3].elm} />
</div>
