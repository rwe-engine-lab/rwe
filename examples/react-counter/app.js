function Counter() {
  const [count, setCount] = React.useState(0);
  return React.createElement('section', null,
    React.createElement('button', {id:'increment', onClick:() => setCount(n => n + 1)}, 'COUNT: ' + count),
    React.createElement('p', {id:'status'}, count === 0 ? 'CLICK THE BUTTON' : 'REACT STATE WORKS'));
}
ReactDOM.createRoot(document.getElementById('app')).render(React.createElement(Counter));
