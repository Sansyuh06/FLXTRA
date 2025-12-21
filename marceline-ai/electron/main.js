const { app, BrowserWindow, Tray, Menu, ipcMain, globalShortcut } = require('electron');
const path = require('path');

let mainWindow = null;
let tray = null;
const isDev = process.env.NODE_ENV === 'development' || !app.isPackaged;

function createWindow() {
    mainWindow = new BrowserWindow({
        width: 450,
        height: 700,
        minWidth: 350,
        minHeight: 500,
        frame: false,
        transparent: true,
        alwaysOnTop: true,
        skipTaskbar: false,
        resizable: true,
        webPreferences: {
            nodeIntegration: false,
            contextIsolation: true,
            preload: path.join(__dirname, 'preload.js')
        }
    });

    // Load the app
    if (isDev) {
        mainWindow.loadURL('http://localhost:3000');
        // Open DevTools in development
        // mainWindow.webContents.openDevTools({ mode: 'detach' });
    } else {
        mainWindow.loadFile(path.join(__dirname, '../frontend/dist/index.html'));
    }

    // Show on all workspaces (macOS)
    mainWindow.setVisibleOnAllWorkspaces(true, { visibleOnFullScreen: true });

    // Handle close - hide instead of quit
    mainWindow.on('close', (event) => {
        if (!app.isQuitting) {
            event.preventDefault();
            mainWindow.hide();
        }
    });

    mainWindow.on('closed', () => {
        mainWindow = null;
    });
}

function createTray() {
    // Use a simple icon for the tray
    const iconPath = isDev
        ? path.join(__dirname, '../frontend/public/marceline.svg')
        : path.join(__dirname, '../frontend/dist/marceline.svg');

    try {
        tray = new Tray(iconPath);
    } catch (e) {
        // Fallback - create tray without icon
        console.log('Could not load tray icon, using default');
    }

    const contextMenu = Menu.buildFromTemplate([
        {
            label: 'Show Marceline',
            click: () => {
                if (mainWindow) {
                    mainWindow.show();
                    mainWindow.focus();
                }
            }
        },
        {
            label: 'Hide',
            click: () => {
                if (mainWindow) {
                    mainWindow.hide();
                }
            }
        },
        { type: 'separator' },
        {
            label: 'Always on Top',
            type: 'checkbox',
            checked: true,
            click: (menuItem) => {
                if (mainWindow) {
                    mainWindow.setAlwaysOnTop(menuItem.checked);
                }
            }
        },
        {
            label: 'Start with System',
            type: 'checkbox',
            checked: app.getLoginItemSettings().openAtLogin,
            click: (menuItem) => {
                app.setLoginItemSettings({ openAtLogin: menuItem.checked });
            }
        },
        { type: 'separator' },
        {
            label: 'Quit Marceline',
            click: () => {
                app.isQuitting = true;
                app.quit();
            }
        }
    ]);

    if (tray) {
        tray.setToolTip('Marceline AI Assistant');
        tray.setContextMenu(contextMenu);

        // Toggle window on tray click
        tray.on('click', () => {
            if (mainWindow) {
                if (mainWindow.isVisible()) {
                    mainWindow.hide();
                } else {
                    mainWindow.show();
                    mainWindow.focus();
                }
            }
        });
    }
}

function registerShortcuts() {
    // Global shortcut to toggle window
    globalShortcut.register('CommandOrControl+Shift+M', () => {
        if (mainWindow) {
            if (mainWindow.isVisible()) {
                mainWindow.hide();
            } else {
                mainWindow.show();
                mainWindow.focus();
            }
        }
    });
}

// App lifecycle
app.whenReady().then(() => {
    createWindow();
    createTray();
    registerShortcuts();

    app.on('activate', () => {
        if (BrowserWindow.getAllWindows().length === 0) {
            createWindow();
        } else if (mainWindow) {
            mainWindow.show();
        }
    });
});

app.on('window-all-closed', () => {
    if (process.platform !== 'darwin') {
        app.quit();
    }
});

app.on('before-quit', () => {
    app.isQuitting = true;
});

app.on('will-quit', () => {
    globalShortcut.unregisterAll();
});

// IPC handlers
ipcMain.on('minimize-window', () => {
    if (mainWindow) mainWindow.minimize();
});

ipcMain.on('close-window', () => {
    if (mainWindow) mainWindow.hide();
});

ipcMain.on('toggle-always-on-top', (event, value) => {
    if (mainWindow) mainWindow.setAlwaysOnTop(value);
});

// Prevent multiple instances
const gotTheLock = app.requestSingleInstanceLock();

if (!gotTheLock) {
    app.quit();
} else {
    app.on('second-instance', () => {
        if (mainWindow) {
            if (mainWindow.isMinimized()) mainWindow.restore();
            mainWindow.show();
            mainWindow.focus();
        }
    });
}
