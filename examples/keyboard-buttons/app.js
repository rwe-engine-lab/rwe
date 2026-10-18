let count = 0;
const show = () => { document.getElementById('count').textContent = 'COUNT: ' + count; };
document.getElementById('increment').addEventListener('click', () => { count++; show(); });
document.getElementById('decrement').addEventListener('click', () => { count--; show(); });
document.getElementById('reset').addEventListener('click', () => { count = 0; show(); });
document.addEventListener('focusin', event => {
  document.getElementById('focus').textContent = 'FOCUS: ' + event.target.id.toUpperCase();
});
document.addEventListener('focusout', () => {
  document.getElementById('focus').textContent = 'FOCUS: NONE';
});
document.addEventListener('keydown', event => {
  if (event.key.toLowerCase() === 'r' && !event.ctrlKey && !event.altKey && !event.metaKey) {
    event.preventDefault(); count = 0; show();
  }
});
