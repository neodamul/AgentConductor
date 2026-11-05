// #546: a dialog opened by a click must not show a focus ring on the button or check box the browser focuses first.
// Text fields keep their focus so typing works; the dialog itself takes the focus otherwise and Tab still reaches every control.
(() => {
  const open = HTMLDialogElement.prototype.showModal;
  HTMLDialogElement.prototype.showModal = function () {
    open.apply(this, arguments);
    const active = document.activeElement;
    if (active && active !== this && this.contains(active) && active.matches('button,input[type=checkbox],input[type=radio],summary,a[href]')) {
      this.tabIndex = -1;
      this.focus({ preventScroll: true });
    }
  };
})();
