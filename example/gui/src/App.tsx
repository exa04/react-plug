import {usePluginContext} from './bindings/PluginProvider';
import {useState} from "react";
import {BinaryIcon, ChevronDown, ChevronRight, FunctionSquareIcon, HashIcon, TextIcon} from "lucide-react";

function App() {
  const pluginContext = usePluginContext();
  const params = pluginContext.parameters;
  const [editing, setEditing] = useState<null | string>();

  return <div className="bg-white text-slate-800 dark:bg-zinc-950 dark:text-zinc-200 flex h-svh">
    <div
      className="bg-slate-100 border-slate-300 dark:bg-zinc-900 border-r dark:border-zinc-700 grow w-full max-w-sm px-8 py-4"
      onMouseDown={() => setEditing(null)}>
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
                    defaultValue={param.format(param.value as never)}
                    onKeyUp={e => {
                      if (e.key === 'Enter') {
                        // @ts-ignore
                        param.setValue(parseFloat(e.target.value))
                        setEditing(null);
                      }
                    }}
                  />
                  : <div>{param.format(param.value as never)}</div>
                }
                {param.unit && <div>{param.unit}</div>}
              </div>
              <input type="range" className="slider"
                     min={0} max={1} step={0.001}
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
              className="bg-slate-300 hover:bg-slate-200 dark:bg-zinc-700 hover:dark:bg-zinc-600 cursor-default h-8 px-4 rounded-md"
              onClick={() => param.resetValue()}>
              Reset
            </button>
          </div>
        )}
      </div>
    </div>
    <div className="h-svh overflow-y-auto w-full grow pl-8 py-4 text-sm">
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
          let typeOrder = order.indexOf(typeof a[1]) - order.indexOf(typeof b[1]);
          if (typeOrder !== 0) return typeOrder;
          else return a[0].localeCompare(b[0]);
        })
        .map(([key, value]) => {
          switch (typeof value) {
            case "object":
              return <ObjectTree object={value} name={key}/>
            case "string":
              return <div className="flex gap-2 items-center text-orange-600 dark:text-orange-300">
                <TextIcon className="size-4"/>
                <span className="grow">{key}</span>
                <span className="w-32">"{value}"</span>
              </div>
            case "number":
              return <div className="flex gap-2 items-center text-sky-600 dark:text-sky-300">
                <HashIcon className="size-4"/>
                <span className="grow">{key}</span>
                <span className="w-32">{value.toPrecision(2)}</span>
              </div>
            case "boolean":
              return <div className="flex gap-2 items-center text-green-600 dark:text-green-300">
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
