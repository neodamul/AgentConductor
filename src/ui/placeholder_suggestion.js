// Opt-in suggestions can become user input; examples and instructional hints keep native behavior.
(() => {
  const selector = 'textarea[data-placeholder-suggestion],input[data-placeholder-suggestion]';
  document.addEventListener('keydown', event => {
    const field = event.target;
    if (!field.matches?.(selector) || field.readOnly || field.disabled || !field.placeholder
        || field.hasAttribute('data-placeholder-dismissed') || event.isComposing || event.keyCode === 229
        || event.altKey || event.ctrlKey || event.metaKey || event.shiftKey
        || ['Alt', 'Control', 'Meta', 'Shift'].includes(event.key)) return;
    if (!field.value && ['Tab', 'ArrowRight'].includes(event.key)) {
      event.preventDefault();
      field.value = field.placeholder;
      field.setSelectionRange(field.value.length, field.value.length);
      field.dispatchEvent(new Event('input', { bubbles: true }));
    } else if (!['Tab', 'ArrowRight'].includes(event.key)) {
      field.setAttribute('data-placeholder-dismissed', '');
    }
  }, true);
  document.addEventListener('focusout', event => {
    if (event.target.matches?.(selector)) event.target.removeAttribute('data-placeholder-dismissed');
  });
})();
