import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App.tsx'
import PluginProvider from "./bindings/PluginProvider.tsx";

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <PluginProvider>
      <App />
    </PluginProvider>
  </React.StrictMode>,
)
