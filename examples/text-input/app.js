const name = document.getElementById('name');
const preview = document.getElementById('preview');
const status = document.getElementById('status');
const show = () => { preview.textContent = 'HELLO, ' + (name.value || 'STRANGER'); };
name.addEventListener('input', show);
name.addEventListener('change', () => { status.textContent = 'COMMITTED: ' + (name.value || '(EMPTY)'); });
document.getElementById('clear').addEventListener('click', () => {
  name.value = ''; show(); status.textContent = 'CLEARED'; name.focus();
});

name.addEventListener('compositionstart', () => { status.textContent = 'IME STARTED'; });
name.addEventListener('compositionupdate', event => { status.textContent = 'PREEDIT: ' + event.data; });
name.addEventListener('compositionend', event => {
  status.textContent = event.data ? 'IME COMMITTED: ' + event.data : 'IME CANCELED';
});
