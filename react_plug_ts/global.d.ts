declare global {
  export interface Window {
    ipc: { postMessage: (message: string) => void };
    onPluginMessage: (message: Object) => void;
  }
}