import {usePluginContext} from './bindings/PluginProvider';
import './styles.css';

function App() {
  const gain = usePluginContext().parameters.gain;
  const boolTest = usePluginContext().parameters.boolTest;
  const intTest = usePluginContext().parameters.intTest;

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
          <div>{gain.name}: {gain.value}{gain.unit}</div>
          <input type="range" className="slider"
                 min={0} max={1} step={0.001}
                 value={gain.range.normalize(gain.rawValue)}
                 onChange={e => {
                   gain.setValue(gain.range.unnormalize(Number(e.target.value)))
                 }}
          />
        </div>
        <button onClick={() => gain.setValue(gain.defaultValue)}>Reset</button>
      </div>
      <div className="input-group">
        <div className="labeled-input">
          <div>{intTest.name}: {intTest.value}{intTest.unit}</div>
          <input type="range" className="slider"
                 min={0} max={intTest.range.getMax()} step={1}
                 value={intTest.rawValue}
                 onChange={e => {
                   intTest.setValue(Number(e.target.value))
                 }}
          />
        </div>
        <button onClick={() => intTest.setValue(intTest.defaultValue)}>Reset</button>
      </div>
      <div className="input-group">
        <div className="labeled-input">
          <div>{boolTest.name}: {boolTest.value}</div>
        </div>
        <a onClick={() => boolTest.toggle()}>Toggle</a>
      </div>
    </div>
  )
}

export default App
