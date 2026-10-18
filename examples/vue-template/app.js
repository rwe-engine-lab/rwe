const {createApp, ref} = Vue;
createApp({
  setup() { return {count:ref(0)}; },
  template: `<section><button id="increment" @click="count++">COUNT: {{count}}</button><p id="status">{{count === 0 ? 'CLICK THE BUTTON' : 'VUE REACTIVITY WORKS'}}</p><ul><li v-for="n in count" :key="n">ITEM {{n}}</li></ul></section>`
}).mount('#app');
