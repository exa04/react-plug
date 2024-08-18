import './styles.css';

import {usePluginContext} from './bindings/PluginProvider';
import {PluginMessage} from "./bindings/PluginMessage";
import {useEffect, useState} from "react";

function App() {
  const ctx = usePluginContext();

  const gain = ctx.parameters.gain;
  const boolTest = ctx.parameters.boolTest;
  const intTest = ctx.parameters.intTest;
  const enumTest = ctx.parameters.enumTest;

  const [pongCount, setPongCount] = useState(0);

  useEffect(() => {
    const handler = (message: PluginMessage) => {
      if (message === "Pong")
        setPongCount(prevCount => prevCount + 1);
    };

    ctx.addMessageListener(handler);

    return () => ctx.removeMessageListener(handler);
  }, [ctx]);

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
      <div className="input-group">
        <div className="labeled-input">
          <div>{enumTest.name}: {enumTest.rawValue}</div>
        </div>
        <select value={enumTest.rawValue} onChange={e => enumTest.setValue(e.target.value)}>
          {Object.entries(enumTest.variants).map(variant =>
            <option value={variant[0]}>
              {variant[1]}
            </option>
          )}
        </select>
      </div>
      <hr/>
      <div className="message-group">
        <button onClick={() => ctx.sendToPlugin("Ping")}>Send Ping</button>
        <div>Pong counter: {pongCount}</div>
      </div>
    </div>
  )
}

export default App
