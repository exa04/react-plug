import {usePluginContext} from './bindings/PluginProvider';

function App() {
  const gain = usePluginContext().parameters.gain;

  return (
    <div>
      <div>{gain.name}: {gain.value}</div>
      <input type="range"
             min={gain.range[0]} max={gain.range[1]} step={0.01}
             value={gain.rawValue}
             onChange={e => gain.setValue(Number(e.target.value))}
      /><br/>
      <button onClick={() => gain.setValue(gain.defaultValue)}>Reset</button>
    </div>
  )
}

export default App
