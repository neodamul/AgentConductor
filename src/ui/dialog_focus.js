// #546: a dialog opened by a click must not show a focus ring on the control the browser focuses first (the confirmation
// dialogs focus Cancel on purpose, so the focus stays where it is). Rings stay hidden until the first key press inside the dialog.
(() => {
  const open = HTMLDialogElement.prototype.showModal;
  HTMLDialogElement.prototype.showModal = function () {
    open.apply(this, arguments);
    this.classList.add('quiet-focus');
    const loud = () => this.classList.remove('quiet-focus');
    this.addEventListener('keydown', loud, { once: true, capture: true });
    this.addEventListener('close', loud, { once: true });
  };
})();
