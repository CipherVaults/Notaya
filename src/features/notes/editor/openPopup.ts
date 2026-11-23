export const openRadixPopup = '[data-radix-popper-content-wrapper] > [data-state="open"]';
export const openBlockMenu = '[data-block-menu="open"]';
export const openEditorPopup = [
  openRadixPopup,
  openBlockMenu,
  '[data-slash-menu="open"]',
  "[data-tippy-root]",
  '[role="menu"]:not([data-state="closed"])',
  '[role="listbox"]',
].join(", ");
