const { contextBridge, ipcRenderer } = require('electron');

// Expose protected methods to the renderer process
contextBridge.exposeInMainWorld('electronAPI', {
    // Window controls
    minimize: () => ipcRenderer.send('minimize-window'),
    close: () => ipcRenderer.send('close-window'),
    toggleAlwaysOnTop: (value) => ipcRenderer.send('toggle-always-on-top', value),

    // Check if running in Electron
    isElectron: true,

    // Platform info
    platform: process.platform
});
