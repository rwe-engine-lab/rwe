document.getElementById('update').onclick = () => {
  const host = document.getElementById('content');
  host.innerHTML = '<p>动态段落一<p>动态段落二 &copy;<!--保留注释-->';
  document.getElementById('status').textContent = '已更新：' + host.children.length + ' 个段落；注释类型 ' + host.lastChild.lastChild.nodeType;
};
