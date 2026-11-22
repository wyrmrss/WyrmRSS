import type { ReadMode } from "../types/ReadMode";

/** On open and on scroll both mark a post read when it's opened, after the read delay. */
export const marksReadOnOpen = (mode: ReadMode | undefined): boolean =>
  mode === "on_open" || mode === "on_scroll";
