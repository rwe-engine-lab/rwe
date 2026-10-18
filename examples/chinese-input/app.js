const name = document.getElementById('name');
const preview = document.getElementById('preview');
const status = document.getElementById('status');
const show = () => { preview.textContent = '你好，' + (name.value || '朋友') + '！'; };
name.addEventListener('input', show);
name.addEventListener('change', () => { status.textContent = '已提交：' + name.value; });
name.addEventListener('compositionstart', () => { status.textContent = '输入法组合开始'; });
name.addEventListener('compositionupdate', event => { status.textContent = '正在组合：' + event.data; });
name.addEventListener('compositionend', event => { status.textContent = event.data ? '组合已确认：' + event.data : '组合已取消'; });
document.getElementById('clear').addEventListener('click', () => {
  name.value = ''; show(); status.textContent = '已清空'; name.focus();
});
