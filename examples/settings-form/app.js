Vue.createApp({
  data() { return {displayName:'RWE USER',notify:true,mode:'simple',submitted:'尚未提交',status:'修改选项后，可提交并预览表单数据。'}; },
  methods: {
    submit(event) {
      const data = new FormData(event.currentTarget);
      this.submitted = Array.from(data).map(([key,value]) => key + '=' + value).join('; ');
      this.status = '已在应用内提交表单';
    },
    reset() {
      this.displayName='RWE USER'; this.notify=true; this.mode='simple';
      this.submitted='尚未提交'; this.status='已恢复默认设置';
    },
  },
  template: `<main><h1>应用设置</h1>
    <form id=settings @submit.prevent="submit" @reset.prevent="reset">
      <label for=display-name>显示名称</label>
      <input id=display-name name=displayName v-model="displayName" maxlength=24>
      <div class=choice><input id=notify type=checkbox name=notify value=yes v-model="notify">
      <label id=notify-label for=notify>接收更新通知</label></div>
      <div class=choice><input id=simple type=radio name=mode value=simple v-model="mode">
      <label for=simple>简洁模式</label></div>
      <div class=choice><input id=advanced type=radio name=mode value=advanced v-model="mode">
      <label id=advanced-label for=advanced>高级模式</label></div>
      <div class=choice><input id=locked type=checkbox checked disabled>
      <label for=locked>组织策略锁定（不可修改）</label></div>
      <div class=actions><button id=submit type=submit>提交并预览</button><button id=reset type=reset>恢复默认</button></div>
    </form>
    <p id=state>当前：{{displayName}} / 通知：{{notify ? '开启' : '关闭'}} / 模式：{{mode}}</p>
    <p id=status>{{status}}</p><p id=submitted>{{submitted}}</p>
  </main>`,
}).mount('#app');
