const {createApp, h, ref} = Vue;
createApp({ setup() { const count = ref(0); return () => h('section', null, [
  h('button', {id:'increment', onClick:() => count.value++}, 'COUNT: ' + count.value),
  h('p', {id:'status'}, count.value === 0 ? 'CLICK THE BUTTON' : 'VUE REACTIVITY WORKS'),
]); }}).mount('#app');
