import './styles.css';

import {usePluginContext} from './bindings/PluginProvider';

function App() {
  const ctx = usePluginContext();

  const params = ctx.parameters;
  const gain = params.gain;

  return (
    <div className="container">
      <div className="description">
        <div className="leading">
          React-Plug
        </div>
        <h1>Gain Example</h1>
        <div>
          A basic example of a gain plug-in. Created using the <b>React-Plug</b> framework.
        </div>
      </div>
      <hr/>
      <div className="input-group">
        <div className="labeled-input">
          <div>{gain.name}: {gain.format(gain.value)}{gain.unit}</div>
          <input type="range" className="slider"
                 min={0} max={1} step={0.001}
                 value={gain.normalizedValue}
                 onChange={e => {
                   console.log('onChange', e.target.value)
                   gain.setNormalizedValue(parseFloat(e.target.value))
                 }}
          />
        </div>
        {/*<button onClick={() => gain.setValue(gain.defaultValue)}>Reset</button>*/}
      </div>
      <pre>
{JSON.stringify(params, function (_key, value) {
  if (typeof value === 'number') {
    return parseFloat(value.toFixed(2));
  }
  return value;
}, 2)}
      </pre>
    </div>
  )
}

export default App
