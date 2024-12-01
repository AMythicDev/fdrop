import { getContext, setContext } from "svelte";
import { type Writable, writable } from "svelte/store";

export type UserConfig = {
  user: string;
  instance_name: string;
  fdrop_dir: string;
};

type Page = {
  text: String;
  elm?: HTMLElement;
};

export function getPages(): Page[] {
  return [
    { text: "Initial setup", elm: undefined },
    { text: "Key Generation", elm: undefined },
    { text: "Link Your Device", elm: undefined },
    { text: "Finish", elm: undefined },
  ];
}

export function createActivePage() {
  let pageIndex = writable<number>(0);
  setContext("activePage", pageIndex);
}

export function getActivePage(): Writable<number> {
  return getContext("activePage");
}

export function transitionPage(
  newActivePage: number,
  oldActivePage: number,
  pages: Page[],
) {
  if (newActivePage > oldActivePage) {
    continueAhead(oldActivePage, pages);
  } else {
    goBack(oldActivePage, pages);
  }
}

function continueAhead(activePage: number, pages: Page[]) {
  let currentPage = pages[activePage];
  let nextPage = pages[activePage + 1];

  if (currentPage.elm != undefined && nextPage.elm != undefined) {
    currentPage.elm.style.transform = "translateX(-70%)";
    nextPage.elm!.style.transform = "translateX(100%)";

    setTimeout(() => {
      nextPage.elm!.style.transform = "translateX(0)";
    }, 15);
  }
}

function goBack(activePage: number, pages: Page[]) {
  let currentPage = pages[activePage];
  let prevPage = pages[activePage - 1];
  if (currentPage.elm != undefined && prevPage.elm != undefined) {
    currentPage.elm.style.transform = "translateX(100%)";

    setTimeout(() => {
      activePage--;
    }, 150);
    setTimeout(() => {
      prevPage.elm!.style.transform = "translateX(0)";
    }, 155);
  }
}
