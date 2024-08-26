import './styles.css';

import {usePluginContext} from './bindings/PluginProvider';

function App() {
  const ctx = usePluginContext();

  const params = ctx.parameters;

  return (
    <div className="container">
      <div className="params">
        <h2>Parameters</h2>
        {Object.entries(params).map(([key, param]) => (
          <div className="input-group" key={key}>
            <div className="labeled-input">
              <div>{param.name}: {param.format(param.value)}{param.unit || ""}</div>
              <input type="range" className="slider"
                     min={0} max={1} step={0.001}
                     value={param.normalizedValue}
                     onChange={e => {
                       param.setNormalizedValue(parseFloat(e.target.value))
                     }}
              />
            </div>
            <button onClick={() => param.resetValue()}>Reset</button>
          </div>
        ))}
      </div>
      <div className="json">
        <h2>Internal representation</h2>
        <pre>
{JSON.stringify(params, function (_key, value) {
  if (typeof value === 'number') {
    return parseFloat(value.toFixed(2));
  }
  return value;
}, 4)}
      </pre>
      </div>
    </div>
  )
}

export default App
