import {usePluginContext} from './bindings/PluginProvider';
import {useEffect, useState} from "react";
import {BinaryIcon, ChevronDown, ChevronRight, FunctionSquareIcon, HashIcon, TextIcon} from "lucide-react";

function App() {
  const pluginContext = usePluginContext();
  const params = pluginContext.parameters;
  const [pluginMessages, setPluginMessages] = useState<{ message: any, time: Date }[]>([]);
  const [editing, setEditing] = useState<null | string>();

  const [fooMessage, setFooMessage] = useState("");
  const [barMessage, setBarMessage] = useState<{ a: number, b: number }>({a: 0, b: 0});

  useEffect(() => {
    const listener = (message: any) => {
      setPluginMessages(prev => [{message, time: new Date()}, ...prev]);
    };

    pluginContext.addMessageListener(listener);

    return () => pluginContext.removeMessageListener(listener);
  }, [pluginContext]);

  return <div className="bg-white text-slate-800 dark:bg-zinc-950 dark:text-zinc-200 flex h-svh">
    <div
      className="grow w-full max-w-sm p-4 flex flex-col gap-2"
      onMouseDown={() => setEditing(null)}>
      <div
        className="bg-slate-100 dark:bg-zinc-900 border-slate-200 dark:border-zinc-800 p-6 pt-3 rounded-lg border">
        <h2 className="text-xl font-bold mb-4">Parameters</h2>
        <div className="space-y-4">
          {Object.entries(params).map(([key, param]) =>
            <div className="flex items-center gap-4"
                 onMouseDown={(e) => {
                   if (e.ctrlKey && typeof param.value === 'number') {
                     e.preventDefault();
                     e.stopPropagation();
                     setEditing(key);
                   }
                 }}
            >
              <div className="grow">
                <div className="flex gap-1">
                  <div>{param.name}:</div>
                  {(typeof param.value === 'number') && editing == key ?
                    <input
                      autoFocus
                      type="number"
                      className="w-16 bg-transparent outline-none"
                      defaultValue={param.value_to_string(param.value as never)}
                      onKeyUp={e => {
                        if (e.key === 'Enter') {
                          // @ts-ignore
                          param.setValue(parseFloat(e.target.value))
                          setEditing(null);
                        }
                      }}
                    />
                    : <div>{param.value_to_string(param.value as never)}</div>
                  }
                  {param.unit && <div>{param.unit}</div>}
                </div>
                <input type="range" className="slider"
                       min={0} max={1} step={0.01}
                       value={param.normalizedValue}
                       onChange={e => {
                         param.setNormalizedValue(parseFloat(e.target.value))
                       }}
                       onDoubleClick={() => param.resetValue()}
                       onMouseDown={(e) => {
                         if (e.ctrlKey) {
                           e.preventDefault();
                           e.stopPropagation();
                           setEditing(key);
                         }
                       }}
                />
              </div>
              <button
                className="bg-slate-300 active:bg-slate-200 dark:bg-zinc-700 active:dark:bg-zinc-600 cursor-default h-8 px-4 rounded-md"
                onClick={() => param.resetValue()}>
                Reset
              </button>
            </div>
          )}
        </div>
      </div>
      <div
        className="bg-slate-100 dark:bg-zinc-900 border-slate-200 dark:border-zinc-800 p-6 pt-3 rounded-lg border grow flex flex-col">
        <h2 className="text-xl font-bold mb-2">Messages</h2>
        <div className="flex flex-col space-y-2">
          <button
            className="bg-slate-300 active:bg-slate-200 dark:bg-zinc-700 active:dark:bg-zinc-600 cursor-default h-8 px-4 rounded-md"
            onClick={() => pluginContext.sendToPlugin('Ping')}
          >
            Send Ping
          </button>
          <div className="flex gap-2 justify-between items-center">
            <div className="w-10">Foo</div>
            <input value={fooMessage} onChange={e => setFooMessage(e.target.value)}
                   className="grow shrink w-0 px-2 h-full bg-white dark:bg-zinc-950 border border-slate-200 dark:border-zinc-800 rounded-md"
                   onSubmit={() => pluginContext.sendToPlugin({'Foo': fooMessage})}
                   onKeyDown={e => {
                     if (e.key === 'Enter')
                       pluginContext.sendToPlugin({'Foo': fooMessage})
                   }}/>
            <button
              className="bg-slate-300 active:bg-slate-200 dark:bg-zinc-700 active:dark:bg-zinc-600 cursor-default h-8 px-4 rounded-md"
              onClick={() => pluginContext.sendToPlugin({'Foo': fooMessage})}
            >
              Send
            </button>
          </div>
          <div className="flex gap-2 justify-between items-center">
            <div className="w-10">Bar</div>
            <input value={barMessage.a} onChange={e => setBarMessage((bar) => {
              return {
                a: Number.parseFloat(e.target.value),
                b: bar.b
              }
            })}
                   className="grow shrink w-0 px-1 h-full bg-white dark:bg-zinc-950 border border-slate-200 dark:border-zinc-800 rounded-md"
                   onKeyDown={e => {
                     if (e.key === 'Enter')
                       pluginContext.sendToPlugin({'Bar': barMessage})
                   }}/>
            <input value={barMessage.b} onChange={e => setBarMessage((bar) => {
              return {
                a: bar.a,
                b: Number.parseFloat(e.target.value)
              }
            })}
                   className="grow shrink w-0 px-1 h-full bg-white dark:bg-zinc-950 border border-slate-200 dark:border-zinc-800 rounded-md"
                   onKeyDown={e => {
                     if (e.key === 'Enter')
                       pluginContext.sendToPlugin({'Bar': barMessage})
                   }}/>
            <button
              className="bg-slate-300 active:bg-slate-200 dark:bg-zinc-700 active:dark:bg-zinc-600 cursor-default h-8 px-4 rounded-md"
              onClick={() => pluginContext.sendToPlugin({'Bar': barMessage})}
            >
              Send
            </button>
          </div>
        </div>
        <div
          className="rounded-lg h-20 py-2 mt-2 bg-white dark:bg-zinc-950 overflow-y-auto grow">
          {pluginMessages.map((message, i) =>
            <div
              className="px-4 flex items-center"
              key={i}
            >
              <div className="grow">
                {JSON.stringify(message.message, function (_key, val) {
                  return val.toFixed ? Number(val.toFixed(3)) : val;
                })}
              </div>
              <div className="text-slate-500 dark:text-zinc-500 tabular-nums text-xs">
                {`${message.time.getHours().toString().padStart(2, '0')}:${message.time.getMinutes().toString().padStart(2, '0')}:${message.time.getSeconds().toString().padStart(2, '0')}`}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
    <div className="h-svh overflow-y-auto w-full grow pl-2 py-4 text-sm">
      <h2 className="text-xl font-bold mb-2">Plugin Context</h2>
      <div className="max-w-lg">
        <ObjectTree object={pluginContext} name="pluginContext"/>
      </div>
    </div>
  </div>
}

function ObjectTree({object, name}: { object: object, name: string }) {
  const [expanded, setExpanded] = useState<boolean>(true);

  return <div>
    <div className="flex gap-2 items-center">
      <div className="hover:dark:text-zinc-50 hover:text-slate-950 text-slate-400 dark:text-zinc-500"
           onClick={() => setExpanded((expanded) => !expanded)}>
        {expanded ? <ChevronDown className="size-4"/> : <ChevronRight className="size-4"/>}
      </div>
      <span>{name}</span>
    </div>
    <div className="pl-4 ml-2 border-l border-slate-300 dark:border-zinc-800">{
      expanded && Object.entries(object)
        .sort((a, b) => {
          const order = ["object", "string", "number", "bigint", "boolean", "function"];
          return order.indexOf(typeof a[1]) - order.indexOf(typeof b[1]);
        })
        .map(([key, value]) => {
          switch (typeof value) {
            case "object":
              return <ObjectTree object={value} name={key}/>
            case "string":
              return <div className="flex gap-2 items-center text-green-600 dark:text-green-300">
                <TextIcon className="size-4"/>
                <span className="grow">{key}</span>
                <span className="w-32">"{value}"</span>
              </div>
            case "number":
              return <div className="flex gap-2 items-center text-purple-600 dark:text-purple-300">
                <HashIcon className="size-4"/>
                <span className="grow">{key}</span>
                <span className="w-32 tabular-nums">{value.toPrecision(2)}</span>
              </div>
            case "boolean":
              return <div className="flex gap-2 items-center text-cyan-600 dark:text-cyan-300">
                <BinaryIcon className="size-4"/>
                <span className="grow">{key}</span>
                <span className="w-32">{value ? "true" : "false"}</span>
              </div>
            case "function":
              return <div className="flex gap-2 items-center text-slate-600 dark:text-zinc-400">
                <FunctionSquareIcon className="size-4"/>
                <span>{key}()</span>
              </div>
          }
        })
    }
    </div>
  </div>
}

export default App
